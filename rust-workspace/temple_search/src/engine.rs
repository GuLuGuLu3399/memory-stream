//! 搜索引擎核心 — 索引管理 + 搜索 + Snippet 高亮

use std::path::Path;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};

use temple_core::error::{ErrorCode, TempleError, TempleResult};

use crate::schema::{temple_schema, TempleFields};
use crate::tokenizer::JiebaTokenizer;
use crate::types::{IndexStats, MatchType, SearchResult};

/// 搜索引擎实例
pub struct SearchEngine {
    index: Index,
    fields: TempleFields,
    query_parser: QueryParser,
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

impl SearchEngine {
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

        // 注册中文分词器
        index.tokenizers().register("jieba", JiebaTokenizer);

        let query_parser =
            QueryParser::for_index(&index, vec![fields.title, fields.body, fields.tags]);

        Ok(Self {
            index,
            fields,
            query_parser,
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

        Ok(Self {
            index,
            fields,
            query_parser,
        })
    }

    /// 获取 index writer（用于批量索引）
    pub fn writer(&self) -> TempleResult<IndexWriter> {
        self.index
            .writer(50_000_000) // 50MB heap
            .map_err(|e| index_err("create_writer", e))
    }

    /// 获取 schema 字段引用（用于批量索引文档）
    pub fn fields(&self) -> &TempleFields {
        &self.fields
    }

    /// 添加文档到索引
    pub fn add_document(
        writer: &mut IndexWriter,
        fields: &TempleFields,
        path: &str,
        title: &str,
        body: &str,
        tags: &[String],
        wikilinks: &[String],
    ) -> TempleResult<()> {
        let tags_str = tags.join(" ");
        let links_str = wikilinks.join(" ");

        writer
            .add_document(doc!(
                fields.path => path,
                fields.title => title,
                fields.body => body,
                fields.tags => tags_str,
                fields.wikilinks => links_str,
            ))
            .map_err(|e| index_err("add_document", e))?;

        Ok(())
    }

    /// 提交索引变更
    pub fn commit(mut writer: IndexWriter) -> TempleResult<()> {
        writer.commit().map_err(|e| index_err("commit_index", e))?;
        Ok(())
    }

    /// 执行搜索
    pub fn search(&self, query_str: &str, limit: usize) -> TempleResult<Vec<SearchResult>> {
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

            // 简单 snippet: 取 body 前 200 字符
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

    /// 获取索引统计
    pub fn stats(&self) -> TempleResult<IndexStats> {
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .map_err(|e| index_err("create_reader", e))?;

        Ok(IndexStats {
            total_docs: reader.searcher().num_docs(),
            index_size_bytes: 0, // 精确大小需要遍历文件
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_and_search() -> Result<(), Box<dyn std::error::Error>> {
        let engine = SearchEngine::open_in_memory()?;
        let mut writer = engine.writer()?;

        SearchEngine::add_document(
            &mut writer,
            &engine.fields,
            "/vault/rust.md",
            "Rust 编程入门",
            "Rust 是一门系统编程语言，注重安全和性能",
            &["rust".to_string(), "编程".to_string()],
            &[],
        )?;

        SearchEngine::add_document(
            &mut writer,
            &engine.fields,
            "/vault/go.md",
            "Go 并发模式",
            "Go 语言有强大的 goroutine 并发模型",
            &["go".to_string()],
            &[],
        )?;

        SearchEngine::commit(writer)?;

        // 搜索
        let results = engine.search("Rust", 10)?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust 编程入门");
        Ok(())
    }

    #[test]
    fn test_chinese_search() -> Result<(), Box<dyn std::error::Error>> {
        let engine = SearchEngine::open_in_memory()?;
        let mut writer = engine.writer()?;

        SearchEngine::add_document(
            &mut writer,
            &engine.fields,
            "/vault/test.md",
            "知识管理系统",
            "这是一个基于双链的知识管理系统",
            &[],
            &[],
        )?;

        SearchEngine::commit(writer)?;

        let results = engine.search("知识管理", 10)?;
        assert!(!results.is_empty());
        Ok(())
    }
}
