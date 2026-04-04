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

// Default Vue Flow edge options used across the graph.
// Ensure edges render with a consistent aesthetic (smoothstep).
// This is a lightweight, opt-in configuration to unify edge drawing
// without altering per-edge data animation/behavior logic elsewhere.
export const defaultEdgeOptions = {
  type: "smoothstep",
};

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

    const mappedEdges: Edge[] = result.edges.map((edge) => ({
      id: `e-${edge.source}-${edge.target}`,
      source: edge.source,
      target: edge.target,
      animated: edge.relation === "sequence",
      data: { type: edge.relation },
      type: "smoothstep",
      style:
        edge.relation === "sequence"
          ? { stroke: "#00e5ff", strokeWidth: 2 }
          : { stroke: "#71717a", strokeWidth: 1.5, strokeDasharray: "5 5" },
    }));
    edges.value = mappedEdges;
  }

  return { nodes, edges, loading, error, isEmpty, load, loadFullGraph };
}
