// ────────────────────────────────────────────────────────────────
// Graph model — mirrors ms-graph/src/model.rs
// ────────────────────────────────────────────────────────────────

export interface GraphNode {
  id: string;
  title: string;
  x?: number;
  y?: number;
  type?: string;
}

export interface GraphEdge {
  source: string;
  target: string;
  relation: string;
}

export interface SubgraphResult {
  center: string;
  depth: number;
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export interface NodePosition {
  id: string;
  x: number;
  y: number;
}

export interface LayoutResult {
  positions: NodePosition[];
  width: number;
  height: number;
}

export interface FullGraph {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export interface BacklinkItem {
  uuid: string;
  title: string;
  relation_type: string;
}
