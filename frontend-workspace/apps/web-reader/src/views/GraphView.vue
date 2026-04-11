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

import { shallowRef, ref, computed, onMounted, nextTick, provide } from "vue";
import {
    VueFlow,
    useVueFlow,
    Position,
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
import { layoutMultiComponent } from "../utils/graphLayout";
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
const edges = shallowRef<Edge[]>([]);

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

// ── 右键菜单 ──
const contextMenu = ref<{ x: number; y: number; nodeId: string } | null>(null);

function onNodeClick(event: NodeMouseEvent) {
    // 记录浏览时间
    recordView(event.node.id);
    store.selectNode(event.node.id);
}

onPaneClick(() => {
    store.selectNode(null);
    contextMenu.value = null;
});

// ── 右键菜单：提取潜流 ──
function onNodeContextMenu(event: NodeMouseEvent) {
    event.event.preventDefault();
    const nodeId = event.node.id;
    const vfEdges = getEdges.value as Edge[];

    if (isInSeqChain(nodeId, vfEdges)) {
        contextMenu.value = {
            x: (event.event as MouseEvent).clientX,
            y: (event.event as MouseEvent).clientY,
            nodeId,
        };
    }
}

function openFlowReader() {
    if (!contextMenu.value) return;

    const vfEdges = getEdges.value as Edge[];
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

function resetLayout() {
    try {
        const measuredNodes = getNodes.value as Node[];
        const currentEdges = getEdges.value as Edge[];
        const layouted = layoutMultiComponent(measuredNodes, currentEdges);
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

onNodesInitialized(() => {
    if (layoutDone) return;
    layoutDone = true;

    try {
        const measuredNodes = getNodes.value as Node[];
        const currentEdges = getEdges.value as Edge[];
        const layouted = layoutMultiComponent(measuredNodes, currentEdges);
        nodes.value = layouted;
    } catch (err) {
        console.error("❌ Dagre 布局失败:", err);
    }

    nextTick(() => {
        fitView({ padding: 0.2, duration: 800, maxZoom: 1.2 });
    });
});

// ── 加载数据 + 构建语义化边 ──
onMounted(async () => {
    await loadFullGraph();

    nodes.value = apiNodes.value as Node[];

    edges.value = (apiEdges.value as Edge[]).map((edge) => {
        const relation = (edge.data as { type?: string })?.type;
        const isSequence = relation === "sequence";

        return {
            ...edge,
            type: isSequence ? "smoothstep" : "default",
            animated: isSequence,
            sourcePosition: Position.Right,
            targetPosition: Position.Left,
            style: isSequence
                ? { stroke: "#00e5ff", strokeWidth: 2 }
                : { stroke: "#71717a", strokeWidth: 1.5, strokeDasharray: "5 5" },
        };
    });
});
</script>

<template>
    <div class="graph-container">
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
        <VueFlow
            v-else
            :nodes="nodes"
            :edges="edges"
            :fit-view-on-init="true"
            :default-viewport="{ zoom: 1 }"
            @node-click="onNodeClick"
            @node-context-menu="onNodeContextMenu"
        >
            <Background :pattern-color="'#27272a'" :gap="20" />
            <Controls />

            <!-- 显式插槽映射：node type "card" → CardNode -->
            <template #node-card="nodeProps">
                <CardNode v-bind="nodeProps" />
            </template>
        </VueFlow>

        <!-- 右键菜单 -->
        <Transition name="ctx-fade">
            <div
                v-if="contextMenu"
                class="context-menu"
                :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
            >
                <button class="context-menu__item" @click="openFlowReader">
                    <Flame :size="12" />
                    <span>提取潜流</span>
                </button>
            </div>
        </Transition>

        <!-- 右上角工具栏 -->
        <div class="toolbar">
            <button
                class="toolbar-btn"
                :class="{ 'toolbar-btn--active': heatmapEnabled }"
                :title="heatmapEnabled ? '关闭遗忘热力学' : '开启遗忘热力学'"
                @click="heatmapEnabled = !heatmapEnabled"
            >
                <Eye :size="14" />
            </button>
            <button
                class="toolbar-btn"
                title="视界归位"
                @click="resetViewport"
            >
                <Crosshair :size="14" />
            </button>
            <button
                class="toolbar-btn"
                title="物理重置"
                @click="resetLayout"
            >
                <RotateCw :size="14" />
            </button>
        </div>

        <!-- 潜流阅读器 -->
        <FlowReader
            :open="flowReaderOpen"
            :chain-ids="flowChainIds"
            :current-node-id="flowCurrentNodeId"
            @close="closeFlowReader"
            @navigate="onFlowNavigate"
        />
    </div>
</template>

<style scoped>
.graph-container {
    height: 100%;
    width: 100%;
    background:
        radial-gradient(ellipse 80% 60% at 50% 50%, rgba(166, 38, 38, 0.03) 0%, transparent 70%),
        radial-gradient(circle at 20% 80%, rgba(201, 168, 76, 0.02) 0%, transparent 40%),
        #09090b;
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

/* ── Controls 暗色主题 ── */
:deep(.vue-flow__controls) {
    background: #18181b;
    border: 1px solid #27272a;
    border-radius: 3px;
}

:deep(.vue-flow__controls-button) {
    background: #18181b;
    border-bottom: 1px solid #27272a;
    fill: #71717a;
}

:deep(.vue-flow__controls-button:hover) {
    background: #27272a;
    fill: #e4e4e7;
}

/* ── 右键菜单 — hard entity shadow ── */
.context-menu {
    position: fixed;
    z-index: 40;
    background: #18181b;
    border: 1px solid #3a3228;
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
    color: #c8bfa8;
    background: transparent;
    border: none;
    cursor: pointer;
    transition: all 150ms ease;
}

.context-menu__item:hover {
    background: rgba(201, 168, 76, 0.08);
    color: #c9a84c;
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
    background: #18181b;
    border: 1px solid #3a3228;
    border-radius: 3px;
    color: #71717a;
    box-shadow: 2px 2px 0 0 rgba(0, 0, 0, 0.5);
    cursor: pointer;
    transition: transform 0.15s ease, box-shadow 0.15s ease, color 0.15s ease, background 0.15s ease;
}

.toolbar-btn:hover {
    background: #27272a;
    color: #c8bfa8;
    transform: translate(-1px, -1px);
    box-shadow: 3px 3px 0 0 rgba(0, 0, 0, 0.5);
}

.toolbar-btn:active {
    transform: translate(1px, 1px);
    box-shadow: 0px 0px 0 0 rgba(0, 0, 0, 0.5);
}

.toolbar-btn--active {
    color: #c9a84c;
    border-color: rgba(201, 168, 76, 0.3);
    box-shadow: 0 0 6px rgba(201, 168, 76, 0.15), 2px 2px 0 0 rgba(0, 0, 0, 0.5);
}
</style>
