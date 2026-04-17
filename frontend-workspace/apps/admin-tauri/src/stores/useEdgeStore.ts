/**
 * @module useEdgeStore
 *
 * 边操作 Store — Edge HTTP 和 WebSocket 操作
 *
 * 所有网络请求通过 Rust 的 api_request IPC 或 WebSocket 命令发出。
 */

import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "./useToast";
import { extractMsg } from "../composables/useTempleError";

export const useEdgeStore = defineStore("edge", () => {
  const toast = useToast();

  // ============================================================================
  // Edge HTTP 操作 — 全部通过 Rust api_request 网关
  // ============================================================================

  /** 通过 HTTP 创建图谱边 */
  async function createEdgeHttp(
    sourceId: string,
    targetId: string,
    relationType: string = "reference",
  ) {
    try {
      await invoke("api_request", {
        method: "POST",
        endpoint: "/edges",
        body: {
          source_id: sourceId,
          target_id: targetId,
          relation_type: relationType,
        },
      });
      toast.addToast("连线已创建 ✓", "success");
    } catch (e) {
      toast.addToast("创建连线失败: " + extractMsg(e), "error");
      throw e;
    }
  }

  /** 通过 HTTP 删除图谱边 */
  async function deleteEdgeHttp(sourceId: string, targetId: string) {
    try {
      await invoke("api_request", {
        method: "DELETE",
        endpoint: "/edges",
        body: { source_id: sourceId, target_id: targetId },
      });
      toast.addToast("连线已断开 ✓", "success");
    } catch (e) {
      toast.addToast("删除连线失败: " + extractMsg(e), "error");
      throw e;
    }
  }

  /** 更新图谱边的关联类型 */
  async function updateEdgeType(
    sourceId: string,
    targetId: string,
    relationType: string,
  ) {
    try {
      await invoke("api_request", {
        method: "PATCH",
        endpoint: "/edges",
        body: {
          source_id: sourceId,
          target_id: targetId,
          relation_type: relationType,
        },
      });
      toast.addToast("连线类型已更新 ✓", "success");
    } catch (e) {
      toast.addToast("更新连线失败: " + extractMsg(e), "error");
      throw e;
    }
  }

  // ============================================================================
  // WebSocket 边操作（保留，用于实时同步场景）
  // ============================================================================

  /** 通过 WebSocket 创建边（实时同步） */
  async function createEdge(source: string, target: string, relation: string) {
    try {
      await invoke("create_edge_cmd", { source, target, rel: relation });
    } catch (e) {
      console.error("[EdgeStore] createEdge failed:", e);
      throw e;
    }
  }

  /** 通过 WebSocket 删除边（实时同步） */
  async function deleteEdge(source: string, target: string) {
    try {
      await invoke("delete_edge_cmd", { source, target });
    } catch (e) {
      console.error("[EdgeStore] deleteEdge failed:", e);
      throw e;
    }
  }

  return {
    createEdgeHttp,
    deleteEdgeHttp,
    updateEdgeType,
    createEdge,
    deleteEdge,
  };
});
