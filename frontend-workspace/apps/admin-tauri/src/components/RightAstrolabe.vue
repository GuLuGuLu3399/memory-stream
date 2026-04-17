<script setup lang="ts">
import { computed, ref, shallowRef, watch, onMounted, onUnmounted, defineAsyncComponent } from "vue";
import { useVueFlow, type Connection } from "@vue-flow/core";
import { invoke } from "@tauri-apps/api/core";
import "@vue-flow/core/dist/style.css";
import "@vue-flow/core/dist/theme-default.css";
import "@vue-flow/controls/dist/style.css";
import { useKnowledgeStore } from "../stores/knowledge";
import { useConfirmDialog } from "../composables/useConfirmDialog";
import { hexForKey } from "../composables/useCategoryTheme";
import { storeToRefs } from "pinia";

const VueFlow = defineAsyncComponent(() => import("@vue-flow/core").then(m => m.VueFlow));
const Background = defineAsyncComponent(() => import("@vue-flow/background").then(m => m.Background));
const Controls = defineAsyncComponent(() => import("@vue-flow/controls").then(m => m.Controls));

const store = useKnowledgeStore();
const { localNodes, localEdges, activeCard, orphanCards, recentCards, categories } = storeToRefs(store);

const { onConnect } = useVueFlow();
const { confirm } = useConfirmDialog();

// ===== Edge selection & context menu state =====
interface SelectedEdge {
  source: string;
  target: string;
  relation: string;
}
const selectedEdge = ref<SelectedEdge | null>(null);
const contextMenu = ref<{ x: number; y: number; edge: SelectedEdge } | null>(null);

// ── 聚光灯：hover 节点时只照亮直连边 ──
const hoveredNodeId = ref<string | null>(null);

function onNodeMouseEnter(event: { node: { id: string } }) {
    hoveredNodeId.value = event.node.id;
}

function onNodeMouseLeave() {
    hoveredNodeId.value = null;
}

const showSummon = ref(false);
const summonQuery = ref("");
const summonedNodeIds = ref<Set<string>>(new Set());
const summonLoading = ref(false);
const summonError = ref("");

interface SummonCardItem {
  id: string;
  title: string;
  x: number;
  y: number;
}

interface CardListPage {
  data: SummonCardItem[];
  next_cursor?: string | null;
}

const summonPool = shallowRef<SummonCardItem[]>([]);

async function loadSummonPool() {
  summonLoading.value = true;
  summonError.value = "";
  try {
    const cards: SummonCardItem[] = [];
    let cursor: string | null = null;
    for (let i = 0; i < 20; i += 1) {
      const endpoint: string = cursor
        ? `/cards?limit=200&cursor=${encodeURIComponent(cursor)}`
        : "/cards?limit=200";
      const page: CardListPage = await invoke("api_request", {
        method: "GET",
        endpoint,
      }) as CardListPage;
      cards.push(...(page.data || []));
      cursor = page.next_cursor || null;
      if (!cursor) break;
    }
    summonPool.value = cards.length > 1000 ? cards.slice(-1000) : cards;
  } catch (e) {
    summonError.value = e instanceof Error ? e.message : String(e);
    summonPool.value = [];
  } finally {
    summonLoading.value = false;
  }
}

// Cards available for summoning (orphans not already on canvas)
const summonableCards = computed(() => {
  const visibleIds = new Set(localNodes.value.map((n) => n.id));
  const allCards = summonPool.value.length > 0
    ? summonPool.value
    : [...orphanCards.value, ...recentCards.value];
  // dedupe by id
  const seen = new Set<string>();
  const unique = allCards.filter((c) => {
    if (seen.has(c.id) || visibleIds.has(c.id)) return false;
    seen.add(c.id);
    return true;
  });
  if (!summonQuery.value) return unique.slice(0, 10);
  const q = summonQuery.value.toLowerCase();
  return unique.filter((c) => c.title.toLowerCase().includes(q)).slice(0, 10);
});

const categoryColorMap = computed(() => {
  const map = new Map<number, string>();
  for (const cat of categories.value) {
    const hex = hexForKey(cat.theme_color);
    if (hex) map.set(cat.id, hex);
  }
  return map;
});

function summonNode(card: { id: string; title: string; x: number; y: number }) {
  // Add to localNodes at center-ish position
  const maxY = localNodes.value.reduce((max, n) => Math.max(max, n.y), 0);
  localNodes.value = [
    ...localNodes.value,
    { id: card.id, title: card.title, x: 200, y: maxY + 120 },
  ];
  summonedNodeIds.value.add(card.id);
  summonQuery.value = "";
  showSummon.value = false;
  store.addToast(`已召唤「${card.title}」至画布`, "success");
}

function toggleSummonPanel() {
  showSummon.value = !showSummon.value;
  if (showSummon.value) {
    loadSummonPool();
    setTimeout(() => {
      const input = document.querySelector('.summon-panel input') as HTMLInputElement | null;
      input?.focus();
    }, 50);
  }
}

// ===== Dagre Worker — layout off main thread =====

const layoutPositions = shallowRef<Record<string, { x: number; y: number }>>({});
let layoutWorker: Worker | null = null;
let layoutTimer: ReturnType<typeof setTimeout> | null = null;

function getLayoutWorker(): Worker {
  if (!layoutWorker) {
    layoutWorker = new Worker(
      new URL("./dagre.worker.ts", import.meta.url),
      { type: "module" },
    );
    layoutWorker.onmessage = (e: MessageEvent<{ positions: Record<string, { x: number; y: number }> }>) => {
      layoutPositions.value = e.data.positions;
    };
  }
  return layoutWorker;
}

function terminateLayoutWorker() {
  if (layoutTimer) {
    clearTimeout(layoutTimer);
    layoutTimer = null;
  }
  if (layoutWorker) {
    layoutWorker.terminate();
    layoutWorker = null;
  }
}

const flowNodes = computed(() => {
  return localNodes.value.map((n, idx) => {
    const pos = layoutPositions.value[n.id];
    const isActive = n.id === activeCard.value?.id;

    const allCards = [...recentCards.value, ...orphanCards.value];
    const card = allCards.find(c => c.id === n.id);
    const catColor = card?.category_id ? categoryColorMap.value.get(card.category_id) : null;

    const fallbackX = (idx % 5) * 180;
    const fallbackY = Math.floor(idx / 5) * 100;

    return {
      id: n.id,
      label: n.title.length > 12 ? n.title.slice(0, 12) + "..." : n.title,
      position: pos ?? { x: fallbackX, y: fallbackY },
      class: [
        'astrolabe-node',
        isActive ? 'astrolabe-node--active' : '',
        catColor && !isActive ? 'astrolabe-node--category' : '',
        !isActive && !catColor ? 'astrolabe-node--default' : '',
      ].filter(Boolean).join(' '),
      style: catColor && !isActive ? { '--cat-color': catColor } as Record<string, string> : {},
    };
  });
});

// Debounced layout: only recompute when nodes/edges change
watch(
  [localNodes, localEdges],
  () => {
    if (layoutTimer) clearTimeout(layoutTimer);
    layoutTimer = setTimeout(() => {
      const worker = getLayoutWorker();
      worker.postMessage({
        nodes: localNodes.value.map(n => ({ id: n.id })),
        edges: localEdges.value.map(e => ({ source: e.source, target: e.target })),
      });
    }, 150);
  },
  { immediate: true },
);

// 🌟 过滤幽灵边 + 唯一 ID + selectable + 选中高亮
const flowEdges = computed(() => {
  const validNodeIds = new Set(localNodes.value.map((n) => n.id));

  const allCards = [...recentCards.value, ...orphanCards.value];
  const cardCatMap = new Map<string, number | null>();
  for (const c of allCards) {
    cardCatMap.set(c.id, c.category_id ?? null);
  }

  // Detect bidirectional edge pairs (A→B AND B→A)
  const edgePairCount = new Map<string, number>()
  for (const e of localEdges.value) {
    if (validNodeIds.has(e.source) && validNodeIds.has(e.target)) {
      const pairKey = [e.source, e.target].sort().join('::')
      edgePairCount.set(pairKey, (edgePairCount.get(pairKey) || 0) + 1)
    }
  }

  return localEdges.value
    .filter((e) => validNodeIds.has(e.source) && validNodeIds.has(e.target))
    .map((e, i) => {
      const isSelected =
        selectedEdge.value?.source === e.source &&
        selectedEdge.value?.target === e.target;

      const sourceCatId = cardCatMap.get(e.source);
      const edgeColor = sourceCatId ? categoryColorMap.value.get(sourceCatId) : null;

      const pairKey = [e.source, e.target].sort().join('::')
      const isBidirectional = (edgePairCount.get(pairKey) || 0) > 1

      let stroke: string;
      let strokeWidth: number;

      if (isSelected) {
        stroke = edgeColor || "#00e5ff";
        strokeWidth = 3;
      } else if (isBidirectional) {
        stroke = edgeColor || "#d97706";
        strokeWidth = 2.5;
      } else if (e.relation === "sequence") {
        stroke = edgeColor || "#00e5ff";
        strokeWidth = 2;
      } else {
        stroke = edgeColor || "#555555";
        strokeWidth = 1;
      }

      // Spotlight: dim all edges, brighten connected ones
      const isConnected = hoveredNodeId.value !== null &&
        (e.source === hoveredNodeId.value || e.target === hoveredNodeId.value);
      const opacity = hoveredNodeId.value !== null
        ? (isConnected ? 1 : 0.15)
        : 1;

      return {
        id: `edge-${e.source}-${e.target}-${e.relation || "rel"}-${i}`,
        source: e.source,
        target: e.target,
        type: "default",
        selectable: true,
        // 不用 Vue Flow animated（底层是 animated stroke-dasharray，导致主干变虚线）
        animated: false,
        class: [
          e.relation === "sequence" ? 'astrolabe-edge--sequence' : '',
          isSelected && e.relation === "sequence" ? 'astrolabe-edge--animated' : '',
          isBidirectional ? 'astrolabe-edge--bidirectional' : '',
        ].filter(Boolean).join(' '),
        style: {
          stroke,
          strokeWidth,
          opacity,
          // 参考线始终虚线，主干始终实线
          ...(e.relation !== "sequence" ? { strokeDasharray: "5 5" } : {}),
        },
      };
    });
});

onConnect((params: Connection) => {
  // Always create sequence edges by default. Prohibit creating reference edges via drag.
  const p = params as Connection & { relation?: string; edge?: { relation?: string } };
  const potentialRelation = p.relation || p.edge?.relation || "sequence";
  if (potentialRelation === "reference" || p.relation === "reference") {
    // Notify user that reference edges are auto-generated from wikilinks and cannot be created via drag
    if (store?.addToast) {
      store.addToast("参考连线由内容中的 [[wikilinks]] 自动生成，无法手动创建。", "info");
    } else {
      // Fallback toast
      window.alert("参考连线由内容中的 [[wikilinks]] 自动生成，无法手动创建。");
    }
    return;
  }
  // Default to sequence edge creation
  store.createEdgeHttp(params.source, params.target, "sequence");
});

// ===== Pane click → clear selection =====
function onPaneClick() {
  selectedEdge.value = null;
  contextMenu.value = null;
}

function onNodeClick(event: { node: { id: string } }) {
  const nodeId = event.node.id;
  // Close context menu on node click
  contextMenu.value = null;
  selectedEdge.value = null;
  if (nodeId && nodeId !== activeCard.value?.id) {
    // 从后端拉取完整卡片数据（含 content），而非使用图谱节点的空 content
    store.loadAndActivateCard(nodeId);
  }
}

// ===== Edge click → select =====
function onEdgeClick(event: { edge: { source: string; target: string } }) {
  const edge = localEdges.value.find(
    (e) => e.source === event.edge.source && e.target === event.edge.target,
  );
  if (edge) {
    selectedEdge.value = { ...edge };
  }
  contextMenu.value = null;
}

// ===== Edge right-click → context menu =====

function onEdgeContextMenu(event: unknown) {
  const ev = (event as { event: { preventDefault(): void; stopPropagation(): void; clientX: number; clientY: number } }).event;
  ev.preventDefault();
  ev.stopPropagation();
  const edgeData = (event as { edge: { source: string; target: string } }).edge;
  const edge = localEdges.value.find(
    (e: { source: string; target: string }) => e.source === edgeData.source && e.target === edgeData.target,
  );
  if (edge) {
    selectedEdge.value = { ...edge };
    contextMenu.value = {
      x: ev.clientX,
      y: ev.clientY,
      edge: { ...edge },
    };
  }
}

// ===== Context menu actions =====
function setEdgeType(relation: string) {
  if (!contextMenu.value) return;
  const { source, target } = contextMenu.value.edge;
  store.updateEdgeType(source, target, relation);
  closeContextMenu();
}

async function handleDeleteEdge() {
  if (!selectedEdge.value) return;
  const { source, target } = selectedEdge.value;
  const ok = await confirm("确定要断开这条连线吗？", {
    title: "断开连线",
    confirmText: "断开",
    danger: true,
  });
  if (ok) {
    store.deleteEdgeHttp(source, target);
    selectedEdge.value = null;
  }
  closeContextMenu();
}

function closeContextMenu() {
  contextMenu.value = null;
}

// ===== Double-click empty canvas → open summon panel =====
function onPaneDblClick() {
  showSummon.value = true;
  loadSummonPool();
  // Focus search input after panel opens
  setTimeout(() => {
    const input = document.querySelector('.summon-panel input') as HTMLInputElement;
    input?.focus();
  }, 50);
}

// ===== Click outside to close summon panel =====
function handleSummonPanelClick(event: MouseEvent) {
  const target = event.target as HTMLElement;
  // Close if clicking outside the panel content
  if (target.closest('.summon-panel-content')) return;
  showSummon.value = false;
}

// ===== Keyboard shortcuts: Delete/Backspace =====
function handleKeydown(e: KeyboardEvent) {
  if ((e.key === "Delete" || e.key === "Backspace") && selectedEdge.value) {
    // Don't delete if user is typing in an input/textarea
    const tag = (e.target as HTMLElement)?.tagName?.toLowerCase();
    if (tag === "input" || tag === "textarea") return;
    e.preventDefault();
    handleDeleteEdge();
  }
  if (e.key === "Escape") {
    selectedEdge.value = null;
    closeContextMenu();
    showSummon.value = false;
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});
onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
  terminateLayoutWorker();
});
</script>

<template>
  <aside class="w-[380px] bg-ms-panel flex flex-col border-l border-ms-border shrink-0">

    <div class="h-12 flex items-center px-4 border-b border-ms-border bg-ms-carbon shrink-0">
      <span class="font-bold text-slate-400 text-xs font-mono tracking-wider">局部拓扑</span>
      <span class="ml-2 text-2xs text-slate-600 font-mono">双击空白召唤 · 右击连线操作</span>
      <span v-if="selectedEdge" class="ml-auto text-xs bg-neon/10 text-neon px-2 py-0.5 rounded-sm">
        已选中连线
      </span>
    </div>
    <div class="flex-1 relative" @click="closeContextMenu">
      <!-- 空状态保护：有节点才渲染 VueFlow -->
      <VueFlow v-if="flowNodes.length > 0" :nodes="flowNodes" :edges="flowEdges" fit-view-on-init
        class="astrolabe-canvas" @node-click="onNodeClick" @edge-click="onEdgeClick"
        @edge-context-menu="onEdgeContextMenu" @pane-click="onPaneClick" @pane-dbl-click="onPaneDblClick"
        @node-mouse-enter="onNodeMouseEnter" @node-mouse-leave="onNodeMouseLeave">
        <Background pattern-color="#333" :gap="16" />
        <Controls />
      </VueFlow>
      <div v-else class="flex-1 flex items-center justify-center h-full text-slate-500 text-sm">
        暂无图谱数据
      </div>
      <div class="absolute top-4 right-4 z-10 flex gap-1.5">
        <!-- Summon button -->
        <button @click.stop="toggleSummonPanel" title="召唤节点至画布"
          class="flex items-center justify-center w-8 h-8 bg-ms-carbon/90 backdrop-blur border border-ms-border rounded-sm shadow-sm text-slate-500 hover:text-neon hover:border-neon/30 transition-all">
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </button>
      </div>

      <!-- Summon Panel Dropdown -->
      <Teleport to="body">
        <div v-if="showSummon" class="summon-panel fixed inset-0 z-dropdown" @click="handleSummonPanelClick">
          <div
            class="summon-panel-content fixed right-[400px] top-1/2 -translate-y-1/2 w-72 bg-ms-carbon rounded-sm shadow-2xl border border-ms-border overflow-hidden"
            @click.stop>
            <!-- Search input -->
            <div class="p-3 border-b border-ms-border bg-ms-deep">
              <input v-model="summonQuery" placeholder="搜索卡片..."
                class="summon-search w-full text-sm px-3 py-2 border border-ms-border rounded-sm outline-none bg-ms-panel text-slate-300 placeholder-slate-600 font-mono" />
            </div>
            <!-- Card list -->
            <div class="max-h-64 overflow-y-auto p-2">
              <div v-if="summonLoading" class="text-xs text-slate-600 px-3 py-3 text-center font-mono">
                正在加载可召唤卡片...
              </div>
              <div v-else-if="summonError" class="text-xs text-ms-danger px-3 py-3 text-center font-mono">
                召唤列表加载失败: {{ summonError }}
              </div>
              <button v-for="card in summonableCards" :key="card.id" @click="summonNode(card)"
                class="summon-card w-full text-left px-3 py-2.5 text-sm rounded-sm text-slate-400 truncate transition font-mono">
                {{ card.title || "无标题" }}
              </button>
              <div v-if="!summonLoading && !summonError && summonableCards.length === 0"
                class="summon-empty text-xs text-slate-600 px-3 py-4 text-center font-mono">
                无可召唤的卡片
              </div>
            </div>
          </div>
        </div>
      </Teleport>

    </div>

    <!-- Edge Context Menu (teleported to body for correct positioning) -->
    <Teleport to="body">
      <div v-if="contextMenu"
        class="astrolabe-context-menu fixed z-dropdown bg-ms-carbon rounded-sm shadow-xl border border-ms-border py-0.5 min-w-[148px]"
        :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }" @click.stop>
        <button @click="setEdgeType('sequence')"
          class="w-full text-left px-2.5 py-1.5 text-xs hover:bg-ms-surface flex items-center gap-2 transition"
          :class="contextMenu.edge.relation === 'sequence' ? 'astrolabe-context-item--active' : 'text-slate-400'">
          <span class="w-2 h-2 rounded-sm bg-neon"></span>
          设为主干 (Sequence)
        </button>
        <!-- Reference edge setting removed: reference edges are not user-configurable -->
        <div class="border-t border-ms-border my-1"></div>
        <button @click="handleDeleteEdge"
          class="w-full text-left px-2.5 py-1.5 text-xs text-ms-danger hover:bg-ms-danger/10 flex items-center gap-2 transition">
          <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
          断开连接
        </button>
      </div>
    </Teleport>
    <!-- 连线类型图例 -->
    <div class="px-3 pb-2 text-2xs font-mono">
      <div class="legend-item">━━ 实线 = 主干（手动连接）</div>
      <div class="legend-item legend-item--bidi">&#9646;&#8644;&#9646; 琥珀 = 双向连接</div>
    </div>
  </aside>
</template>

<style scoped>
/* ===== Brushed-metal canvas ===== */
.astrolabe-canvas {
  background-color: #0d0d0d;
  background-image:
    radial-gradient(ellipse at 20% 30%, rgba(255, 255, 255, 0.02) 0%, transparent 50%),
    radial-gradient(ellipse at 80% 70%, rgba(201, 168, 76, 0.015) 0%, transparent 50%),
    radial-gradient(ellipse at 50% 50%, rgba(0, 229, 255, 0.01) 0%, transparent 70%);
}

/* ===== Node base styles ===== */
.astrolabe-node {
  font-family: 'JetBrains Mono', 'Fira Code', Consolas, monospace;
  font-size: 11px;
  padding: 6px 10px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.astrolabe-node--default {
  background-color: rgba(42, 42, 42, 0.3);
  color: #cbd5e1;
  border: 1px solid #2a2a2a;
}

.astrolabe-node--active {
  background-color: rgba(0, 229, 255, 0.05);
  color: #00e5ff;
  font-weight: bold;
  border: 1px solid #00e5ff;
  box-shadow: 0 0 8px rgba(0, 229, 255, 0.2), 0 0 16px rgba(0, 229, 255, 0.1);
}

.astrolabe-node--category {
  background-color: rgba(var(--cat-rgb, 200, 200, 200), 0.05);
  border: 1px solid var(--cat-color, #666);
  box-shadow: 0 0 6px var(--cat-color-alpha, rgba(100, 100, 100, 0.3));
}

/* ===== Neon flowing edge animation (only selected edges) ===== */
:deep(.vue-flow__edge-path) {
    transition: opacity 0.25s ease;
}

.astrolabe-edge--sequence {
  /* static visual indicator for sequence edges */
}

.astrolabe-edge--animated {
  animation: edgeFlow 1.5s linear infinite;
}

@keyframes edgeFlow {
  0% {
    stroke-dashoffset: 0;
  }

  100% {
    stroke-dashoffset: -20;
  }
}

/* ===== Bidirectional edge glow ===== */
.astrolabe-edge--bidirectional {
  filter: drop-shadow(0 0 4px rgba(217, 119, 6, 0.5));
}

/* ===== Edge type legend (brass color scheme) ===== */
.legend-item {
  color: #c9a84c;
  opacity: 0.7;
}

.legend-item--bidi {
  color: #d97706;
  opacity: 1;
}

/* ===== Summon panel ===== */
.summon-search {
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}

.summon-search:focus {
  border-color: #c9a84c;
  box-shadow: 0 0 0 2px rgba(201, 168, 76, 0.1);
}

.summon-card {
  transition: background-color 0.15s ease;
}

.summon-card:hover {
  background-color: rgba(0, 229, 255, 0.1);
  color: #00e5ff;
}

.summon-empty {
  transition: color 0.2s ease;
}

/* ===== Edge context menu ===== */
.astrolabe-context-item--active {
  color: #00e5ff;
  font-weight: 500;
}

.astrolabe-context-menu::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, transparent, #c9a84c, transparent);
  opacity: 0.5;
}

.astrolabe-context-menu::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, #c9a84c, transparent);
  opacity: 0.3;
}

/* ===== Category color parsing for node styles ===== */
/* Note: CSS custom properties for RGB values are set inline via style attribute */
</style>
