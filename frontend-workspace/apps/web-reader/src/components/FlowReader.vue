<script setup lang="ts">
/**
 * FlowReader — 拓扑潜流阅读器
 *
 * 将 SEQ 链条中所有卡片按序拼接成线性阅读流。
 * 全屏覆盖层，从底部滑入，沉浸式阅读。
 *
 * 视觉：玄色基底 + 金缮分割线 + WASM Markdown 渲染
 */

import { ref, watch, computed, nextTick } from "vue";
import { ChevronLeft, ChevronRight, BookOpen, X } from "lucide-vue-next";
import MarkdownViewer from "@memory-stream/ui-shared/components/MarkdownViewer.vue";
import SkeletonBlock from "@memory-stream/ui-shared/components/SkeletonBlock.vue";
import { useCards } from "../composables/useCards";
import { resolveWikilinkTarget } from "../composables/useWikilinkNavigation";
import type { CardDetail } from "../composables/useCards";

const props = defineProps<{
    open: boolean;
    chainIds: string[];
    currentNodeId: string | null;
}>();

const emit = defineEmits<{
    close: [];
    navigate: [nodeId: string];
}>();

const { loadDetail } = useCards();

const cards = ref<Map<string, CardDetail>>(new Map());
const loading = ref(false);
const currentIndex = ref(0);
const scrollContainer = ref<HTMLElement | null>(null);
const touchStartX = ref(0);
const touchDeltaX = ref(0);

const currentNode = computed(() => {
    const id = props.chainIds[currentIndex.value];
    return id ? cards.value.get(id) : null;
});

const totalCount = computed(() => props.chainIds.length);
const hasPrev = computed(() => currentIndex.value > 0);
const hasNext = computed(() => currentIndex.value < totalCount.value - 1);

// 当 open 且 chainIds 变化时，加载所有卡片
watch(
    () => [props.open, props.chainIds] as const,
    async ([isOpen, ids]) => {
        if (!isOpen || ids.length === 0) return;

        // 定位当前节点
        if (props.currentNodeId) {
            const idx = ids.indexOf(props.currentNodeId);
            currentIndex.value = idx >= 0 ? idx : 0;
        }

        // 加载所有卡片详情
        loading.value = true;
        const loaded = new Map<string, CardDetail>();

        for (const id of ids) {
            try {
                const detail = await loadDetail(id);
                if (detail) loaded.set(id, detail);
            } catch {
                // 跳过加载失败的卡片
            }
        }

        cards.value = loaded;
        loading.value = false;

        // 滚动到顶部
        await nextTick();
        if (scrollContainer.value) {
            scrollContainer.value.scrollTop = 0;
        }
    },
    { immediate: true },
);

function close() {
    emit("close");
}

function goTo(index: number) {
    if (index < 0 || index >= totalCount.value) return;
    currentIndex.value = index;
    emit("navigate", props.chainIds[index]);

    // 滚动到顶部
    nextTick(() => {
        if (scrollContainer.value) {
            scrollContainer.value.scrollTop = 0;
        }
    });
}

function goPrev() {
    goTo(currentIndex.value - 1);
}

function goNext() {
    goTo(currentIndex.value + 1);
}

function handleKeydown(e: KeyboardEvent) {
    if (!props.open) return;
    if (e.key === "Escape") close();
    if (e.key === "ArrowLeft") goPrev();
    if (e.key === "ArrowRight") goNext();
}

function onTouchStart(e: TouchEvent) {
    touchStartX.value = e.changedTouches[0]?.clientX ?? 0;
    touchDeltaX.value = 0;
}

function onTouchMove(e: TouchEvent) {
    const currentX = e.changedTouches[0]?.clientX ?? touchStartX.value;
    touchDeltaX.value = currentX - touchStartX.value;
}

function onTouchEnd() {
    if (touchDeltaX.value <= -50) {
        goNext();
    } else if (touchDeltaX.value >= 50) {
        goPrev();
    }
    touchDeltaX.value = 0;
}

// 🗡️ Wikilink 点击：导航到目标卡片
async function onWikilinkClick(targetId: string) {
    const resolvedId = await resolveWikilinkTarget(targetId);
    if (!resolvedId) return;

    const idx = props.chainIds.indexOf(resolvedId);
    if (idx >= 0) {
        goTo(idx);
    } else {
        // 如果不在当前 chain 中，也可以直接导航（由父组件决定）
        emit('navigate', resolvedId);
    }
}
</script>

<template>
    <Transition name="flow-slide">
        <div v-if="open" class="flow-reader z-fullscreen" tabindex="0" @keydown="handleKeydown"
            @touchstart.passive="onTouchStart" @touchmove.passive="onTouchMove" @touchend.passive="onTouchEnd">
            <!-- 禅系金缮顶线 -->
            <div class="flow-zen-topline">
                <svg class="flow-brush-svg" viewBox="0 0 1200 2" preserveAspectRatio="none">
                    <path class="flow-brush-path" d="M 0 1 Q 300 1, 600 1 T 1200 1" fill="none" stroke="currentColor"
                        stroke-width="2" stroke-linecap="round" />
                </svg>
            </div>

            <!-- 禅系边缘导线 -->
            <div class="flow-guideline-left"></div>
            <div class="flow-guideline-right"></div>

            <!-- 顶栏 -->
            <div class="flow-header">
                <div class="flow-header__left">
                    <BookOpen :size="14" class="text-ms-gold" />
                    <span class="flow-header__label">潜流阅读</span>
                    <span class="flow-header__counter font-mono">
                        {{ currentIndex + 1 }} / {{ totalCount }}
                    </span>
                </div>
                <div class="flow-header__right">
                    <button class="flow-nav-btn" :disabled="!hasPrev" @click="goPrev">
                        <ChevronLeft :size="16" />
                    </button>
                    <button class="flow-nav-btn" :disabled="!hasNext" @click="goNext">
                        <ChevronRight :size="16" />
                    </button>
                    <button class="flow-close-btn" @click="close">
                        <X :size="16" />
                    </button>
                </div>
            </div>

            <!-- 金缮分割线 -->
            <div class="flow-gold-rule" />

            <!-- 内容区 -->
            <div ref="scrollContainer" class="flow-body scrollbar-thin">
                <!-- 加载态 -->
                <div v-if="loading" class="flow-loading">
                    <SkeletonBlock variant="text" :lines="6" />
                    <div class="pt-6 border-t border-ms-copper/30">
                        <SkeletonBlock variant="text" :lines="4" />
                    </div>
                </div>

                <!-- 卡片内容 -->
                <div v-else-if="currentNode" class="flow-content prose-container">
                    <!-- 卡片标题 -->
                    <h2 class="flow-card-title">{{ currentNode.title }}</h2>

                    <!-- 卡片序号指示 -->
                    <div v-if="totalCount > 1" class="flow-chain-dots">
                        <button v-for="(id, idx) in chainIds" :key="id" class="flow-chain-dot"
                            :class="{ 'flow-chain-dot--active': idx === currentIndex }" @click="goTo(idx)" />
                    </div>

                    <!-- Markdown 渲染 -->
                    <MarkdownViewer :html-content="currentNode.html" @wikilink-click="onWikilinkClick" />
                </div>

                <!-- 空态 -->
                <div v-else class="flow-empty">
                    <span class="text-ms-smoke text-sm">无法加载内容</span>
                </div>
            </div>

            <!-- 底栏：进度条 -->
            <div v-if="totalCount > 1" class="flow-progress">
                <div class="flow-progress__fill" :style="{ width: `${((currentIndex + 1) / totalCount) * 100}%` }" />
            </div>

        </div>
    </Transition>
</template>

<style scoped>
/* ═══ 全屏覆盖层 ═══ */
.flow-reader {
    position: fixed;
    inset: 0;
    display: flex;
    flex-direction: column;
    background: theme("colors.ms-xuan");
    outline: none;
    height: 100dvh;
    isolation: isolate;
}

/* ── 禅系暗场 ── */
.flow-reader::after {
    content: '';
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 0;
    background: radial-gradient(ellipse 70% 60% at 50% 50%, transparent 50%, rgba(10, 8, 6, 0.38) 100%);
}

/* ═══ 顶栏 ═══ */
.flow-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    background: rgba(22, 20, 17, 0.88);
    border-bottom: 1px solid rgba(201, 168, 76, 0.16);
    flex-shrink: 0;
    position: relative;
    z-index: 2;
}

.flow-header__left {
    display: flex;
    align-items: center;
    gap: 8px;
}

.flow-header__label {
    font-size: 12px;
    font-weight: 600;
    color: theme("colors.ms-gold");
    letter-spacing: 0.05em;
}

.flow-header__counter {
    font-size: 10px;
    color: theme("colors.ms-ash");
}

.flow-header__right {
    display: flex;
    align-items: center;
    gap: 4px;
}

/* 导航按钮 — mechanical stamp */
.flow-nav-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    color: theme("colors.ms-smoke");
    background: theme("colors.ms-xiang");
    border: 1px solid theme("colors.ms-copper");
    box-shadow: 2px 2px 0 0 rgba(0, 0, 0, 0.5);
    border-radius: 2px;
    cursor: pointer;
    transition: transform 0.15s ease, box-shadow 0.15s ease, color 0.15s ease;
}

.flow-nav-btn:hover:not(:disabled) {
    color: theme("colors.ms-bone");
    transform: translate(-1px, -1px);
    box-shadow: 3px 3px 0 0 rgba(0, 0, 0, 0.5);
}

.flow-nav-btn:active:not(:disabled) {
    transform: translate(1px, 1px);
    box-shadow: 0px 0px 0 0 rgba(0, 0, 0, 0.5);
}

.flow-nav-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
}

.flow-close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    color: theme("colors.ms-bone");
    background: rgba(166, 38, 38, 0.18);
    border: 1px solid rgba(166, 38, 38, 0.45);
    border-radius: 2px;
}

/* ═══ 金缮分割线 ═══ */
.flow-gold-rule {
    height: 1px;
    flex-shrink: 0;
    position: relative;
    z-index: 2;
    background: linear-gradient(90deg,
            transparent 0%,
            theme("colors.ms-gold") 20%,
            theme("colors.ms-gold-dim") 50%,
            theme("colors.ms-gold") 80%,
            transparent 100%);
}

/* ── 禅系笔触顶线 ── */
.flow-zen-topline {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    z-index: 3;
    overflow: hidden;
}

.flow-brush-svg {
    width: 100%;
    height: 100%;
    color: theme("colors.ms-gold");
}

.flow-brush-path {
    stroke-dasharray: 1200;
    stroke-dashoffset: 1200;
    animation: flow-brush-draw 2.2s cubic-bezier(0.37, 0, 0.63, 1) forwards;
}

/* ── 禅系边缘导线 ── */
.flow-guideline-left,
.flow-guideline-right {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    z-index: 1;
    background: linear-gradient(180deg,
            transparent 0%,
            rgba(166, 38, 38, 0.1) 8%,
            rgba(166, 38, 38, 0.16) 35%,
            rgba(166, 38, 38, 0.1) 72%,
            transparent 100%);
}

.flow-guideline-left {
    left: 14px;
}

.flow-guideline-right {
    right: 14px;
}


/* ═══ 内容区 ═══ */
.flow-body {
    flex: 1;
    overflow-y: auto;
    padding: 32px 40px;
    -webkit-overflow-scrolling: touch;
    overscroll-behavior: contain;
    position: relative;
    z-index: 2;
}

.flow-loading {
    max-width: 72ch;
    margin: 0 auto;
    padding: 40px 0;
}

.flow-content {
    max-width: 72ch;
    margin: 0 auto;
}

/* 卡片标题 */
.flow-card-title {
    font-size: 20px;
    font-weight: 700;
    color: theme("colors.ms-ivory");
    letter-spacing: 0.02em;
    line-height: 1.4;
    margin: 0 0 24px;
}

/* 链条圆点指示器 */
.flow-chain-dots {
    display: flex;
    gap: 6px;
    margin-bottom: 28px;
}

.flow-chain-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: theme("colors.ms-copper");
    border: none;
    cursor: pointer;
    padding: 0;
    transition: all 150ms ease;
}

.flow-chain-dot:hover {
    background: theme("colors.ms-ash");
    transform: scale(1.3);
}

.flow-chain-dot--active {
    background: theme("colors.ms-gold");
    box-shadow: 0 0 4px rgba(201, 168, 76, 0.4), 1px 1px 0 0 rgba(0, 0, 0, 0.5);
}

.flow-chain-dot--active:hover {
    background: theme("colors.ms-gold");
}

/* 空态 */
.flow-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 200px;
}

/* ═══ 底栏进度条 ═══ */
.flow-progress {
    height: 2px;
    background: theme("colors.ms-mo");
    flex-shrink: 0;
    position: relative;
    z-index: 2;
}

.flow-progress__fill {
    height: 100%;
    background: linear-gradient(90deg, rgba(166, 38, 38, 0.9), rgba(201, 168, 76, 0.95));
    transition: width 0.3s ease;
}

@media (max-width: 900px) {
    .flow-header {
        padding: 10px 12px;
    }

    .flow-header__label,
    .flow-header__counter {
        font-size: 11px;
    }

    .flow-body {
        padding: 18px 14px calc(18px + env(safe-area-inset-bottom));
    }

    .flow-card-title {
        font-size: 18px;
        margin-bottom: 16px;
    }

    .flow-chain-dots {
        gap: 8px;
        margin-bottom: 20px;
        overflow-x: auto;
        padding-bottom: 2px;
    }

    .flow-chain-dot {
        min-width: 8px;
        width: 8px;
        height: 8px;
    }

}

@keyframes flow-brush-draw {
    to {
        stroke-dashoffset: 0;
    }
}

/* ═══ Transition ═══ */
.flow-slide-enter-active,
.flow-slide-leave-active {
    transition: transform 0.35s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.25s ease;
}

.flow-slide-enter-from,
.flow-slide-leave-to {
    transform: translateY(100%);
    opacity: 0;
}

/* ═══ Prose overrides ═══ */
.prose-container :deep(.prose) {
    color: theme("colors.ms-bone-dim");
}

.prose-container :deep(.prose h1) {
    color: theme("colors.ms-ivory");
}

.prose-container :deep(.prose h2) {
    color: theme("colors.ms-bone");
    border-bottom-color: theme("colors.ms-copper");
}

.prose-container :deep(.prose a) {
    color: theme("colors.xuepo.DEFAULT");
}

.prose-container :deep(.prose code) {
    background: rgba(166, 38, 38, 0.12);
    color: theme("colors.xuepo.600");
}
</style>
