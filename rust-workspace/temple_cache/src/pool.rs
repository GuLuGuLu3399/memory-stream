//! Moka LRU 热数据池 — 带容量边界的并发文档存储
//!
//! 使用 moka 并发缓存替代无边界的 DashMap，实现：
//! - **容量上限**：最多缓存 10,000 篇完整文档
//! - **TTI 淘汰**：1 小时未访问的冷数据自动回收
//! - **元数据常驻**：轻量 `DocumentMeta` 不受淘汰影响

use dashmap::DashMap;
use moka::sync::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::document::{Document, DocumentMeta};

/// 默认最大缓存文档数
const DEFAULT_MAX_CAPACITY: usize = 10_000;
/// 默认空闲淘汰时间（1 小时）
const DEFAULT_TTI: Duration = Duration::from_secs(3600);

/// 高并发文档内存池（带 LRU 淘汰）
///
/// 采用双层架构：
/// - **热数据层**（`moka::sync::Cache`）：完整文档，有容量上限和 TTI 淘汰
/// - **目录层**（`DashMap`）：轻量元数据，常驻内存，用于列表和搜索
///
/// 当缓存满时，moka 自动淘汰最近最少使用的文档。
/// `list_metadata()` 和 `search_by_title()` 始终基于完整目录，
/// 不受缓存淘汰影响。
pub struct DocumentPool {
    /// 热文档缓存（LRU 淘汰，容量受限）
    cache: Cache<String, Arc<Document>>,
    /// 轻量元数据目录（常驻，不受淘汰）
    meta: DashMap<String, DocumentMeta>,
}

impl DocumentPool {
    /// 创建带默认配置的文档池（最大 10,000 篇，1 小时 TTI）
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_MAX_CAPACITY)
    }

    /// 创建指定容量上限的文档池
    pub fn with_capacity(max_capacity: usize) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity as u64)
            .time_to_idle(DEFAULT_TTI)
            .build();
        Self {
            cache,
            meta: DashMap::new(),
        }
    }

    /// 获取单篇文档（Arc clone）。如果文档已被 LRU 淘汰则返回 None。
    pub fn get(&self, path: &str) -> Option<Arc<Document>> {
        self.cache.get(path)
    }

    /// 插入或更新文档（同时更新热缓存和元数据目录）
    pub fn upsert(&self, doc: Document) {
        let meta = DocumentMeta::from(&doc);
        let path = doc.path.clone();
        self.cache.insert(path.clone(), Arc::new(doc));
        self.meta.insert(path, meta);
    }

    /// 移除文档（同时从缓存和目录中删除）
    pub fn remove(&self, path: &str) -> Option<Arc<Document>> {
        let old = self.cache.get(path);
        self.cache.invalidate(path);
        self.meta.remove(path);
        old
    }

    /// 获取所有文档的轻量元数据（基于完整目录，不受缓存淘汰影响）
    pub fn list_metadata(&self) -> Vec<DocumentMeta> {
        self.meta.iter().map(|r| r.value().clone()).collect()
    }

    /// 获取已知文档数量（基于元数据目录）
    pub fn len(&self) -> usize {
        self.meta.len()
    }

    /// 池是否为空
    pub fn is_empty(&self) -> bool {
        self.meta.is_empty()
    }

    /// 清空文档池（同时清空缓存和目录）
    pub fn clear(&self) {
        self.cache.invalidate_all();
        self.meta.clear();
    }

    /// 检查文档是否存在于目录中
    pub fn contains(&self, path: &str) -> bool {
        self.meta.contains_key(path)
    }

    /// 按标题模糊搜索（基于元数据目录，不受缓存淘汰影响）
    pub fn search_by_title(&self, query: &str) -> Vec<DocumentMeta> {
        let query_lower = query.to_lowercase();
        self.meta
            .iter()
            .filter(|r| r.value().title.to_lowercase().contains(&query_lower))
            .map(|r| r.value().clone())
            .collect()
    }

    /// 获取热缓存中的文档数（可能小于目录总数，因为部分已被淘汰）
    pub fn cache_entry_count(&self) -> u64 {
        self.cache.entry_count()
    }
}

impl Default for DocumentPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_doc(path: &str, title: &str) -> Document {
        Document {
            path: path.to_string(),
            filename: title.to_string(),
            title: title.to_string(),
            raw_md: format!("# {title}\nContent here"),
            excerpt: "Content here".to_string(),
            html: format!("<h1>{title}</h1><p>Content here</p>"),
            ast_json: "{}".to_string(),
            extracted_links: vec![],
            updated_at: 1000,
        }
    }

    #[test]
    fn test_upsert_and_get() -> Result<(), Box<dyn std::error::Error>> {
        let pool = DocumentPool::new();
        pool.upsert(test_doc("/vault/test.md", "Test Card"));

        let got = pool.get("/vault/test.md");
        assert!(got.is_some());
        assert_eq!(got.ok_or("expected Some")?.title, "Test Card");
        Ok(())
    }

    #[test]
    fn test_remove() -> Result<(), Box<dyn std::error::Error>> {
        let pool = DocumentPool::new();
        pool.upsert(test_doc("/vault/a.md", "A"));
        pool.upsert(test_doc("/vault/b.md", "B"));

        let removed = pool.remove("/vault/a.md");
        assert!(removed.is_some());
        assert_eq!(removed.ok_or("expected Some")?.title, "A");
        assert!(pool.get("/vault/a.md").is_none());
        assert_eq!(pool.len(), 1);
        Ok(())
    }

    #[test]
    fn test_list_metadata() {
        let pool = DocumentPool::new();
        pool.upsert(test_doc("/vault/a.md", "Card A"));
        pool.upsert(test_doc("/vault/b.md", "Card B"));

        let meta = pool.list_metadata();
        assert_eq!(meta.len(), 2);
        assert!(meta.iter().all(|m| !m.title.is_empty()));
    }

    #[test]
    fn test_search_by_title() {
        let pool = DocumentPool::new();
        pool.upsert(test_doc("/vault/rust.md", "Rust Basics"));
        pool.upsert(test_doc("/vault/go.md", "Go Patterns"));
        pool.upsert(test_doc("/vault/ts.md", "TypeScript Tips"));

        let results = pool.search_by_title("rust");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Basics");

        let results = pool.search_by_title("t");
        assert_eq!(results.len(), 3); // "RusT", "PaTTerns", "TypeScripT"
    }

    #[test]
    fn test_concurrent_access() -> Result<(), Box<dyn std::error::Error>> {
        use std::sync::Arc;
        use std::thread;

        let pool = Arc::new(DocumentPool::new());
        let mut handles = vec![];

        for i in 0..10 {
            let p = pool.clone();
            handles.push(thread::spawn(move || {
                p.upsert(test_doc(&format!("/vault/{i}.md"), &format!("Card {i}")));
            }));
        }

        for h in handles {
            h.join().map_err(|e| format!("thread panicked: {:?}", e))?;
        }

        assert_eq!(pool.len(), 10);
        Ok(())
    }

    #[test]
    fn test_metadata_survives_cache_eviction() {
        // 创建容量仅为 2 的池
        let pool = DocumentPool::with_capacity(2);
        pool.upsert(test_doc("/vault/a.md", "Card A"));
        pool.upsert(test_doc("/vault/b.md", "Card B"));
        pool.upsert(test_doc("/vault/c.md", "Card C"));

        // 元数据目录应包含全部 3 篇
        assert_eq!(pool.len(), 3);
        let meta = pool.list_metadata();
        assert_eq!(meta.len(), 3);
    }

    #[test]
    fn test_contains_after_remove() {
        let pool = DocumentPool::new();
        pool.upsert(test_doc("/vault/a.md", "A"));
        assert!(pool.contains("/vault/a.md"));

        pool.remove("/vault/a.md");
        assert!(!pool.contains("/vault/a.md"));
    }
}
