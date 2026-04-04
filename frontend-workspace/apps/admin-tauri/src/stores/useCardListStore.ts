/**
 * @module useCardListStore
 *
 * 卡片列表 Store — Orphan 和 Recent 卡片列表管理
 *
 * 所有网络请求通过 Rust 的 api_request IPC 命令发出。
 */

import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

/** 侧边栏卡片列表项 — Card 的精简表示 */
export interface CardItem {
  id: string;
  title: string;
  content: string;
  x: number;
  y: number;
  updated_at?: string;
  category_id?: number | null;
}

export const useCardListStore = defineStore("cardList", () => {
  // ---- 响应式状态 ----
  const orphanCards = ref<CardItem[]>([]);
  const recentCards = ref<CardItem[]>([]);
  const searchQuery = ref("");
  // 当前选中的分类 ID，用于对卡片列表进行分类筛选
  const selectedCategoryId = ref<number | null>(null); // null = show all

  // ---- 计算属性 ----

  /** 按搜索关键词过滤的孤儿卡片 */
  const filteredOrphans = computed(() => {
    let result = orphanCards.value;
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter((c) => c.title.toLowerCase().includes(q));
    }
    if (selectedCategoryId.value !== null) {
      result = result.filter((c) => c.category_id === selectedCategoryId.value);
    }
    return result;
  });

  /** 按搜索关键词过滤的最近卡片 */
  const filteredRecent = computed(() => {
    let result = recentCards.value;
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter((c) => c.title.toLowerCase().includes(q));
    }
    if (selectedCategoryId.value !== null) {
      result = result.filter((c) => c.category_id === selectedCategoryId.value);
    }
    return result;
  });

  // ============================================================================
  // 卡片列表加载 — 全部通过 Rust api_request 网关
  // ============================================================================

  /** 加载孤儿卡片（未建立关联的卡片，匹配 Go Discover PaginatedResult） */
  async function loadOrphans() {
    try {
      const data = await invoke<Record<string, unknown>>("api_request", {
        method: "GET",
        endpoint: "/cards/discover",
      });
      // Go GetDiscover 返回 { data: [...], has_more, total_count }
      const list = (data.data || data.cards || []) as Record<string, unknown>[];
      orphanCards.value = list.map((c) => ({
        id: c.id as string,
        title: (c.title as string) || "无标题",
        content: (c.excerpt as string) || (c.raw_md as string) || "",
        x: (c.x as number) || 0,
        y: (c.y as number) || 0,
        updated_at: c.updated_at as string,
        category_id: c.category_id != null ? (c.category_id as number) : null,
      }));
    } catch (e) {
      console.error("[CardListStore] loadOrphans failed:", e);
    }
  }

  /** 加载最近编辑的卡片列表（匹配 Go PaginatedResult 结构） */
  async function loadRecent() {
    try {
      const data = await invoke<Record<string, unknown>>("api_request", {
        method: "GET",
        endpoint: "/cards",
      });
      // Go 返回 { data: [...], has_more, next_cursor, total_count }
      const list = (data.data || data.cards || []) as Record<string, unknown>[];
      recentCards.value = list.map((c) => ({
        id: c.id as string,
        title: (c.title as string) || "无标题",
        content: (c.excerpt as string) || (c.raw_md as string) || "",
        x: (c.x as number) || 0,
        y: (c.y as number) || 0,
        updated_at: c.updated_at as string,
        category_id: c.category_id != null ? (c.category_id as number) : null,
      }));
    } catch (e) {
      console.error("[CardListStore] loadRecent failed:", e);
    }
  }

  return {
    orphanCards,
    recentCards,
    searchQuery,
    filteredOrphans,
    filteredRecent,
    selectedCategoryId,
    loadOrphans,
    loadRecent,
  };
});
