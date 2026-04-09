use crate::importer::{ImportCard, ImportError};
use encoding_rs::UTF_8;
use gray_matter::{engine::YAML, Matter, ParsedEntity};
use std::fs;
use std::path::Path;

pub fn parse_markdown_file(path: &Path) -> Result<ImportCard, ImportError> {
    let bytes = fs::read(path).map_err(|e| ImportError::IoError(e.to_string()))?;

    let (content, _encoding, _had_bom) = UTF_8.decode(&bytes);
    let content = content.into_owned();

    let matter = Matter::<YAML>::new();
    let parsed: Option<ParsedEntity> = matter.parse(&content).ok();

    if let Some(parsed) = parsed {
        if let Some(front_matter) = parsed.data {
            let (title, category, tags) = extract_frontmatter(&front_matter, &parsed.content, path);

            let body_md = parsed.content.trim().to_string();

            return Ok(ImportCard {
                title,
                category,
                tags,
                body_md,
                source_path: path.display().to_string(),
            });
        }
    }

    let title = extract_first_heading(&content).unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string()
    });

    Ok(ImportCard {
        title,
        category: None,
        tags: vec![],
        body_md: content.trim().to_string(),
        source_path: path.display().to_string(),
    })
}

fn extract_frontmatter(
    front_matter: &gray_matter::Pod,
    content: &str,
    path: &Path,
) -> (String, Option<String>, Vec<String>) {
    let mut title = String::new();
    let mut category = None;
    let mut tags = vec![];

    if let Ok(hash) = front_matter.as_hashmap() {
        if let Some(t) = hash.get("title") {
            if let Ok(s) = t.as_string() {
                title = s;
            }
        }

        if let Some(c) = hash.get("category") {
            if let Ok(s) = c.as_string() {
                category = Some(s);
            }
        }

        if let Some(tag_list) = hash.get("tags") {
            if let Ok(vec) = tag_list.as_vec() {
                for tag in vec {
                    if let Ok(tag_str) = tag.as_string() {
                        tags.push(tag_str);
                    }
                }
            }
        }
    }

    if title.is_empty() {
        if let Some(heading) = extract_first_heading(content) {
            title = heading;
        } else {
            title = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string();
        }
    }

    (title, category, tags)
}

fn extract_first_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            return Some(stripped.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_parse_valid_frontmatter() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.md");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "---").unwrap();
        writeln!(file, "title: My Card").unwrap();
        writeln!(file, "category: Tech").unwrap();
        writeln!(file, "tags:").unwrap();
        writeln!(file, "  - rust").unwrap();
        writeln!(file, "  - tauri").unwrap();
        writeln!(file, "---").unwrap();
        writeln!(file, "# My Card").unwrap();
        writeln!(file, "Content here.").unwrap();

        let result = parse_markdown_file(&file_path).unwrap();
        assert_eq!(result.title, "My Card");
        assert_eq!(result.category, Some("Tech".to_string()));
        assert_eq!(result.tags, vec!["rust", "tauri"]);
        assert!(result.body_md.contains("Content here."));
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.md");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "# No Frontmatter").unwrap();
        writeln!(file, "Just content.").unwrap();

        let result = parse_markdown_file(&file_path).unwrap();
        assert_eq!(result.title, "No Frontmatter");
        assert_eq!(result.category, None);
        assert_eq!(result.tags.len(), 0);
    }

    #[test]
    fn test_utf8_bom() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("bom.md");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();
        writeln!(file, "# BOM Card").unwrap();

        let result = parse_markdown_file(&file_path).unwrap();
        assert_eq!(result.title, "BOM Card");
    }

    #[test]
    fn test_malformed_yaml_fallback() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("bad.md");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "---").unwrap();
        writeln!(file, "invalid: {{{{}}").unwrap();
        writeln!(file, "---").unwrap();
        writeln!(file, "# Fallback Title").unwrap();

        let result = parse_markdown_file(&file_path).unwrap();
        assert_eq!(result.title, "Fallback Title");
    }

    #[test]
    fn test_empty_file_uses_filename() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("MyCard.md");
        let _file = File::create(&file_path).unwrap();

        let result = parse_markdown_file(&file_path).unwrap();
        assert_eq!(result.title, "MyCard");
    }
}
