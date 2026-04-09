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
 */

import Graph from "graphology";
import { connectedComponents } from "graphology-components";
import dagre from "dagre";
import potpack from "potpack";
import type { Node, Edge } from "@vue-flow/core";
import { Position } from "@vue-flow/core";

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
  /** 该分量包含的节点 ID 列表 */
  nodeIds: string[];
  /** 该分量内部的边 */
  edges: Edge[];
  /** 节点相对坐标映射 (nodeId → {x, y}) */
  positions: Map<string, { x: number; y: number }>;
  /** 该分量的 BBox */
  bbox: ComponentBBox;
  /** 是否为孤岛（单节点分量） */
  isOrphan: boolean;
}

// ── 核心导出函数 ──

/** 节点尺寸信息 */
interface NodeDimensions {
  width: number;
  height: number;
}

/**
 * layoutMultiComponent — 多连通分量星图布局主入口
 *
 * 接收 Vue Flow 的 nodes/edges，返回带绝对坐标的新 nodes 数组。
 * 自动识别连通分量，独立布局后通过 potpack 打包到同一画布。
 *
 * @param nodes - Vue Flow 节点数组
 * @param edges - Vue Flow 边数组
 * @returns 带绝对坐标的 Vue Flow 节点数组
 */
export function layoutMultiComponent(nodes: Node[], edges: Edge[]): Node[] {
  if (nodes.length === 0) return [];

  // Step 1: 构建 graphology 逻辑图
  const graph = buildGraphology(nodes, edges);

  // Step 2: 切割连通分量
  const components = connectedComponents(graph);

  if (components.length === 0) return nodes;

  // Step 2.5: 构建节点尺寸映射（优先使用实际尺寸，回退到默认值）
  const nodeDimensions = new Map<string, NodeDimensions>(
    nodes.map((n) => [
      n.id,
      n.dimensions
        ? { width: n.dimensions.width, height: n.dimensions.height }
        : { width: NODE_WIDTH, height: NODE_HEIGHT },
    ]),
  );

  // Step 3: 每个分量独立 Dagre 布局
  const nodeMap = new Map(nodes.map((n) => [n.id, n]));
  const edgeSet = new Set(edges.map((e) => `${e.source}:${e.target}`));
  const layouts: ComponentLayout[] = [];

  for (const componentNodeIds of components) {
    const isOrphan = componentNodeIds.length === 1;
    const componentEdges = filterComponentEdges(componentNodeIds, edgeSet);

    const positions = layoutComponent(
      componentNodeIds,
      componentEdges,
      isOrphan,
      nodeDimensions,
    );
    const bbox = computeBoundingBox(positions, isOrphan, nodeDimensions);

    layouts.push({
      nodeIds: componentNodeIds,
      edges: componentEdges,
      positions,
      bbox,
      isOrphan,
    });
  }

  // Step 4: potpack 矩阵打包
  const boxes: PotpackBox[] = layouts.map((l) => ({
    w: l.bbox.width + COMPONENT_GAP,
    h: l.bbox.height + COMPONENT_GAP,
    x: 0,
    y: 0,
  }));
  potpack(boxes);

  // Step 5: 绝对坐标映射
  const result: Node[] = [];

  for (let i = 0; i < layouts.length; i++) {
    const layout = layouts[i];
    const offset = boxes[i];
    // potpack 返回的 x, y 就是该矩形在整体中的偏移
    const offsetX = offset.x + COMPONENT_GAP / 2;
    const offsetY = offset.y + COMPONENT_GAP / 2;

    for (const nodeId of layout.nodeIds) {
      const originalNode = nodeMap.get(nodeId);
      if (!originalNode) continue;

      const relPos = layout.positions.get(nodeId)!;

      result.push({
        ...originalNode,
        targetPosition: Position.Left,
        sourcePosition: Position.Right,
        position: {
          x: relPos.x - layout.bbox.minX + offsetX,
          y: relPos.y - layout.bbox.minY + offsetY,
        },
      });
    }
  }

  return result;
}

/**
 * 获取聚光灯模式下 N 度邻居的节点 ID 集合
 *
 * @param nodes - 所有节点
 * @param edges - 所有边
 * @param focusId - 聚焦的节点 ID
 * @param depth - 聚焦深度（1-3）
 * @returns 应该高亮的节点 ID 集合
 */
export function getSpotlightNeighbors(
  nodes: Node[],
  edges: Edge[],
  focusId: string,
  depth: number,
): Set<string> {
  // 构建邻接表
  const adjacency = new Map<string, Set<string>>();
  for (const node of nodes) {
    adjacency.set(node.id, new Set());
  }
  for (const edge of edges) {
    adjacency.get(edge.source)?.add(edge.target);
    adjacency.get(edge.target)?.add(edge.source);
  }

  // BFS 搜索 N 度邻居（提前终止：不扩展已达 depth 的节点）
  const visited = new Set<string>();
  const queue: Array<{ id: string; d: number }> = [{ id: focusId, d: 0 }];
  visited.add(focusId);
  let head = 0; // 用索引代替 shift()，避免 O(n) 数组拷贝

  while (head < queue.length) {
    const { id, d } = queue[head++];
    // 已达最大深度，不扩展邻居
    if (d >= depth) continue;

    const neighbors = adjacency.get(id);
    if (neighbors) {
      for (const neighborId of neighbors) {
        if (!visited.has(neighborId)) {
          visited.add(neighborId);
          queue.push({ id: neighborId, d: d + 1 });
        }
      }
    }
  }

  return visited;
}

// ── 内部工具函数 ──

/**
 * 构建 graphology 无向图
 */
function buildGraphology(nodes: Node[], edges: Edge[]): Graph {
  const graph = new Graph({ multi: false, type: "undirected" });

  for (const node of nodes) {
    graph.addNode(node.id);
  }

  for (const edge of edges) {
    // 跳过自环和重复边
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

  // 注意：孤岛节点已经被 addNode 添加，但没有边。
  // connectedComponents 会正确返回它们作为单独的分量。

  return graph;
}

/**
 * 筛选属于某个分量的边
 */
function filterComponentEdges(
  componentNodeIds: string[],
  edgeSet: Set<string>,
): Edge[] {
  const nodeIdSet = new Set(componentNodeIds);

  // 从全局边中筛选两端都在该分量内的边
  // 这里需要重新构造边对象
  const result: Edge[] = [];
  // 我们需要从原始 edges 中查找
  // 为了性能，使用 source:target 格式匹配
  for (const edgeKey of edgeSet) {
    const [source, target] = edgeKey.split(":");
    if (nodeIdSet.has(source) && nodeIdSet.has(target)) {
      result.push({
        id: `e-${source}-${target}`,
        source,
        target,
      } as Edge);
    }
  }

  return result;
}

/**
 * 对单个连通分量执行 Dagre 布局
 *
 * @param nodeIds - 该分量的节点 ID
 * @param componentEdges - 该分量的边
 * @param isOrphan - 是否为孤岛
 * @param nodeDimensions - 节点尺寸映射
 * @returns 节点相对坐标映射
 */
function layoutComponent(
  nodeIds: string[],
  componentEdges: Edge[],
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
