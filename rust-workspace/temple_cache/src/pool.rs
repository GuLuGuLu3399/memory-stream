//! DashMap 热数据池 — 高并发内存文档存储

use dashmap::DashMap;
use std::sync::Arc;

use crate::document::{Document, DocumentMeta};

/// 高并发文档内存池
///
/// 使用 DashMap 实现无锁并发读写，所有文档常驻内存。
/// Key = 文件绝对路径，Value = Arc<Document>（clone 仅复制指针）。
pub struct DocumentPool {
    docs: Arc<DashMap<String, Arc<Document>>>,
}

impl DocumentPool {
    /// 创建空的文档池
    pub fn new() -> Self {
        Self {
            docs: Arc::new(DashMap::new()),
        }
    }

    /// 获取单篇文档（Arc clone，零堆分配）
    pub fn get(&self, path: &str) -> Option<Arc<Document>> {
        self.docs.get(path).map(|r| r.value().clone())
    }

    /// 插入或更新文档
    pub fn upsert(&self, doc: Document) {
        self.docs.insert(doc.path.clone(), Arc::new(doc));
    }

    /// 移除文档
    pub fn remove(&self, path: &str) -> Option<Arc<Document>> {
        self.docs.remove(path).map(|(_, v)| v)
    }

    /// 获取所有文档的轻量元数据（列表页使用，零 Document clone）
    pub fn list_metadata(&self) -> Vec<DocumentMeta> {
        self.docs
            .iter()
            .map(|r| DocumentMeta::from(r.value().as_ref()))
            .collect()
    }

    /// 获取池中文档数量
    pub fn len(&self) -> usize {
        self.docs.len()
    }

    /// 池是否为空
    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }

    /// 清空文档池
    pub fn clear(&self) {
        self.docs.clear();
    }

    /// 检查文档是否存在
    pub fn contains(&self, path: &str) -> bool {
        self.docs.contains_key(path)
    }

    /// 按标题模糊搜索（简单的子串匹配，后续由 temple_search 替代）
    pub fn search_by_title(&self, query: &str) -> Vec<DocumentMeta> {
        let query_lower = query.to_lowercase();
        self.docs
            .iter()
            .filter(|r| r.value().title.to_lowercase().contains(&query_lower))
            .map(|r| DocumentMeta::from(r.value().as_ref()))
            .collect()
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
    fn test_upsert_and_get() {
        let pool = DocumentPool::new();
        pool.upsert(test_doc("/vault/test.md", "Test Card"));

        let got = pool.get("/vault/test.md").unwrap();
        assert_eq!(got.title, "Test Card");
    }

    #[test]
    fn test_remove() {
        let pool = DocumentPool::new();
        pool.upsert(test_doc("/vault/a.md", "A"));
        pool.upsert(test_doc("/vault/b.md", "B"));

        let removed = pool.remove("/vault/a.md").unwrap();
        assert_eq!(removed.title, "A");
        assert!(pool.get("/vault/a.md").is_none());
        assert_eq!(pool.len(), 1);
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
    fn test_concurrent_access() {
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
            h.join().unwrap();
        }

        assert_eq!(pool.len(), 10);
    }
}
