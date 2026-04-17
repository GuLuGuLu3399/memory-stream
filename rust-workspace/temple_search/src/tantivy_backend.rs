//! Tantivy 搜索后端 — 基于 Tantivy 的 [`SearchBackend`] 实现。

use std::path::Path;
use std::sync::Mutex;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};

use temple_core::error::{ErrorCode, TempleError, TempleResult};

use crate::backend::SearchBackend;
use crate::schema::{temple_schema, TempleFields};
use crate::tokenizer::JiebaTokenizer;
use crate::types::{IndexStats, MatchType, SearchResult};

/// 基于 Tantivy 的搜索后端。
///
/// 封装索引管理、文档写入和搜索查询，实现 [`SearchBackend`] trait。
/// 内部使用 `Mutex<IndexWriter>` 保证线程安全的文档写入。
pub struct TantivyBackend {
    index: Index,
    fields: TempleFields,
    query_parser: QueryParser,
    writer: Mutex<IndexWriter>,
}

fn index_err(action: &str, err: impl std::fmt::Display) -> TempleError {
    TempleError::new(
        ErrorCode::IndexNotReady,
        format!("temple_search::{action} 失败: {err}"),
    )
}

fn query_err(action: &str, err: impl std::fmt::Display) -> TempleError {
    TempleError::new(
        ErrorCode::SearchQueryFailed,
        format!("temple_search::{action} 失败: {err}"),
    )
}

impl TantivyBackend {
    /// 在指定目录下创建或打开索引
    pub fn open(index_dir: &str) -> TempleResult<Self> {
        let schema = temple_schema();
        let fields = TempleFields::new(&schema)?;

        let index_path = Path::new(index_dir);
        std::fs::create_dir_all(index_path)?;

        let index = if index_path.join(".tantivy-meta").exists() {
            Index::open_in_dir(index_path).map_err(|e| index_err("open_index", e))?
        } else {
            Index::create_in_dir(index_path, schema).map_err(|e| index_err("create_index", e))?
        };

        index.tokenizers().register("jieba", JiebaTokenizer);

        let query_parser =
            QueryParser::for_index(&index, vec![fields.title, fields.body, fields.tags]);

        let writer = index
            .writer(50_000_000)
            .map_err(|e| index_err("create_writer", e))?;

        Ok(Self {
            index,
            fields,
            query_parser,
            writer: Mutex::new(writer),
        })
    }

    /// 创建内存索引（测试用）
    pub fn open_in_memory() -> TempleResult<Self> {
        let schema = temple_schema();
        let fields = TempleFields::new(&schema)?;

        let index = Index::create_in_ram(schema);
        index.tokenizers().register("jieba", JiebaTokenizer);

        let query_parser =
            QueryParser::for_index(&index, vec![fields.title, fields.body, fields.tags]);

        let writer = index
            .writer(50_000_000)
            .map_err(|e| index_err("create_writer", e))?;

        Ok(Self {
            index,
            fields,
            query_parser,
            writer: Mutex::new(writer),
        })
    }

    /// 获取底层 Tantivy Index 引用（高级用法）
    pub fn index(&self) -> &Index {
        &self.index
    }

    /// 获取 schema 字段引用（高级用法）
    pub fn fields(&self) -> &TempleFields {
        &self.fields
    }
}

impl SearchBackend for TantivyBackend {
    fn index_document(
        &self,
        path: &str,
        title: &str,
        body: &str,
        tags: &[String],
        wikilinks: &[String],
    ) -> TempleResult<()> {
        let tags_str = tags.join(" ");
        let links_str = wikilinks.join(" ");

        let mut writer = self.writer.lock().map_err(|e| {
            TempleError::new(ErrorCode::IndexNotReady, format!("writer 锁中毒: {e}"))
        })?;

        writer
            .add_document(doc!(
                self.fields.path => path,
                self.fields.title => title,
                self.fields.body => body,
                self.fields.tags => tags_str,
                self.fields.wikilinks => links_str,
            ))
            .map_err(|e| index_err("add_document", e))?;

        writer.commit().map_err(|e| index_err("commit", e))?;
        Ok(())
    }

    fn search(&self, query_str: &str, limit: usize) -> TempleResult<Vec<SearchResult>> {
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .map_err(|e| index_err("create_reader", e))?;

        let searcher = reader.searcher();

        let query = self
            .query_parser
            .parse_query(query_str)
            .map_err(|e| query_err("parse_query", e))?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
            .map_err(|e| query_err("execute_search", e))?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(|e| query_err("read_document", e))?;

            let path = doc
                .get_first(self.fields.path)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let title = doc
                .get_first(self.fields.title)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let body = doc
                .get_first(self.fields.body)
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let snippet = if body.chars().count() > 200 {
                let prefix: String = body.chars().take(200).collect();
                format!("{prefix}...")
            } else {
                body.to_string()
            };

            results.push(SearchResult {
                path,
                title,
                score,
                snippet,
                match_type: MatchType::Body,
            });
        }

        Ok(results)
    }

    fn delete_document(&self, doc_id: &str) -> TempleResult<()> {
        let mut writer = self.writer.lock().map_err(|e| {
            TempleError::new(ErrorCode::IndexNotReady, format!("writer 锁中毒: {e}"))
        })?;

        let term = tantivy::schema::Term::from_field_text(self.fields.path, doc_id);
        writer.delete_term(term);
        writer.commit().map_err(|e| index_err("commit_delete", e))?;
        Ok(())
    }

    fn stats(&self) -> TempleResult<IndexStats> {
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .map_err(|e| index_err("create_reader", e))?;

        Ok(IndexStats {
            total_docs: reader.searcher().num_docs(),
            index_size_bytes: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_and_search_via_trait() -> Result<(), Box<dyn std::error::Error>> {
        let backend = TantivyBackend::open_in_memory()?;

        backend
            .index_document(
                "/vault/rust.md",
                "Rust 编程入门",
                "Rust 是一门系统编程语言，注重安全和性能",
                &["rust".to_string(), "编程".to_string()],
                &[],
            )?;

        backend
            .index_document(
                "/vault/go.md",
                "Go 并发模式",
                "Go 语言有强大的 goroutine 并发模型",
                &["go".to_string()],
                &[],
            )?;

        let results = backend.search("Rust", 10)?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust 编程入门");
        Ok(())
    }

    #[test]
    fn test_chinese_search_via_trait() -> Result<(), Box<dyn std::error::Error>> {
        let backend = TantivyBackend::open_in_memory()?;

        backend
            .index_document(
                "/vault/test.md",
                "知识管理系统",
                "这是一个基于双链的知识管理系统",
                &[],
                &[],
            )?;

        let results = backend.search("知识管理", 10)?;
        assert!(!results.is_empty());
        Ok(())
    }

    #[test]
    fn test_delete_document() -> Result<(), Box<dyn std::error::Error>> {
        let backend = TantivyBackend::open_in_memory()?;

        backend
            .index_document(
                "/vault/rust.md",
                "Rust 入门",
                "Rust 内容",
                &[],
                &[],
            )?;

        // Verify it's there
        let results = backend.search("Rust", 10)?;
        assert_eq!(results.len(), 1);

        // Delete and verify
        backend.delete_document("/vault/rust.md")?;
        let stats = backend.stats()?;
        assert_eq!(stats.total_docs, 0);
        Ok(())
    }

    #[test]
    fn test_trait_object() -> Result<(), Box<dyn std::error::Error>> {
        let backend = TantivyBackend::open_in_memory()?;
        let _dyn_backend: Box<dyn SearchBackend> = Box::new(backend);
        Ok(())
    }
}
