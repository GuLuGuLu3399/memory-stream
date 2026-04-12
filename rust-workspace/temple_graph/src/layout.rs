//! 层级布局算法 — 替代前端 Dagre

use crate::graph::KnowledgeGraph;
use crate::types::{LayoutResult, NodePosition};

const LAYER_GAP: f64 = 200.0; // 层间距
const NODE_GAP: f64 = 180.0;  // 同层节点间距

/// 基于拓扑层级计算 Sugiyama 风格布局
///
/// 返回每个节点的 {x, y} 坐标，前端 Vue Flow 退化为纯渲染层。
pub fn compute_hierarchical_layout(
    graph: &KnowledgeGraph,
    node_ids: &[String],
) -> LayoutResult {
    let layers = graph.topological_layers();

    match layers {
        Some(layer_groups) => {
            let mut positions = Vec::new();
            let mut max_width = 0.0_f64;

            for (layer_idx, layer_nodes) in layer_groups.iter().enumerate() {
                // 只布局请求的节点（如果有指定）
                let filtered: Vec<&String> = if node_ids.is_empty() {
                    layer_nodes.iter().collect()
                } else {
                    layer_nodes
                        .iter()
                        .filter(|id| node_ids.contains(id))
                        .collect()
                };

                let count = filtered.len();
                let layer_width = (count.saturating_sub(1) as f64) * NODE_GAP;
                if layer_width > max_width {
                    max_width = layer_width;
                }

                for (i, id) in filtered.iter().enumerate() {
                    positions.push(NodePosition {
                        id: (*id).clone(),
                        x: (i as f64) * NODE_GAP - layer_width / 2.0,
                        y: (layer_idx as f64) * LAYER_GAP,
                    });
                }
            }

            let max_layer = layer_groups.len();
            LayoutResult {
                positions,
                width: max_width,
                height: (max_layer.saturating_sub(1) as f64) * LAYER_GAP,
            }
        }
        None => {
            // 有环图 fallback: 圆形布局
            let count = node_ids.len();
            let radius = (count as f64) * 50.0;
            let positions: Vec<NodePosition> = node_ids
                .iter()
                .enumerate()
                .map(|(i, id)| {
                    let angle = (i as f64) / (count as f64) * 2.0 * std::f64::consts::PI;
                    NodePosition {
                        id: id.clone(),
                        x: radius * angle.cos(),
                        y: radius * angle.sin(),
                    }
                })
                .collect();

            LayoutResult {
                positions,
                width: radius * 2.0,
                height: radius * 2.0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::KnowledgeGraph;
    use crate::types::{GraphEdge, GraphNode};

    fn make_node(id: &str) -> GraphNode {
        GraphNode { id: id.to_string(), title: id.to_string() }
    }

    fn make_edge(s: &str, t: &str) -> GraphEdge {
        GraphEdge { source: s.to_string(), target: t.to_string(), relation: "ref".to_string() }
    }

    #[test]
    fn test_hierarchical_layout() {
        let mut g = KnowledgeGraph::new();
        g.add_node(make_node("a"));
        g.add_node(make_node("b"));
        g.add_node(make_node("c"));
        g.add_edge("a", "b", make_edge("a", "b")).unwrap();
        g.add_edge("b", "c", make_edge("b", "c")).unwrap();

        let result = compute_hierarchical_layout(&g, &[]);
        assert_eq!(result.positions.len(), 3);

        // a should be at y=0 (layer 0), b at y=200 (layer 1), c at y=400 (layer 2)
        let a = result.positions.iter().find(|p| p.id == "a").unwrap();
        let c = result.positions.iter().find(|p| p.id == "c").unwrap();
        assert!(a.y < c.y);
    }

    #[test]
    fn test_circular_fallback() {
        let mut g = KnowledgeGraph::new();
        g.add_node(make_node("a"));
        g.add_node(make_node("b"));
        g.add_edge("a", "b", make_edge("a", "b")).unwrap();
        g.add_edge("b", "a", make_edge("b", "a")).unwrap(); // cycle!

        // toposort fails → circular fallback
        let result = compute_hierarchical_layout(&g, &["a".to_string(), "b".to_string()]);
        assert_eq!(result.positions.len(), 2);
    }
}
