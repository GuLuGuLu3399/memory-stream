//! 图谱数据模型

use serde::{Deserialize, Serialize};

/// 图谱节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
}

/// 图谱有向边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
}

/// 子图提取结果（BFS N 度邻居）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubgraphResult {
    /// 中心节点 ID
    pub center: String,
    /// 搜索深度
    pub depth: usize,
    /// 子图节点
    pub nodes: Vec<GraphNode>,
    /// 子图边
    pub edges: Vec<GraphEdge>,
}
