//! # ms-graph — 知识图谱核心引擎
//!
//! 基于 Petgraph 的有向图算法库，提供 BFS 子图提取和拓扑排序。
//! 纯内存计算，零 IO 依赖（`graph` feature 仅拉取 petgraph）。

pub mod error;
pub mod model;

#[cfg(feature = "graph")]
pub mod knowledge_graph;

pub use error::{ErrorCode, GraphError, GraphResult};

#[cfg(feature = "graph")]
pub use knowledge_graph::KnowledgeGraph;
#[cfg(feature = "graph")]
pub use model::*;
