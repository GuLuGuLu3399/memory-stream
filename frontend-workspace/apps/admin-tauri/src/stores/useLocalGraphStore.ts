/**
 * @module useLocalGraphStore
 *
 * 局部图谱 Store — 当前卡片的邻居节点和边关系
 *
 * 所有网络请求通过 Rust 的 api_request IPC 命令发出。
 */

import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

/** 图谱可视化节点 — 包含布局坐标 */
export interface LocalGraphNode {
  id: string;
  title: string;
  x: number;
  y: number;
}

/** 图谱可视化边 */
export interface LocalGraphEdge {
  source: string;
  target: string;
  relation: string;
}

/** 局部图谱 API 响应 */
interface GraphDetailResponse {
  nodes: Array<{ id: string; title?: string }>;
  edges: Array<{ source: string; target: string; relation?: string }>;
}

export const useLocalGraphStore = defineStore("localGraph", () => {
  // ---- 响应式状态 ----
  const localNodes = ref<LocalGraphNode[]>([]);
  const localEdges = ref<LocalGraphEdge[]>([]);

  /** 加载指定卡片的局部图谱（邻居节点 + 边） */
  async function loadLocalGraph(cardId: string) {
    if (!cardId) return;
    try {
      const data = await invoke<GraphDetailResponse>("api_request", {
        method: "GET",
        endpoint: `/graph/detail/${cardId}`,
      });
      localNodes.value = (data.nodes || []).map((n) => ({
        id: n.id,
        title: n.title || n.id,
        x: 0,
        y: 0,
      }));
      localEdges.value = (data.edges || []).map((e) => ({
        source: e.source,
        target: e.target,
        relation: e.relation || "reference",
      }));
    } catch (e) {
      console.error("[LocalGraphStore] loadLocalGraph failed:", e);
    }
  }

  /** 清空局部图谱 */
  function clearLocalGraph() {
    localNodes.value = [];
    localEdges.value = [];
  }

  return {
    localNodes,
    localEdges,
    loadLocalGraph,
    clearLocalGraph,
  };
});
