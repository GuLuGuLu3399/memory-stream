//! Tantivy 索引 Schema 定义

use tantivy::schema::*;

use temple_core::error::{ErrorCode, TempleError, TempleResult};

/// 创建 Temple 搜索引擎的 Schema
///
/// 文本字段使用 "jieba" 分词器，支持中文全文搜索。
/// 注意：使用前需在 Index 上注册 `jieba` tokenizer。
pub fn temple_schema() -> Schema {
    let mut builder = Schema::builder();

    let text_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("jieba")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();

    let text_options_unstored = TextOptions::default().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("jieba")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions),
    );

    builder.add_text_field("path", STRING | STORED);
    builder.add_text_field("title", text_options.clone());
    builder.add_text_field("body", text_options);
    builder.add_text_field("tags", text_options_unstored.clone());
    builder.add_text_field("wikilinks", text_options_unstored);

    builder.build()
}

/// 获取 schema 中的字段引用
pub struct TempleFields {
    pub path: Field,
    pub title: Field,
    pub body: Field,
    pub tags: Field,
    pub wikilinks: Field,
}

fn schema_field_err(field: &str, err: impl std::fmt::Display) -> TempleError {
    TempleError::new(
        ErrorCode::IndexNotReady,
        format!("temple_search::schema 字段 `{field}` 失败: {err}"),
    )
}

impl TempleFields {
    pub fn new(schema: &Schema) -> TempleResult<Self> {
        let path = schema
            .get_field("path")
            .map_err(|e| schema_field_err("path", e))?;
        let title = schema
            .get_field("title")
            .map_err(|e| schema_field_err("title", e))?;
        let body = schema
            .get_field("body")
            .map_err(|e| schema_field_err("body", e))?;
        let tags = schema
            .get_field("tags")
            .map_err(|e| schema_field_err("tags", e))?;
        let wikilinks = schema
            .get_field("wikilinks")
            .map_err(|e| schema_field_err("wikilinks", e))?;

        Ok(Self {
            path,
            title,
            body,
            tags,
            wikilinks,
        })
    }
}
