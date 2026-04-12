<script setup lang="ts">
/**
 * StatsWidget — 灵签长条（血肉神殿）
 *
 * 水平长条固定在列表底部，替代旧的方格折叠态。
 * 常驻显示所有数据：排序切换 · 总数 · 今日 · 均热 · sparkline
 * 不再需要折叠/展开交互。
 */

import { ref, computed } from "vue";

const props = defineProps<{
    totalNodes: number;
    todayCount: number;
    avgHot: string;
    sortLabel: string;
    sparklineData: number[];
}>();

const emit = defineEmits<{
    toggleSort: [];
}>();

const hoveredPoint = ref<number | null>(null);

const sparklinePoints = computed(() => {
    const data = props.sparklineData;
    if (!data || data.length < 2) return "";
    const max = Math.max(...data, 1);
    const w = 160;
    const h = 24;
    const step = w / (data.length - 1);
    return data
        .map((v, i) => `${(i * step).toFixed(1)},${(h - (v / max) * h * 0.75 - h * 0.1).toFixed(1)}`)
        .join(" ");
});

const sparklinePolygon = computed(() => {
    const points = sparklinePoints.value;
    if (!points) return "";
    return `0,24 ${points} 160,24`;
});

const dataPoints = computed(() => {
    const data = props.sparklineData;
    if (!data || data.length < 2) return [];
    const max = Math.max(...data, 1);
    const w = 160;
    const h = 24;
    const step = w / (data.length - 1);
    return data.map((v, i) => ({
        x: i * step,
        y: h - (v / max) * h * 0.75 - h * 0.1,
        value: Math.round(v),
        index: i,
    }));
});

const handlePointHover = (index: number | null) => {
    hoveredPoint.value = index;
};
</script>

<template>
    <div class="fortune-strip">
        <!-- 左侧金色装饰端 -->
        <div class="fortune-strip__endcap" />

        <!-- 排序切换 -->
        <button class="fortune-strip__sort" @click="emit('toggleSort')">
            <svg class="w-3 h-3 transition-transform duration-300" :class="sortLabel === '热度排序' ? 'rotate-180' : ''"
                fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
            <span>{{ sortLabel }}</span>
        </button>

        <!-- 分隔符 -->
        <div class="fortune-strip__sep" />

        <!-- 数据区 -->
        <div class="fortune-strip__stats">
            <!-- 总数 -->
            <div class="fortune-strip__stat fortune-strip__stat--gold">
                <span class="fortune-strip__value fortune-strip__value--gold">{{ totalNodes }}</span>
                <span class="fortune-strip__label">穴位</span>
            </div>

            <!-- 今日 -->
            <div class="fortune-strip__stat" :class="todayCount > 0 ? 'fortune-strip__stat--active' : ''">
                <div class="fortune-strip__value-row">
                    <span class="fortune-strip__value" :class="todayCount > 0 ? 'text-xuepo' : 'text-ms-ash'">
                        {{ todayCount > 0 ? `+${todayCount}` : '0' }}
                    </span>
                    <svg v-if="todayCount > 0" class="fortune-strip__trend" viewBox="0 0 10 10" fill="none">
                        <path d="M5 1L9 6H1L5 1Z" fill="currentColor" />
                    </svg>
                </div>
                <span class="fortune-strip__label">今日</span>
            </div>

            <!-- 均热 -->
            <div class="fortune-strip__stat fortune-strip__stat--ember">
                <span class="fortune-strip__value fortune-strip__value--ember">{{ avgHot }}</span>
                <span class="fortune-strip__label">均热</span>
            </div>
        </div>

        <!-- 分隔符 -->
        <div class="fortune-strip__sep" />

        <!-- Sparkline -->
        <div v-if="sparklineData.length >= 2" class="fortune-strip__sparkline">
            <svg class="w-full h-full" viewBox="0 0 160 24" preserveAspectRatio="none">
                <defs>
                    <linearGradient id="sparkGradStrip" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stop-color="rgba(166,38,38,0.2)" />
                        <stop offset="100%" stop-color="rgba(166,38,38,0)" />
                    </linearGradient>
                </defs>
                <polygon v-if="sparklinePolygon" :points="sparklinePolygon" fill="url(#sparkGradStrip)" />
                <polyline v-if="sparklinePoints" :points="sparklinePoints" fill="none" stroke="currentColor"
                    stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="text-xuepo/70" />
                <!-- 悬停数据点 -->
                <g v-if="hoveredPoint !== null && dataPoints[hoveredPoint]">
                    <circle :cx="dataPoints[hoveredPoint].x" :cy="dataPoints[hoveredPoint].y" r="2.5"
                        class="fill-xuepo" />
                </g>
            </svg>
            <div v-for="(point, idx) in dataPoints" :key="idx" class="sparkline-zone"
                :style="{ left: `${(point.x / 160) * 100}%`, width: `${100 / dataPoints.length}%` }"
                @mouseenter="handlePointHover(idx)">
                <div v-if="hoveredPoint === idx" class="sparkline-tip">
                    {{ point.value }}
                </div>
            </div>
        </div>

        <!-- 右侧金色装饰端 -->
        <div class="fortune-strip__endcap" />
    </div>
</template>

<style scoped>
/* ═══ 灵签长条 ═══ */
.fortune-strip {
    display: flex;
    align-items: center;
    height: 40px;
    background: #141210;
    border-top: 1px solid rgba(58, 50, 40, 0.5);
    padding: 0 16px;
    gap: 0;
    user-select: none;
    flex-shrink: 0;
}

/* 金色装饰端帽 */
.fortune-strip__endcap {
    width: 2px;
    height: 16px;
    background: linear-gradient(180deg, transparent, rgba(201, 168, 76, 0.3), transparent);
    flex-shrink: 0;
}

/* 排序按钮 */
.fortune-strip__sort {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    font-size: 12px;
    font-family: 'JetBrains Mono', monospace;
    color: #c8bfa8;
    background: transparent;
    border: 1px solid rgba(58, 50, 40, 0.4);
    cursor: pointer;
    transition: all 150ms ease;
    flex-shrink: 0;
}

.fortune-strip__sort:hover {
    color: #e8dfd0;
    border-color: rgba(58, 50, 40, 0.7);
    background: rgba(58, 50, 40, 0.15);
}

.fortune-strip__sort:active {
    transform: translateY(1px);
}

/* 分隔符 */
.fortune-strip__sep {
    width: 1px;
    height: 18px;
    background: rgba(58, 50, 40, 0.4);
    margin: 0 12px;
    flex-shrink: 0;
}

/* 数据区 */
.fortune-strip__stats {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-shrink: 0;
}

.fortune-strip__stat {
    display: flex;
    align-items: center;
    gap: 6px;
}

.fortune-strip__value-row {
    display: flex;
    align-items: center;
    gap: 3px;
}

.fortune-strip__value {
    font-size: 14px;
    font-weight: 700;
    font-family: 'JetBrains Mono', monospace;
    line-height: 1;
}

.fortune-strip__value--gold {
    color: #c9a84c;
}

.fortune-strip__value--ember {
    color: #d97706;
}

.fortune-strip__label {
    font-size: 11px;
    font-family: 'JetBrains Mono', monospace;
    color: #5a4f3e;
}

.fortune-strip__trend {
    width: 7px;
    height: 7px;
    color: #a62626;
}

.fortune-strip__stat--active .fortune-strip__trend {
    animation: trendBounce 2s ease-in-out infinite;
}

@keyframes trendBounce {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-1.5px); }
}

/* Sparkline */
.fortune-strip__sparkline {
    position: relative;
    width: 160px;
    height: 24px;
    flex-shrink: 0;
    margin-left: auto;
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
    padding: 2px 5px;
    font-size: 10px;
    font-family: 'JetBrains Mono', monospace;
    color: #e8dfd0;
    background: #12100c;
    border: 1px solid #3a3228;
    border-radius: 2px;
    white-space: nowrap;
    pointer-events: none;
}
</style>
