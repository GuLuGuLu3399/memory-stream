//! 元数据与索引管理 — 提供 SQLite 本地元数据库的抽象层。
//!
//! 本模块是 Memory Stream 的本地存储引擎，负责管理卡片索引、关系图谱、
//! 全文搜索和资产引用等元数据。
//!
//! # 核心功能
//!
//! - **卡片索引** (`CardIndex`)：维护 UUID 到文件路径的映射，追踪文件哈希和同步状态
//! - **关系图谱** (`RelationRecord`)：存储卡片间的连接关系（链接、主干等）
//! - **全文搜索** (`FtsHit`)：基于 FTS5 的标题/内容全文索引
//! - **资产引用** (`AssetRef`)：追踪图片等资源的引用计数，支持垃圾回收
//! - **同步状态** (`SyncStatus`)：管理卡片的云端同步状态
//!
//! # 数据库设计
//!
//! - 使用 SQLite 作为本地存储引擎
//! - 三个核心表：`card_index`、`relation_index`、`card_fts`
//! - 事务支持，保证关系更新的原子性
//!
//! # 线程安全
//!
//! `MetaDb` 支持多线程并发访问（通过 `rusqlite` 的连接池）

mod db;
mod error;
mod fts;
mod repo;
mod types;

pub use error::{MetaDbError, MetaResult};
pub use db::MetaDb;
pub use types::{AssetRef, CardIndex, FtsHit, RelationRecord, SyncStatus};
