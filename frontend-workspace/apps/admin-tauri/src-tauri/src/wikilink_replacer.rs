//! # Wikilink Replacer Module
//!
//! AST-aware wikilink replacement for Markdown files.
//! Scans vault for wikilinks and replaces them while skipping code blocks.

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use ts_rs::TS;

// ============================================================================
// Core Data Types
// ============================================================================

/// Result of previewing a merge operation
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export_to = ".")]
pub struct MergePreview {
    /// Number of files that will be modified
    pub files_to_modify: usize,
    /// Total number of wikilinks to be replaced
    pub total_wikilinks: usize,
    /// Details of each affected file
    pub affected_files: Vec<FileImpact>,
}

/// Impact on a single file
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export_to = ".")]
pub struct FileImpact {
    /// File path
    pub path: String,
    /// Number of wikilinks in this file
    pub link_count: usize,
}

// ============================================================================
// Scanner Functions
// ============================================================================

/// Scan a vault directory for all .md files containing a specific wikilink.
///
/// # Arguments
/// * `vault_path` - Root directory of the vault
/// * `victim_title` - The wikilink title to search for (without brackets)
///
/// # Returns
/// List of file paths that contain `[[victim_title]]`
pub fn scan_vault_for_wikilinks(vault_path: &Path, victim_title: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let wikilink_pattern = format!("[[{}]]", victim_title);

    if let Ok(entries) = fs::read_dir(vault_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                result.extend(scan_vault_for_wikilinks(&path, victim_title));
            } else if path.extension().is_some_and(|ext| ext == "md") {
                // Check if this .md file contains the wikilink
                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains(&wikilink_pattern) {
                        result.push(path);
                    }
                }
            }
        }
    }

    result
}

// ============================================================================
// AST-Aware Code Block Detection
// ============================================================================

/// Represents a range in the source text that is inside a code block
#[derive(Debug, Clone, Copy)]
struct CodeRange {
    start: usize,
    end: usize,
}

/// Extract byte ranges of all code blocks and inline code from markdown.
///
/// Uses pulldown-cmark parser to track CodeBlock and Code events.
/// Returns a list of (start, end) byte ranges that are inside code.
fn extract_code_ranges(content: &str) -> Vec<CodeRange> {
    let mut code_ranges = Vec::new();
    let mut code_block_start: Option<usize> = None;

    let parser = Parser::new_ext(content, Options::empty());

    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) => {
                code_block_start = Some(range.start);
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(start) = code_block_start.take() {
                    code_ranges.push(CodeRange {
                        start,
                        end: range.end,
                    });
                }
            }
            Event::Code(_) => {
                code_ranges.push(CodeRange {
                    start: range.start,
                    end: range.end,
                });
            }
            _ => {}
        }
    }

    code_ranges
}

/// Check if a byte position is inside any of the code ranges
fn is_in_code_range(pos: usize, code_ranges: &[CodeRange]) -> bool {
    code_ranges
        .iter()
        .any(|range| pos >= range.start && pos < range.end)
}

/// Count wikilinks outside code blocks in a markdown string.
///
/// # Arguments
/// * `content` - Markdown content
/// * `title` - Wikilink title to count (without brackets)
///
/// # Returns
/// Number of occurrences of `[[title]]` outside code blocks
pub fn count_wikilinks_outside_codeblocks(content: &str, title: &str) -> usize {
    let code_ranges = extract_code_ranges(content);
    let wikilink = format!("[[{}]]", title);
    let mut count = 0;
    let mut search_start = 0;

    while let Some(pos) = content[search_start..].find(&wikilink) {
        let absolute_pos = search_start + pos;

        if !is_in_code_range(absolute_pos, &code_ranges) {
            count += 1;
        }

        search_start = absolute_pos + wikilink.len();
    }

    count
}

// ============================================================================
// Replacement Functions
// ============================================================================

/// Replace wikilinks in memory while skipping code blocks.
///
/// # Arguments
/// * `files` - HashMap of file paths to their content (modified in place)
/// * `victim_title` - The wikilink title to replace (without brackets)
/// * `survivor_title` - The replacement title (without brackets)
///
/// # Returns
/// Total count of replacements made across all files
///
/// # Exact Matching
/// Only replaces `[[victim_title]]` exactly - does NOT match:
/// - `[[victim_title extended]]`
/// - `[[prefix victim_title]]`
#[allow(dead_code)]
pub fn replace_wikilinks_in_memory(
    files: &mut HashMap<PathBuf, String>,
    victim_title: &str,
    survivor_title: &str,
) -> Result<usize, String> {
    let old_link = format!("[[{}]]", victim_title);
    let new_link = format!("[[{}]]", survivor_title);
    let mut total_replaced = 0;

    for content in files.values_mut() {
        let code_ranges = extract_code_ranges(content);
        let mut new_content = String::with_capacity(content.len());
        let mut last_end = 0;
        let mut search_start = 0;
        let mut replaced_in_file = 0;

        while let Some(pos) = content[search_start..].find(&old_link) {
            let absolute_pos = search_start + pos;

            if is_in_code_range(absolute_pos, &code_ranges) {
                // Inside code block - don't replace
                search_start = absolute_pos + old_link.len();
            } else {
                // Outside code block - replace
                new_content.push_str(&content[last_end..absolute_pos]);
                new_content.push_str(&new_link);
                last_end = absolute_pos + old_link.len();
                search_start = last_end;
                replaced_in_file += 1;
            }
        }

        // Append remaining content
        if replaced_in_file > 0 {
            new_content.push_str(&content[last_end..]);
            *content = new_content;
        }

        total_replaced += replaced_in_file;
    }

    Ok(total_replaced)
}

// ============================================================================
// Tauri Command
// ============================================================================

/// Preview the impact of merging wikilinks.
///
/// Scans all .md files in the vault for wikilinks matching any of the victim titles,
/// counting only those outside code blocks.
///
/// # Arguments
/// * `vault_path` - Root directory of the vault
/// * `victim_titles` - List of wikilink titles to search for (without brackets)
///
/// # Returns
/// MergePreview with file count, link count, and file details
#[tauri::command]
pub fn preview_merge_impact(
    state: tauri::State<'_, std::sync::Mutex<crate::AppState>>,
    victim_titles: Vec<String>,
) -> Result<MergePreview, String> {
    let vault_path = match state
        .lock()
        .map_err(|e| format!("lock failed: {}", e))?
        .watcher
        .as_ref()
        .map(|w| w.watch_dir().clone())
    {
        Some(path) => path,
        None => {
            return Ok(MergePreview {
                files_to_modify: 0,
                total_wikilinks: 0,
                affected_files: vec![],
            });
        }
    };

    let vault = Path::new(&vault_path);
    if !vault.exists() {
        return Ok(MergePreview {
            files_to_modify: 0,
            total_wikilinks: 0,
            affected_files: vec![],
        });
    }

    let mut affected_files_map: HashMap<String, usize> = HashMap::new();

    for victim in &victim_titles {
        let files = scan_vault_for_wikilinks(vault, victim);
        for file in files {
            if let Ok(content) = fs::read_to_string(&file) {
                let count = count_wikilinks_outside_codeblocks(&content, victim);
                if count > 0 {
                    let path_str = file.to_string_lossy().to_string();
                    *affected_files_map.entry(path_str).or_insert(0) += count;
                }
            }
        }
    }

    let affected_files: Vec<FileImpact> = affected_files_map
        .into_iter()
        .map(|(path, link_count)| FileImpact { path, link_count })
        .collect();

    let total_links = affected_files.iter().map(|f| f.link_count).sum();
    let total_files = affected_files.len();

    Ok(MergePreview {
        files_to_modify: total_files,
        total_wikilinks: total_links,
        affected_files,
    })
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_exact_title_replacement() {
        let mut files = HashMap::new();
        files.insert(
            PathBuf::from("test.md"),
            "This is [[Alpha]] and [[Alpha Beta]] and [[Alpha]].".to_string(),
        );

        let count = replace_wikilinks_in_memory(&mut files, "Alpha", "Gamma").unwrap();

        assert_eq!(count, 2);
        let content = files.get(&PathBuf::from("test.md")).unwrap();
        assert_eq!(
            content,
            "This is [[Gamma]] and [[Alpha Beta]] and [[Gamma]]."
        );
    }

    #[test]
    fn test_code_block_skip() {
        let mut files = HashMap::new();
        files.insert(
            PathBuf::from("test.md"),
            "Text [[Victim]] text\n```\n[[Victim]] should not change\n```\n[[Victim]] after"
                .to_string(),
        );

        let count = replace_wikilinks_in_memory(&mut files, "Victim", "Survivor").unwrap();

        assert_eq!(count, 2);
        let content = files.get(&PathBuf::from("test.md")).unwrap();
        assert!(content.contains("[[Survivor]] text"));
        assert!(content.contains("[[Victim]] should not change"));
        assert!(content.contains("[[Survivor]] after"));
    }

    #[test]
    fn test_inline_code_skip() {
        let mut files = HashMap::new();
        files.insert(
            PathBuf::from("test.md"),
            "Text [[Victim]] text `[[Victim]] inline` [[Victim]] end".to_string(),
        );

        let count = replace_wikilinks_in_memory(&mut files, "Victim", "Survivor").unwrap();

        assert_eq!(count, 2);
        let content = files.get(&PathBuf::from("test.md")).unwrap();
        assert!(content.starts_with("Text [[Survivor]] text"));
        assert!(content.contains("[[Victim]] inline"));
        assert!(content.ends_with("[[Survivor]] end"));
    }

    #[test]
    fn test_unicode_title() {
        let mut files = HashMap::new();
        files.insert(
            PathBuf::from("test.md"),
            "This is [[中文节点]] and [[中文节点扩展]].".to_string(),
        );

        let count = replace_wikilinks_in_memory(&mut files, "中文节点", "新节点").unwrap();

        assert_eq!(count, 1);
        let content = files.get(&PathBuf::from("test.md")).unwrap();
        assert_eq!(content, "This is [[新节点]] and [[中文节点扩展]].");
    }

    #[test]
    fn test_multiple_occurrences() {
        let mut files = HashMap::new();
        files.insert(
            PathBuf::from("test.md"),
            "[[Victim]] first [[Victim]] second [[Victim]] third".to_string(),
        );

        let count = replace_wikilinks_in_memory(&mut files, "Victim", "Survivor").unwrap();

        assert_eq!(count, 3);
        let content = files.get(&PathBuf::from("test.md")).unwrap();
        assert_eq!(
            content,
            "[[Survivor]] first [[Survivor]] second [[Survivor]] third"
        );
    }

    #[test]
    fn test_count_wikilinks_outside_codeblocks() {
        let content = r#"
Text [[Alpha]] here
```
[[Alpha]] in code block
```
`[[Alpha]] inline` and [[Alpha]] outside
"#;

        let count = count_wikilinks_outside_codeblocks(content, "Alpha");
        // Should count only the first and last occurrence (2 total)
        assert_eq!(count, 2);
    }

    #[test]
    fn test_scan_vault_for_wikilinks() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Create test files
        let file1 = vault_path.join("doc1.md");
        let mut f1 = fs::File::create(&file1).unwrap();
        f1.write_all(b"This has [[Target]] wikilink").unwrap();

        let file2 = vault_path.join("doc2.md");
        let mut f2 = fs::File::create(&file2).unwrap();
        f2.write_all(b"This has no wikilink").unwrap();

        let subdir = vault_path.join("subdir");
        fs::create_dir(&subdir).unwrap();
        let file3 = subdir.join("doc3.md");
        let mut f3 = fs::File::create(&file3).unwrap();
        f3.write_all(b"Another [[Target]] here").unwrap();

        // Scan for wikilinks
        let results = scan_vault_for_wikilinks(vault_path, "Target");

        assert_eq!(results.len(), 2);
        assert!(results.contains(&file1));
        assert!(results.contains(&file3));
        assert!(!results.contains(&file2));
    }

    #[test]
    fn test_no_partial_match() {
        let mut files = HashMap::new();
        files.insert(
            PathBuf::from("test.md"),
            "[[Victim Extended]] [[Prefix Victim]] [[Victim]]".to_string(),
        );

        let count = replace_wikilinks_in_memory(&mut files, "Victim", "Survivor").unwrap();

        assert_eq!(count, 1);
        let content = files.get(&PathBuf::from("test.md")).unwrap();
        assert_eq!(
            content,
            "[[Victim Extended]] [[Prefix Victim]] [[Survivor]]"
        );
    }

    #[test]
    fn test_empty_files() {
        let mut files: HashMap<PathBuf, String> = HashMap::new();

        let count = replace_wikilinks_in_memory(&mut files, "Victim", "Survivor").unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_no_wikilinks() {
        let mut files = HashMap::new();
        files.insert(
            PathBuf::from("test.md"),
            "Just some text without wikilinks".to_string(),
        );

        let count = replace_wikilinks_in_memory(&mut files, "Victim", "Survivor").unwrap();

        assert_eq!(count, 0);
        let content = files.get(&PathBuf::from("test.md")).unwrap();
        assert_eq!(content, "Just some text without wikilinks");
    }
}
