//! 知识库导出 — ZIP 打包

use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use tokio::task;

use crate::error::{IoError, IoResult};

/// 导出卡片数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbCard {
    pub category_name: String,
    pub title: String,
    pub raw_md: String,
    #[serde(default)]
    pub images: Vec<KbImage>,
}

/// 导出图片数据
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
        const CHARS: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
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

    #[allow(clippy::unnecessary_wraps)]
    fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
        const CHARS: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let s = s.trim_end_matches('=');
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

/// 导出统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSummary {
    pub total_cards: usize,
    pub total_images: usize,
    pub zip_size_bytes: u64,
}

/// 导出选项
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

/// 导出卡片到 ZIP 文件
pub async fn export_to_zip(
    cards: Vec<KbCard>,
    dest_path: &Path,
    options: ExportOptions,
) -> IoResult<ExportSummary> {
    let dest = dest_path.to_path_buf();
    let total_cards = cards.len();
    let total_images: usize = cards.iter().map(|c| c.images.len()).sum();

    task::spawn_blocking(move || {
        let file = std::fs::File::create(&dest)
            .map_err(|e| IoError::ExportCreateFailed(e.to_string()))?;
        let mut zip = zip::ZipWriter::new(file);
        let zip_options: zip::write::FileOptions<'_, zip::write::ExtendedFileOptions> =
            zip::write::FileOptions::default().compression_method(options.compression);

        for card in &cards {
            let category = sanitize_filename(&card.category_name);
            let title = sanitize_filename(&card.title);

            let md_path = format!("{category}/{title}.md");
            zip.start_file(&md_path, zip_options.clone())
                .map_err(|e| IoError::ExportZipFailed(e.to_string()))?;
            zip.write_all(card.raw_md.as_bytes())?;

            for img in &card.images {
                let img_dir = if options.flatten_images {
                    "images".to_string()
                } else {
                    category.clone()
                };
                let img_path = format!("{}/{}", img_dir, sanitize_filename(&img.filename));
                zip.start_file(&img_path, zip_options.clone())
                    .map_err(|e| IoError::ExportZipFailed(e.to_string()))?;
                zip.write_all(&img.data)?;
            }
        }

        zip.finish()
            .map_err(|e| IoError::ExportZipFailed(e.to_string()))?;

        let zip_size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);

        Ok(ExportSummary {
            total_cards,
            total_images,
            zip_size_bytes: zip_size,
        })
    })
    .await
    .map_err(extract_panic)?
}

/// 使用 fetcher 函数异步拉取卡片后导出
pub async fn export_with_fetcher<F, Fut>(
    card_ids: Vec<String>,
    dest_path: &Path,
    fetcher: F,
    concurrency: usize,
) -> IoResult<ExportSummary>
where
    F: Fn(String) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = IoResult<KbCard>> + Send,
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
        .collect::<IoResult<Vec<_>>>()?;

    export_to_zip(cards, &dest, ExportOptions::default()).await
}

fn extract_panic(join_err: tokio::task::JoinError) -> IoError {
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
    IoError::TaskPanic { reason }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ if c.is_control() => '_',
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

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello/world"), "hello_world");
        assert_eq!(sanitize_filename(".hidden"), "hidden");
        assert_eq!(sanitize_filename("normal.md"), "normal.md");
    }

    #[tokio::test]
    async fn test_export_to_zip() -> IoResult<()> {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("export.zip");

        let cards = vec![KbCard {
            category_name: "notes".into(),
            title: "test".into(),
            raw_md: "# Hello".into(),
            images: vec![],
        }];

        let summary = export_to_zip(cards, &dest, ExportOptions::default()).await?;
        assert_eq!(summary.total_cards, 1);
        assert!(dest.exists());
        Ok(())
    }
}
