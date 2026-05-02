//! 极速扫盘 — 带"防火墙"的 Vault 目录扫描

use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use crate::error::{IoError, IoResult};

/// 隐藏文件/目录检测：忽略所有以 `.` 开头的条目。
///
/// 碰见 `.bunker`、`.git`、`.obsidian` 直接跳过整个子树，
/// 避免把 SQLite 数据库或版本控制文件当笔记读。
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

/// 临时目录检测：忽略扫描过程中产生的工作目录。
fn is_temp_dir(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| matches!(s, "tmp" | "temp" | "bunlk" | ".bunlk" | ".bulk"))
        .unwrap_or(false)
}

/// 冲突副本检测：同步冲突时保存的服务器版本文件，不应作为普通卡片显示。
fn is_conflict_copy(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.contains("云端冲突副本") || s.contains(".conflict."))
        .unwrap_or(false)
}

/// 扫描 Vault 目录下所有 `.md` 文件。
///
/// 自动过滤隐藏目录（`.bunker`、`.git`、`.obsidian`），
/// 返回相对于 `vault_root` 的路径列表。
pub fn scan_markdown_files(vault_root: &Path) -> IoResult<Vec<PathBuf>> {
    if !vault_root.exists() {
        return Err(IoError::VaultNotFound(vault_root.display().to_string()));
    }

    Ok(WalkDir::new(vault_root)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || (!is_hidden(e) && !is_temp_dir(e)))
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .filter(|e| !is_conflict_copy(e))
        .map(|e| e.path().to_path_buf())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vault(structure: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for path in structure {
            let full = dir.path().join(path);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(&full, "content").unwrap();
        }
        dir
    }

    #[test]
    fn test_scan_finds_md_files() {
        let vault = make_vault(&["a.md", "sub/b.md", "sub/c.md"]);
        let files = scan_markdown_files(vault.path()).unwrap();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_scan_skips_hidden_dirs() {
        let vault = make_vault(&[
            "good.md",
            ".bunker/index.db",
            ".bunker/notes.md",
            ".git/config",
            ".obsidian/workspace",
        ]);
        let files = scan_markdown_files(vault.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("good.md"));
    }

    #[test]
    fn test_scan_skips_temp_dirs() {
        let vault = make_vault(&[
            "good.md",
            "tmp/draft.md",
            "temp/working.md",
            "bunlk/cache.md",
            ".bunlk/scratch.md",
            ".bulk/scratch.md",
        ]);
        let files = scan_markdown_files(vault.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("good.md"));
    }

    #[test]
    fn test_scan_skips_non_md() {
        let vault = make_vault(&["note.md", "image.png", "data.json"]);
        let files = scan_markdown_files(vault.path()).unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_scan_skips_conflict_copies() {
        let vault = make_vault(&[
            "good.md",
            "card(云端冲突副本 1700000000).md",
            "sub/note(云端冲突副本 1700000001).md",
            "note.conflict.1700000000.md",
        ]);
        let files = scan_markdown_files(vault.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("good.md"));
    }

    #[test]
    fn test_scan_nonexistent_dir() {
        let result = scan_markdown_files(Path::new("/no/such/dir"));
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_empty_dir() {
        let vault = tempfile::tempdir().unwrap();
        let files = scan_markdown_files(vault.path()).unwrap();
        assert!(files.is_empty());
    }
}
