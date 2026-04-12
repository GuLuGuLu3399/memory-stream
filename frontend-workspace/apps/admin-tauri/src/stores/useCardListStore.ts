/**
 * @module useCardListStore
 *
 * 卡片列表 Store — Orphan 和 Recent 卡片列表管理
 *
 * 所有网络请求通过 Rust 的 api_request IPC 命令发出。
 */

import { defineStore } from "pinia";
import { ref, shallowRef, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { PaginatedResponse, CardListItem } from "@memory-stream/types";

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
  const orphanCards = shallowRef<CardItem[]>([]);
  const recentCards = shallowRef<CardItem[]>([]);
  const searchQuery = ref("");
  const debouncedQuery = ref("");
  let _debounceTimer: ReturnType<typeof setTimeout> | null = null;

  watch(searchQuery, (v) => {
    if (_debounceTimer) clearTimeout(_debounceTimer);
    _debounceTimer = setTimeout(() => {
      debouncedQuery.value = v;
    }, 300);
  });

  // 当前选中的分类 ID，用于对卡片列表进行分类筛选
  const selectedCategoryId = ref<number | null>(null); // null = show all

  // ---- 计算属性 ----

  /** 按搜索关键词过滤的孤儿卡片 */
  const filteredOrphans = computed(() => {
    let result = orphanCards.value;
    if (debouncedQuery.value) {
      const q = debouncedQuery.value.toLowerCase();
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
    if (debouncedQuery.value) {
      const q = debouncedQuery.value.toLowerCase();
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
      const result = await invoke<PaginatedResponse<CardListItem>>("api_request", {
        method: "GET",
        endpoint: "/cards/discover",
      });
      orphanCards.value = result.data.map((c) => ({
        id: c.id,
        title: c.title || "无标题",
        content: c.excerpt || c.raw_md || "",
        x: c.x || 0,
        y: c.y || 0,
        updated_at: c.updated_at,
        category_id: c.category_id,
      }));
    } catch (e) {
      console.error("[CardListStore] loadOrphans failed:", e);
    }
  }

  /** 加载最近编辑的卡片列表（匹配 Go PaginatedResult 结构） */
  async function loadRecent() {
    try {
      const result = await invoke<PaginatedResponse<CardListItem>>("api_request", {
        method: "GET",
        endpoint: "/cards",
      });
      recentCards.value = result.data.map((c) => ({
        id: c.id,
        title: c.title || "无标题",
        content: c.excerpt || c.raw_md || "",
        x: c.x || 0,
        y: c.y || 0,
        updated_at: c.updated_at,
        category_id: c.category_id,
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
