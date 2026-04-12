//! FS Watcher — 文件变更 → 内存池增量同步
//!
//! 监听 Vault 目录的文件变更，实时更新 DocumentPool。
//! 通过回调通知上层（Tauri Event 层），不依赖 Tauri 本身。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::document::DocumentMeta;
use crate::pool::DocumentPool;
use crate::preheat::parse_single_file;
use temple_core::error::{ErrorCode, TempleError, TempleResult};

/// 文件变更通知（通过回调发送给上层）
#[derive(Debug, Clone)]
pub enum ChangeNotification {
    /// 新文件被解析并加入池
    Created(DocumentMeta),
    /// 已有文件被重新解析
    Updated(DocumentMeta),
    /// 文件被删除，从池中移除
    Removed { path: String },
}

/// 文件系统监听器 — 桥接 notify → DocumentPool
pub struct VaultWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<Result<Event, notify::Error>>,
    watch_dir: PathBuf,
}

impl VaultWatcher {
    /// 创建监听器（不启动）
    pub fn new(watch_dir: &str) -> TempleResult<Self> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .map_err(|e| TempleError::new(ErrorCode::InternalError, format!("创建 FS Watcher 失败: {e}")))?;

        let dir = PathBuf::from(watch_dir);
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }

        Ok(Self {
            _watcher: watcher,
            rx,
            watch_dir: dir,
        })
    }

    /// 启动监听
    pub fn start(&mut self) -> TempleResult<()> {
        self._watcher
            .watch(&self.watch_dir, RecursiveMode::Recursive)
            .map_err(|e| {
                TempleError::new(ErrorCode::InternalError, format!("启动 FS Watcher 失败: {e}"))
            })
    }

    /// 处理所有挂起的文件变更，更新内存池
    ///
    /// 返回变更通知列表（上层可用于推送到前端）。
    /// 同路径去重：同一次 poll 中相同路径只保留最后一个事件。
    pub fn poll_and_sync(&self, pool: &DocumentPool) -> Vec<ChangeNotification> {
        let mut dedup: HashMap<String, FileChangeKind> = HashMap::new();

        // 收集所有挂起事件，去重
        while let Ok(result) = self.rx.try_recv() {
            match result {
                Ok(event) => {
                    if !Self::is_markdown_event(&event) {
                        continue;
                    }
                    let kind = match event.kind {
                        EventKind::Create(_) => FileChangeKind::Create,
                        EventKind::Modify(_) => FileChangeKind::Modify,
                        EventKind::Remove(_) => FileChangeKind::Remove,
                        _ => continue,
                    };
                    for path in &event.paths {
                        if Self::is_markdown_file(path) {
                            dedup.insert(path.to_string_lossy().to_string(), kind);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("FS Watcher 错误: {:?}", e);
                }
            }
        }

        // 按去重后的事件更新内存池
        let mut notifications = Vec::new();
        for (path_str, kind) in dedup {
            let path = PathBuf::from(&path_str);
            match kind {
                FileChangeKind::Create | FileChangeKind::Modify => {
                    match parse_single_file(&path) {
                        Ok(doc) => {
                            let meta = DocumentMeta::from(&doc);
                            pool.upsert(doc);
                            notifications.push(match kind {
                                FileChangeKind::Create => ChangeNotification::Created(meta),
                                _ => ChangeNotification::Updated(meta),
                            });
                        }
                        Err(e) => {
                            eprintln!("文件变更解析失败 {}: {e}", path_str);
                        }
                    }
                }
                FileChangeKind::Remove => {
                    if let Some(_doc) = pool.remove(&path_str) {
                        notifications.push(ChangeNotification::Removed { path: path_str });
                    }
                }
            }
        }

        notifications
    }

    /// 获取监听目录
    #[allow(dead_code)]
    pub fn watch_dir(&self) -> &Path {
        &self.watch_dir
    }

    fn is_markdown_event(event: &Event) -> bool {
        event.paths.iter().any(|p| Self::is_markdown_file(p))
    }

    fn is_markdown_file(path: &Path) -> bool {
        path.extension()
            .map(|ext| ext == "md" || ext == "markdown")
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FileChangeKind {
    Create,
    Modify,
    Remove,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_is_markdown_file() {
        assert!(VaultWatcher::is_markdown_file(Path::new("test.md")));
        assert!(VaultWatcher::is_markdown_file(Path::new("notes/readme.markdown")));
        assert!(!VaultWatcher::is_markdown_file(Path::new("test.txt")));
        assert!(!VaultWatcher::is_markdown_file(Path::new("image.png")));
    }

    #[test]
    fn test_watcher_creation() {
        let tmp_dir = std::env::temp_dir().join("temple_cache_test_watcher");
        let _ = fs::create_dir_all(&tmp_dir);

        let result = VaultWatcher::new(tmp_dir.to_str().unwrap());
        assert!(result.is_ok());

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
