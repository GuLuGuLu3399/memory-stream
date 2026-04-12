//! 图谱类型定义

use serde::{Deserialize, Serialize};

/// 图谱节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
}

/// 图谱边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
}

/// 子图提取结果（前端请求"局部星图"时返回）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubgraphResult {
    /// 中心节点 ID
    pub center: String,
    /// 搜索深度
    pub depth: usize,
    /// 子图中的节点
    pub nodes: Vec<GraphNode>,
    /// 子图中的边
    pub edges: Vec<GraphEdge>,
}

/// 节点布局位置（替代前端 Dagre 计算）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub id: String,
    pub x: f64,
    pub y: f64,
}

/// 布局计算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutResult {
    pub positions: Vec<NodePosition>,
    pub width: f64,
    pub height: f64,
}
