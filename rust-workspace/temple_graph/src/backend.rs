//! 图谱存储后端 trait — 解耦具体实现（Petgraph / Neo4j / 远端 API）。

use crate::types::{GraphEdge, GraphNode, SubgraphResult};
use temple_core::error::TempleResult;

/// 图谱存储后端接口。
///
/// 抽象图谱的增删查操作，使上层业务不依赖具体的图引擎实现。
/// 当前实现为 [`KnowledgeGraph`](crate::graph::KnowledgeGraph)。
///
/// # 线程安全
/// 实现者必须满足 `Send + Sync`，允许在 `Arc<Mutex<dyn GraphStore>>` 中使用。
pub trait GraphStore: Send + Sync {
    /// 添加节点，返回节点 ID。
    fn add_node(&mut self, node: GraphNode) -> TempleResult<String>;

    /// 添加有向边。
    fn add_edge(&mut self, source: &str, target: &str, edge: GraphEdge) -> TempleResult<()>;

    /// 获取以 `center_id` 为中心、`depth` 度范围内的子图。
    fn get_subgraph(&self, center_id: &str, depth: usize) -> TempleResult<SubgraphResult>;
}
