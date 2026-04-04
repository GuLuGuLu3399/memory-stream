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

export const useLocalGraphStore = defineStore("localGraph", () => {
  // ---- 响应式状态 ----
  const localNodes = ref<LocalGraphNode[]>([]);
  const localEdges = ref<LocalGraphEdge[]>([]);

  /** 加载指定卡片的局部图谱（邻居节点 + 边） */
  async function loadLocalGraph(cardId: string) {
    if (!cardId) return;
    try {
      const data = await invoke<Record<string, unknown>>("api_request", {
        method: "GET",
        endpoint: `/graph/detail/${cardId}`,
      });
      const nodes = (data.nodes || []) as Record<string, unknown>[];
      const edges = (data.edges || []) as Record<string, unknown>[];
      localNodes.value = nodes.map((n) => ({
        id: n.id as string,
        title: (n.title as string) || (n.id as string),
        x: 0,
        y: 0,
      }));
      localEdges.value = edges.map((e) => ({
        source: e.source as string,
        target: e.target as string,
        relation: (e.relation as string) || "reference",
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
