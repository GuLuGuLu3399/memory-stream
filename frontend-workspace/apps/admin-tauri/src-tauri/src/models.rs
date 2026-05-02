use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CardMeta {
    pub uuid: String,
    pub title: String,
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentStats {
    pub lines: usize,
    pub chars: usize,
    pub words: usize,
    pub read_time: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TocItem {
    pub level: u8,
    pub text: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocAnalysis {
    pub stats: DocumentStats,
    pub toc: Vec<TocItem>,
    pub excerpt: String,
    pub outbound_links: usize,
}

#[derive(Serialize)]
pub struct ParsedDocument {
    pub meta: Option<CardMeta>,
    pub analysis: DocAnalysis,
    pub ast: Value,
    pub content: String,
}

pub(crate) fn render_frontmatter(meta: &CardMeta) -> Result<String, String> {
    let yaml = serde_yaml::to_string(meta).map_err(|e| e.to_string())?;
    Ok(format!("---\n{}---\n\n", yaml))
}

pub(crate) fn compose_document(meta: &CardMeta, body: &str) -> Result<String, String> {
    let header = render_frontmatter(meta)?;
    if body.is_empty() {
        Ok(header.trim_end().to_string())
    } else {
        Ok(format!("{}{}", header, body))
    }
}

const WINDOWS_RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL",
    "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
    "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

pub(crate) fn sanitize_file_stem(title: &str) -> String {
    let mut cleaned = String::with_capacity(title.len());
    for ch in title.trim().chars() {
        match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | '\0' => cleaned.push('_'),
            c if c.is_control() => cleaned.push('_'),
            _ => cleaned.push(ch),
        }
    }

    let cleaned = cleaned.trim().trim_matches('.').to_string();
    if cleaned.is_empty() {
        return "untitled".to_string();
    }
    if WINDOWS_RESERVED.contains(&cleaned.to_uppercase().as_str()) {
        format!("{cleaned}_")
    } else {
        cleaned
    }
}

pub(crate) fn card_file_path(dir: &Path, title: &str) -> PathBuf {
    dir.join(format!("{}.md", sanitize_file_stem(title)))
}

pub(crate) fn category_path(vault_root: &Path, category: &str) -> Result<PathBuf, String> {
    let trimmed = category.trim();
    if trimmed.is_empty() || trimmed == "." || trimmed == "/" || trimmed == "\\" {
        return Ok(PathBuf::from(vault_root));
    }

    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return Err(format!("不支持嵌套分类: {trimmed}"));
    }

    let mut path = PathBuf::from(vault_root);
    path.push(trimmed);

    let abs_path = std::path::absolute(&path).unwrap_or(path.clone());
    let abs_root = std::path::absolute(vault_root).unwrap_or_else(|_| vault_root.to_path_buf());
    if !abs_path.starts_with(&abs_root) {
        return Err("路径越界".to_string());
    }

    Ok(path)
}

pub(crate) fn article_path(
    vault_root: &Path,
    category: &str,
    title: &str,
) -> Result<PathBuf, String> {
    let dir = category_path(vault_root, category)?;
    Ok(card_file_path(&dir, title))
}

pub(crate) fn reserve_card_path(dir: &Path, title: &str) -> (String, PathBuf) {
    use std::fs::OpenOptions;
    let base_title = sanitize_file_stem(title);
    let mut candidate_title = base_title.clone();
    let mut candidate_path = card_file_path(dir, &candidate_title);
    let mut suffix = 2;

    loop {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate_path)
        {
            Ok(_) => return (candidate_title, candidate_path),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                candidate_title = format!("{} {}", base_title, suffix);
                candidate_path = card_file_path(dir, &candidate_title);
                suffix += 1;
            }
            Err(_) => return (candidate_title, candidate_path),
        }
    }
}

pub(crate) fn path_stem(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}
