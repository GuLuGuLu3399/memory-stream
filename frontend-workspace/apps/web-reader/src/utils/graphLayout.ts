/**
 * 🌌 graphLayout — 多连通分量星图布局引擎
 *
 * 核心流程：
 * 1. 构建 graphology 逻辑图
 * 2. connectedComponents() 切割所有连通分量和孤岛
 * 3. 每个分量独立 Dagre 布局 → 相对坐标
 * 4. 计算每个分量的 BBox
 * 5. potpack 矩形打包 → 绝对偏移量
 * 6. 相对坐标 + 偏移 → 绝对坐标
 *
 * 架构分层：
 * - computePositions() — 纯计算层，零 Vue Flow 依赖（Worker 安全）
 * - layoutMultiComponent() — 同步 Vue Flow 集成
 * - layoutMultiComponentAsync() — Worker 异步 Vue Flow 集成
 */

import Graph from "graphology";
import { connectedComponents } from "graphology-components";
import dagre from "dagre";
import potpack from "potpack";
import type { Node, Edge } from "@vue-flow/core";

// ── 布局常量 ──
const NODE_WIDTH = 220;
const NODE_HEIGHT = 80;
const RANK_SEP = 280;
const NODE_SEP = 60;
const COMPONENT_GAP = 60; // 子图之间的间距
const ORPHAN_NODE_SIZE = 154; // 孤岛节点尺寸 (220 * 0.7)

// ── 类型定义 ──

/** 子图 BBox 信息 */
interface ComponentBBox {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
  width: number;
  height: number;
}

/** potpack 返回的带偏移量盒子 */
interface PotpackBox {
  w: number;
  h: number;
  x: number;
  y: number;
}

/** 子图布局结果 */
interface ComponentLayout {
  nodeIds: string[];
  positions: Map<string, { x: number; y: number }>;
  bbox: ComponentBBox;
  isOrphan: boolean;
}

/** 节点尺寸信息 */
interface NodeDimensions {
  width: number;
  height: number;
}

/** 序列化边（Worker 传输用） */
export interface SerializableEdge {
  source: string;
  target: string;
}

/** 序列化节点（Worker 传输用） */
export interface SerializableNode {
  id: string;
  width: number;
  height: number;
}

/** 从 Vue Flow Node 安全提取尺寸（dimensions 存在于运行时但不在 TS 类型中） */
function getNodeDimensions(n: Node): { width: number; height: number } {
  const dims = (n as unknown as { dimensions?: { width: number; height: number } }).dimensions;
  return dims ?? { width: NODE_WIDTH, height: NODE_HEIGHT };
}

// ── 内部工具函数 ──

/**
 * 构建 graphology 无向图
 */
function buildGraphologyGraph(
  nodeIds: string[],
  edges: SerializableEdge[],
): Graph {
  const graph = new Graph({ multi: false, type: "undirected" });

  for (const id of nodeIds) {
    graph.addNode(id);
  }

  for (const edge of edges) {
    if (
      edge.source === edge.target ||
      graph.hasEdge(edge.source, edge.target)
    ) {
      continue;
    }
    try {
      graph.addEdge(edge.source, edge.target);
    } catch {
      // 节点不存在时跳过
    }
  }

  return graph;
}

/**
 * 筛选属于某个分量的边
 */
function filterComponentEdges(
  componentNodeIds: string[],
  edgeSet: Set<string>,
): SerializableEdge[] {
  const nodeIdSet = new Set(componentNodeIds);
  const result: SerializableEdge[] = [];

  for (const edgeKey of edgeSet) {
    const colonIdx = edgeKey.indexOf(":");
    const source = edgeKey.substring(0, colonIdx);
    const target = edgeKey.substring(colonIdx + 1);
    if (nodeIdSet.has(source) && nodeIdSet.has(target)) {
      result.push({ source, target });
    }
  }

  return result;
}

/**
 * 对单个连通分量执行 Dagre 布局
 */
function layoutComponentDagre(
  nodeIds: string[],
  componentEdges: SerializableEdge[],
  isOrphan: boolean,
  nodeDimensions: Map<string, NodeDimensions>,
): Map<string, { x: number; y: number }> {
  const positions = new Map<string, { x: number; y: number }>();

  if (isOrphan) {
    positions.set(nodeIds[0], { x: 0, y: 0 });
    return positions;
  }

  const g = new dagre.graphlib.Graph();
  g.setDefaultEdgeLabel(() => ({}));
  g.setGraph({
    rankdir: "LR",
    nodesep: NODE_SEP,
    ranksep: RANK_SEP,
    marginx: 20,
    marginy: 20,
  });

  for (const nodeId of nodeIds) {
    const dims = nodeDimensions.get(nodeId) ?? { width: NODE_WIDTH, height: NODE_HEIGHT };
    g.setNode(nodeId, { width: dims.width, height: dims.height });
  }

  for (const edge of componentEdges) {
    g.setEdge(edge.source, edge.target);
  }

  dagre.layout(g);

  for (const nodeId of nodeIds) {
    const pos = g.node(nodeId);
    const dims = nodeDimensions.get(nodeId) ?? { width: NODE_WIDTH, height: NODE_HEIGHT };
    if (pos) {
      positions.set(nodeId, {
        x: pos.x - dims.width / 2,
        y: pos.y - dims.height / 2,
      });
    }
  }

  return positions;
}

/**
 * 计算子图的 BoundingBox
 */
function computeBoundingBox(
  positions: Map<string, { x: number; y: number }>,
  isOrphan: boolean,
  nodeDimensions: Map<string, NodeDimensions>,
): ComponentBBox {
  let minX = Infinity,
    maxX = -Infinity,
    minY = Infinity,
    maxY = -Infinity;

  for (const [nodeId, pos] of positions) {
    const dims = nodeDimensions.get(nodeId) ?? { width: NODE_WIDTH, height: NODE_HEIGHT };
    const w = isOrphan ? Math.min(dims.width, ORPHAN_NODE_SIZE) : dims.width;
    const h = isOrphan ? Math.min(dims.height, ORPHAN_NODE_SIZE * (80 / 220)) : dims.height;

    minX = Math.min(minX, pos.x);
    maxX = Math.max(maxX, pos.x + w);
    minY = Math.min(minY, pos.y);
    maxY = Math.max(maxY, pos.y + h);
  }

  return {
    minX,
    maxX,
    minY,
    maxY,
    width: maxX - minX,
    height: maxY - minY,
  };
}

// ── 纯计算层（Worker 安全，零 Vue Flow 依赖） ──

/**
 * computePositions — 纯坐标计算
 *
 * 接收序列化的节点/边数据，返回节点绝对坐标映射。
 * 不依赖 Vue Flow，可在 Web Worker 中执行。
 */
export function computePositions(
  nodes: SerializableNode[],
  edges: SerializableEdge[],
): Record<string, { x: number; y: number }> {
  if (nodes.length === 0) return {};

  const nodeIds = nodes.map((n) => n.id);
  const graph = buildGraphologyGraph(nodeIds, edges);
  const components = connectedComponents(graph);

  if (components.length === 0) return {};

  const nodeDimensions = new Map<string, NodeDimensions>(
    nodes.map((n) => [n.id, { width: n.width, height: n.height }]),
  );
  const edgeSet = new Set(edges.map((e) => `${e.source}:${e.target}`));
  const layouts: ComponentLayout[] = [];

  for (const componentNodeIds of components) {
    const isOrphan = componentNodeIds.length === 1;
    const componentEdges = filterComponentEdges(componentNodeIds, edgeSet);
    const positions = layoutComponentDagre(componentNodeIds, componentEdges, isOrphan, nodeDimensions);
    const bbox = computeBoundingBox(positions, isOrphan, nodeDimensions);

    layouts.push({ nodeIds: componentNodeIds, positions, bbox, isOrphan });
  }

  // potpack 矩阵打包
  const boxes: PotpackBox[] = layouts.map((l) => ({
    w: l.bbox.width + COMPONENT_GAP,
    h: l.bbox.height + COMPONENT_GAP,
    x: 0,
    y: 0,
  }));
  potpack(boxes);

  // 绝对坐标映射
  const result: Record<string, { x: number; y: number }> = {};

  for (let i = 0; i < layouts.length; i++) {
    const layout = layouts[i];
    const offset = boxes[i];
    const offsetX = offset.x + COMPONENT_GAP / 2;
    const offsetY = offset.y + COMPONENT_GAP / 2;

    for (const nodeId of layout.nodeIds) {
      const relPos = layout.positions.get(nodeId)!;
      result[nodeId] = {
        x: relPos.x - layout.bbox.minX + offsetX,
        y: relPos.y - layout.bbox.minY + offsetY,
      };
    }
  }

  return result;
}

// ── Vue Flow 集成层 ──

/**
 * layoutMultiComponent — 同步版 Vue Flow 布局
 *
 * 接收 Vue Flow 的 nodes/edges，返回带绝对坐标的新 nodes 数组。
 * 小图谱（< ASYNC_THRESHOLD）可直接使用，大图谱请用 layoutMultiComponentAsync。
 */
export function layoutMultiComponent(nodes: Node[], edges: Edge[]): Node[] {
  if (nodes.length === 0) return [];

  const layoutNodes: SerializableNode[] = nodes.map((n) => {
    const dims = getNodeDimensions(n);
    return { id: n.id, width: dims.width, height: dims.height };
  });
  const layoutEdges: SerializableEdge[] = edges.map((e) => ({
    source: e.source,
    target: e.target,
  }));

  const positions = computePositions(layoutNodes, layoutEdges);

  return nodes.map((n) => ({
    ...n,
    position: positions[n.id] ?? n.position ?? { x: 0, y: 0 },
  }));
}

/**
 * layoutMultiComponentAsync — Worker 异步版 Vue Flow 布局
 *
 * 将 Dagre 布局计算移至 Web Worker，避免大图谱阻塞主线程。
 * 适用于 100+ 节点的图谱。
 */
export async function layoutMultiComponentAsync(
  nodes: Node[],
  edges: Edge[],
): Promise<Node[]> {
  if (nodes.length === 0) return [];

  const layoutNodes: SerializableNode[] = nodes.map((n) => {
    const dims = getNodeDimensions(n);
    return { id: n.id, width: dims.width, height: dims.height };
  });
  const layoutEdges: SerializableEdge[] = edges.map((e) => ({
    source: e.source,
    target: e.target,
  }));

  const positions = await runLayoutWorker(layoutNodes, layoutEdges);

  return nodes.map((n) => ({
    ...n,
    position: positions[n.id] ?? n.position ?? { x: 0, y: 0 },
  }));
}

function runLayoutWorker(
  nodes: SerializableNode[],
  edges: SerializableEdge[],
): Promise<Record<string, { x: number; y: number }>> {
  return new Promise((resolve, reject) => {
    const worker = new Worker(
      new URL("./layout.worker.ts", import.meta.url),
      { type: "module" },
    );
    worker.onmessage = (e) => {
      worker.terminate();
      resolve(e.data);
    };
    worker.onerror = (e) => {
      worker.terminate();
      reject(new Error(`Layout worker error: ${e.message}`));
    };
    worker.postMessage({ nodes, edges });
  });
}
