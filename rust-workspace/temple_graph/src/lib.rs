//! # temple_graph — 图谱拓扑与双链计算引擎
//!
//! 基于 Petgraph 的有向图，提供 BFS 子图提取和层级布局计算，
//! 将繁重的图论算力从前端 V8 剥离到 Rust 侧。

pub mod backend;
pub mod graph;
pub mod layout;
pub mod types;

pub use backend::GraphStore;
pub use graph::KnowledgeGraph;
pub use layout::compute_hierarchical_layout;
pub use types::{GraphEdge, GraphNode, LayoutResult, NodePosition, SubgraphResult};
