//! # temple_search — 毫秒级全文搜索引擎
//!
//! 基于 Tantivy 的本地全文搜索，支持中文分词 (jieba-rs)，
//! 为知识库提供即时搜索体验。

pub mod engine;
pub mod schema;
pub mod tokenizer;
pub mod types;

pub use engine::SearchEngine;
pub use types::{IndexStats, MatchType, SearchResult};
