<script setup lang="ts">
/**
 * 🌌 GraphView — 多连通分量星图（孤岛与断裂子图同屏呈现）
 *
 * 布局策略：
 * 1. 全量快照拉取所有节点和边
 * 2. graphology 切割连通分量
 * 3. 每个分量独立 Dagre 布局
 * 4. potpack 矩阵打包到同一画布
 *
 * 聚光灯模式：
 * - 点击节点 → 按 graphDepth 高亮 N 度邻居
 * - 点击空白 → 恢复全景
 */

import { ref, shallowRef, onMounted, onUnmounted, watch, nextTick, type Ref } from "vue";
import { storeToRefs } from "pinia";
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
import { Sparkles, Zap, RefreshCw, Network } from "lucide-vue-next";

import CardNode from "../components/ui/CardNode.vue";
import { useGraph } from "../composables/useGraph";
import { useGraphSync, type MinimalNode, type MinimalEdge } from "../composables/useGraphSync";
import { useGraphStore } from "../store/useGraphStore";
import { layoutMultiComponent, getSpotlightNeighbors } from "../utils/graphLayout";

// Vue Flow 样式
import "@vue-flow/core/dist/style.css";
import "@vue-flow/core/dist/theme-default.css";
import "@vue-flow/controls/dist/style.css";

// ── Pinia Store ──
const store = useGraphStore();
const { graphDepth } = storeToRefs(store);

// ── Vue Flow ──
const { onNodesInitialized, fitView, setNodes, getNodes, setEdges, getEdges, onPaneClick } =
    useVueFlow();

// ── 数据 ──
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

// ── WS 实时增量同步（缩窄类型为最小约束接口，避免 VueFlow 泛型深度实例化） ──
const { connect: connectWS, disconnect: disconnectWS } = useGraphSync(
    nodes as Ref<MinimalNode[]>,
    edges as Ref<MinimalEdge[]>,
);

// ── 聚光灯状态 ──
const spotlightId = ref<string | null>(null);
const spotlightSet = ref<Set<string>>(new Set());

/**
 * 应用聚光灯效果：非邻居节点 opacity 降低
 */
function setNodeClass(id: string, cls: string) {
    const idx = nodes.value.findIndex((n) => n.id === id);
    if (idx >= 0) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        (nodes.value[idx] as any).class = cls;
    }
}

function applySpotlight(focusId: string | null) {
    const currentNodes = getNodes.value;

    if (!focusId) {
        spotlightId.value = null;
        spotlightSet.value = new Set();
        nodes.value = [...currentNodes] as Node[];
        nodes.value.forEach((n) => {
            setNodeClass(n.id, (n.data as any)?.isOrphan ? "is-orphan" : "");
        });
        return;
    }

    spotlightId.value = focusId;
    const neighbors = getSpotlightNeighbors(
        currentNodes,
        getEdges.value,
        focusId,
        graphDepth.value,
    );
    spotlightSet.value = neighbors;

    nodes.value = [...currentNodes] as Node[];
    nodes.value.forEach((n) => {
        const inSpotlight = neighbors.has(n.id);
        const isOrphan = (n.data as any)?.isOrphan;
        let cls = "";
        if (!inSpotlight) cls = "spotlight-dimmed";
        if (isOrphan && inSpotlight) cls = "is-orphan";
        if (isOrphan && !inSpotlight) cls = "is-orphan spotlight-dimmed";
        setNodeClass(n.id, cls);
    });

    const currentEdges = getEdges.value as Edge[];
    const updatedEdges: Edge[] = currentEdges.map((e) => {
        const sourceIn = neighbors.has(e.source);
        const targetIn = neighbors.has(e.target);
        const isActive = sourceIn && targetIn;
        return {
            ...e,
            class: isActive ? "spotlight-active" : "",
        } as Edge;
    });
    edges.value = updatedEdges;
}

// ── 深度变化 → 更新聚光灯范围（不重新拉取数据） ──
watch(graphDepth, () => {
    if (spotlightId.value) {
        applySpotlight(spotlightId.value);
    }
});

// ── 边样式 ──
function enforceEdgePositions(edges: Edge[]): Edge[] {
    return edges.map((edge) => ({
        ...edge,
        sourcePosition: Position.Right,
        targetPosition: Position.Left,
        type: (edge.data as { type?: string })?.type === "sequence"
            ? "smoothstep"
            : "default",
        animated: (edge.data as { type?: string })?.type === "sequence",
    }));
}

// ── 暴露 fitView 给父组件 ──
function fitViewPublic() {
    fitView({ padding: 0.2, duration: 800 });
}
defineExpose({ fitView: fitViewPublic });

// ── 初始化 ──
onMounted(async () => {
    await loadFullGraph();
    nodes.value = (apiNodes.value as Node[]).map((n) => ({ ...n, style: { visibility: "hidden" } }));
    edges.value = apiEdges.value as Edge[];
    connectWS();
});

onUnmounted(() => {
    disconnectWS();
    if (layoutTimer) {
        clearTimeout(layoutTimer);
        layoutTimer = null;
    }
});

// 🌟 关键：等节点真正渲染后，执行多连通分量布局
onNodesInitialized(async () => {
    const layouted = layoutMultiComponent(getNodes.value as Node[], getEdges.value as Edge[]) as Node[];
    nodes.value = layouted.map((n) => ({ ...n, style: { visibility: "visible" } }));
    await nextTick();
    edges.value = enforceEdgePositions(getEdges.value as Edge[]) as Edge[];
    window.requestAnimationFrame(() => {
        fitView({ padding: 0.2, duration: 800 });
    });
});

// ── 缩放事件 → 更新语义层级 ──
function handleViewportChange({ zoom }: { zoom: number }) {
    store.setZoomLevel(zoom);
}

// ── 节点点击 → 聚光灯模式 ──
function onNodeClick(event: NodeMouseEvent) {
    store.selectNode(event.node.id);
    applySpotlight(event.node.id);
}

// ── 画布空白点击 → 退出聚光灯 ──
onPaneClick(() => {
    store.selectNode(null);
    applySpotlight(null);
});

// ── 一键归位 ──
const isLayouting = ref(false);
let layoutTimer: ReturnType<typeof setTimeout> | null = null;

const layout = async () => {
    if (isLayouting.value) return;
    isLayouting.value = true;

    const currentNodes = getNodes.value;
    const currentEdges = getEdges.value;

    const edgesOff = currentEdges.map((e) => ({ ...e, animated: false }));
    setEdges(edgesOff);

    const layoutedNodes = layoutMultiComponent(currentNodes, currentEdges);
    setNodes(layoutedNodes);

    layoutTimer = setTimeout(() => {
        const edgesOn = getEdges.value.map((e) => ({
            ...e,
            animated:
                (e.data as { type?: string })?.type === "sequence",
        }));
        setEdges(edgesOn);
        fitView({ padding: 0.2, duration: 400 });
        isLayouting.value = false;
        layoutTimer = null;
    }, 500);
};
</script>

<template>
    <div class="graph-container" :class="{ 'is-layouting': isLayouting }">
        <!-- ── 加载态（骨架屏节点网格预览） ── -->
        <div v-if="graphLoading" class="empty-state">
            <div class="grid grid-cols-3 gap-4 max-w-md mx-auto">
                <div v-for="i in 6" :key="i"
                    class="h-16 rounded-sm animate-pulse"
                    :class="i % 3 === 0 ? 'col-span-2' : 'col-span-1'"
                    :style="{ background: `rgba(51, 51, 51, ${0.3 + (i * 0.1)})` }">
                    <div class="p-3 space-y-2">
                        <div class="h-2 rounded bg-ms-border/50 w-3/4" />
                        <div class="h-1.5 rounded bg-ms-border/30 w-1/2" />
                    </div>
                </div>
            </div>
            <p class="text-gray-500 text-xs mt-6 font-mono">正在加载全量星图...</p>
        </div>

        <!-- ── 空态：后端无数据 ── -->
        <div v-else-if="graphEmpty" class="empty-state">
            <Network :size="48" class="text-gray-600 mb-4" />
            <h3 class="text-gray-200 text-lg font-semibold mb-2">图谱是空的</h3>
            <p class="text-gray-500 text-sm mb-4">还没有任何记忆卡片，快去创建第一张吧！</p>
            <p class="text-gray-600 text-xs font-mono">在 admin-tauri 中新建卡片后，图谱会自动生成</p>
        </div>

        <!-- ── 错误态：API 不可用 ── -->
        <div v-else-if="graphError" class="empty-state">
            <Zap :size="48" class="text-ms-danger mb-4" />
            <h3 class="text-gray-200 text-lg font-semibold mb-2">无法连接到后端服务</h3>
            <p class="text-gray-500 text-sm mb-4">{{ graphError }}</p>
            <button @click="loadFullGraph()"
                class="px-4 py-2 bg-neon/10 text-neon text-sm rounded-lg border border-neon/20 hover:bg-neon/20 transition-all inline-flex items-center gap-1.5">
                <RefreshCw :size="14" /> 重新连接
            </button>
        </div>

        <!-- ── 正常态：图谱渲染 ── -->
        <VueFlow v-else
            :nodes="nodes"
            :edges="edges"
            :fit-view-on-init="true"
            :default-viewport="{ zoom: 1 }"
            :snap-to-grid="true"
            :snap-grid="[15, 15]"
            :connect-on-click="false"
            :connect-on-drag="false"
            @node-click="onNodeClick"
            @viewport-change="handleViewportChange">
            <Background :pattern-color="'#333333'" :gap="20" />
            <Controls />

            <!-- 🌟 自定义卡片节点 -->
            <template #node-card="nodeProps">
                <CardNode v-bind="nodeProps" />
            </template>

            <!-- ✨ 一键归位浮动按钮 ── -->
            <div class="absolute bottom-4 right-4 z-chrome">
                <button @click="layout" class="layout-btn" :disabled="isLayouting" title="一键归位">
                    <Sparkles :size="18" />
                </button>
            </div>
        </VueFlow>
    </div>
</template>

<style scoped>
.graph-container {
    height: 100%;
    width: 100%;
    background: theme('colors.ms-deep');
    border-radius: 0;
    overflow: hidden;
}

.empty-state {
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 2rem;
}

.layout-btn {
    padding: 12px;
    border-radius: 9999px;
    background: theme('colors.neon.DEFAULT');
    color: theme('colors.ms-deep');
    border: none;
    cursor: pointer;
    font-size: 18px;
    box-shadow: 0 4px 24px rgba(0, 229, 255, 0.4);
    transition: all 0.25s cubic-bezier(0.2, 0, 0, 1);
}

.layout-btn:hover:not(:disabled) {
    background: theme('colors.neon.400');
    transform: scale(1.1);
}

.layout-btn:disabled {
    opacity: 0.5;
    cursor: wait;
}

/* ── 250ms 动效法则 ── */
:deep(.vue-flow__node) {
    transition: transform 0.25s cubic-bezier(0.2, 0, 0, 1),
        opacity 0.25s ease-out;
    will-change: transform, opacity;
    transform: translateZ(0);
    backface-visibility: hidden;
}

.is-layouting :deep(.vue-flow__node) {
    pointer-events: none;
}

/* 隐藏 Vue Flow 所有连接手柄，禁用拖拽连接（只读边） */
:deep(.vue-flow__handle) {
    opacity: 0;
    pointer-events: none;
}

/* ── 聚光灯暗化（blur + grayscale + 透明度） ── */
:deep(.vue-flow__node.spotlight-dimmed) {
    opacity: 0.12 !important;
    filter: blur(3px) grayscale(0.5);
    will-change: filter, opacity;
    pointer-events: none;
}

:deep(.vue-flow__node.spotlight-dimmed:hover) {
    opacity: 0.12 !important;
    filter: blur(3px) grayscale(0.5);
    transform: none !important;
}

/* ── 聚光灯焦点路径边发光 ── */
:deep(.vue-flow__edge.spotlight-active .vue-flow__edge-path) {
    stroke: #00e5ff;
    filter: drop-shadow(0 0 6px rgba(0, 229, 255, 0.6));
    stroke-width: 2.5;
    stroke-dasharray: 8 4;
    animation: spotlight-flow 1s linear infinite;
}

@keyframes spotlight-flow {
    from { stroke-dashoffset: 12; }
    to { stroke-dashoffset: 0; }
}

/* ── 孤岛节点 class ── */
:deep(.vue-flow__node.is-orphan) {
    /* CardNode 内部样式已处理 */
}

/* ── Reference 参考线：虚线灰色 ── */
:deep(.vue-flow__edge.reference .vue-flow__edge-path) {
    stroke: #71717a;
    stroke-width: 1.5;
    stroke-dasharray: 6 4;
}

/* ── Sequence 能量流线：实线 + animated 流动） ── */
:deep(.vue-flow__edge.animated .vue-flow__edge-path) {
    stroke: #00e5ff;
    stroke-width: 3;
    filter: drop-shadow(0 0 6px rgba(0, 229, 255, 0.5));
}

/* ── Controls 主题适配 ── */
:deep(.vue-flow__controls) {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 2px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
}

:deep(.vue-flow__controls-button) {
    background: #1a1a1a;
    border-bottom: 1px solid #333;
    fill: #a3a3a3;
}

:deep(.vue-flow__controls-button:hover) {
    background: #222;
}
</style>
