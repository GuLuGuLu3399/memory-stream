//! # temple_cache — 零 I/O 内存热数据池
//!
//! 提供带 LRU 淘汰策略的并发文档池（基于 moka），
//! 启动时通过 Rayon 并行预热，运行时通过 FS Watcher 增量更新。

pub mod document;
pub mod pool;

#[cfg(feature = "native")]
pub mod preheat;
#[cfg(feature = "native")]
pub mod watcher;

pub use document::{Document, DocumentMeta};
pub use pool::DocumentPool;

#[cfg(feature = "native")]
pub use preheat::{preheat_vault, PreheatStats};
#[cfg(feature = "native")]
pub use watcher::{ChangeNotification, VaultWatcher};
