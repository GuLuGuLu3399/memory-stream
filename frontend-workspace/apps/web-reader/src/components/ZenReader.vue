<script setup lang="ts">
/**
 * ZenReader — 内殿祭坛（全屏禅模式）
 *
 * z-fullscreen 覆盖一切，沉浸式阅读体验。
 * 100% 宽度 Markdown 渲染（max-width: 72ch 或 88ch 可切换）
 * 右下角 FloatingCompass 悬浮目录导航
 * 退出：ESC 键、下滑手势、点击退出按钮
 */

import { ref, watch, nextTick, onMounted, onUnmounted, computed } from "vue";

import { storeToRefs } from "pinia";
import MarkdownViewer from "@memory-stream/ui-shared/components/MarkdownViewer.vue";
import { useGraphStore } from "../store/useGraphStore";
import { useCards } from "../composables/useCards";
import { useActiveHeading } from "../composables/useActiveHeading";
import FloatingCompass from "./FloatingCompass.vue";
import type { CardDetail, TocItem } from "../composables/useCards";

const store = useGraphStore();
const { zenMode, selectedId } = storeToRefs(store);
const { loadDetail } = useCards();

const detail = ref<CardDetail | null>(null);
const tocItems = ref<TocItem[]>([]);
const proseRef = ref<HTMLElement>();

const readProgress = ref(0);
const proseWidth = ref<'prose' | 'reading'>('prose');

const proseMaxWidth = computed(() => {
    return proseWidth.value === 'prose' ? 'max-w-prose' : 'max-w-[88ch]';
});

const calculateProgress = () => {
    if (!proseRef.value) return;
    const { scrollTop, scrollHeight, clientHeight } = proseRef.value;
    const maxScroll = scrollHeight - clientHeight;
    readProgress.value = maxScroll <= 0 ? 100 : Math.round((scrollTop / maxScroll) * 100);
};

const { activeSlug, delayedRefresh } = useActiveHeading(proseRef);

const toggleProseWidth = () => {
    proseWidth.value = proseWidth.value === 'prose' ? 'reading' : 'prose';
};

// 下滑退出手势
let swipeStartY = 0;
let swipeTracking = false;
const SWIPE_THRESHOLD = 100;

const onTouchStart = (e: TouchEvent) => {
    const touch = e.touches[0];
    swipeStartY = touch.clientY;
    swipeTracking = true;
};

const onTouchMove = (e: TouchEvent) => {
    if (!swipeTracking) return;
    const touch = e.touches[0];
    const dy = touch.clientY - swipeStartY;
    const dx = Math.abs(touch.clientX - e.touches[0].clientX);

    // 如果水平滑动大于垂直，取消追踪
    if (dx > Math.abs(dy) && dx > 10) {
        swipeTracking = false;
        return;
    }
};

const onTouchEnd = (e: TouchEvent) => {
    if (!swipeTracking) return;
    swipeTracking = false;

    const touch = e.changedTouches[0];
    const dy = touch.clientY - swipeStartY;

    if (dy > SWIPE_THRESHOLD) {
        store.toggleZenMode();
    }
};

watch([zenMode, selectedId], async ([zen, id]) => {
    if (!zen || !id) {
        detail.value = null;
        tocItems.value = [];
        return;
    }

    try {
        const result = await loadDetail(id);
        if (!result) {
            detail.value = null;
            return;
        }
        detail.value = result;
        tocItems.value = result.tocData ?? [];

        await nextTick();
        delayedRefresh();
    } catch (err) {
        console.error("[ZenReader] load failed:", err);
        detail.value = null;
    }
}, { immediate: true });

watch(zenMode, (zen) => {
    document.body.style.overflow = zen ? "hidden" : "";
});

// ── Esc 键退出禅模式 ──
function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && zenMode.value) {
        e.preventDefault();
        store.toggleZenMode();
    }
}

// ── 金缮帘杆：鼠标靠近顶部时浮现 ──
const showExitRod = ref(false);
let exitRodTimer: ReturnType<typeof setTimeout> | null = null;

function onZenMouseMove(e: MouseEvent) {
    if (!zenMode.value) return;
    if (e.clientY < 30) {
        showExitRod.value = true;
        if (exitRodTimer) { clearTimeout(exitRodTimer); exitRodTimer = null; }
    } else if (showExitRod.value && e.clientY > 80) {
        if (exitRodTimer) clearTimeout(exitRodTimer);
        exitRodTimer = setTimeout(() => { showExitRod.value = false; }, 600);
    }
}

function exitZen() {
    store.toggleZenMode();
}

onMounted(() => {
    document.addEventListener("keydown", onKeydown);
    document.addEventListener("mousemove", onZenMouseMove);
});

onUnmounted(() => {
    document.body.style.overflow = "";
    document.removeEventListener("keydown", onKeydown);
    document.removeEventListener("mousemove", onZenMouseMove);
    if (exitRodTimer) clearTimeout(exitRodTimer);
});
</script>

<template>
    <Teleport to="body">
        <Transition name="ms-scale">
            <div v-if="zenMode && detail"
                class="fixed inset-0 z-fullscreen bg-ms-xuan flex zen-container zen-vignette"
                @touchstart="onTouchStart"
                @touchmove="onTouchMove"
                @touchend="onTouchEnd">

                <!-- 金缮顶线装饰 - 笔触动画 -->
                <div class="zen-gold-topline">
                    <svg class="zen-brush-svg" viewBox="0 0 1200 2" preserveAspectRatio="none">
                        <path class="zen-brush-path" d="M 0 1 Q 300 1, 600 1 T 1200 1"
                            fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                    </svg>
                </div>

                <!-- 血珀参考线 - 左右边缘 -->
                <div class="zen-guideline-left"></div>
                <div class="zen-guideline-right"></div>

                <!-- 金缮帘杆 — 鼠标靠近顶部浮现的退出入口 -->
                <Transition name="zen-rod">
                    <button v-if="showExitRod"
                        class="zen-exit-rod"
                        @click="exitZen"
                        title="退出禅模式 (ESC)">
                        <span class="zen-exit-rod__line" />
                        <span class="zen-exit-rod__diamond">
                            <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                                <path d="M5 1L9 5L5 9L1 5Z" stroke="currentColor" stroke-width="1" fill="none" />
                            </svg>
                        </span>
                        <span class="zen-exit-rod__line" />
                    </button>
                </Transition>

                <!-- 全屏 Markdown 阅读 -->
                <div class="flex-1 overflow-y-auto scrollbar-thin text-stone-300 relative z-[2]" ref="proseRef" @scroll="calculateProgress">
                    <div class="mx-auto px-8 py-16 zen-content-area [&_li_p]:m-0 [&_li]:my-1" :class="proseMaxWidth" @dblclick="toggleProseWidth">
                        <!-- 标题 - 淡入上滑动画 -->
                        <Transition name="ms-fade-slide-up" appear>
                            <h1 class="text-3xl font-bold text-zinc-200 mb-8 leading-tight font-serif">
                                {{ detail.title }}
                            </h1>
                        </Transition>

                        <!-- 正文 -->
                        <MarkdownViewer :html-content="detail.html" />
                    </div>
                </div>

                <!-- 禅进度条 - 底部金色细线 -->
                <div class="fixed bottom-0 left-0 right-0 h-[2px] bg-ms-mo z-[2]">
                    <div class="zen-progress-fill" :style="{ width: `${readProgress}%` }"></div>
                </div>

                <!-- 悬浮阅读罗盘 -->
                <div v-if="tocItems.length > 0" class="fixed bottom-6 right-6 z-[1]">
                    <FloatingCompass :toc-items="tocItems" :active-slug="activeSlug" :container-el="proseRef"
                        :read-progress="readProgress" />
                </div>

                <!-- 宽度切换提示 -->
                <Transition name="fade">
                    <div v-if="proseWidth === 'reading'" class="fixed bottom-4 left-4 z-[1]">
                        <span class="text-2xs font-mono text-ms-smoke bg-ms-xuan/90 border border-ms-copper/30 px-2 py-1 rounded">
                            88ch 阅读宽
                        </span>
                    </div>
                </Transition>
            </div>
        </Transition>
    </Teleport>
</template>

<style scoped>
/* ── 环境暗场：沉浸深渊暗角 ── */
.zen-vignette::after {
    content: '';
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 1;
    background: radial-gradient(
        ellipse 70% 60% at 50% 50%,
        transparent 50%,
        rgba(10, 8, 6, 0.4) 100%
    );
}

/* ── 金缮帘杆：退出禅境的隐形入口 ── */
.zen-exit-rod {
    position: fixed;
    top: 0;
    left: 50%;
    transform: translateX(-50%);
    z-index: 80;
    display: flex;
    align-items: center;
    justify-content: center;
    width: max-content;
    background: none;
    border: none;
    padding: 10px 20px 6px;
    cursor: pointer;
}

.zen-exit-rod__line {
    display: block;
    width: 80px;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(201, 168, 76, 0.35), transparent);
    transition: all 400ms cubic-bezier(0.16, 1, 0.3, 1);
}

.zen-exit-rod__diamond {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    color: rgba(201, 168, 76, 0.4);
    transform: rotate(0deg);
    transition: all 400ms cubic-bezier(0.16, 1, 0.3, 1);
}

.zen-exit-rod:hover .zen-exit-rod__line {
    width: 120px;
    background: linear-gradient(90deg, transparent, rgba(201, 168, 76, 0.6), transparent);
}

.zen-exit-rod:hover .zen-exit-rod__diamond {
    color: rgba(201, 168, 76, 0.8);
    transform: rotate(45deg) scale(1.2);
    filter: drop-shadow(0 0 4px rgba(201, 168, 76, 0.4));
}

.zen-exit-rod:active .zen-exit-rod__diamond {
    color: rgba(166, 38, 38, 0.9);
    transform: rotate(45deg) scale(0.9);
    filter: drop-shadow(0 0 6px rgba(166, 38, 38, 0.5));
    transition-duration: 100ms;
}

/* 帘杆出入动画 */
.zen-rod-enter-active {
    transition: all 400ms cubic-bezier(0.16, 1, 0.3, 1);
}

.zen-rod-leave-active {
    transition: all 300ms cubic-bezier(0.4, 0, 0.2, 1);
}

.zen-rod-enter-from {
    opacity: 0;
    transform: translateX(-50%) translateY(-8px);
}

.zen-rod-leave-to {
    opacity: 0;
    transform: translateX(-50%) translateY(-4px);
}

/* ── 金缮顶线装饰：笔触动画 ── */
.zen-gold-topline {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    z-index: 2;
    overflow: hidden;
}

.zen-brush-svg {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    color: theme('colors.ms-gold');
}

.zen-brush-path {
    stroke-dasharray: 1200;
    stroke-dashoffset: 1200;
    animation: zen-brush-draw 2.5s cubic-bezier(0.37, 0, 0.63, 1) forwards,
               zen-gold-pulse 3s ease-in-out infinite 2.5s;
}

@keyframes zen-brush-draw {
    to {
        stroke-dashoffset: 0;
    }
}

@keyframes zen-gold-pulse {
    0%, 100% {
        filter: drop-shadow(0 0 4px rgba(201, 168, 76, 0.3));
    }
    50% {
        filter: drop-shadow(0 0 12px rgba(201, 168, 76, 0.5));
    }
}

/* ── 血珀参考线：左右边缘微弱脉动 ── */
.zen-guideline-left,
.zen-guideline-right {
    position: fixed;
    top: 0;
    bottom: 0;
    width: 1px;
    background: linear-gradient(to bottom,
        transparent 0%,
        rgba(166, 38, 38, 0.08) 20%,
        rgba(166, 38, 38, 0.12) 50%,
        rgba(166, 38, 38, 0.08) 80%,
        transparent 100%);
    z-index: 1;
    animation: zen-guideline-pulse 4s ease-in-out infinite;
    pointer-events: none;
}

.zen-guideline-left {
    left: max(calc(50% - 36ch), 16px);
}

.zen-guideline-right {
    right: max(calc(50% - 36ch), 16px);
}

@keyframes zen-guideline-pulse {
    0%, 100% {
        opacity: 0.6;
    }
    50% {
        opacity: 1;
    }
}

/* ── 禅进度条：金色辉光 ── */
.zen-progress-fill {
    height: 100%;
    background: theme('colors.ms-gold');
    box-shadow:
        0 0 8px rgba(201, 168, 76, 0.4),
        0 0 16px rgba(201, 168, 76, 0.2);
    transition: width 150ms ease-out;
}

/* ── 标题淡入上滑动画 ── */
.ms-fade-slide-up-enter-active {
    transition: all 600ms cubic-bezier(0.16, 1, 0.3, 1);
}

.ms-fade-slide-up-enter-from {
    opacity: 0;
    transform: translateY(20px);
}

.ms-fade-slide-up-enter-to {
    opacity: 1;
    transform: translateY(0);
}

/* ── 宽度切换提示淡入淡出 ── */
.fade-enter-active,
.fade-leave-active {
    transition: opacity 300ms ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}

/* ── 缩放过渡动画 ── */
.ms-scale-enter-active {
    transition: all 300ms cubic-bezier(0.16, 1, 0.3, 1);
}

.ms-scale-leave-active {
    transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
}

.ms-scale-enter-from,
.ms-scale-leave-to {
    opacity: 0;
    transform: scale(0.98);
}

/* ── 内容区域：双击提示 ── */
.zen-content-area {
    cursor: default;
    user-select: text;
}

.zen-content-area:active {
    cursor: text;
}
</style>
