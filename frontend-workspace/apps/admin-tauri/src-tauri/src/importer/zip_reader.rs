use crate::importer::{ImportCard, ImportError, ImportImage};
use gray_matter::{engine::YAML, Matter, Pod};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

pub fn extract_zip_archive(
    path: &Path,
) -> Result<(Vec<ImportCard>, Vec<ImportImage>), ImportError> {
    let file = File::open(path).map_err(|e| ImportError::IoError(e.to_string()))?;
    let mut archive = ZipArchive::new(file).map_err(|e| ImportError::ParseError(e.to_string()))?;

    let mut cards = vec![];
    let mut images = vec![];

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| ImportError::ParseError(e.to_string()))?;

        let file_name = file.name().to_string();

        if file_name.ends_with(".md") {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| ImportError::IoError(e.to_string()))?;

            let matter = Matter::<YAML>::new();
            let parsed = match matter.parse::<Pod>(&content) {
                Ok(p) => p,
                Err(_) => {
                    cards.push(ImportCard {
                        title: Path::new(&file_name)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Untitled")
                            .to_string(),
                        category: None,
                        tags: vec![],
                        body_md: content.trim().to_string(),
                        source_path: file_name.clone(),
                    });
                    continue;
                }
            };

            let mut title = String::new();
            let mut category = None;
            let mut tags = vec![];
            let body_md = parsed.content.trim().to_string();

            if let Some(front_matter) = parsed.data {
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
            }

            if title.is_empty() {
                if let Some(heading) = extract_first_heading(&body_md) {
                    title = heading;
                } else {
                    title = Path::new(&file_name)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Untitled")
                        .to_string();
                }
            }

            cards.push(ImportCard {
                title,
                category,
                tags,
                body_md,
                source_path: file_name.clone(),
            });
        } else if file_name.ends_with(".png")
            || file_name.ends_with(".jpg")
            || file_name.ends_with(".jpeg")
            || file_name.ends_with(".gif")
            || file_name.ends_with(".webp")
        {
            let mut data = vec![];
            file.read_to_end(&mut data)
                .map_err(|e| ImportError::IoError(e.to_string()))?;

            images.push(ImportImage {
                filename: file_name.clone(),
                data,
            });
        }
    }

    Ok((cards, images))
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
