/**
 * 🌟 文件系统监听器 — Markdown 变更自动同步
 *
 * 监听本地 Markdown 文件目录，一旦文件变动：
 * 1. 解析元数据（frontmatter）
 * 2. 通过 HTTP 推送至 Go 服务端
 * 3. 触发 LayoutManager 的增量位置计算
 */
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use ts_rs::TS;

/// 文件变更事件（传递给前端）
#[derive(Debug, Serialize, Clone, TS)]
#[ts(export_to = ".")]
pub struct FileChangeEvent {
    pub path: String,
    pub kind: String,
    pub timestamp: String,
}

/// Markdown 文件监听器
pub struct MarkdownWatcher {
    watcher: RecommendedWatcher,
    rx: Receiver<Result<Event, notify::Error>>,
    watch_dir: PathBuf,
}

impl MarkdownWatcher {
    /// 创建新的监听器
    pub fn new(watch_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )?;

        let dir = PathBuf::from(watch_dir);
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }

        Ok(Self {
            watcher,
            rx,
            watch_dir: dir,
        })
    }

    /// 启动监听
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.watcher
            .watch(&self.watch_dir, RecursiveMode::Recursive)?;
        Ok(())
    }

    /// 停止监听
    #[allow(dead_code)]
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.watcher.unwatch(&self.watch_dir)?;
        Ok(())
    }

    /// 非阻塞轮询变更事件（同路径去重：同一次 poll 中相同路径只保留最后一个事件）
    pub fn poll_changes(&self) -> Vec<FileChangeEvent> {
        let mut dedup: HashMap<String, FileChangeEvent> = HashMap::new();

        while let Ok(result) = self.rx.try_recv() {
            match result {
                Ok(event) => {
                    // 只处理 Markdown 文件的创建/修改/删除
                    if !Self::is_markdown_event(&event) {
                        continue;
                    }

                    let kind = match event.kind {
                        EventKind::Create(_) => "created",
                        EventKind::Modify(_) => "modified",
                        EventKind::Remove(_) => "removed",
                        _ => continue,
                    };

                    for path in &event.paths {
                        if Self::is_markdown_file(path) {
                            let key = path.to_string_lossy().to_string();
                            dedup.insert(
                                key,
                                FileChangeEvent {
                                    path: path.to_string_lossy().to_string(),
                                    kind: kind.to_string(),
                                    timestamp: chrono_now(),
                                },
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ 文件监听错误: {:?}", e);
                }
            }
        }

        dedup.into_values().collect()
    }

    /// 检查事件是否涉及 Markdown 文件
    fn is_markdown_event(event: &Event) -> bool {
        event.paths.iter().any(Self::is_markdown_file)
    }

    /// 检查路径是否为 Markdown 文件
    #[allow(clippy::ptr_arg)]
    fn is_markdown_file(path: &PathBuf) -> bool {
        path.extension()
            .map(|ext| ext == "md" || ext == "markdown")
            .unwrap_or(false)
    }

    /// 获取监听目录
    #[allow(dead_code)]
    pub fn watch_dir(&self) -> &PathBuf {
        &self.watch_dir
    }
}

/// 简易时间戳（避免引入 chrono 依赖）
fn chrono_now() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}", duration.as_secs(), duration.subsec_millis())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_is_markdown_file() {
        assert!(MarkdownWatcher::is_markdown_file(&PathBuf::from("test.md")));
        assert!(MarkdownWatcher::is_markdown_file(&PathBuf::from(
            "notes/readme.markdown"
        )));
        assert!(!MarkdownWatcher::is_markdown_file(&PathBuf::from(
            "test.txt"
        )));
        assert!(!MarkdownWatcher::is_markdown_file(&PathBuf::from(
            "image.png"
        )));
    }

    #[test]
    fn test_watcher_creation() {
        let tmp_dir = std::env::temp_dir().join("memory-stream-test-watcher");
        let _ = fs::create_dir_all(&tmp_dir);

        let result = MarkdownWatcher::new(tmp_dir.to_str().unwrap());
        assert!(result.is_ok());

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
