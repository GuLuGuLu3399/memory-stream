<script setup lang="ts">
/**
 * StatsWidget — 灵签（血肉神殿）
 *
 * 竖直签条设计，模拟寺庙抽签：
 * - 折叠态：窄长竹签形态（38×140px），签头顶部圆弧 + 数字封印
 * - 展开态：签文展开，纵向堆叠数据 + sparkline
 * - 金缮装饰线、血珀脉动、墨玉签文
 */

import { ref, computed, onUnmounted, watch } from "vue";
import { useBreakpoints } from "../composables/useBreakpoints";

const { isMobile } = useBreakpoints();

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

const expanded = ref(false);
const collapsed = ref(true);
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
    const w = 160;
    const h = 28;
    const step = w / (data.length - 1);
    return data
        .map((v, i) => `${(i * step).toFixed(1)},${(h - (v / max) * h * 0.75 - h * 0.1).toFixed(1)}`)
        .join(" ");
});

const sparklinePolygon = computed(() => {
    const points = sparklinePoints.value;
    if (!points) return "";
    return `0,28 ${points} 160,28`;
});

const dataPoints = computed(() => {
    const data = props.sparklineData;
    if (!data || data.length < 2) return [];
    const max = Math.max(...data, 1);
    const w = 160;
    const h = 28;
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

// ── 点击外部收签 ──
const paperRef = ref<HTMLElement | null>(null);

function onDocClick(e: MouseEvent) {
    if (!expanded.value) return;
    const target = e.target as HTMLElement;
    // 排除签文面板自身和签条触发按钮
    if (paperRef.value?.contains(target)) return;
    if (target.closest('.fortune-stick')) return;
    expanded.value = false;
}

watch(expanded, (open) => {
    if (open) {
        // 延迟添加，避免触发按钮的 click 同步触发关闭
        setTimeout(() => document.addEventListener('click', onDocClick), 0);
    } else {
        document.removeEventListener('click', onDocClick);
    }
});

onUnmounted(() => {
    document.removeEventListener('click', onDocClick);
});
</script>

<template>
    <div class="fixed z-30 select-none"
        :class="isMobile ? 'right-3 bottom-16' : 'right-6 bottom-6'">
        <!-- 折叠态：竖直签条（竹签形态） -->
        <Transition name="stick-pop">
            <button v-if="collapsed" @click="openLot" class="fortune-stick"
                :class="[todayCount > 0 ? 'fortune-stick--alive' : 'fortune-stick--dormant', isMobile ? 'fortune-stick--mobile' : '']">
                <!-- 签头圆弧 -->
                <div class="fortune-stick__cap" />
                <!-- 签身 — 主数字 -->
                <div class="fortune-stick__body">
                    <span class="fortune-stick__number"
                        :class="todayCount > 0 ? 'text-xuepo' : 'text-ms-gold'">
                        {{ todayCount > 0 ? `+${todayCount}` : totalNodes }}
                    </span>
                    <span class="fortune-stick__sub">
                        {{ todayCount > 0 ? '今日' : '穴位' }}
                    </span>
                </div>
                <!-- 签底金线 -->
                <div class="fortune-stick__foot" />
            </button>
        </Transition>

        <!-- 展开态：签文面板 -->
        <Transition name="lot-unfold" @after-leave="onLotAfterLeave">
            <div v-if="expanded" ref="paperRef" class="fortune-paper">
                <!-- 顶部金缮线 -->
                <div class="fortune-paper__rule" />

                <!-- 签号头部 -->
                <div class="fortune-paper__header">
                    <span class="fortune-paper__seal" />
                    <span class="fortune-paper__title">灵签</span>
                </div>

                <!-- 签文数据 — 纵向堆叠 -->
                <div class="fortune-paper__stats">
                    <div class="fortune-paper__stat">
                        <span class="fortune-paper__value fortune-paper__value--gold">{{ totalNodes }}</span>
                        <span class="fortune-paper__label">穴位</span>
                    </div>
                    <div class="fortune-paper__divider" />
                    <div class="fortune-paper__stat" :class="todayCount > 0 ? 'fortune-paper__stat--active' : ''">
                        <div class="fortune-paper__value-row">
                            <span class="fortune-paper__value"
                                :class="todayCount > 0 ? 'text-xuepo' : 'text-ms-ash'">
                                {{ todayCount > 0 ? `+${todayCount}` : '0' }}
                            </span>
                            <svg v-if="todayCount > 0" class="fortune-paper__trend" viewBox="0 0 10 10" fill="none">
                                <path d="M5 1L9 6H1L5 1Z" fill="currentColor" />
                            </svg>
                        </div>
                        <span class="fortune-paper__label">今日</span>
                    </div>
                    <div class="fortune-paper__divider" />
                    <div class="fortune-paper__stat">
                        <span class="fortune-paper__value fortune-paper__value--ember">{{ avgHot }}</span>
                        <span class="fortune-paper__label">均热</span>
                    </div>
                </div>

                <!-- Sparkline -->
                <div v-if="sparklineData.length >= 2" class="fortune-paper__sparkline">
                    <svg class="w-full h-full" viewBox="0 0 160 28" preserveAspectRatio="none">
                        <defs>
                            <linearGradient id="sparkGradFortune" x1="0" y1="0" x2="0" y2="1">
                                <stop offset="0%" stop-color="rgba(166,38,38,0.2)" />
                                <stop offset="100%" stop-color="rgba(166,38,38,0)" />
                            </linearGradient>
                        </defs>
                        <polygon v-if="sparklinePolygon" :points="sparklinePolygon" fill="url(#sparkGradFortune)" />
                        <polyline v-if="sparklinePoints" :points="sparklinePoints" fill="none" stroke="currentColor"
                            stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="text-xuepo/70" />
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

                <!-- 排序切换 -->
                <div class="fortune-paper__footer">
                    <button class="fortune-paper__sort" @click="emit('toggleSort')">
                        <svg class="w-3 h-3 transition-transform duration-300"
                            :class="sortLabel === '热度排序' ? 'rotate-180' : ''" fill="none" stroke="currentColor"
                            viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                        </svg>
                        {{ sortLabel }}
                    </button>
                </div>
            </div>
        </Transition>
    </div>
</template>

<style scoped>
/* ═══ 折叠态：竖直签条 ═══ */
.fortune-stick {
    position: relative;
    width: 38px;
    height: 140px;
    display: flex;
    flex-direction: column;
    align-items: center;
    cursor: pointer;
    transition: all 300ms cubic-bezier(0.16, 1, 0.3, 1);
}

/* Mobile: compact stick */
.fortune-stick--mobile {
    width: 32px;
    height: 100px;
}

.fortune-stick--mobile .fortune-stick__cap {
    width: 24px;
    height: 12px;
}

.fortune-stick--mobile .fortune-stick__body {
    width: 24px;
}

.fortune-stick--mobile .fortune-stick__number {
    font-size: 14px;
}

.fortune-stick--mobile .fortune-stick__foot {
    width: 24px;
}

/* 签头圆弧 */
.fortune-stick__cap {
    width: 30px;
    height: 15px;
    border-radius: 15px 15px 0 0;
    transition: all 300ms ease;
}

.fortune-stick--dormant .fortune-stick__cap {
    background: linear-gradient(180deg, #2a2218, #1c1814);
    border: 1px solid rgba(58, 50, 40, 0.4);
    border-bottom: none;
}

.fortune-stick--alive .fortune-stick__cap {
    background: linear-gradient(180deg, rgba(166, 38, 38, 0.2), rgba(166, 38, 38, 0.08));
    border: 1px solid rgba(166, 38, 38, 0.3);
    border-bottom: none;
    box-shadow: 0 -4px 12px rgba(166, 38, 38, 0.1);
}

/* 签身 */
.fortune-stick__body {
    flex: 1;
    width: 30px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    transition: all 300ms ease;
}

.fortune-stick--dormant .fortune-stick__body {
    background: #1c1814;
    border-left: 1px solid rgba(58, 50, 40, 0.4);
    border-right: 1px solid rgba(58, 50, 40, 0.4);
}

.fortune-stick--alive .fortune-stick__body {
    background: linear-gradient(180deg, rgba(166, 38, 38, 0.08), rgba(18, 16, 12, 0.8), rgba(166, 38, 38, 0.08));
    border-left: 1px solid rgba(166, 38, 38, 0.2);
    border-right: 1px solid rgba(166, 38, 38, 0.2);
    animation: stickPulse 3s ease-in-out infinite;
}

@keyframes stickPulse {
    0%, 100% { box-shadow: 0 0 6px rgba(166, 38, 38, 0.06); }
    50% { box-shadow: 0 0 14px rgba(166, 38, 38, 0.14); }
}

.fortune-stick__number {
    font-size: 18px;
    font-weight: 700;
    font-family: 'JetBrains Mono', monospace;
    line-height: 1;
}

.fortune-stick__sub {
    font-size: 9px;
    font-family: 'JetBrains Mono', monospace;
    color: #5a4f3e;
}

/* 签底金线 */
.fortune-stick__foot {
    width: 30px;
    height: 2px;
    background: linear-gradient(90deg, transparent, rgba(201, 168, 76, 0.3), transparent);
    transition: all 300ms ease;
}

/* Hover 效果 */
.fortune-stick:hover {
    transform: translateY(-4px);
}

.fortune-stick--dormant:hover .fortune-stick__body {
    background: #221e18;
    border-color: rgba(58, 50, 40, 0.6);
}

.fortune-stick--alive:hover .fortune-stick__body {
    background: linear-gradient(180deg, rgba(166, 38, 38, 0.15), rgba(18, 16, 12, 0.9), rgba(166, 38, 38, 0.15));
    border-color: rgba(166, 38, 38, 0.35);
    animation: none;
}

.fortune-stick:hover .fortune-stick__foot {
    background: linear-gradient(90deg, transparent, rgba(201, 168, 76, 0.5), transparent);
}

.fortune-stick:active {
    transform: translateY(-1px) scale(0.97);
    transition-duration: 100ms;
}

/* ═══ 展开态：签文面板 ═══ */
.fortune-paper {
    width: 200px;
    background: #16140f;
    border: 1px solid rgba(58, 50, 40, 0.5);
    border-radius: 2px;
    padding: 16px;
    box-shadow:
        0 4px 20px rgba(0, 0, 0, 0.5),
        0 0 30px rgba(0, 0, 0, 0.3);
    transform-origin: bottom right;
}

/* 顶部金缮线 */
.fortune-paper__rule {
    height: 1px;
    margin-bottom: 14px;
    background: linear-gradient(90deg, transparent, rgba(201, 168, 76, 0.3) 30%, rgba(201, 168, 76, 0.3) 70%, transparent);
}

/* 头部 */
.fortune-paper__header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 16px;
}

.fortune-paper__seal {
    width: 6px;
    height: 6px;
    background: #a62626;
    border-radius: 50%;
    animation: sealPulse 2.5s ease-in-out infinite;
}

@keyframes sealPulse {
    0%, 100% { opacity: 0.5; }
    50% { opacity: 1; }
}

.fortune-paper__title {
    font-size: 12px;
    font-family: 'JetBrains Mono', monospace;
    color: #8a7e6e;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    flex: 1;
}

/* 纵向数据堆叠 */
.fortune-paper__stats {
    display: flex;
    flex-direction: column;
    gap: 0;
    margin-bottom: 14px;
}

.fortune-paper__stat {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 4px;
}

.fortune-paper__stat--active {
    background: linear-gradient(90deg, rgba(166, 38, 38, 0.06), transparent);
    margin: 0 -4px;
    padding: 8px 8px;
    border-radius: 2px;
}

.fortune-paper__value-row {
    display: flex;
    align-items: center;
    gap: 4px;
}

.fortune-paper__value {
    font-size: 16px;
    font-weight: 700;
    font-family: 'JetBrains Mono', monospace;
    line-height: 1;
}

.fortune-paper__value--gold {
    color: #c9a84c;
}

.fortune-paper__value--ember {
    color: #d97706;
}

.fortune-paper__trend {
    width: 7px;
    height: 7px;
    color: #a62626;
    animation: trendBounce 2s ease-in-out infinite;
}

@keyframes trendBounce {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-1.5px); }
}

.fortune-paper__label {
    font-size: 11px;
    font-family: 'JetBrains Mono', monospace;
    color: #5a4f3e;
}

.fortune-paper__divider {
    height: 1px;
    background: rgba(58, 50, 40, 0.3);
}

/* Sparkline */
.fortune-paper__sparkline {
    position: relative;
    height: 28px;
    margin-bottom: 12px;
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

/* 底部排序 */
.fortune-paper__footer {
    padding-top: 10px;
    border-top: 1px solid rgba(58, 50, 40, 0.35);
}

.fortune-paper__sort {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 5px 8px;
    font-size: 11px;
    font-family: 'JetBrains Mono', monospace;
    color: #8a7e6e;
    background: transparent;
    border: 1px solid rgba(58, 50, 40, 0.3);
    cursor: pointer;
    transition: all 150ms ease;
}

.fortune-paper__sort:hover {
    color: #c8bfa8;
    border-color: rgba(58, 50, 40, 0.5);
    background: rgba(58, 50, 40, 0.1);
}

/* ═══ 签条出入动画 ═══ */
.stick-pop-enter-active {
    transition: all 250ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.stick-pop-leave-active {
    transition: all 180ms cubic-bezier(0.4, 0, 0.2, 1);
}

.stick-pop-enter-from {
    opacity: 0;
    transform: translateY(20px) scaleY(0.8);
}

.stick-pop-leave-to {
    opacity: 0;
    transform: translateY(10px) scaleY(0.9);
}

/* ═══ 签文展开动画 ═══ */
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
</style>
