//! 搜索引擎后端 trait — 解耦具体实现（Tantivy / SQLite FTS5 / 远端 API）。

use crate::types::{IndexStats, SearchResult};
use temple_core::error::TempleResult;

/// 搜索引擎后端接口。
///
/// 抽象全文索引操作，使上层业务不依赖具体的搜索引擎实现。
/// 当前实现为 [`TantivyBackend`](crate::tantivy_backend::TantivyBackend)。
///
/// # 线程安全
/// 实现者必须满足 `Send + Sync`，允许在 `Arc<dyn SearchBackend>` 中使用。
pub trait SearchBackend: Send + Sync {
    /// 索引一篇文档（自动提交）。
    fn index_document(
        &self,
        path: &str,
        title: &str,
        body: &str,
        tags: &[String],
        wikilinks: &[String],
    ) -> TempleResult<()>;

    /// 执行全文搜索。
    fn search(&self, query: &str, limit: usize) -> TempleResult<Vec<SearchResult>>;

    /// 从索引中删除指定文档。
    fn delete_document(&self, doc_id: &str) -> TempleResult<()>;

    /// 获取索引统计信息。
    fn stats(&self) -> TempleResult<IndexStats>;
}
