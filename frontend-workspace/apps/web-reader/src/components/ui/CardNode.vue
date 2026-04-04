<script setup lang="ts">
/**
 * 🌟 CardNode — 统一的卡片渲染原子组件
 *
 * 支持：
 * - Handle 锚点：Left(target) / Right(source) 水平数据流
 * - Sequence 节点：深色细边框 + 霓虹青光晕
 * - Reference 节点：zinc 虚线边框 + 半透明背景
 * - 热度角标：右上角 font-mono（暗色主题协调）
 * - 三级语义缩放：outline / summary / detail
 * - 选中状态：霓虹呼吸发光（GPU 加速）
 */

import { computed, ref } from "vue";
import { storeToRefs } from "pinia";
import { Handle, Position } from "@vue-flow/core";
import { useGraphStore } from "../../store/useGraphStore";

interface CardNodeData {
    title: string;
    date?: string;
    type?: string; // "sequence" | "reference"
    heat?: number;
    preview?: string; // Markdown 预览文本
    isOrphan?: boolean; // 孤岛标记：无任何边连接的节点
}

const props = defineProps<{
    data: CardNodeData;
    id: string;
    selected?: boolean;
    highlighted?: boolean;
}>();

// ── 从 Pinia Store 获取 tier 和 selectedId ──
const graphStore = useGraphStore();
const { selectedId, semanticTier } = storeToRefs(graphStore);

const isSequence = computed(() => props.data.type === "sequence");
const isSelected = computed(() => selectedId.value === props.id);
const isHighlighted = computed(() => graphStore.highlightedId === props.id);
const tier = computed(() => semanticTier.value || "summary");

// 语义缩放：根据 tier 切换显示内容
const showOutline = computed(() => tier.value === "outline");
const showSummary = computed(() => tier.value === "summary");
const showDetail = computed(() => tier.value === "detail");

// 悬停状态
const isHovered = ref(false);
</script>

<template>
    <div class="card-node" :class="{
        'card-sequence': isSequence && !data.isOrphan,
        'card-reference': !isSequence && !data.isOrphan,
        'card-orphan': data.isOrphan,
        'card-outline': showOutline,
        'card-detail': showDetail,
        'card-selected': isSelected,
        'card-highlighted': isHighlighted,
        'card-hovered': isHovered,
    }" @mouseenter="isHovered = true" @mouseleave="isHovered = false">
        <!-- ── 左侧入锚点 (target) ── -->
        <Handle type="target" :position="Position.Left" class="handle-node handle-target" />

        <!-- 热度角标 -->
        <span v-if="data.heat && !showOutline" class="heat-badge font-mono">
            🔥 {{ data.heat }}
        </span>

        <!-- Outline 模式：仅显示分类标题 -->
        <template v-if="showOutline">
            <div class="node-title text-xs truncate">{{ data.title }}</div>
        </template>

        <!-- Summary 模式：标题 + 日期 -->
        <template v-else-if="showSummary">
            <div class="node-title">{{ data.title }}</div>
            <div v-if="data.date" class="node-date">{{ data.date }}</div>
        </template>

        <!-- Detail 模式：标题 + 日期 + Markdown 预览 -->
        <template v-else>
            <div class="node-title text-sm">{{ data.title }}</div>
            <div v-if="data.date" class="node-date">{{ data.date }}</div>
            <div v-if="data.preview" class="node-preview">{{ data.preview }}</div>
        </template>

        <!-- 悬停 Popover（Phase W-2 进阶） -->
        <Transition name="popover">
            <div v-if="isHovered && data.preview && !showOutline" class="card-popover">
                <p class="text-xs text-gray-400 line-clamp-3">{{ data.preview }}</p>
            </div>
        </Transition>

        <!-- ── 右侧出锚点 (source) ── -->
        <Handle type="source" :position="Position.Right" class="handle-node handle-source" />
    </div>
</template>

<style scoped>
/* ── Design Tokens (via Tailwind theme) ── */
.card-node {
    /* 颜色 tokens — 统一引用 tailwind.config.js */
    --c-border: theme('colors.ms-border');
    --c-carbon: theme('colors.ms-carbon');
    --c-panel: theme('colors.ms-panel');
    --c-neon: theme('colors.neon.DEFAULT');
    --c-text: theme('colors.gray.200');
    --c-text-dim: theme('colors.gray.500');
    --c-text-muted: theme('colors.gray.400');
    --c-zinc-500: theme('colors.zinc.500');
    --c-zinc-600: theme('colors.zinc.600');
    --c-handle-bg: theme('colors.ms-border');
    --c-handle-border: theme('colors.gray.600');

    /* RGB 分量用于 box-shadow / gradient 中的 alpha 组合 */
    --neon-rgb: 0, 229, 255;
    --carbon-rgb: 26, 26, 26;
    --panel-rgb: 34, 34, 34;
}

/* ── 基础卡片 ── */
.card-node {
    position: relative;
    min-width: 180px;
    max-width: 280px;
    padding: 12px 16px;
    border-radius: 2px;
    cursor: pointer;
    border: 1px solid var(--c-border);
    background: var(--c-carbon);
    transition:
        min-width 0.3s cubic-bezier(0.16, 1, 0.3, 1),
        max-width 0.3s cubic-bezier(0.16, 1, 0.3, 1),
        padding 0.3s cubic-bezier(0.16, 1, 0.3, 1),
        box-shadow 0.3s ease,
        border-color 0.3s ease,
        transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    will-change: transform, box-shadow;
    transform: translateZ(0);
    backface-visibility: hidden;
}

/* ── Hover 态：霓虹青向外晕染 ── */
.card-node:hover,
.card-hovered {
    border-color: rgba(var(--neon-rgb), 0.3);
    box-shadow:
        0 0 16px rgba(var(--neon-rgb), 0.2),
        0 0 32px rgba(var(--neon-rgb), 0.08);
    transform: scale(1.03);
}

/* ── Sequence 节点：左侧霓虹微光标识线 ── */
.card-sequence {
    box-shadow: 0 2px 12px rgba(var(--neon-rgb), 0.08);
}

.card-sequence::after {
    content: "";
    position: absolute;
    left: 0;
    top: 20%;
    bottom: 20%;
    width: 2px;
    background: linear-gradient(to bottom,
            transparent,
            rgba(var(--neon-rgb), 0.6),
            transparent);
    border-radius: 1px;
}

.card-sequence:hover {
    box-shadow:
        0 0 20px rgba(var(--neon-rgb), 0.25),
        0 0 40px rgba(var(--neon-rgb), 0.1);
}

/* ── Reference 节点：zinc 虚线 + 半透明 ── */
.card-reference {
    background: rgba(var(--carbon-rgb), 0.5);
    border: 1px dashed var(--c-zinc-500);
}

.card-reference:hover {
    border-color: var(--c-text-muted);
    background: rgba(var(--carbon-rgb), 0.7);
}

/* ── ☄️ 孤岛星尘 (Orphan Stardust) ── */
.card-orphan {
    min-width: 120px;
    max-width: 180px;
    padding: 8px 12px;
    background: rgba(var(--carbon-rgb), 0.3);
    border: 1px dashed var(--c-zinc-600);
    opacity: 0.5;
    transform: scale(0.7);
    transform-origin: center center;
}

.card-orphan .node-title {
    font-size: 11px;
    color: var(--c-text-muted);
}

.card-orphan:hover,
.card-orphan.card-hovered {
    opacity: 1;
    border-color: rgba(var(--neon-rgb), 0.3);
    background: rgba(var(--carbon-rgb), 0.7);
    transform: scale(0.85);
    box-shadow:
        0 0 12px rgba(var(--neon-rgb), 0.15),
        0 0 24px rgba(var(--neon-rgb), 0.05);
}

.card-orphan.card-selected {
    transform: scale(0.85);
}

.card-orphan .handle-node {
    width: 6px;
    height: 6px;
    opacity: 0.3;
}

.card-orphan:hover .handle-node {
    opacity: 1;
}

/* ── 选中状态：霓虹呼吸发光 ── */
.card-selected {
    border-color: var(--c-neon) !important;
    border-style: solid !important;
    box-shadow:
        0 0 16px rgba(var(--neon-rgb), 0.35),
        0 0 32px rgba(var(--neon-rgb), 0.15) !important;
    animation: neon-breathe 2s ease-in-out infinite;
}

/* ── 高亮状态（双向悬停） ── */
.card-highlighted {
    border-color: var(--c-neon) !important;
    box-shadow: 0 0 20px rgba(var(--neon-rgb), 0.35) !important;
}

@keyframes neon-breathe {

    0%,
    100% {
        box-shadow:
            0 0 16px rgba(var(--neon-rgb), 0.35),
            0 0 32px rgba(var(--neon-rgb), 0.15);
    }

    50% {
        box-shadow:
            0 0 24px rgba(var(--neon-rgb), 0.5),
            0 0 48px rgba(var(--neon-rgb), 0.2);
    }
}

/* ── Outline 缩小模式 ── */
.card-outline {
    min-width: 100px;
    max-width: 140px;
    padding: 6px 10px;
}

/* ── Detail 展开模式 ── */
.card-detail {
    min-width: 240px;
    max-width: 360px;
}

/* ── 文字样式 ── */
.node-title {
    font-weight: 600;
    font-size: 13px;
    color: var(--c-text);
    line-height: 1.4;
}

.node-date {
    font-size: 11px;
    color: var(--c-text-dim);
    margin-top: 4px;
}

.node-preview {
    font-size: 12px;
    color: var(--c-text-muted);
    margin-top: 6px;
    line-height: 1.5;
    max-height: 60px;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
}

/* ── 热度角标（暗色主题协调） ── */
.heat-badge {
    position: absolute;
    top: -6px;
    right: -6px;
    font-size: 10px;
    background: rgba(249, 115, 22, 0.15);
    color: theme('colors.orange.400');
    border: 1px solid rgba(249, 115, 22, 0.25);
    padding: 2px 6px;
    border-radius: 10px;
    line-height: 1;
}

/* ── Handle 锚点样式 ── */
.handle-node {
    width: 8px;
    height: 8px;
    background: var(--c-handle-bg);
    border: 2px solid var(--c-handle-border);
    transition: all 0.2s ease;
}

.handle-node:hover {
    background: var(--c-neon);
    border-color: var(--c-neon);
    box-shadow: 0 0 8px rgba(var(--neon-rgb), 0.5);
}

.card-selected .handle-node {
    background: var(--c-neon);
    border-color: var(--c-neon);
}

/* ── 悬停 Popover ── */
.card-popover {
    position: absolute;
    left: calc(100% + 12px);
    top: 0;
    width: 240px;
    padding: 12px;
    background: rgba(var(--panel-rgb), 0.9);
    backdrop-filter: blur(12px);
    border: 1px solid var(--c-border);
    border-radius: 2px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    z-index: 50;
    pointer-events: none;
}

.popover-enter-active,
.popover-leave-active {
    transition: opacity 150ms ease, transform 150ms ease;
}

.popover-enter-from,
.popover-leave-to {
    opacity: 0;
    transform: translateX(-8px);
}
</style>
