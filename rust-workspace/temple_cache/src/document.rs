//! 内存驻留的文档结构 — 解析后的完整文档数据

use serde::{Deserialize, Serialize};

/// 内存中的文档快照（零 I/O 读取目标）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// 文件绝对路径（作为 DashMap key）
    pub path: String,
    /// 文件名（不含扩展名），作为标题的回退
    pub filename: String,
    /// 从 frontmatter 或首行 H1 提取的标题
    pub title: String,
    /// 原始 Markdown 文本
    pub raw_md: String,
    /// 纯文本摘要（前 150 字符）
    pub excerpt: String,
    /// 渲染后的 HTML
    pub html: String,
    /// AST JSON 字符串
    pub ast_json: String,
    /// 从 Markdown 中提取的 [[wikilink]] 列表
    pub extracted_links: Vec<String>,
    /// Unix 时间戳 (毫秒)
    pub updated_at: u64,
}

/// 轻量元数据（列表页使用，避免传输全文 HTML）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMeta {
    pub path: String,
    pub filename: String,
    pub title: String,
    pub excerpt: String,
    pub extracted_links: Vec<String>,
    pub updated_at: u64,
}

impl From<&Document> for DocumentMeta {
    fn from(doc: &Document) -> Self {
        Self {
            path: doc.path.clone(),
            filename: doc.filename.clone(),
            title: doc.title.clone(),
            excerpt: doc.excerpt.clone(),
            extracted_links: doc.extracted_links.clone(),
            updated_at: doc.updated_at,
        }
    }
}
