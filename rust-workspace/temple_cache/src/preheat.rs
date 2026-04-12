//! 启动预热 — Rayon 多线程并行扫描 Vault 目录

use std::path::Path;

use rayon::prelude::*;

use ast_renderer::render_to_html;
use md_parser::{extract_wikilinks, parse_markdown};

use crate::document::Document;
use crate::pool::DocumentPool;
use temple_core::error::{ErrorCode, TempleError, TempleResult};

/// 预热统计
#[derive(Debug, Clone)]
pub struct PreheatStats {
    pub total_files: usize,
    pub parsed_ok: usize,
    pub parse_errors: usize,
    pub elapsed_ms: u64,
}

/// 从文件路径提取文件名（不含扩展名）
fn extract_filename(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// 从 Markdown 内容提取标题（首行 H1 或回退到文件名）
fn extract_title(content: &str, filename: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            return title.to_string();
        }
        if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with("---") {
            break;
        }
    }
    filename.to_string()
}

/// 从内容提取纯文本摘要
fn extract_excerpt(content: &str, max_len: usize) -> String {
    let mut result = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "---" {
            continue;
        }
        let cleaned = trimmed
            .replace(['*', '_', '`'], "")
            .replace(['[', ']', '('], "")
            .trim_start_matches('-')
            .trim()
            .to_string();
        if !cleaned.is_empty() {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&cleaned);
        }
        if result.chars().count() >= max_len {
            break;
        }
    }
    let char_count = result.chars().count();
    if char_count > max_len {
        let truncated: String = result.chars().take(max_len).collect();
        format!("{truncated}...")
    } else {
        result
    }
}

/// 当前时间戳（毫秒）
fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// 解析单个 Markdown 文件为 Document
pub(crate) fn parse_single_file(path: &Path) -> Result<Document, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("read failed: {e}"))?;
    let filename = extract_filename(path);
    let title = extract_title(&content, &filename);
    let excerpt = extract_excerpt(&content, 150);
    let extracted_links = extract_wikilinks(&content);

    // 先完成所有借用操作（AST borrow content），再移动 content
    let ast = parse_markdown(&content)
        .map_err(|e| format!("parse failed: {e:?}"))?;
    let html = render_to_html(&ast).map_err(|e| format!("render failed: {e:?}"))?;
    let ast_json = serde_json::to_string(&ast).map_err(|e| format!("serialize failed: {e}"))?;
    // AST 借用在此结束，content 可以被移动

    Ok(Document {
        path: path.to_string_lossy().to_string(),
        filename,
        title,
        raw_md: content,
        excerpt,
        html,
        ast_json,
        extracted_links,
        updated_at: now_millis(),
    })
}

/// 收集目录下所有 Markdown 文件
fn collect_md_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, TempleError> {
    let mut files = Vec::new();
    let _ = std::fs::read_dir(dir).map_err(|e| {
        TempleError::new(ErrorCode::DirectoryNotFound, format!("无法读取目录 {}: {e}", dir.display()))
    })?;

    fn walk(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<(), TempleError> {
        let entries = std::fs::read_dir(dir).map_err(|e| {
            TempleError::new(ErrorCode::DirectoryNotFound, format!("遍历目录失败: {e}"))
        })?;
        for entry in entries {
            let entry = entry.map_err(|e| {
                TempleError::new(ErrorCode::FileReadFailed, format!("读取目录项失败: {e}"))
            })?;
            let path = entry.path();
            if path.is_dir() {
                // 跳过隐藏目录和 .obsidian
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with('.') || name == ".obsidian" || name == ".git" {
                    continue;
                }
                walk(&path, files)?;
            } else if path
                .extension()
                .map(|ext| ext == "md" || ext == "markdown")
                .unwrap_or(false)
            {
                files.push(path);
            }
        }
        Ok(())
    }

    walk(dir, &mut files)?;
    Ok(files)
}

/// 并行预热：扫描整个 Vault 目录，解析所有 Markdown 文件
pub fn preheat_vault(pool: &DocumentPool, vault_path: &str) -> TempleResult<PreheatStats> {
    let start = std::time::Instant::now();
    let dir = Path::new(vault_path);

    if !dir.exists() {
        return Err(TempleError::new(
            ErrorCode::DirectoryNotFound,
            format!("Vault 目录不存在: {vault_path}"),
        ));
    }

    let files = collect_md_files(dir)?;
    let total_files = files.len();

    // Rayon 并行解析
    let parsed: Vec<Result<Document, String>> = files
        .par_iter()
        .map(|path| parse_single_file(path))
        .collect();

    let mut parsed_ok = 0;
    let mut parse_errors = 0;

    for result in parsed {
        match result {
            Ok(doc) => {
                pool.upsert(doc);
                parsed_ok += 1;
            }
            Err(_e) => {
                parse_errors += 1;
            }
        }
    }

    let elapsed = start.elapsed();
    Ok(PreheatStats {
        total_files,
        parsed_ok,
        parse_errors,
        elapsed_ms: elapsed.as_millis() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_extract_title_from_h1() {
        assert_eq!(extract_title("# Hello World\nbody", "file"), "Hello World");
    }

    #[test]
    fn test_extract_title_fallback_to_filename() {
        assert_eq!(extract_title("No heading here", "myfile"), "myfile");
    }

    #[test]
    fn test_extract_excerpt() {
        let content = "# Title\n\nThis is the first paragraph.\n\nSecond paragraph here.";
        let excerpt = extract_excerpt(content, 50);
        assert!(excerpt.contains("first paragraph"));
    }

    #[test]
    fn test_preheat_empty_dir() {
        let tmp = std::env::temp_dir().join("temple_cache_test_empty");
        let _ = fs::create_dir_all(&tmp);

        let pool = DocumentPool::new();
        let stats = preheat_vault(&pool, tmp.to_str().unwrap()).unwrap();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.parsed_ok, 0);

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_preheat_with_files() {
        let tmp = std::env::temp_dir().join("temple_cache_test_files");
        let _ = fs::create_dir_all(&tmp);

        fs::write(tmp.join("card1.md"), "# Card One\nContent of card 1.").unwrap();
        fs::write(tmp.join("card2.md"), "# Card Two\nContent of card 2.").unwrap();
        fs::write(tmp.join("readme.txt"), "Not a markdown file").unwrap();

        let pool = DocumentPool::new();
        let stats = preheat_vault(&pool, tmp.to_str().unwrap()).unwrap();

        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.parsed_ok, 2);
        assert_eq!(pool.len(), 2);

        let doc = pool.get(tmp.join("card1.md").to_str().unwrap()).unwrap();
        assert_eq!(doc.title, "Card One");
        assert!(doc.html.contains("Card One"));

        let _ = fs::remove_dir_all(&tmp);
    }
}
