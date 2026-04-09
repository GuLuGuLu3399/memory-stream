//! # 知识库导出命令
//!
//! 对接 `ms-kb-exporter` Crate，将卡片数据打包为 ZIP 文件。
//! 所有命令均为 async，避免阻塞 Tauri 主线程。

use ms_kb_exporter::{ExportOptions, KbCard, KbImage, export_to_zip};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 前端传来的卡片导出数据（包含可选的 Base64 图片）
#[derive(Deserialize)]
pub struct ExportCardInput {
    category_name: String,
    title: String,
    raw_md: String,
    #[serde(default)]
    images: Vec<ExportImageInput>,
}

/// 前端传来的图片数据（Base64 编码）
#[derive(Deserialize)]
pub struct ExportImageInput {
    filename: String,
    /// Base64 编码的图片数据
    data: String,
}

/// 导出结果摘要
#[derive(Serialize, TS)]
#[ts(export_to = ".")]
pub struct ExportSummaryDto {
    total_cards: usize,
    total_images: usize,
    #[ts(type = "number")]
    zip_size_bytes: u64,
}

impl From<ms_kb_exporter::ExportSummary> for ExportSummaryDto {
    fn from(s: ms_kb_exporter::ExportSummary) -> Self {
        Self {
            total_cards: s.total_cards,
            total_images: s.total_images,
            zip_size_bytes: s.zip_size_bytes,
        }
    }
}

/// 导出知识库为 ZIP 文件
///
/// 接收前端传来的卡片数组，调用 `ms-kb-exporter` 打包为 ZIP。
/// 使用 `async` + `spawn_blocking` 确保不阻塞主线程。
///
/// # 参数
/// - `cards`: 卡片数组（含分类、标题、内容、图片）
/// - `dest_path`: 用户通过原生对话框选择的保存路径
#[tauri::command]
pub async fn export_knowledge_base(
    cards: Vec<ExportCardInput>,
    dest_path: String,
) -> Result<ExportSummaryDto, String> {
    let kb_cards: Vec<KbCard> = cards
        .into_iter()
        .map(|c| KbCard {
            category_name: c.category_name,
            title: c.title,
            raw_md: c.raw_md,
            images: c
                .images
                .into_iter()
                .filter_map(|img| {
                    // Base64 解码图片数据
                    let data = base64_decode(&img.data)?;
                    Some(KbImage {
                        filename: img.filename,
                        data,
                    })
                })
                .collect(),
        })
        .collect();

    let dest = std::path::PathBuf::from(&dest_path);
    let options = ExportOptions::default();

    let summary = export_to_zip(kb_cards, &dest, options)
        .await
        .map_err(|e| format!("导出失败: {:?}", e))?;

    Ok(ExportSummaryDto::from(summary))
}

/// 简单的 Base64 解码（不依赖额外 crate）
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let s = input.trim_end_matches('=');
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
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    #[test]
    fn test_base64_decode_hello() {
        let result = base64_decode("SGVsbG8=").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_base64_decode_empty() {
        let result = base64_decode("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_base64_decode_binary() {
        let input = base64::engine::general_purpose::STANDARD.encode(b"\x00\x01\x02\xff");
        let result = base64_decode(&input).unwrap();
        assert_eq!(result, vec![0x00, 0x01, 0x02, 0xFF]);
    }
}
