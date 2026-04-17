/**
 * 🌌 useGraph — 全量图谱数据管理（多连通分量星图）
 *
 * 从后端 API 拉取全量节点和边数据，转换为 Vue Flow 格式。
 * 支持全量快照模式和深度遍历模式。
 * 孤岛节点通过 connectedComponentIds 标记。
 */

import { ref, shallowRef } from "vue";
import { api, type GraphResult } from "../api";
import { GraphResultSchema } from "../api/schemas";
import type { Node, Edge } from "@vue-flow/core";

/** Vue Flow 节点数据 — 卡片节点展示信息 */
export interface CardNodeData {
  title: string;
  date: string;
  type?: string;
  /** 孤岛标记：该节点没有任何边连接 */
  isOrphan?: boolean;
}

/** Vue Flow 边数据扩展 — 支持双向引用标记 */
export interface EdgeData {
  type?: string; // 'sequence' | 'reference'
  /** 双向引用标记：同时存在 A→B 和 B→A 时为 true */
  isBidirectional?: boolean;
}

// Default Vue Flow edge options used across the graph.
// Ensure edges render with a consistent aesthetic (smoothstep).
// This is a lightweight, opt-in configuration to unify edge drawing
// without altering per-edge data animation/behavior logic elsewhere.
export const defaultEdgeOptions = {
  type: "smoothstep",
};

/**
 * 🔄 双向边融合算法：消除相互重叠的双线
 *
 * 场景：用户在 A 中写 [[B]]，在 B 中也写了 [[A]]（互相引用）。
 * 结果：后端返回 A→B 和 B→A 两条有向边。
 * 问题：图谱里会看到两条交叉或重叠的线，很混乱。
 *
 * 解法：检测互相引用的双边，合并为一条边，并标记 isBidirectional = true。
 * 渲染时：在线的两头都画箭头，视觉表达双向性。
 *
 * @param rawEdges - 从后端获取的原始边数组
 * @returns 融合后的边数组，消除相互重叠的双线
 */
function deduplicateAndMergeEdges(
  rawEdges: Array<{ source: string; target: string; relation: string }>,
): Edge[] {
  const mergedEdges: Edge[] = [];
  const edgeMap = new Map<string, Edge>();

  rawEdges.forEach((edge) => {
    // 生成无向键：将 A-B 和 B-A 都映射到同一个标准键
    // 按字母排序保证唯一性（如 'A' 和 'B' 排序，总是 'A|B'）
    const [node1, node2] = [edge.source, edge.target].sort();
    const edgeType = edge.relation;
    const uniqueKey = `${node1}|${node2}|${edgeType}`;

    if (edgeMap.has(uniqueKey)) {
      // 发现反向线！比如之前存了 A→B，现在又来了 B→A
      const existingEdge = edgeMap.get(uniqueKey)!;
      // 将原有的单向边，标记为"互相引用（Bidirectional）"
      const existingData = (existingEdge.data as EdgeData) || {};
      existingData.isBidirectional = true;
      existingEdge.data = existingData;
    } else {
      // 第一次见到的边，先存起来
      const newEdge: Edge = {
        id: `e-${edge.source}-${edge.target}`,
        source: edge.source,
        target: edge.target,
        data: {
          type: edgeType,
          isBidirectional: false,
        } as EdgeData,
      };
      edgeMap.set(uniqueKey, newEdge);
      mergedEdges.push(newEdge);
    }
  });

  return mergedEdges;
}

/**
 * useGraph — 图谱数据管理 Composable（多连通分量星图）。
 *
 * 支持两种加载模式：
 * 1. `loadFullGraph()` — 全量快照，展示所有连通分量和孤岛
 * 2. `load(cardId, depth)` — 深度遍历，以指定卡片为中心
 *
 * @returns Vue Flow 格式的 nodes/edges、加载状态、加载方法
 */
export function useGraph() {
  const nodes = shallowRef<Node[]>([]);
  const edges = shallowRef<Edge[]>([]);
  const loading = ref(false);
  const error = ref("");
  const isEmpty = ref(false);

  /**
   * 加载全量图谱数据（所有节点 + 所有边，含孤岛）。
   * 调用后端 GET /graph/all 接口。
   */
  async function loadFullGraph() {
    loading.value = true;
    error.value = "";
    isEmpty.value = false;

    try {
      const rawResult: GraphResult = await api.getFullGraph();

      // ── Zod 运行时校验：拦截脏数据 ──
      const result = GraphResultSchema.parse(rawResult);

      if (!result.nodes || result.nodes.length === 0) {
        nodes.value = [];
        edges.value = [];
        isEmpty.value = true;
        return;
      }

      convertToVueFlow(result);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      console.error("❌ 全量图谱数据加载失败:", message);
      nodes.value = [];
      edges.value = [];
      error.value = message;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 加载指定卡片的子图数据（深度遍历模式）。
   *
   * @param cardId - 中心卡片 ID（"root" 自动解析为根卡片）
   * @param depth - 遍历深度，默认 2（范围 1-5）
   */
  async function load(cardId: string, depth = 2) {
    loading.value = true;
    error.value = "";
    isEmpty.value = false;

    try {
      const rawResult: GraphResult = await api.getGraph(cardId, depth);

      // ── Zod 运行时校验：拦截脏数据 ──
      const result = GraphResultSchema.parse(rawResult);

      if (!result.nodes || result.nodes.length === 0) {
        nodes.value = [];
        edges.value = [];
        isEmpty.value = true;
        return;
      }

      convertToVueFlow(result);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      console.error("❌ 图谱数据加载失败:", message);
      nodes.value = [];
      edges.value = [];
      error.value = message;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 将后端 GraphResult 转换为 Vue Flow 节点/边格式。
   * position 设为 {0,0} 占位，实际布局由 GraphView.vue 中的布局引擎计算。
   *
   * 孤岛检测：没有任何边引用的节点标记为 isOrphan。
   * 从 sequence 边反推节点类型。
   *
   * @param result - 后端返回的图谱数据（nodes + edges）
   */
  function convertToVueFlow(result: GraphResult) {
    // 1. 收集所有有边连接的节点 ID
    const connectedNodeIds = new Set<string>();
    result.edges.forEach((edge) => {
      connectedNodeIds.add(edge.source);
      connectedNodeIds.add(edge.target);
    });

    // 2. 从 sequence 边收集主干节点 ID
    const mainChainIds = new Set<string>();
    result.edges.forEach((edge) => {
      if (edge.relation === "sequence") {
        mainChainIds.add(edge.source);
        mainChainIds.add(edge.target);
      }
    });

    // 3. 为节点设置正确的 type 和 isOrphan 标记
    const mappedNodes: Node[] = result.nodes.map((node) => ({
      id: node.id,
      type: "card",
      position: { x: 0, y: 0 },
      data: {
        title: node.title,
        date: "",
        type: mainChainIds.has(node.id) ? "sequence" : "reference",
        isOrphan: !connectedNodeIds.has(node.id),
      } as CardNodeData,
    }));
    nodes.value = mappedNodes;

    // 🔄 先执行边去重融合：消除双线乱飞
    const mergedEdgeData = deduplicateAndMergeEdges(result.edges);

    const mappedEdges: Edge[] = mergedEdgeData.map((edge) => {
      const edgeData = (edge.data as EdgeData) || {};
      const isReference = edgeData.type !== "sequence";
      const isBiDir = edgeData.isBidirectional || false;

      return {
        id: edge.id,
        source: edge.source,
        target: edge.target,
        // 🗡️ 第二击：连线形态解耦（视觉隔离）
        // 主干用平滑折线 (smoothstep) + 动画，参考线用贝塞尔曲线 (default) + 静止
        type: isReference ? "default" : "smoothstep",
        animated: !isReference, // 主干线流动，参考线静止
        data: edgeData,
        // 🔄 双向边视觉映射：双向引用时两头都有箭头
        // 这部分在 GraphView 的 edges computed 中处理，这里只保留数据
        // 动画与样式压制：主干线醒目，参考线极度暗淡
        style: isReference
          ? {
              stroke: isBiDir ? "#888" : "#555", // 双向参考线稍亮一点
              strokeWidth: isBiDir ? 1.5 : 1,
              opacity: 0.2, // 参考线：极细、极暗、极淡
            }
          : {
              stroke: "#00e5ff",
              strokeWidth: 2,
              opacity: 1, // 主干线：醒目、科幻蓝
            },
        // zIndex：让参考线在 DOM 层级中垫底
        zIndex: isReference ? 0 : 10,
      };
    });
    edges.value = mappedEdges;
  }

  return { nodes, edges, loading, error, isEmpty, load, loadFullGraph };
}
