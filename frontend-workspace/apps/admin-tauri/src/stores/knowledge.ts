/**
 * @module knowledge
 *
 * 知识管理核心 Store — Pinia Composition API 风格
 *
 * 所有网络请求已统一通过 Rust 的 `api_request` IPC 命令发出，
 * Vue 前端不再直接访问网络，Rust 充当唯一的 API 网关。
 *
 * 架构优势：
 * - 彻底无视 CORS（桌面端请求由 Rust 发出）
 * - 连接池复用（全局 reqwest::Client）
 * - 未来凭据安全（Token 存储在系统安全区）
 *
 * 子 Store 架构：
 * - useToast: Toast 通知系统
 * - useCategoryStore: 分类 CRUD
 * - useEdgeStore: 边操作
 * - useCardListStore: 卡片列表
 * - useLocalGraphStore: 局部图谱
 */

import { defineStore, storeToRefs } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useConfirmDialog } from "../composables/useConfirmDialog";

// Sub-stores
import { useToast } from "./useToast";
import { useCategoryStore } from "./useCategoryStore";
import { useEdgeStore } from "./useEdgeStore";
import { useCardListStore, type CardItem } from "./useCardListStore";
import { useLocalGraphStore } from "./useLocalGraphStore";

// Re-export types for backward compatibility
export type { ToastMessage } from "./useToast";
export type { LocalGraphNode, LocalGraphEdge } from "./useLocalGraphStore";
// CardItem is already imported above, re-export it
export type { CardItem } from "./useCardListStore";

// ============================================================================
// Store 定义
// ============================================================================

export const useKnowledgeStore = defineStore("knowledge", () => {
  // ---- 初始化子 Store ----
  const toastStore = useToast();
  const categoryStore = useCategoryStore();
  const edgeStore = useEdgeStore();
  const cardListStore = useCardListStore();
  const localGraphStore = useLocalGraphStore();

  // ---- 从子 Store 解构响应式引用 ----
  const { toasts } = storeToRefs(toastStore);
  const { categories } = storeToRefs(categoryStore);
  const { orphanCards, recentCards, searchQuery, filteredOrphans, filteredRecent, selectedCategoryId } =
    storeToRefs(cardListStore);
  const { localNodes, localEdges } = storeToRefs(localGraphStore);

  // ---- 核心状态（保留在主 Store） ----
  const activeCard = ref<CardItem | null>(null);
  const isLoading = ref(false);
  const isDirty = ref(false);
  const justSaved = ref(false);
  const isSaving = ref(false);
  const searchFocused = ref(false);

  interface BacklinkItem {
    source_id: string;
    source_title: string;
    relation_type: string;
    context_snippet?: string;
  }
  const backlinks = ref<BacklinkItem[]>([]);

  // ---- 防抖工具 ----
  let _refreshTimer: ReturnType<typeof setTimeout> | null = null;

  // ---- 脏状态追踪 ----

  let savedSnapshot = "";

  /** 保存当前卡片快照，用于脏检测基线 */
  function captureSnapshot() {
    if (!activeCard.value) return;
    savedSnapshot = JSON.stringify({
      title: activeCard.value.title,
      content: activeCard.value.content,
      category_id: activeCard.value.category_id,
    });
    isDirty.value = false;
  }

  /** 检查当前卡片是否被修改 */
  function checkDirty() {
    if (!activeCard.value) return;
    const current = JSON.stringify({
      title: activeCard.value.title,
      content: activeCard.value.content,
      category_id: activeCard.value.category_id,
    });
    isDirty.value = current !== savedSnapshot;
  }

  /** 导航拦截：如果有未保存的修改，弹窗确认（异步版） */
  async function confirmDirtyDiscard(): Promise<boolean> {
    if (isDirty.value && activeCard.value) {
      const { confirm } = useConfirmDialog();
      return await confirm("当前卡片未保存，是否放弃修改？", {
        title: "未保存的修改",
        confirmText: "放弃修改",
        cancelText: "继续编辑",
        danger: true,
      });
    }
    return true;
  }

  // ---- 卡片激活 ----

  /** 设置当前激活的卡片（不加载完整内容） */
  async function setActiveCard(card: CardItem) {
    if (!(await confirmDirtyDiscard())) return;
    activeCard.value = { ...card };
    captureSnapshot();
    if (card.id) localGraphStore.loadLocalGraph(card.id);
  }

  /**
   * 数据水合：从后端拉取完整 raw_md 再激活卡片
   *
   * 通过 Rust IPC `get_card_detail` 获取完整卡片数据，
   * 包含 raw_md 内容用于编辑器加载。
   */
  async function loadAndActivateCard(cardId: string) {
    if (!cardId) return;
    if (!(await confirmDirtyDiscard())) return;
    isLoading.value = true;
    try {
      const detail = await invoke<Record<string, unknown>>("get_card_detail", {
        id: cardId,
      });
      const card: CardItem = {
        id: detail.id as string,
        title: (detail.title as string) || "无标题",
        content: (detail.raw_md as string) || "",
        x: (detail.x as number) || 0,
        y: (detail.y as number) || 0,
        updated_at: detail.updated_at as string,
        category_id:
          detail.category_id != null ? (detail.category_id as number) : null,
      };
      activeCard.value = card;
      captureSnapshot();
      localGraphStore.loadLocalGraph(cardId);
      fetchBacklinks(cardId);
    } catch (e) {
      console.error("[Store] loadAndActivateCard failed:", e);
      toastStore.addToast("加载卡片失败: " + String(e), "error");
    } finally {
      isLoading.value = false;
    }
  }

  function loadAndActivateCardByTitle(title: string) {
    const allCards = [...recentCards.value, ...orphanCards.value];
    const found = allCards.find((c) => c.title === title);
    if (found) {
      loadAndActivateCard(found.id);
    } else {
      activeCard.value = { id: "", title, content: "", x: 0, y: 0 } as CardItem;
      localNodes.value = [];
      localEdges.value = [];
      isDirty.value = false;
      savedSnapshot = "";
    }
  }

  /** 创建新的空白卡片 */
  function newCard() {
    activeCard.value = { id: "", title: "", content: "", x: 0, y: 0 };
    localNodes.value = [];
    localEdges.value = [];
    isDirty.value = false;
    savedSnapshot = "";
  }

  // ============================================================================
  // 卡片保存 — 混合策略（Rust AST 解析 + Rust HTTP 网关）
  // ============================================================================

  /**
   * 保存当前激活的卡片
   *
   * 流程：
   * 1. 调用 Rust `process_markdown` 本地解析 AST
   * 2. 有 ID → 通过 Rust `update_card` 更新
   * 3. 无 ID → 通过 Rust `create_card_with_relation` 创建
   */
  async function saveCard() {
    if (!activeCard.value || isSaving.value) return;
    isSaving.value = true;
    isLoading.value = true;
    try {
      const card = activeCard.value;

      const renderResult = await invoke<{
        html: string;
        ast_json: string;
        excerpt: string;
        extracted_links: string[];
      }>("process_markdown", { content: card.content });

      // Step 2: 从 AST 提取 TOC 目录树
      let tocData: unknown = null;
      try {
        tocData = await invoke("extract_toc", {
          astJson: renderResult.ast_json,
        });
      } catch (e) {
        console.warn("[Store] extract_toc failed, saving without TOC:", e);
      }

      // Step 3: 智能分流 — 有 ID 更新，无 ID 创建
      if (card.id) {
        // UPDATE 路径 — 通过 Rust api_request 网关
        const body: Record<string, unknown> = {
          title: card.title,
          raw_md: card.content,
          excerpt: renderResult.excerpt,
          ast_data: renderResult.ast_json,
          toc_data: tocData,
          extracted_links: renderResult.extracted_links,
        };
        if (card.category_id != null) {
          body.category_id = card.category_id;
        }
        await invoke("api_request", {
          method: "PUT",
          endpoint: `/cards/${card.id}`,
          body,
        });
      } else {
        // CREATE 路径 — 通过 Rust 专用命令（含关联关系）
        const cardId = await invoke<string>("create_card_with_relation", {
          title: card.title,
          content: card.content,
          astData: renderResult.ast_json,
          excerpt: renderResult.excerpt,
          tocData: tocData ? JSON.stringify(tocData) : null,
          parentId: null,
          relation: "sequence",
        });
        if (cardId) activeCard.value.id = cardId;
      }

      captureSnapshot();
      toastStore.addToast("卡片已保存 ✓", "success");
      justSaved.value = true;
      setTimeout(() => {
        justSaved.value = false;
      }, 1500);
      await refreshWorkspace();
    } catch (e) {
      console.error("[Store] saveCard failed:", e);
      toastStore.addToast("保存失败: " + String(e), "error");
    } finally {
      isSaving.value = false;
      isLoading.value = false;
    }
  }

  /**
   * 删除指定卡片
   *
   * 通过 Rust `delete_card` IPC 命令执行，级联删除关联边和布局数据。
   */
  async function deleteCard(cardId: string) {
    if (!cardId) return;
    isLoading.value = true;
    try {
      await invoke("delete_card", { id: cardId });
      toastStore.addToast("卡片已删除 ✓", "success");
      if (activeCard.value?.id === cardId) {
        activeCard.value = null;
        localNodes.value = [];
        localEdges.value = [];
        isDirty.value = false;
        savedSnapshot = "";
      }
      await refreshWorkspace();
    } catch (e) {
      console.error("[Store] deleteCard failed:", e);
      toastStore.addToast("删除失败: " + String(e), "error");
    } finally {
      isLoading.value = false;
    }
  }

  // ============================================================================
  // 本地草稿（ms-local-draft）— 离线自动保存
  // ============================================================================

  /** 保存当前卡片到本地 SQLite 草稿库 */
  async function saveDraft() {
    if (!activeCard.value?.id) return;
    try {
      await invoke("save_draft", {
        cardId: activeCard.value.id,
        rawMd: activeCard.value.content,
        astData: null as string | null,
      });
    } catch (e) {
      console.error("[Store] saveDraft failed:", e);
    }
  }

  /** 从本地草稿库恢复指定卡片内容 */
  async function loadDraft(cardId: string): Promise<string | null> {
    try {
      const draft = await invoke<{
        card_id: string;
        raw_md: string;
        ast_data: string | null;
        updated_at: number;
      } | null>("load_draft", { cardId });
      return draft?.raw_md ?? null;
    } catch (e) {
      console.error("[Store] loadDraft failed:", e);
      return null;
    }
  }

  /** 列出所有未同步的本地草稿 */
  async function listDrafts(): Promise<
    { card_id: string; raw_md: string; updated_at: number }[]
  > {
    try {
      return await invoke("list_drafts");
    } catch (e) {
      console.error("[Store] listDrafts failed:", e);
      return [];
    }
  }

  /** 删除指定卡片的本地草稿（通常在同步成功后调用） */
  async function deleteDraft(cardId: string) {
    try {
      await invoke("delete_draft", { cardId });
    } catch (e) {
      console.error("[Store] deleteDraft failed:", e);
    }
  }

  // ============================================================================
  // 知识库导出（ms-kb-exporter）
  // ============================================================================

  /**
   * 导出知识库为 ZIP 文件
   *
   * 1. 弹出原生保存对话框选择路径
   * 2. 收集当前卡片数据
   * 3. 调用 Rust 层 ms-kb-exporter 进行 ZIP 打包
   */
  async function exportKb(): Promise<boolean> {
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const destPath = await save({
        filters: [{ name: "Archive", extensions: ["zip"] }],
        defaultPath: "memory-stream-export.zip",
      });
      if (!destPath) return false;

      // 收集所有卡片数据用于导出
      const allCards = [...orphanCards.value, ...recentCards.value];
      const exportCards = allCards.map((c) => ({
        category_name: "未分类",
        title: c.title || "无标题",
        raw_md: c.content || "",
        images: [],
      }));

      const summary = await invoke<{
        total_cards: number;
        total_images: number;
        zip_size_bytes: number;
      }>("export_knowledge_base", {
        cards: exportCards,
        destPath,
      });

      toastStore.addToast(
        `导出完成: ${summary.total_cards} 张卡片, ${(summary.zip_size_bytes / 1024).toFixed(1)} KB`,
        "success",
      );
      return true;
    } catch (e) {
      toastStore.addToast("导出失败: " + String(e), "error");
      return false;
    }
  }

  // ============================================================================
  // 目录树提取（ms-toc-extractor）
  // ============================================================================

  /** 从 AST JSON 提取层级目录树 */
  async function extractToc(
    astJson: string,
  ): Promise<
    { level: number; text: string; slug: string; children: unknown[] }[]
  > {
    try {
      return await invoke("extract_toc", { astJson });
    } catch (e) {
      console.error("[Store] extractToc failed:", e);
      return [];
    }
  }

  // ============================================================================
  // 智能刷新（Smart Refresh）— 精准定向，按需加载
  // ============================================================================

  /**
   * 智能刷新工作区 — 替代原来散落的 Promise.all([loadOrphans(), loadRecent()])
   *
   * 策略：
   * 1. 永远刷新：卡片列表 + 分类（核心数据）
   * 2. 按需刷新：orphan 列表（仅 sidebar 展开时才拉）
   * 3. 按需刷新：局部图谱（仅选中卡片时才拉）
   *
   * 注意：保留 /cards/discover 接口，因为后端游标分页下
   * 本地无法从部分列表准确计算全局 orphans。
   */
   async function refreshWorkspace() {
     const promises: Promise<void>[] = [
      cardListStore.loadRecent(),
      categoryStore.loadCategories(),
    ];

    // 加载 orphans + recent
    promises.push(cardListStore.loadOrphans());

    // 仅选中卡片时才刷新局部图谱
    if (activeCard.value?.id) {
      promises.push(localGraphStore.loadLocalGraph(activeCard.value.id));
    }

    await Promise.allSettled(promises);
  }

  /**
   * 无感刷新 — WS 事件触发后，500ms 防抖静默刷新
   *
   * 不影响当前 activeCard 的编辑状态，只刷新侧边栏和图谱。
   * 多个 WS 事件在 500ms 内到达只会触发一次刷新，避免刷新风暴。
   */
  function silentRefresh() {
    if (_refreshTimer) clearTimeout(_refreshTimer);
    _refreshTimer = setTimeout(() => {
      refreshWorkspace();
    }, 500);
  }

  // ============================================================================
  // 布局更新
  // ============================================================================

  /** 批量更新节点布局坐标 */
  function updateLayouts(layouts: { id: string; x: number; y: number }[]) {
    const map = new Map(layouts.map((l) => [l.id, l]));
    localNodes.value = localNodes.value.map((n) => {
      const p = map.get(n.id);
      return p ? { ...n, x: p.x, y: p.y } : n;
    });
    orphanCards.value = orphanCards.value.map((c) => {
      const p = map.get(c.id);
      return p ? { ...c, x: p.x, y: p.y } : c;
    });
    recentCards.value = recentCards.value.map((c) => {
      const p = map.get(c.id);
      return p ? { ...c, x: p.x, y: p.y } : c;
    });
  }

  // ============================================================================
  // 边操作包装器（添加 refreshWorkspace）
  // ============================================================================

  /** 通过 HTTP 创建图谱边（带刷新） */
  async function createEdgeHttp(
    sourceId: string,
    targetId: string,
    relationType: string = "reference",
  ) {
    try {
      await edgeStore.createEdgeHttp(sourceId, targetId, relationType);
      await refreshWorkspace();
    } catch (e) {
      // Error already handled in edgeStore with toast
    }
  }

  /** 通过 HTTP 删除图谱边（带刷新） */
  async function deleteEdgeHttp(sourceId: string, targetId: string) {
    try {
      await edgeStore.deleteEdgeHttp(sourceId, targetId);
      await refreshWorkspace();
    } catch (e) {
      // Error already handled in edgeStore with toast
    }
  }

  /** 更新图谱边的关联类型（带刷新） */
  async function updateEdgeType(
    sourceId: string,
    targetId: string,
    relationType: string,
  ) {
    try {
      await edgeStore.updateEdgeType(sourceId, targetId, relationType);
      await refreshWorkspace();
    } catch (e) {
      // Error already handled in edgeStore with toast
    }
  }

  /** 将卡片从分类中解绑（设置 category_id 为 null） */
  async function unlinkCardFromCategory(cardId: string) {
    try {
      await invoke("api_request", {
        method: "PUT",
        endpoint: `/cards/${cardId}`,
        body: { category_id: null },
      });
      toastStore.addToast("已从分类中移除", "success");
      await refreshWorkspace();
    } catch (e) {
      console.error("[Store] unlinkCardFromCategory failed:", e);
      toastStore.addToast("移除失败: " + String(e), "error");
    }
  }

  async function updateCardCategory(cardId: string, categoryId: number) {
    try {
      await invoke("api_request", {
        method: "PUT",
        endpoint: `/cards/${cardId}`,
        body: { category_id: categoryId },
      });
      toastStore.addToast("卡片已迁移", "success");
      await refreshWorkspace();
    } catch (e) {
      console.error("[Store] updateCardCategory failed:", e);
      toastStore.addToast("迁移失败: " + String(e), "error");
    }
  }

  async function fetchBacklinks(cardId: string) {
    try {
      const data = await invoke<Record<string, unknown>>("api_request", {
        method: "GET",
        endpoint: `/cards/${cardId}/backlinks`,
      });
      backlinks.value = (data.backlinks as BacklinkItem[]) || [];
    } catch {
      backlinks.value = [];
    }
  }

  // ---- 导出 ----
  return {
    // 核心状态
    activeCard,
    isLoading,
    isSaving,
    isDirty,
    justSaved,
    searchFocused,
    // 从子 Store
    toasts,
    categories,
    orphanCards,
    recentCards,
    localNodes,
    localEdges,
    searchQuery,
    // 计算属性
    filteredOrphans,
    filteredRecent,
    selectedCategoryId,
    // Toast
    addToast: toastStore.addToast,
    // 脏状态
    checkDirty,
    // 卡片激活
    setActiveCard,
    loadAndActivateCard,
    loadAndActivateCardByTitle,
    newCard,
    // Category（委托到子 Store）
    loadCategories: categoryStore.loadCategories,
    createCategory: categoryStore.createCategory,
    updateCategory: categoryStore.updateCategory,
    deleteCategory: categoryStore.deleteCategory,
    // Edge（包装器）
    createEdgeHttp,
    deleteEdgeHttp,
    updateEdgeType,
    // Card List（委托到子 Store）
    loadOrphans: cardListStore.loadOrphans,
    loadRecent: cardListStore.loadRecent,
    // Local Graph（委托到子 Store）
    loadLocalGraph: localGraphStore.loadLocalGraph,
    // Card CRUD
    saveCard,
    deleteCard,
    unlinkCardFromCategory,
    updateCardCategory,
    // Edge WebSocket（委托到子 Store）
    createEdge: edgeStore.createEdge,
    deleteEdge: edgeStore.deleteEdge,
    // 本地草稿
    saveDraft,
    loadDraft,
    listDrafts,
    deleteDraft,
    // 知识库导出
    exportKb,
    // 目录树提取
    extractToc,
    // 刷新
    refreshWorkspace,
    silentRefresh,
    // 布局
    updateLayouts,
    // 反向引力
    backlinks,
    fetchBacklinks,
  };
});
