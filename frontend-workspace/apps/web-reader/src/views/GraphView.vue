<script setup lang="ts">
/**
 * GraphView — 图谱渲染主组件（重铸版）
 *
 * 三大基石：
 * 1. 显式插槽映射 #node-card → CardNode
 * 2. 语义化边（sequence=青色smoothstep, reference=灰色虚线bezier）
 * 3. Dagre LR 布局仅在 onNodesInitialized 后执行
 *
 * 扩展功能：
 * - 遗忘热力学：节点 opacity 按浏览时间衰减
 * - 拓扑潜流：右键提取 SEQ 链，FlowReader 线性阅读
 */

import { shallowRef, ref, computed, onMounted, nextTick, provide, onUnmounted } from "vue";
import {
    VueFlow,
    useVueFlow,
    MarkerType,
    type NodeMouseEvent,
    type Node,
    type Edge,
} from "@vue-flow/core";
import { Background } from "@vue-flow/background";
import { Controls } from "@vue-flow/controls";
import { Eye, Flame, Crosshair, RotateCw } from "lucide-vue-next";

import CardNode from "../components/ui/CardNode.vue";
import FlowReader from "../components/FlowReader.vue";
import { useGraph } from "../composables/useGraph";
import { useGraphStore } from "../store/useGraphStore";
import { layoutMultiComponent, layoutMultiComponentAsync } from "../utils/graphLayout";
import { useOblivionHeatmap } from "../composables/useOblivionHeatmap";
import { extractSeqChain, isInSeqChain } from "../utils/seqTraversal";

// VueFlow 样式
import "@vue-flow/core/dist/style.css";
import "@vue-flow/core/dist/theme-default.css";
import "@vue-flow/controls/dist/style.css";

// ── 数据层 ──
const {
    nodes: apiNodes,
    edges: apiEdges,
    loading: graphLoading,
    error: graphError,
    isEmpty: graphEmpty,
    loadFullGraph,
} = useGraph();

const nodes = shallowRef<Node[]>([]);
const rawEdges = shallowRef<Edge[]>([]);

// ── 聚光灯：hover 节点时只点亮关联的参考线 ──
const hoveredNodeId = ref<string | null>(null);

// ── 语义化边：参考线支持 Hover 点亮 ──
const edges = computed(() => {
    return rawEdges.value.map((edge) => {
        const edgeData = (edge.data as { type?: string; isBidirectional?: boolean }) || {};
        const relation = edgeData.type;
        const isReference = relation !== "sequence"; // 参考线判断
        const isBiDir = edgeData.isBidirectional || false; // 双向标记

        // 🗡️ 第三击：交互点亮（Hover 降噪）
        // 只有参考线支持点亮，序列线保持常态
        let style: Record<string, any>;

        if (isReference) {
            // 参考线：Hover 时从暗淡变橙亮
            const isConnected = hoveredNodeId.value !== null &&
                (edge.source === hoveredNodeId.value || edge.target === hoveredNodeId.value);
            
            if (isConnected) {
                // 点亮状态：橙色 + 高透明度 + 增粗
                style = {
                    stroke: "#ffaa00",
                    strokeWidth: 2,
                    opacity: 1,
                };
            } else if (hoveredNodeId.value !== null) {
                // 其他参考线在 Hover 时进一步暗淡
                style = {
                    stroke: "#555",
                    strokeWidth: 1,
                    opacity: 0.1,
                };
            } else {
                // 默认状态：极暗（双向线稍亮一点）
                style = {
                    stroke: isBiDir ? "#888" : "#555",
                    strokeWidth: isBiDir ? 1.5 : 1,
                    opacity: 0.2,
                };
            }
        } else {
            // 序列线：始终保持醒目状态，不受 Hover 影响
            style = {
                stroke: "#00e5ff",
                strokeWidth: 2,
                opacity: 1,
            };
        }

        // 🎨 双向边视觉映射：两头都有箭头
        return {
            ...edge,
            type: isReference ? "default" : "smoothstep",
            animated: !isReference,
            style,
            zIndex: isReference ? 0 : 10,
            // 核心：双向边在两端都显示箭头
            markerEnd: MarkerType.ArrowClosed,
            markerStart: isBiDir ? MarkerType.ArrowClosed : undefined,
        };
    });
});

function onNodeMouseEnter(event: NodeMouseEvent) {
    hoveredNodeId.value = event.node.id;
}

function onNodeMouseLeave() {
    hoveredNodeId.value = null;
}

// ── VueFlow 钩子 ──
const { onNodesInitialized, fitView, getNodes, getEdges, onPaneClick } = useVueFlow();

// ── Store（驱动 DetailDrawer 抽屉） ──
const store = useGraphStore();

// ── 遗忘热力学 ──
const { recordView } = useOblivionHeatmap();
const heatmapEnabled = ref(true);
provide("heatmapEnabled", heatmapEnabled);

// ── 拓扑潜流 ──
const flowReaderOpen = ref(false);
const flowChainIds = ref<string[]>([]);
const flowCurrentNodeId = ref<string | null>(null);
const LONG_PRESS_MS = 450;
const ASYNC_LAYOUT_THRESHOLD = 120;
const CONTEXT_MENU_WIDTH = 140;
const CONTEXT_MENU_HEIGHT = 42;
const CONTEXT_MENU_GAP = 8;

let longPressTimer: ReturnType<typeof setTimeout> | null = null;
let longPressMoved = false;

// ── 右键菜单 ──
const contextMenu = ref<{ x: number; y: number; nodeId: string } | null>(null);

const canExtractFlowFromContextMenu = computed(() => {
    if (!contextMenu.value) return false;
    const vfEdges = getEdges.value as Edge[];
    return isInSeqChain(contextMenu.value.nodeId, vfEdges);
});

function onNodeClick(event: NodeMouseEvent) {
    // 记录浏览时间
    recordView(event.node.id);
    store.selectNode(event.node.id);
}

onPaneClick(() => {
    store.selectNode(null);
    contextMenu.value = null;
});

// ── 右键菜单：节点快捷操作 ──
function onNodeContextMenu(event: NodeMouseEvent) {
    event.event.preventDefault();
    const nodeId = event.node.id;
    const vfEdges = getEdges.value as Edge[];
    if (!isInSeqChain(nodeId, vfEdges)) {
        contextMenu.value = null;
        return;
    }

    const mouseEvent = event.event as MouseEvent;
    openContextMenuAt(mouseEvent.clientX, mouseEvent.clientY, nodeId);
}

function getClampedContextMenuPosition(clientX: number, clientY: number) {
    const maxX = window.innerWidth - CONTEXT_MENU_WIDTH - CONTEXT_MENU_GAP;
    const maxY = window.innerHeight - CONTEXT_MENU_HEIGHT - CONTEXT_MENU_GAP;
    return {
        x: Math.max(CONTEXT_MENU_GAP, Math.min(clientX, maxX)),
        y: Math.max(CONTEXT_MENU_GAP, Math.min(clientY, maxY)),
    };
}

function openContextMenuAt(clientX: number, clientY: number, nodeId: string) {
    const { x, y } = getClampedContextMenuPosition(clientX, clientY);
    contextMenu.value = { x, y, nodeId };
}

function clearLongPressTimer() {
    if (longPressTimer) {
        clearTimeout(longPressTimer);
        longPressTimer = null;
    }
}

function handleTouchStart(e: TouchEvent) {
    const touch = e.touches[0];
    if (!touch) return;

    const target = e.target as HTMLElement | null;
    const nodeEl = target?.closest(".vue-flow__node") as HTMLElement | null;
    const nodeId = nodeEl?.dataset.id;
    if (!nodeId) return;

    longPressMoved = false;
    clearLongPressTimer();

    longPressTimer = setTimeout(() => {
        if (longPressMoved) return;

        const vfEdges = getEdges.value as Edge[];
        if (!isInSeqChain(nodeId, vfEdges)) return;

        store.selectNode(nodeId);
        openContextMenuAt(touch.clientX, touch.clientY, nodeId);
    }, LONG_PRESS_MS);
}

function handleTouchMove() {
    longPressMoved = true;
    clearLongPressTimer();
}

function handleTouchEnd() {
    clearLongPressTimer();
}

function openFlowReader() {
    if (!contextMenu.value) return;

    const vfEdges = getEdges.value as Edge[];
    if (!isInSeqChain(contextMenu.value.nodeId, vfEdges)) return;

    const chain = extractSeqChain(contextMenu.value.nodeId, vfEdges);

    if (chain.length > 0) {
        flowChainIds.value = chain;
        flowCurrentNodeId.value = contextMenu.value.nodeId;
        flowReaderOpen.value = true;
    }

    contextMenu.value = null;
}

function closeFlowReader() {
    flowReaderOpen.value = false;
    flowChainIds.value = [];
    flowCurrentNodeId.value = null;
}

function onFlowNavigate(nodeId: string) {
    recordView(nodeId);
}

function resetViewport() {
    fitView({ padding: 0.2, duration: 800, maxZoom: 1.2 });
}

async function applyAdaptiveLayout(currentNodes: Node[], currentEdges: Edge[]) {
    if (currentNodes.length >= ASYNC_LAYOUT_THRESHOLD) {
        return layoutMultiComponentAsync(currentNodes, currentEdges);
    }
    return layoutMultiComponent(currentNodes, currentEdges);
}

async function resetLayout() {
    try {
        const measuredNodes = getNodes.value as Node[];
        const currentEdges = getEdges.value as Edge[];
        const layouted = await applyAdaptiveLayout(measuredNodes, currentEdges);
        nodes.value = layouted;
    } catch (err) {
        console.error("❌ Dagre 重置布局失败:", err);
    }

    nextTick(() => {
        fitView({ padding: 0.2, duration: 800, maxZoom: 1.2 });
    });
}

// ── Dagre 布局锁（防重入） ──
let layoutDone = false;

onNodesInitialized(async () => {
    if (layoutDone) return;
    layoutDone = true;

    try {
        const measuredNodes = getNodes.value as Node[];
        const currentEdges = getEdges.value as Edge[];
        const layouted = await applyAdaptiveLayout(measuredNodes, currentEdges);
        nodes.value = layouted;
    } catch (err) {
        console.error("❌ Dagre 布局失败:", err);
    }

    nextTick(() => {
        fitView({ padding: 0.2, duration: 800, maxZoom: 1.2 });
    });
});

// ── 加载数据 ──
onMounted(async () => {
    await loadFullGraph();

    nodes.value = apiNodes.value as Node[];
    rawEdges.value = apiEdges.value as Edge[];
});

onUnmounted(() => {
    clearLongPressTimer();
});
</script>

<template>
    <div class="graph-container" @touchstart.passive="handleTouchStart" @touchmove.passive="handleTouchMove"
        @touchend.passive="handleTouchEnd" @touchcancel.passive="handleTouchEnd">
        <!-- 加载态 -->
        <div v-if="graphLoading" class="state-overlay">
            <p class="text-zinc-500 text-xs font-mono animate-pulse">正在加载图谱...</p>
        </div>

        <!-- 空态 -->
        <div v-else-if="graphEmpty" class="state-overlay">
            <p class="text-zinc-500 text-sm">图谱为空</p>
        </div>

        <!-- 错误态 -->
        <div v-else-if="graphError" class="state-overlay">
            <p class="text-red-400 text-sm">{{ graphError }}</p>
        </div>

        <!-- 图谱渲染 -->
        <VueFlow v-else :nodes="nodes" :edges="edges" :fit-view-on-init="true" :default-viewport="{ zoom: 1 }"
            @node-click="onNodeClick" @node-context-menu="onNodeContextMenu"
            @node-mouse-enter="onNodeMouseEnter" @node-mouse-leave="onNodeMouseLeave">
            <Background :pattern-color="'#1a1a1a'" :gap="20" />
            <Controls />

            <!-- 显式插槽映射：node type "card" → CardNode -->
            <template #node-card="nodeProps">
                <CardNode v-bind="nodeProps" />
            </template>
        </VueFlow>

        <!-- 右键菜单 -->
        <Teleport to="body">
            <Transition name="ctx-fade">
                <div v-if="contextMenu" class="context-menu z-dropdown"
                    :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }">
                    <button class="context-menu__item" :disabled="!canExtractFlowFromContextMenu" @click="openFlowReader">
                        <Flame :size="12" />
                        <span>提取潜流</span>
                    </button>
                </div>
            </Transition>
        </Teleport>

        <!-- 右上角工具栏 -->
        <div class="toolbar">
            <button class="toolbar-btn" :class="{ 'toolbar-btn--active': heatmapEnabled }"
                :title="heatmapEnabled ? '关闭遗忘热力学' : '开启遗忘热力学'" @click="heatmapEnabled = !heatmapEnabled">
                <Eye :size="14" />
            </button>
            <button class="toolbar-btn" title="视界归位" @click="resetViewport">
                <Crosshair :size="14" />
            </button>
            <button class="toolbar-btn" title="物理重置" @click="resetLayout">
                <RotateCw :size="14" />
            </button>
        </div>

        <!-- 潜流阅读器 -->
        <FlowReader :open="flowReaderOpen" :chain-ids="flowChainIds" :current-node-id="flowCurrentNodeId"
            @close="closeFlowReader" @navigate="onFlowNavigate" />
    </div>
</template>

<style scoped>
.graph-container {
    height: 100%;
    width: 100%;
    background:
        radial-gradient(ellipse at 20% 30%, rgba(255, 255, 255, 0.02) 0%, transparent 50%),
        radial-gradient(ellipse at 80% 70%, rgba(0, 229, 255, 0.015) 0%, transparent 50%),
        radial-gradient(ellipse at 50% 50%, rgba(0, 229, 255, 0.01) 0%, transparent 70%),
        #0d0d0d;
    position: relative;
    overflow: hidden;
}

.state-overlay {
    height: 100%;
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
}

/* ── Controls dark industrial theme ── */
:deep(.vue-flow__controls) {
    background: #141414;
    border: 1px solid #2a2a2a;
    border-radius: 3px;
}

:deep(.vue-flow__controls-button) {
    background: #141414;
    border-bottom: 1px solid #2a2a2a;
    fill: #71717a;
}

:deep(.vue-flow__controls-button:hover) {
    background: #2a2a2a;
    fill: #cbd5e1;
}

/* ── 右键菜单 — hard entity shadow ── */
.context-menu {
    position: fixed;
    background: #141414;
    border: 1px solid #2a2a2a;
    border-radius: 3px;
    padding: 4px 0;
    min-width: 140px;
    box-shadow:
        inset 0 1px 0 0 rgba(255, 255, 255, 0.04),
        3px 3px 0 0 rgba(0, 0, 0, 0.6);
}

.context-menu__item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 14px;
    font-size: 12px;
    color: #cbd5e1;
    background: transparent;
    border: none;
    cursor: pointer;
    transition: all 150ms ease;
}

.context-menu__item:hover {
    background: rgba(0, 229, 255, 0.08);
    color: #00e5ff;
}

.context-menu__item:disabled {
    opacity: 0.4;
    cursor: not-allowed;
}

.context-menu__item:disabled:hover {
    background: transparent;
    color: #cbd5e1;
}

.ctx-fade-enter-active,
.ctx-fade-leave-active {
    transition: opacity 0.15s ease;
}

.ctx-fade-enter-from,
.ctx-fade-leave-to {
    opacity: 0;
}

/* ── 右上角工具栏 ── */
.toolbar {
    position: absolute;
    top: 12px;
    right: 12px;
    z-index: 10;
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.toolbar-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #141414;
    border: 1px solid #2a2a2a;
    border-radius: 3px;
    color: #71717a;
    box-shadow: 2px 2px 0 0 rgba(0, 0, 0, 0.5);
    cursor: pointer;
    transition: transform 0.15s ease, box-shadow 0.15s ease, color 0.15s ease, background 0.15s ease;
}

.toolbar-btn:hover {
    background: #2a2a2a;
    color: #cbd5e1;
    transform: translate(-1px, -1px);
    box-shadow: 3px 3px 0 0 rgba(0, 0, 0, 0.5);
}

.toolbar-btn:active {
    transform: translate(1px, 1px);
    box-shadow: 0px 0px 0 0 rgba(0, 0, 0, 0.5);
}

.toolbar-btn--active {
    color: #00e5ff;
    border-color: rgba(0, 229, 255, 0.3);
    box-shadow: 0 0 6px rgba(0, 229, 255, 0.15), 2px 2px 0 0 rgba(0, 0, 0, 0.5);
}

@media (max-width: 900px) {
    .toolbar {
        top: auto;
        bottom: calc(12px + env(safe-area-inset-bottom));
        right: 10px;
        gap: 6px;
    }

    .toolbar-btn {
        width: 36px;
        height: 36px;
    }

}

/* ── Edge spotlight transitions ── */
:deep(.vue-flow__edge-path) {
    transition: opacity 0.25s ease, stroke 0.25s ease;
}

:deep(.vue-flow__edge:hover) {
    cursor: pointer;
}
</style>
