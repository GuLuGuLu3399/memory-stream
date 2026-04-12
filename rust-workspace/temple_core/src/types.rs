//! IPC 数据类型 — 前后端共享的结构化类型定义
//!
//! 从 admin-tauri/lib.rs 迁移而来，统一管理所有跨 IPC 边界的数据结构。

use serde::{Deserialize, Serialize};

// ============================================================================
// Markdown 渲染
// ============================================================================

/// Markdown 解析与渲染结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResult {
    /// 渲染后的 HTML 字符串
    pub html: String,
    /// AST 结构化 JSON 字符串
    pub ast_json: String,
    /// 从原文提取的纯文本摘要
    pub excerpt: String,
    /// 从原文提取的 wikilink 链接
    pub extracted_links: Vec<String>,
}

// ============================================================================
// 缓存与同步
// ============================================================================

/// 本地布局缓存中的单个卡片记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedLayout {
    pub card_id: String,
    pub x: f64,
    pub y: f64,
    pub title: String,
    pub category_id: Option<String>,
    pub hot_score: f64,
    pub updated_at: String,
}

/// 本地布局缓存中的单条边记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEdge {
    pub source_id: String,
    pub target_id: String,
    pub relation: String,
}

/// 本地布局缓存查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutCacheResult {
    pub layouts: Vec<CachedLayout>,
    pub edges: Vec<CachedEdge>,
    pub count: i64,
    pub last_sync: Option<String>,
}

/// 服务端同步操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub synced_count: i64,
}

// ============================================================================
// 卡片 CRUD
// ============================================================================

/// 创建卡片请求载荷
#[derive(Debug, Clone, Serialize)]
pub struct CreateCardRequest {
    pub title: String,
    pub raw_md: String,
    pub excerpt: String,
    pub ast_data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toc_data: Option<serde_json::Value>,
    pub parent_id: Option<String>,
    pub relation_type: Option<String>,
}

/// 创建卡片响应
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCardResponse {
    pub card_id: String,
}

// ============================================================================
// 图谱
// ============================================================================

/// 图谱节点响应
#[derive(Debug, Clone, Deserialize)]
pub struct GraphNodeResponse {
    pub id: String,
    pub title: String,
}

/// 图谱边响应
#[derive(Debug, Clone, Deserialize)]
pub struct GraphEdgeResponse {
    pub source: String,
    pub target: String,
    pub relation: String,
}

/// 图谱概览响应
#[derive(Debug, Clone, Deserialize)]
pub struct OutlineResponse {
    pub nodes: Vec<GraphNodeResponse>,
    pub edges: Vec<GraphEdgeResponse>,
}

// ============================================================================
// 文件变更
// ============================================================================

/// 文件系统变更事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileChangeKind {
    Create,
    Modify,
    Remove,
}

/// 文件系统变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeEvent {
    pub path: String,
    pub kind: FileChangeKind,
}
