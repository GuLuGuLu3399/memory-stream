/**
 * 🌟 useGraphStore — 全局图谱状态管理 (Pinia)
 *
 * 单一真相来源 (SSOT)，统一管理：
 * - 当前选中节点
 * - 高亮节点（双向悬停预留）
 * - 缩放等级 + 语义层级
 * - 视图模式 (list / graph)
 * - 浮动指挥岛控制状态（排序/密度/过滤/深度/聚光灯/禅模式）
 */

import { ref, computed } from "vue";
import { defineStore } from "pinia";

export const useGraphStore = defineStore("graph", () => {
  // ── 核心状态 ──
  const selectedId = ref<string | null>(null);
  const highlightedId = ref<string | null>(null);
  const zoomLevel = ref(1.0);
  const viewMode = ref<"list" | "graph">("graph");

  // ── 列表视图控制 ──
  /** 排序方式：按更新时间 / 按热度 */
  const sortBy = ref<"updated" | "hot">("updated");
  /** 列表密度：舒适（带摘要）/ 紧凑（仅标题） */
  const density = ref<"cozy" | "compact">("cozy");
  /** 分类过滤：null = 全部 */
  const categoryFilter = ref<string | null>(null);

  // ── 图谱视图控制 ──
  /** 图谱发散深度 (1-3) */
  const graphDepth = ref(3);
  /** 聚光灯模式：仅高亮 hover 节点及其直接连线 */
  const spotlightMode = ref(false);

  // ── 全局 UI 状态 ──
  /** 禅模式：全屏阅读 + TOC */
  const zenMode = ref(false);
  /** Cmd+K 命令面板 */
  const commandPaletteOpen = ref(false);

  // ── Actions ──
  function selectNode(id: string | null) {
    selectedId.value = id;
  }

  function highlightNode(id: string | null) {
    highlightedId.value = id;
  }

  function setZoomLevel(zoom: number) {
    zoomLevel.value = Math.max(0.1, Math.min(4.0, zoom));
  }

  function setViewMode(mode: "list" | "graph") {
    viewMode.value = mode;
  }

  function setSortBy(sort: "updated" | "hot") {
    sortBy.value = sort;
  }

  function setDensity(d: "cozy" | "compact") {
    density.value = d;
  }

  function setCategoryFilter(cat: string | null) {
    categoryFilter.value = cat;
  }

  function setGraphDepth(depth: number) {
    graphDepth.value = Math.max(1, Math.min(3, depth));
  }

  function toggleSpotlight() {
    spotlightMode.value = !spotlightMode.value;
  }

  function toggleZenMode() {
    zenMode.value = !zenMode.value;
  }

  function toggleCommandPalette() {
    commandPaletteOpen.value = !commandPaletteOpen.value;
  }

  // ── Getters ──

  /**
   * 根据缩放等级返回语义层级
   * - outline: 缩小（≤0.5x），仅显示分类大纲
   * - summary: 正常（0.5x~1.2x），显示标题+日期
   * - detail: 放大（≥1.2x），展开 Markdown 预览
   */
  const semanticTier = computed((): "outline" | "summary" | "detail" => {
    if (zoomLevel.value <= 0.5) return "outline";
    if (zoomLevel.value >= 1.2) return "detail";
    return "summary";
  });

  return {
    // 核心状态
    selectedId,
    highlightedId,
    zoomLevel,
    viewMode,
    semanticTier,
    // 列表控制
    sortBy,
    density,
    categoryFilter,
    // 图谱控制
    graphDepth,
    spotlightMode,
    // UI 状态
    zenMode,
    commandPaletteOpen,
    // Actions
    selectNode,
    highlightNode,
    setZoomLevel,
    setViewMode,
    setSortBy,
    setDensity,
    setCategoryFilter,
    setGraphDepth,
    toggleSpotlight,
    toggleZenMode,
    toggleCommandPalette,
  };
});
