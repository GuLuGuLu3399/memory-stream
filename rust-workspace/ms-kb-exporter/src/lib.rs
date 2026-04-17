#![cfg(feature = "native")]

pub mod error;

use error::{ExportError, ExportResult};
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use tokio::task;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbCard {
    pub category_name: String,
    pub title: String,
    pub raw_md: String,
    #[serde(default)]
    pub images: Vec<KbImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbImage {
    pub filename: String,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

mod serde_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(data: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&base64_encode(data))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        base64_decode(&s).map_err(serde::de::Error::custom)
    }

    fn base64_encode(data: &[u8]) -> String {
        use std::fmt::Write;
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
        for chunk in data.chunks(3) {
            let b0 = u32::from(chunk[0]);
            let b1 = if chunk.len() > 1 { u32::from(chunk[1]) } else { 0 };
            let b2 = if chunk.len() > 2 { u32::from(chunk[2]) } else { 0 };
            let triple = (b0 << 16) | (b1 << 8) | b2;
            let _ = write!(
                out,
                "{}{}{}{}",
                CHARS[((triple >> 18) & 0x3F) as usize] as char,
                CHARS[((triple >> 12) & 0x3F) as usize] as char,
                if chunk.len() > 1 {
                    CHARS[((triple >> 6) & 0x3F) as usize] as char
                } else {
                    '='
                },
                if chunk.len() > 2 {
                    CHARS[(triple & 0x3F) as usize] as char
                } else {
                    '='
                }
            );
        }
        out
    }

    // CC-理由: 保持 API 一致性，与其他序列化/反序列化函数统一返回 Result
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::cast_possible_truncation)]
    fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let s = s.trim_end_matches('=');
        // CC-理由: Base64 解码的字符位置不会超过 63，符合 u32 范围
        #[allow(clippy::cast_possible_truncation)]
        let mut out = Vec::with_capacity(s.len() * 3 / 4);
        for chunk in s.as_bytes().chunks(4) {
            let mut acc = 0u32;
            let mut bits = 0;
            for &b in chunk {
                if let Some(pos) = CHARS.iter().position(|&c| c == b) {
                    acc = (acc << 6) | pos as u32;
                    bits += 6;
                }
            }
            while bits >= 8 {
                bits -= 8;
                out.push(((acc >> bits) & 0xFF) as u8);
            }
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSummary {
    pub total_cards: usize,
    pub total_images: usize,
    pub zip_size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub compression: zip::CompressionMethod,
    pub flatten_images: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            compression: zip::CompressionMethod::Deflated,
            flatten_images: true,
        }
    }
}

/// 导出卡片到 ZIP 文件。
///
/// # Errors
/// 返回错误如果文件创建或压缩失败。
pub async fn export_to_zip(
    cards: Vec<KbCard>,
    dest_path: &Path,
    options: ExportOptions,
) -> ExportResult<ExportSummary> {
    let dest = dest_path.to_path_buf();
    let total_cards = cards.len();
    let total_images: usize = cards.iter().map(|c| c.images.len()).sum();

    task::spawn_blocking(move || {
        let file = std::fs::File::create(&dest)
            .map_err(|e| ExportError::FileCreateError(e.to_string()))?;
        let mut zip = zip::ZipWriter::new(file);
        let zip_options: zip::write::FileOptions<'_, zip::write::ExtendedFileOptions> =
            zip::write::FileOptions::default().compression_method(options.compression);

        for card in &cards {
            let category = sanitize_filename(&card.category_name);
            let title = sanitize_filename(&card.title);

            let md_path = format!("{category}/{title}.md");
            zip.start_file(&md_path, zip_options.clone())?;
            zip.write_all(card.raw_md.as_bytes())?;

            for img in &card.images {
                let img_dir = if options.flatten_images {
                    "images".to_string()
                } else {
                    category.clone()
                };
                let img_path = format!("{}/{}", img_dir, sanitize_filename(&img.filename));
                zip.start_file(&img_path, zip_options.clone())?;
                zip.write_all(&img.data)?;
            }
        }

        zip.finish()?;

        let zip_size = std::fs::metadata(&dest)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(ExportSummary {
            total_cards,
            total_images,
            zip_size_bytes: zip_size,
        })
    })
    .await
    .map_err(extract_panic_source)?
}

/// 使用 fetcher 函数导出卡片到 ZIP 文件。
///
/// # Errors
/// 返回错误如果获取卡片或导出失败。
pub async fn export_with_fetcher<F, Fut>(
    card_ids: Vec<String>,
    dest_path: &Path,
    fetcher: F,
    concurrency: usize,
) -> ExportResult<ExportSummary>
where
    F: Fn(String) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ExportResult<KbCard>> + Send,
{
    let dest = dest_path.to_path_buf();
    let fetcher = std::sync::Arc::new(fetcher);

    let cards: Vec<KbCard> = stream::iter(card_ids)
        .map(|id| {
            let f = fetcher.clone();
            async move { f(id).await }
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<ExportResult<Vec<_>>>()?;

    export_to_zip(cards, &dest, ExportOptions::default()).await
}

fn extract_panic_source(join_err: tokio::task::JoinError) -> ExportError {
    let reason = if join_err.is_panic() {
        let panic_err = join_err.into_panic();
        if let Some(s) = panic_err.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_err.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        }
    } else {
        "Task cancelled".to_string()
    };
    ExportError::TaskPanic { reason }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ if c.is_control() => '_', // 控制字符替换为下划线
            _ => c,
        })
        .collect::<String>()
        .trim()
        .trim_start_matches('.')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use zip::read::ZipArchive;

    fn test_card(category: &str, title: &str, md: &str) -> KbCard {
        KbCard {
            category_name: category.to_string(),
            title: title.to_string(),
            raw_md: md.to_string(),
            images: vec![],
        }
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Hello/World"), "Hello_World");
        assert_eq!(sanitize_filename("a:b*c?d"), "a_b_c_d");
        assert_eq!(sanitize_filename("normal-name.md"), "normal-name.md");
        // 控制字符被替换
        assert_eq!(sanitize_filename("file\x00name"), "file_name");
        assert_eq!(sanitize_filename("path\x1b"), "path_");
        // 前导点被移除（防止隐藏文件）
        assert_eq!(sanitize_filename(".hidden"), "hidden");
    }

    #[tokio::test]
    async fn test_export_single_card() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new()?;
        let zip_path = dir.path().join("export.zip");

        let cards = vec![test_card("Rust", "入门指南", "# Hello Rust\n\n这是一个测试。")];
        let summary = export_to_zip(cards, &zip_path, ExportOptions::default())
            .await?;

        assert_eq!(summary.total_cards, 1);
        assert_eq!(summary.total_images, 0);
        assert!(zip_path.exists());
        assert!(summary.zip_size_bytes > 0);

        let file = std::fs::File::open(&zip_path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut entry = archive.by_index(0)?;
        let mut content = String::new();
        std::io::Read::read_to_string(&mut entry, &mut content)?;
        assert!(content.contains("# Hello Rust"));

        Ok(())
    }

    #[tokio::test]
    async fn test_export_multiple_categories() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new()?;
        let zip_path = dir.path().join("export.zip");

        let cards = vec![
            test_card("Go", "后端优化", "# Go 后端优化"),
            test_card("Rust", "所有权", "# Rust 所有权"),
            test_card("Rust", "借用", "# Rust 借用"),
        ];

        let summary = export_to_zip(cards, &zip_path, ExportOptions::default())
            .await?;

        assert_eq!(summary.total_cards, 3);

        let file = std::fs::File::open(&zip_path)?;
        let archive = ZipArchive::new(file)?;
        let names: Vec<String> = archive.file_names().map(std::string::ToString::to_string).collect();
        assert!(names.iter().any(|n| n.starts_with("Go/")));
        assert!(names.iter().any(|n| n.starts_with("Rust/")));
        assert_eq!(names.len(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_export_with_images() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new()?;
        let zip_path = dir.path().join("export.zip");

        let cards = vec![KbCard {
            category_name: "笔记".to_string(),
            title: "图文测试".to_string(),
            raw_md: "![img](photo.png)".to_string(),
            images: vec![KbImage {
                filename: "photo.png".to_string(),
                data: vec![0x89, 0x50, 0x4E, 0x47],
            }],
        }];

        let options = ExportOptions {
            flatten_images: true,
            ..Default::default()
        };
        let summary = export_to_zip(cards, &zip_path, options).await?;
        assert_eq!(summary.total_images, 1);

        let file = std::fs::File::open(&zip_path)?;
        let archive = ZipArchive::new(file)?;
        let names: Vec<String> = archive.file_names().map(std::string::ToString::to_string).collect();
        assert!(names.iter().any(|n| n.starts_with("images/")));

        Ok(())
    }

    #[tokio::test]
    async fn test_export_with_images_nested_dirs() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new()?;
        let zip_path = dir.path().join("export.zip");

        let cards = vec![KbCard {
            category_name: "技术".to_string(),
            title: "测试".to_string(),
            raw_md: "text".to_string(),
            images: vec![KbImage {
                filename: "diagram.png".to_string(),
                data: vec![1, 2, 3],
            }],
        }];

        let options = ExportOptions {
            flatten_images: false,
            ..Default::default()
        };
        let summary = export_to_zip(cards, &zip_path, options).await?;
        assert_eq!(summary.total_images, 1);

        let file = std::fs::File::open(&zip_path)?;
        let archive = ZipArchive::new(file)?;
        let names: Vec<String> = archive.file_names().map(std::string::ToString::to_string).collect();
        assert!(names.iter().any(|n| n == "技术/diagram.png"));

        Ok(())
    }

    #[tokio::test]
    async fn test_export_empty_cards() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new()?;
        let zip_path = dir.path().join("export.zip");

        let summary = export_to_zip(vec![], &zip_path, ExportOptions::default())
            .await?;
        assert_eq!(summary.total_cards, 0);
        assert!(zip_path.exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_export_special_chars_in_filename() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new()?;
        let zip_path = dir.path().join("export.zip");

        let cards = vec![test_card(
            "A/B:C",
            "测试<文件>?",
            "# 特殊字符",
        )];
        let summary = export_to_zip(cards, &zip_path, ExportOptions::default())
            .await?;
        assert_eq!(summary.total_cards, 1);

        let file = std::fs::File::open(&zip_path)?;
        let archive = ZipArchive::new(file)?;
        let names: Vec<String> = archive.file_names().map(std::string::ToString::to_string).collect();
        assert_eq!(names.len(), 1);
        assert!(!names[0].contains(':'));
        assert!(!names[0].contains('?'));

        Ok(())
    }
}
