//! 知识图谱 — Petgraph 有向图引擎
//!
//! 在内存中构建所有笔记的有向图，提供 BFS 子图提取和拓扑排序。

use std::collections::HashMap;

use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{Bfs, EdgeRef};

use temple_core::error::{ErrorCode, TempleError, TempleResult};

use crate::backend::GraphStore;
use crate::types::{GraphEdge, GraphNode, SubgraphResult};

/// 知识图谱引擎
#[derive(Clone)]
pub struct KnowledgeGraph {
    /// Petgraph 有向图
    inner: DiGraph<GraphNode, GraphEdge>,
    /// card_id → NodeIndex 快速查找
    node_index: HashMap<String, NodeIndex>,
}

impl KnowledgeGraph {
    /// 创建空图
    pub fn new() -> Self {
        Self {
            inner: DiGraph::new(),
            node_index: HashMap::new(),
        }
    }

    /// 添加节点，返回 NodeIndex
    pub fn add_node(&mut self, node: GraphNode) -> NodeIndex {
        let id = node.id.clone();
        let idx = self.inner.add_node(node);
        self.node_index.insert(id, idx);
        idx
    }

    /// 添加有向边
    pub fn add_edge(&mut self, source_id: &str, target_id: &str, edge: GraphEdge) -> TempleResult<()> {
        let source = self.node_index.get(source_id).ok_or_else(|| {
            TempleError::new(ErrorCode::GraphNodeNotFound, format!("节点不存在: {source_id}"))
        })?;
        let target = self.node_index.get(target_id).ok_or_else(|| {
            TempleError::new(ErrorCode::GraphNodeNotFound, format!("节点不存在: {target_id}"))
        })?;
        self.inner.add_edge(*source, *target, edge);
        Ok(())
    }

    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    /// 获取边数量
    pub fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }

    /// 检查节点是否存在
    pub fn contains_node(&self, id: &str) -> bool {
        self.node_index.contains_key(id)
    }

    /// BFS N 度子图提取 — 前端请求"局部星图"时调用
    pub fn subgraph(&self, center_id: &str, depth: usize) -> TempleResult<SubgraphResult> {
        let start = self.node_index.get(center_id).ok_or_else(|| {
            TempleError::new(ErrorCode::GraphNodeNotFound, format!("节点不存在: {center_id}"))
        })?;

        let mut bfs = Bfs::new(&self.inner, *start);
        let mut visited_indices: Vec<NodeIndex> = Vec::new();
        let mut distances: HashMap<NodeIndex, usize> = HashMap::new();
        distances.insert(*start, 0);

        // BFS 逐层展开，记录距离
        while let Some(nx) = bfs.next(&self.inner) {
            if let Some(&d) = distances.get(&nx) {
                if d >= depth {
                    continue;
                }
            }
            visited_indices.push(nx);

            let current_dist = distances.get(&nx).copied().unwrap_or(0);
            if current_dist < depth {
                for neighbor in self.inner.neighbors(nx) {
                    distances.entry(neighbor).or_insert(current_dist + 1);
                }
            }
        }

        // 收集子图节点和边
        let visited_set: std::collections::HashSet<NodeIndex> = visited_indices.iter().copied().collect();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for &idx in &visited_set {
            if let Some(node) = self.inner.node_weight(idx) {
                nodes.push(node.clone());
            }
        }

        for edge_ref in self.inner.edge_references() {
            let source = edge_ref.source();
            let target = edge_ref.target();
            if visited_set.contains(&source) || visited_set.contains(&target) {
                edges.push(edge_ref.weight().clone());
            }
        }

        Ok(SubgraphResult {
            center: center_id.to_string(),
            depth,
            nodes,
            edges,
        })
    }

    /// 拓扑层级排序 — 计算层级布局
    pub fn topological_layers(&self) -> Option<Vec<Vec<String>>> {
        let sorted = toposort(&self.inner, None).ok()?;
        let mut layers: HashMap<NodeIndex, usize> = HashMap::new();

        // 计算每个节点的层级（最长路径）
        for &idx in &sorted {
            let parent_layer = self
                .inner
                .neighbors_directed(idx, petgraph::Direction::Incoming)
                .filter_map(|p| layers.get(&p))
                .max()
                .copied();
            let layer = parent_layer.map_or(0, |l| l + 1);
            layers.insert(idx, layer);
        }

        // 按层级分组
        let max_layer = layers.values().copied().max().unwrap_or(0);
        let mut result: Vec<Vec<String>> = vec![Vec::new(); max_layer + 1];
        for (&idx, &layer) in &layers {
            if let Some(node) = self.inner.node_weight(idx) {
                result[layer].push(node.id.clone());
            }
        }

        Some(result)
    }

    /// 获取所有节点的出度（连接数统计）
    pub fn out_degree(&self, id: &str) -> Option<usize> {
        self.node_index.get(id).map(|&idx| {
            self.inner.neighbors_directed(idx, petgraph::Direction::Outgoing).count()
        })
    }

    /// 获取所有节点的入度
    pub fn in_degree(&self, id: &str) -> Option<usize> {
        self.node_index.get(id).map(|&idx| {
            self.inner.neighbors_directed(idx, petgraph::Direction::Incoming).count()
        })
    }

    /// 批量构建图谱 — 从节点和边列表
    pub fn build_from(nodes: Vec<GraphNode>, edges: Vec<GraphEdge>) -> Self {
        let mut graph = Self::new();
        for node in nodes {
            graph.add_node(node);
        }
        for edge in &edges {
            let _ = graph.add_edge(&edge.source, &edge.target, edge.clone());
        }
        graph
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphStore for KnowledgeGraph {
    fn add_node(&mut self, node: GraphNode) -> TempleResult<String> {
        let id = node.id.clone();
        self.add_node(node);
        Ok(id)
    }

    fn add_edge(&mut self, source: &str, target: &str, edge: GraphEdge) -> TempleResult<()> {
        self.add_edge(source, target, edge)
    }

    fn get_subgraph(&self, center_id: &str, depth: usize) -> TempleResult<SubgraphResult> {
        self.subgraph(center_id, depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: &str, title: &str) -> GraphNode {
        GraphNode { id: id.to_string(), title: title.to_string() }
    }

    fn make_edge(source: &str, target: &str) -> GraphEdge {
        GraphEdge { source: source.to_string(), target: target.to_string(), relation: "reference".to_string() }
    }

    #[test]
    fn test_add_nodes_and_edges() -> Result<(), Box<dyn std::error::Error>> {
        let mut g = KnowledgeGraph::new();
        g.add_node(make_node("a", "A"));
        g.add_node(make_node("b", "B"));
        g.add_edge("a", "b", make_edge("a", "b"))?;
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.edge_count(), 1);
        Ok(())
    }

    #[test]
    fn test_subgraph_bfs() -> Result<(), Box<dyn std::error::Error>> {
        let mut g = KnowledgeGraph::new();
        g.add_node(make_node("a", "A"));
        g.add_node(make_node("b", "B"));
        g.add_node(make_node("c", "C"));
        g.add_node(make_node("d", "D"));
        g.add_edge("a", "b", make_edge("a", "b"))?;
        g.add_edge("b", "c", make_edge("b", "c"))?;
        g.add_edge("a", "d", make_edge("a", "d"))?;

        // 1 度子图: a → b, d
        let sg = g.subgraph("a", 1)?;
        assert!(sg.nodes.len() >= 2); // a + at least b or d

        // 2 度子图: a → b, d → c
        let sg2 = g.subgraph("a", 2)?;
        assert!(sg2.nodes.len() >= 3);
        Ok(())
    }

    #[test]
    fn test_subgraph_not_found() {
        let g = KnowledgeGraph::new();
        assert!(g.subgraph("nonexistent", 1).is_err());
    }

    #[test]
    fn test_topological_layers() -> Result<(), Box<dyn std::error::Error>> {
        let mut g = KnowledgeGraph::new();
        g.add_node(make_node("a", "A"));
        g.add_node(make_node("b", "B"));
        g.add_node(make_node("c", "C"));
        g.add_edge("a", "b", make_edge("a", "b"))?;
        g.add_edge("b", "c", make_edge("b", "c"))?;

        let layers = g.topological_layers().ok_or("topological layers failed")?;
        assert_eq!(layers.len(), 3); // layer 0: a, layer 1: b, layer 2: c
        assert!(layers[0].contains(&"a".to_string()));
        Ok(())
    }

    #[test]
    fn test_degrees() -> Result<(), Box<dyn std::error::Error>> {
        let mut g = KnowledgeGraph::new();
        g.add_node(make_node("a", "A"));
        g.add_node(make_node("b", "B"));
        g.add_node(make_node("c", "C"));
        g.add_edge("a", "b", make_edge("a", "b"))?;
        g.add_edge("c", "b", make_edge("c", "b"))?;

        assert_eq!(g.out_degree("a"), Some(1));
        assert_eq!(g.in_degree("b"), Some(2));
        assert_eq!(g.out_degree("b"), Some(0));
        Ok(())
    }

    #[test]
    fn test_build_from() {
        let nodes = vec![make_node("x", "X"), make_node("y", "Y")];
        let edges = vec![make_edge("x", "y")];
        let g = KnowledgeGraph::build_from(nodes, edges);
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.edge_count(), 1);
    }
}
