//! 搜索类型定义

use serde::{Deserialize, Serialize};

/// 搜索结果条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 文件路径
    pub path: String,
    /// 文件标题
    pub title: String,
    /// 搜索得分
    pub score: f32,
    /// 高亮上下文片段
    pub snippet: String,
    /// 匹配类型
    pub match_type: MatchType,
}

/// 匹配位置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchType {
    Title,
    Body,
    Tag,
}

/// 索引统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_docs: u64,
    pub index_size_bytes: u64,
}
