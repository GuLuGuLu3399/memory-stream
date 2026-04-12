//! Tantivy 索引 Schema 定义

use tantivy::schema::*;

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

    let text_options_unstored = TextOptions::default()
        .set_indexing_options(
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

impl TempleFields {
    pub fn new(schema: &Schema) -> Self {
        Self {
            path: schema.get_field("path").unwrap(),
            title: schema.get_field("title").unwrap(),
            body: schema.get_field("body").unwrap(),
            tags: schema.get_field("tags").unwrap(),
            wikilinks: schema.get_field("wikilinks").unwrap(),
        }
    }
}
