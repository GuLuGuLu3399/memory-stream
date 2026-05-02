//! 原子文件操作 — 绝不损坏数据

use std::path::Path;

use crate::error::IoResult;

/// 原子写入：先写临时文件，再 rename 覆盖。
///
/// `fs::rename` 在同一文件系统上是操作系统级别的原子操作，
/// 即使在写入过程中断电，原文件也毫发无损。
pub fn write_atomic(target: &Path, content: &str) -> IoResult<()> {
    let ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp = target.with_extension(format!("tmp-{ns}"));
    std::fs::write(&tmp, content).map_err(|e| {
        crate::error::IoError::AtomicWriteFailed(format!("{}: {e}", target.display()))
    })?;
    std::fs::rename(&tmp, target).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        crate::error::IoError::AtomicWriteFailed(format!("{}: rename failed: {e}", target.display()))
    })?;
    Ok(())
}

/// 确保目录存在，不存在则递归创建。
pub fn ensure_dir(path: &Path) -> IoResult<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_atomic_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        write_atomic(&path, "hello world").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello world");
    }

    #[test]
    fn test_write_atomic_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        write_atomic(&path, "v1").unwrap();
        write_atomic(&path, "v2").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "v2");
    }

    #[test]
    fn test_write_atomic_no_tmp_leftover() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        write_atomic(&path, "content").unwrap();
        // No tmp-* files should remain
        for entry in std::fs::read_dir(dir.path()).unwrap() {
            let name = entry.unwrap().file_name();
            assert!(!name.to_string_lossy().starts_with("test.tmp-"));
        }
    }

    #[test]
    fn test_ensure_dir_creates_nested() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a/b/c");
        ensure_dir(&nested).unwrap();
        assert!(nested.exists());
    }

    #[test]
    fn test_ensure_dir_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub");
        ensure_dir(&path).unwrap();
        ensure_dir(&path).unwrap();
        assert!(path.exists());
    }
}
