<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { VueFlow, useVueFlow } from "@vue-flow/core";
import { Background } from "@vue-flow/background";
import { Controls } from "@vue-flow/controls";
import dagre from "dagre";
import "@vue-flow/core/dist/style.css";
import "@vue-flow/core/dist/theme-default.css";
import "@vue-flow/controls/dist/style.css";
import { useKnowledgeStore } from "../stores/knowledge";
import { useConfirmDialog } from "../composables/useConfirmDialog";
import { hexForKey } from "../composables/useCategoryTheme";
import { storeToRefs } from "pinia";

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
const showSummon = ref(false);
const summonQuery = ref("");
const summonedNodeIds = ref<Set<string>>(new Set());

// Cards available for summoning (orphans not already on canvas)
const summonableCards = computed(() => {
  const visibleIds = new Set(localNodes.value.map((n) => n.id));
  const allCards = [...orphanCards.value, ...recentCards.value];
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

const flowNodes = computed(() => {
  const g = new dagre.graphlib.Graph();
  g.setDefaultEdgeLabel(() => ({}));
  g.setGraph({ rankdir: "TB", nodesep: 60, ranksep: 80 });

  const validNodeIds = new Set(localNodes.value.map((n) => n.id));

  for (const n of localNodes.value) {
    g.setNode(n.id, { width: 140, height: 36 });
  }

  for (const e of localEdges.value) {
    if (validNodeIds.has(e.source) && validNodeIds.has(e.target)) {
      g.setEdge(e.source, e.target);
    }
  }

  dagre.layout(g);

  return localNodes.value.map((n, idx) => {
    const nodePos = g.node(n.id);
    const isActive = n.id === activeCard.value?.id;

    const allCards = [...recentCards.value, ...orphanCards.value];
    const card = allCards.find(c => c.id === n.id);
    const catColor = card?.category_id ? categoryColorMap.value.get(card.category_id) : null;

    const baseStyle: Record<string, string> = {
      fontFamily: '"JetBrains Mono", "Fira Code", Consolas, monospace',
      fontSize: "11px",
      padding: "6px 10px",
      borderRadius: "2px",
      cursor: "pointer",
    };

    if (isActive) {
      baseStyle.background = "rgba(0, 229, 255, 0.05)";
      baseStyle.color = "#00e5ff";
      baseStyle.fontWeight = "bold";
      baseStyle.border = "1px solid #00e5ff";
      baseStyle.boxShadow = "0 0 8px rgba(0, 229, 255, 0.2)";
    } else if (catColor) {
      baseStyle.background = `rgba(${parseInt(catColor.slice(1,3),16)},${parseInt(catColor.slice(3,5),16)},${parseInt(catColor.slice(5,7),16)},0.05)`;
      baseStyle.color = catColor;
      baseStyle.border = `1px solid ${catColor}`;
      baseStyle.boxShadow = `0 0 6px ${catColor}33`;
    } else {
      baseStyle.background = "rgba(42, 42, 42, 0.3)";
      baseStyle.color = "#cbd5e1";
      baseStyle.border = "1px solid #2a2a2a";
    }

    const fallbackX = (idx % 5) * 180;
    const fallbackY = Math.floor(idx / 5) * 100;

    return {
      id: n.id,
      label: n.title.length > 12 ? n.title.slice(0, 12) + "..." : n.title,
      position: nodePos ? { x: nodePos.x - 70, y: nodePos.y - 18 } : { x: fallbackX, y: fallbackY },
      style: baseStyle,
    };
  });
});

// 🌟 过滤幽灵边 + 唯一 ID + selectable + 选中高亮
const flowEdges = computed(() => {
  const validNodeIds = new Set(localNodes.value.map((n) => n.id));

  const allCards = [...recentCards.value, ...orphanCards.value];
  const cardCatMap = new Map<string, number | null>();
  for (const c of allCards) {
    cardCatMap.set(c.id, c.category_id ?? null);
  }

  return localEdges.value
    .filter((e) => validNodeIds.has(e.source) && validNodeIds.has(e.target))
    .map((e, i) => {
      const isSelected =
        selectedEdge.value?.source === e.source &&
        selectedEdge.value?.target === e.target;

      const sourceCatId = cardCatMap.get(e.source);
      const edgeColor = sourceCatId ? categoryColorMap.value.get(sourceCatId) : null;

      let stroke: string;
      let strokeWidth: number;

      if (isSelected) {
        stroke = edgeColor || "#00e5ff";
        strokeWidth = 3;
      } else if (e.relation === "sequence") {
        stroke = edgeColor || "#00e5ff";
        strokeWidth = 2;
      } else {
        stroke = edgeColor || "#555555";
        strokeWidth = 1;
      }

      return {
        id: `edge-${e.source}-${e.target}-${e.relation || "rel"}-${i}`,
        source: e.source,
        target: e.target,
        type: "smoothstep",
        selectable: true,
        // 不用 Vue Flow animated（底层是 animated stroke-dasharray，导致主干变虚线）
        animated: false,
        style: {
          stroke,
          strokeWidth,
          // 参考线始终虚线，主干始终实线
          ...(e.relation !== "sequence" ? { strokeDasharray: "5 5" } : {}),
        },
      };
    });
});

onConnect((params: any) => {
  // Always create sequence edges by default. Prohibit creating reference edges via drag.
  const potentialRelation = (params && (params.relation || (params.edge && (params.edge as any).relation))) || "sequence";
  if (potentialRelation === "reference" || (params && (params as any).relation === "reference")) {
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
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});
onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <aside class="w-[380px] bg-ms-panel flex flex-col border-l border-ms-border shrink-0">

    <div class="h-12 flex items-center px-4 border-b border-ms-border bg-ms-carbon shrink-0">
      <span class="font-bold text-slate-400 text-xs font-mono tracking-wider">局部拓扑</span>
      <span class="ml-2 text-[10px] text-slate-600 font-mono">点击节点跳转 · 右击连线操作</span>
      <span v-if="selectedEdge" class="ml-auto text-xs bg-neon/10 text-neon px-2 py-0.5 rounded-sm">
        已选中连线
      </span>
    </div>
    <div class="flex-1 relative" @click="closeContextMenu">
      <!-- 空状态保护：有节点才渲染 VueFlow -->
      <VueFlow v-if="flowNodes.length > 0" :nodes="flowNodes" :edges="flowEdges" fit-view-on-init class="bg-ms-deep"
        @node-click="onNodeClick" @edge-click="onEdgeClick" @edge-context-menu="onEdgeContextMenu"
        @pane-click="onPaneClick">
        <Background pattern-color="#333" :gap="16" />
        <Controls />
      </VueFlow>
      <div v-else class="flex-1 flex items-center justify-center h-full text-slate-500 text-sm">
        暂无图谱数据
      </div>
      <div class="absolute top-4 right-4 z-10 flex gap-1.5">
        <!-- Summon button -->
        <div class="relative">
          <button @click.stop="showSummon = !showSummon" title="召唤节点至画布"
            class="flex items-center justify-center w-8 h-8 bg-ms-carbon/90 backdrop-blur border border-ms-border rounded-sm shadow-sm text-slate-500 hover:text-neon hover:border-neon/30 transition-all">
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </button>
          <div v-if="showSummon" @click.stop
            class="absolute right-0 top-10 w-64 bg-ms-carbon rounded-sm shadow-xl border border-ms-border p-2 z-20">
            <input v-model="summonQuery" placeholder="搜索卡片..."
              class="w-full text-sm px-3 py-1.5 border border-ms-border rounded-sm outline-none focus:border-neon bg-ms-deep text-slate-300" />
            <div class="max-h-48 overflow-y-auto mt-1">
              <button v-for="card in summonableCards" :key="card.id" @click="summonNode(card)"
                class="w-full text-left px-3 py-2 text-sm rounded-sm hover:bg-neon/10 text-slate-400 truncate transition">
                {{ card.title || "无标题" }}
              </button>
              <div v-if="summonableCards.length === 0" class="text-xs text-slate-500 px-3 py-2">无可召唤的卡片</div>
            </div>
          </div>
        </div>
      </div>

    </div>

    <!-- Edge Context Menu (teleported to body for correct positioning) -->
    <Teleport to="body">
      <div v-if="contextMenu"
        class="fixed z-dropdown bg-ms-carbon rounded-sm shadow-xl border border-ms-border py-1 min-w-[160px]"
        :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }" @click.stop>
        <button @click="setEdgeType('sequence')"
          class="w-full text-left px-3 py-2 text-sm hover:bg-ms-surface flex items-center gap-2 transition"
          :class="contextMenu.edge.relation === 'sequence' ? 'text-neon font-medium' : 'text-slate-400'">
          <span class="w-2 h-2 rounded-full bg-neon"></span>
          设为主干 (Sequence)
        </button>
        <!-- Reference edge setting removed: reference edges are not user-configurable -->
        <div class="border-t border-ms-border my-1"></div>
        <button @click="handleDeleteEdge"
          class="w-full text-left px-3 py-2 text-sm text-ms-danger hover:bg-ms-danger/10 flex items-center gap-2 transition">
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
          断开连接
        </button>
      </div>
    </Teleport>
    <!-- 连线类型图例 -->
    <div class="px-3 pb-2 text-[10px] text-slate-600 font-mono">
      <div>━━ 实线 = 主干（手动连接）</div>
      <div>╌╌ 虚线 = 参考（[[链接]] 自动生成）</div>
    </div>
  </aside>
</template>
