<script setup lang="ts">
/**
 * StatsWidget — 灵签（血肉神殿）
 *
 * 折叠态：血珀封印 — 呼吸脉动的方形符印
 * 展开态：墨玉签文 — 三列数据 + sparkline 心电图
 * 关闭：极简下收箭头（▼），暗示"合上签文"
 */

import { ref, computed } from "vue";

const props = defineProps<{
    totalNodes: number;
    todayCount: number;
    avgHot: string;
    sortLabel: string;
    sparklineData: number[];
}>();

const expanded = ref(false);
const collapsed = ref(true); // 封印可见性：仅在签文完全收合后显示
const hoveredPoint = ref<number | null>(null);

function openLot() {
    collapsed.value = false;
    expanded.value = true;
}

function onLotAfterLeave() {
    collapsed.value = true;
}

const sparklinePoints = computed(() => {
    const data = props.sparklineData;
    if (!data || data.length < 2) return "";
    const max = Math.max(...data, 1);
    const w = 200;
    const h = 32;
    const step = w / (data.length - 1);
    return data
        .map((v, i) => `${(i * step).toFixed(1)},${(h - (v / max) * h * 0.8 - h * 0.1).toFixed(1)}`)
        .join(" ");
});

const sparklinePolygon = computed(() => {
    const points = sparklinePoints.value;
    if (!points) return "";
    return `0,32 ${points} 200,32`;
});

const dataPoints = computed(() => {
    const data = props.sparklineData;
    if (!data || data.length < 2) return [];
    const max = Math.max(...data, 1);
    const w = 200;
    const h = 32;
    const step = w / (data.length - 1);
    return data.map((v, i) => ({
        x: i * step,
        y: h - (v / max) * h * 0.8 - h * 0.1,
        value: Math.round(v),
        index: i
    }));
});

const handlePointHover = (index: number | null) => {
    hoveredPoint.value = index;
};

const handlePointLeave = () => {
    hoveredPoint.value = null;
};
</script>

<template>
    <div class="fixed right-6 bottom-6 z-30 select-none">
        <!-- 折叠态：血珀封印（签文离场动画结束后才显示） -->
        <Transition name="seal-pop">
            <button
                v-if="collapsed"
                @click="openLot"
                class="seal"
            :class="todayCount > 0 ? 'seal--alive' : 'seal--dormant'"
        >
            <span class="seal-value" :class="todayCount > 0 ? 'text-xuepo' : 'text-ms-smoke'">
                {{ todayCount > 0 ? `+${todayCount}` : totalNodes }}
            </span>
            <!-- 底部微光边 -->
            <div class="seal-edge" />
        </button>
        </Transition>

        <!-- 展开态：墨玉签文 -->
        <Transition name="lot-unfold" @after-leave="onLotAfterLeave">
            <div v-if="expanded" class="lot-panel">
                <!-- 签文顶部装饰线 -->
                <div class="lot-rule" />

                <!-- 头部 -->
                <div class="lot-header">
                    <div class="flex items-center gap-2">
                        <span class="lot-dot" />
                        <span class="lot-title">灵签</span>
                    </div>
                    <!-- 极简收合指示：下收线 -->
                    <button @click="expanded = false" class="lot-collapse" title="收合签文">
                        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                            <path d="M2 4.5L6 8L10 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
                    </button>
                </div>

                <!-- 三列数据 -->
                <div class="lot-stats">
                    <div class="lot-stat lot-stat--gold">
                        <span class="lot-stat__value lot-stat__value--gold">{{ totalNodes }}</span>
                        <span class="lot-stat__label">穴位</span>
                    </div>
                    <div class="lot-stat" :class="todayCount > 0 ? 'lot-stat--active' : ''">
                        <div class="lot-stat__value-row">
                            <span class="lot-stat__value" :class="todayCount > 0 ? 'text-xuepo' : 'text-ms-ash'">
                                {{ todayCount > 0 ? `+${todayCount}` : '0' }}
                            </span>
                            <svg v-if="todayCount > 0" class="lot-stat__trend" viewBox="0 0 10 10" fill="none">
                                <path d="M5 1L9 6H1L5 1Z" fill="currentColor" />
                            </svg>
                        </div>
                        <span class="lot-stat__label">今日</span>
                    </div>
                    <div class="lot-stat lot-stat--ember">
                        <span class="lot-stat__value lot-stat__value--ember">{{ avgHot }}</span>
                        <span class="lot-stat__label">均热</span>
                    </div>
                </div>

                <!-- Sparkline 心电图 -->
                <div v-if="sparklineData.length >= 2" class="sparkline-container">
                    <svg class="w-full h-full" viewBox="0 0 200 32" preserveAspectRatio="none">
                        <defs>
                            <linearGradient id="sparkGradLot" x1="0" y1="0" x2="0" y2="1">
                                <stop offset="0%" stop-color="rgba(166,38,38,0.25)" />
                                <stop offset="100%" stop-color="rgba(166,38,38,0)" />
                            </linearGradient>
                        </defs>
                        <polygon v-if="sparklinePolygon" :points="sparklinePolygon"
                            fill="url(#sparkGradLot)" class="sparkline-area" />
                        <polyline v-if="sparklinePoints" :points="sparklinePoints" fill="none"
                            stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"
                            class="text-xuepo sparkline-line" />
                        <!-- 悬停数据点 -->
                        <g v-if="hoveredPoint !== null && dataPoints[hoveredPoint]">
                            <circle
                                :cx="dataPoints[hoveredPoint].x"
                                :cy="dataPoints[hoveredPoint].y"
                                r="3"
                                class="fill-xuepo" />
                            <circle
                                :cx="dataPoints[hoveredPoint].x"
                                :cy="dataPoints[hoveredPoint].y"
                                r="6"
                                class="fill-xuepo/15" />
                        </g>
                    </svg>

                    <!-- 悬停热区 + 数值提示 -->
                    <div v-for="(point, idx) in dataPoints" :key="idx"
                        class="sparkline-zone"
                        :style="{ left: `${(point.x / 200) * 100}%`, width: `${100 / dataPoints.length}%` }"
                        @mouseenter="handlePointHover(idx)">
                        <Transition name="lot-tip">
                            <div v-if="hoveredPoint === idx" class="sparkline-tip">
                                {{ point.value }}
                            </div>
                        </Transition>
                    </div>
                </div>

                <!-- 底部签注 -->
                <div class="lot-footer">
                    <span class="lot-sort-badge">{{ sortLabel }}</span>
                </div>
            </div>
        </Transition>
    </div>
</template>

<style scoped>
/* ═══ 折叠态：血珀封印 ═══ */
.seal {
    position: relative;
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--c-bg);
    border: 1px solid var(--c-border);
    cursor: pointer;
    transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1);
}

.seal--dormant {
    --c-bg: #1c1814;
    --c-border: #3a3228;
}

.seal--alive {
    --c-bg: rgba(166, 38, 38, 0.08);
    --c-border: rgba(166, 38, 38, 0.25);
    box-shadow: 0 0 12px rgba(166, 38, 38, 0.12);
    animation: sealBreathe 3s ease-in-out infinite;
}

.seal:hover {
    transform: scale(1.08);
    border-color: #4a4238;
}

.seal--alive:hover {
    border-color: rgba(166, 38, 38, 0.4);
    box-shadow: 0 0 16px rgba(166, 38, 38, 0.2);
}

.seal-value {
    font-size: 15px;
    font-weight: 700;
    font-family: 'JetBrains Mono', monospace;
    position: relative;
    z-index: 1;
}

.seal-edge {
    position: absolute;
    bottom: 0;
    left: 20%;
    right: 20%;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(201, 168, 76, 0.15), transparent);
}

@keyframes sealBreathe {
    0%, 100% { box-shadow: 0 0 8px rgba(166, 38, 38, 0.08); }
    50% { box-shadow: 0 0 16px rgba(166, 38, 38, 0.18); }
}

/* ═══ 展开态：墨玉签文 ═══ */
.lot-panel {
    width: 240px;
    background: #1c1814;
    border: 1px solid #3a3228;
    border-radius: 3px;
    padding: 16px 18px;
    box-shadow:
        0 4px 16px rgba(0, 0, 0, 0.4),
        0 0 24px rgba(0, 0, 0, 0.2);
    transform-origin: bottom right;
}

/* 顶部金缮装饰线 */
.lot-rule {
    height: 1px;
    margin-bottom: 14px;
    background: linear-gradient(90deg, transparent, rgba(201, 168, 76, 0.25) 30%, rgba(201, 168, 76, 0.25) 70%, transparent);
}

/* 头部 */
.lot-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 14px;
}

.lot-dot {
    width: 4px;
    height: 4px;
    background: #a62626;
    border-radius: 50%;
    animation: dotPulse 2.5s ease-in-out infinite;
}

@keyframes dotPulse {
    0%, 100% { opacity: 0.5; }
    50% { opacity: 1; }
}

.lot-title {
    font-size: 11px;
    font-family: 'JetBrains Mono', monospace;
    color: #8a7e6e;
    text-transform: uppercase;
    letter-spacing: 0.12em;
}

/* 收合按钮：极简下收箭头 */
.lot-collapse {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: transparent;
    border: 1px solid transparent;
    color: #5a4f3e;
    cursor: pointer;
    border-radius: 2px;
    transition: all 150ms ease;
}

.lot-collapse:hover {
    color: #c8bfa8;
    background: rgba(58, 50, 40, 0.3);
    border-color: rgba(58, 50, 40, 0.5);
}

/* 三列数据 */
.lot-stats {
    display: flex;
    align-items: stretch;
    justify-content: space-between;
    gap: 6px;
    margin-bottom: 14px;
}

.lot-stat {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 8px 4px;
    border-radius: 3px;
    background: rgba(18, 16, 12, 0.5);
    border: 1px solid rgba(58, 50, 40, 0.2);
    transition: all 200ms ease;
}

.lot-stat--gold {
    background: linear-gradient(135deg, rgba(201, 168, 76, 0.06) 0%, rgba(18, 16, 12, 0.5) 100%);
    border-color: rgba(201, 168, 76, 0.12);
}

.lot-stat--ember {
    background: linear-gradient(135deg, rgba(217, 119, 6, 0.06) 0%, rgba(18, 16, 12, 0.5) 100%);
    border-color: rgba(217, 119, 6, 0.12);
}

.lot-stat--active {
    background: linear-gradient(135deg, rgba(166, 38, 38, 0.1) 0%, rgba(18, 16, 12, 0.5) 100%);
    border-color: rgba(166, 38, 38, 0.25);
    box-shadow: 0 0 8px rgba(166, 38, 38, 0.08);
}

.lot-stat__value-row {
    display: flex;
    align-items: center;
    gap: 3px;
}

.lot-stat__trend {
    width: 8px;
    height: 8px;
    color: #a62626;
    animation: trendBounce 2s ease-in-out infinite;
}

@keyframes trendBounce {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-2px); }
}

.lot-stat__value {
    font-size: 20px;
    font-weight: 700;
    font-family: 'JetBrains Mono', monospace;
    line-height: 1;
}

.lot-stat__value--gold {
    color: #c9a84c;
}

.lot-stat__value--ember {
    color: #d97706;
}

.lot-stat__label {
    font-size: 9px;
    font-family: 'JetBrains Mono', monospace;
    color: #5a4f3e;
    letter-spacing: 0.05em;
}

/* ═══ Sparkline ═══ */
.sparkline-container {
    position: relative;
    height: 32px;
    margin-bottom: 10px;
}

.sparkline-line {
    stroke-dasharray: 600;
    stroke-dashoffset: 600;
    animation: sparkDraw 1s cubic-bezier(0.37, 0, 0.63, 1) forwards;
}

.sparkline-area {
    opacity: 0;
    animation: sparkFade 0.6s ease-out 0.2s forwards;
}

@keyframes sparkDraw {
    to { stroke-dashoffset: 0; }
}

@keyframes sparkFade {
    to { opacity: 1; }
}

.sparkline-zone {
    position: absolute;
    inset: 0;
    cursor: crosshair;
    pointer-events: auto;
}

.sparkline-tip {
    position: absolute;
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%);
    margin-bottom: 4px;
    padding: 2px 6px;
    font-size: 10px;
    font-family: 'JetBrains Mono', monospace;
    color: #e8dfd0;
    background: #12100c;
    border: 1px solid #3a3228;
    border-radius: 2px;
    white-space: nowrap;
    pointer-events: none;
}

/* ═══ 底部签注 ═══ */
.lot-footer {
    padding-top: 10px;
    border-top: 1px solid rgba(58, 50, 40, 0.4);
}

.lot-sort-badge {
    font-size: 10px;
    font-family: 'JetBrains Mono', monospace;
    color: #c8bfa8;
    background: #12100c;
    padding: 3px 8px;
    border: 1px solid #3a3228;
    border-radius: 2px;
}

/* ═══ Tooltip 动画 ═══ */
.lot-tip-enter-active,
.lot-tip-leave-active {
    transition: all 150ms cubic-bezier(0.16, 1, 0.3, 1);
}

.lot-tip-enter-from,
.lot-tip-leave-to {
    opacity: 0;
    transform: translate(-50%, 3px);
}

/* ═══ 面板展开动画 ═══ */
.lot-unfold-enter-active {
    transition: all 280ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.lot-unfold-leave-active {
    transition: all 180ms cubic-bezier(0.4, 0, 0.2, 1);
}

.lot-unfold-enter-from,
.lot-unfold-leave-to {
    opacity: 0;
    transform: scale(0.85);
}

/* ═══ 封印出入动画 ═══ */
.seal-pop-enter-active {
    transition: all 220ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.seal-pop-leave-active {
    transition: all 150ms cubic-bezier(0.4, 0, 0.2, 1);
}

.seal-pop-enter-from {
    opacity: 0;
    transform: scale(0.6);
}

.seal-pop-leave-to {
    opacity: 0;
    transform: scale(0.8);
}
</style>
