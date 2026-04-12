//! # temple_cache — 零 I/O 内存热数据池
//!
//! 提供 DashMap 高并发文档池，启动时通过 Rayon 并行预热，
//! 运行时通过 FS Watcher 增量更新，所有读取直接命中内存。

pub mod document;
pub mod pool;
pub mod preheat;
pub mod watcher;

pub use document::{Document, DocumentMeta};
pub use pool::DocumentPool;
pub use preheat::{preheat_vault, PreheatStats};
pub use watcher::{ChangeNotification, VaultWatcher};
