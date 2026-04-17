/**
 * dagre.worker — Web Worker: 将 Dagre 布局计算移出主线程
 *
 * 避免大图谱（100+ 节点）布局时 UI 冻结。
 * 接收节点/边列表，返回节点坐标映射。
 */
import dagre from "dagre";

interface LayoutRequest {
  nodes: Array<{ id: string }>;
  edges: Array<{ source: string; target: string }>;
}

interface LayoutResponse {
  positions: Record<string, { x: number; y: number }>;
}

self.onmessage = (e: MessageEvent<LayoutRequest>) => {
  const { nodes, edges } = e.data;

  const g = new dagre.graphlib.Graph();
  g.setDefaultEdgeLabel(() => ({}));
  g.setGraph({ rankdir: "TB", nodesep: 60, ranksep: 80 });

  const validNodeIds = new Set(nodes.map((n) => n.id));

  for (const n of nodes) {
    g.setNode(n.id, { width: 140, height: 36 });
  }

  for (const e of edges) {
    if (validNodeIds.has(e.source) && validNodeIds.has(e.target)) {
      g.setEdge(e.source, e.target);
    }
  }

  dagre.layout(g);

  const positions: LayoutResponse["positions"] = {};
  for (const n of nodes) {
    const nodePos = g.node(n.id);
    if (nodePos) {
      positions[n.id] = { x: nodePos.x - 70, y: nodePos.y - 18 };
    }
  }

  self.postMessage({ positions } satisfies LayoutResponse);
};
