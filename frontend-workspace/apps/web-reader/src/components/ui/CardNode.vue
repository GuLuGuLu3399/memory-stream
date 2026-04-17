<script setup lang="ts">
/**
 * CardNode — 墨玉符牌（血肉神殿）
 *
 * 设计理念：每个节点是一块深色玉牌，烛光穿透，余烬明灭。
 *   - 顶部辉光条区分类型（经脉=血珀红 · 引渡=金缮金）
 *   - 悬停时烛光辉映（暖色径向渐变增强）
 *   - 选中时左侧血珀脉动条
 *   - 孤岛节点如幽灵碎玉，若隐若现
 *
 * 物理接口：Left (target) ← node → Right (source) — 匹配 Dagre LR
 * 容错机制：title 缺失显示 "无标题"，min-width/min-height 保底
 */

import { computed, ref, inject, type Ref } from "vue";
import { Handle, Position } from "@vue-flow/core";
import { useOblivionHeatmap } from "../../composables/useOblivionHeatmap";

/** 节点数据接口 */
interface CardNodeData {
    title?: string;
    date?: string;
    type?: string;
    heat?: number;
    preview?: string;
    isOrphan?: boolean;
}

const props = defineProps<{
    data: CardNodeData;
    id: string;
    selected?: boolean;
}>();

const isHovered = ref(false);
const isDragOver = ref(false);

// ── 遗忘热力学：CardNode 自管理衰减 opacity ──
const heatmapEnabled = inject<Ref<boolean>>("heatmapEnabled", { value: true } as Ref<boolean>);
const { getDecay } = useOblivionHeatmap();

const decayOpacity = computed(() => {
    if (!heatmapEnabled.value) return 1;
    return getDecay(props.id);
});

const displayTitle = computed(() => props.data?.title?.trim() || "无标题");
const isSequence = computed(() => props.data?.type === "sequence");
const heatGlow = computed(() => {
    const h = props.data?.heat || 0;
    if (h <= 3) return 0.2;
    if (h <= 7) return 0.35;
    return 0.5;
});
</script>

<template>
    <div
        class="jade"
        :class="{
            'jade-sequence': isSequence && !data?.isOrphan,
            'jade-reference': !isSequence && !data?.isOrphan,
            'jade-orphan': data?.isOrphan,
            'jade-selected': selected,
            'jade-hovered': isHovered,
        }"
        :style="{
            '--heat': `rgba(201,168,76,${heatGlow})`,
            '--decay-opacity': decayOpacity,
        }"
        @mouseenter="isHovered = true"
        @mouseleave="isHovered = false"
        @dragover.prevent="isDragOver = true"
        @dragleave="isDragOver = false"
        @drop="isDragOver = false"
    >
        <!-- Target handle (left) -->
        <Handle
            type="target"
            :position="Position.Left"
            class="copper-ring"
            :class="{ 'ring-lit': isDragOver }"
        />

        <!-- Top accent line -->
        <div class="accent-line" />

        <!-- Selected pulse bar -->
        <div v-if="selected" class="pulse-bar" />

        <!-- Content -->
        <div class="jade-body">
            <span class="jade-title">{{ displayTitle }}</span>
            <span v-if="data?.date" class="jade-date">{{ data.date }}</span>
        </div>

        <!-- Heat ember -->
        <span v-if="data?.heat" class="ember font-mono">{{ Math.round(data.heat) }}</span>

        <!-- Source handle (right) -->
        <Handle
            type="source"
            :position="Position.Right"
            class="copper-ring"
            :class="{ 'ring-lit': isDragOver }"
        />
    </div>
</template>

<style scoped>
/* ═══ Tokens — industrial dark (synced with Admin) ═══ */
.jade {
    --bg: #1a1a1a;
    --bg-warm: #2d2d2d;
    --border: #2a2a2a;
    --border-bright: #3a3a3a;
    --bone: #e0e0e0;
    --bone-dim: #cbd5e1;
    --copper: #333;
    --copper-light: #444;
    --ash: #71717a;
    --smoke: #555;
    --neon: #00e5ff;
    --neon-rgb: 0, 229, 255;
    --gold: #00e5ff;
    --gold-rgb: 0, 229, 255;
    --heat: rgba(0, 229, 255, 0.2);
    --decay-opacity: 1;
}

/* ═══ Base: dark jade — optical elevation, no soft blur ═══ */
.jade {
    position: relative;
    min-width: 180px;
    min-height: 48px;
    padding: 12px 18px;
    border-radius: 3px;
    cursor: pointer;
    opacity: var(--decay-opacity);

    background:
        radial-gradient(ellipse at 25% 0%, rgba(var(--neon-rgb), 0.04) 0%, transparent 50%),
        #1a1a1a;

    border: 1px solid var(--border);

    box-shadow:
        inset 0 1px 0 0 rgba(255, 255, 255, 0.04),
        2px 2px 0 0 rgba(0, 0, 0, 0.6);

    transition:
        opacity 0.4s ease,
        box-shadow 0.15s ease,
        border-color 0.25s ease,
        transform 0.15s ease,
        background 0.3s ease;
    will-change: transform, box-shadow;
    transform: translateZ(0);
}

/* ═══ Hover: candlelight warms the jade — mechanical lift ═══ */
.jade:hover,
.jade-hovered {
    border-color: var(--border-bright);
    transform: translate(-1px, -1px);

    background:
        radial-gradient(ellipse at 30% 0%, rgba(var(--neon-rgb), 0.09) 0%, transparent 50%),
        var(--bg-warm);

    box-shadow:
        inset 0 1px 0 0 rgba(255, 255, 255, 0.06),
        3px 3px 0 0 rgba(0, 0, 0, 0.6);
}

/* ═══ Top accent line — type identifier ═══ */
.accent-line {
    position: absolute;
    top: -1px;
    left: 16px;
    right: 16px;
    height: 1px;
    opacity: 0;
    transition: opacity 0.3s ease;
}

/* Sequence: neon cyan */
.jade-sequence .accent-line {
    background: linear-gradient(90deg, transparent, var(--neon) 30%, var(--neon) 70%, transparent);
    opacity: 0.5;
}

.jade-sequence {
    border-left: 2px solid rgba(var(--neon-rgb), 0.2);
}

.jade-sequence:hover {
    border-left-color: rgba(var(--neon-rgb), 0.4);
}

/* Reference: cool gray accent */
.jade-reference .accent-line {
    background: linear-gradient(90deg, transparent, var(--border-bright) 30%, var(--border-bright) 70%, transparent);
    opacity: 0.35;
}

.jade-reference {
    border: 1px dashed var(--border);
}

.jade-reference:hover {
    border-color: rgba(var(--gold-rgb), 0.5);
}

/* Hover intensifies accent */
.jade:hover .accent-line,
.jade-hovered .accent-line {
    opacity: 0.8;
}

/* ═══ Orphan: ghost jade ═══ */
.jade-orphan {
    min-width: 120px;
    max-width: 180px;
    padding: 8px 12px;
    background: rgba(18, 16, 12, 0.4);
    border: 1px dashed rgba(74, 66, 56, 0.3);
    opacity: 0.55;
    transform: scale(0.7);
    transform-origin: center;
    animation: ghostBreath 4s ease-in-out infinite;
}

@keyframes ghostBreath {
    0%, 100% { opacity: 0.5; }
    50% { opacity: 0.7; }
}

.jade-orphan:hover {
    opacity: 1;
    transform: scale(0.85);
    border-color: var(--border);
    background: rgba(26, 26, 26, 0.9);
    box-shadow:
        inset 0 1px 0 0 rgba(255, 255, 255, 0.03),
        2px 2px 0 0 rgba(0, 0, 0, 0.5);
}

.jade-orphan .jade-title {
    font-size: 11px;
    color: var(--ash);
}

/* ═══ Selected: neon cyan pulse — hard entity shadow ═══ */
.jade-selected {
    border-color: var(--neon) !important;
    border-style: solid !important;
    border-left-width: 3px !important;
    box-shadow:
        inset 0 1px 0 0 rgba(255, 255, 255, 0.05),
        0 0 8px rgba(var(--neon-rgb), 0.2),
        2px 2px 0 0 rgba(0, 0, 0, 0.6) !important;
}

.pulse-bar {
    position: absolute;
    left: 0;
    top: 18%;
    bottom: 18%;
    width: 3px;
    background: var(--neon);
    border-radius: 1px;
    animation: neonPulse 2s ease-in-out infinite;
}

@keyframes neonPulse {
    0%, 100% {
        opacity: 0.4;
        box-shadow: 0 0 4px rgba(var(--neon-rgb), 0.15);
    }
    50% {
        opacity: 1;
        box-shadow: 0 0 10px rgba(var(--neon-rgb), 0.35);
    }
}

/* ═══ Typography ═══ */
.jade-body {
    position: relative;
    z-index: 1;
}

.jade-title {
    display: block;
    color: var(--bone);
    font-weight: 600;
    font-size: 13px;
    line-height: 1.45;
    letter-spacing: 0.02em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.jade-date {
    display: block;
    font-size: 10px;
    color: var(--ash);
    margin-top: 4px;
    letter-spacing: 0.04em;
    font-variant-numeric: tabular-nums;
}

/* ═══ Heat ember — glowing corner badge ═══ */
.ember {
    position: absolute;
    top: -5px;
    right: -5px;
    font-size: 9px;
    line-height: 1;
    padding: 3px 6px;
    border-radius: 10px;
    color: var(--gold);
    background: var(--heat);
    border: 1px solid rgba(var(--gold-rgb), 0.15);
    box-shadow: 0 0 6px var(--heat);
    transition: box-shadow 0.3s ease;
    z-index: 2;
}

.jade:hover .ember {
    box-shadow: 0 0 12px var(--heat);
}

/* ═══ Handles: copper rings hidden until hover ═══ */
.copper-ring {
    width: 8px;
    height: 8px;
    background: var(--copper-light);
    border: 2px solid var(--copper);
    opacity: 0;
    transition:
        opacity 200ms ease,
        background 0.2s ease,
        border-color 0.2s ease,
        box-shadow 0.2s ease;
}

.jade:hover .copper-ring {
    opacity: 1;
}

.copper-ring:hover {
    background: var(--neon);
    border-color: var(--neon);
    box-shadow: 0 0 6px rgba(var(--neon-rgb), 0.3);
}

.ring-lit {
    background: var(--neon) !important;
    border-color: var(--neon) !important;
    box-shadow: 0 0 8px rgba(var(--neon-rgb), 0.4) !important;
    opacity: 1 !important;
}

.jade-orphan .copper-ring {
    width: 6px;
    height: 6px;
    opacity: 0.15;
}

.jade-orphan:hover .copper-ring {
    opacity: 1;
}
</style>
