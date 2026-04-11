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
import { ChevronLeft, ChevronRight, BookOpen } from "lucide-vue-next";
import MarkdownViewer from "@memory-stream/ui-shared/components/MarkdownViewer.vue";
import SkeletonBlock from "@memory-stream/ui-shared/components/SkeletonBlock.vue";
import { useCards } from "../composables/useCards";
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
</script>

<template>
    <Transition name="flow-slide">
        <div
            v-if="open"
            class="flow-reader"
            tabindex="0"
            @keydown="handleKeydown"
        >
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
                    <button
                        class="flow-nav-btn"
                        :disabled="!hasPrev"
                        @click="goPrev"
                    >
                        <ChevronLeft :size="16" />
                    </button>
                    <button
                        class="flow-nav-btn"
                        :disabled="!hasNext"
                        @click="goNext"
                    >
                        <ChevronRight :size="16" />
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
                        <button
                            v-for="(_, idx) in chainIds"
                            :key="idx"
                            class="flow-chain-dot"
                            :class="{ 'flow-chain-dot--active': idx === currentIndex }"
                            @click="goTo(idx)"
                        />
                    </div>

                    <!-- Markdown 渲染 -->
                    <MarkdownViewer :html-content="currentNode.html" />
                </div>

                <!-- 空态 -->
                <div v-else class="flow-empty">
                    <span class="text-ms-smoke text-sm">无法加载内容</span>
                </div>
            </div>

            <!-- 底栏：进度条 -->
            <div v-if="totalCount > 1" class="flow-progress">
                <div
                    class="flow-progress__bar"
                    :style="{ width: `${((currentIndex + 1) / totalCount) * 100}%` }"
                />
            </div>
        </div>
    </Transition>
</template>

<style scoped>
/* ═══ 全屏覆盖层 ═══ */
.flow-reader {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: flex;
    flex-direction: column;
    background: theme("colors.ms-xuan");
    outline: none;
}

/* ═══ 顶栏 ═══ */
.flow-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    background: theme("colors.ms-mo");
    border-bottom: 1px solid theme("colors.ms-copper");
    flex-shrink: 0;
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

/* ═══ 金缮分割线 ═══ */
.flow-gold-rule {
    height: 1px;
    flex-shrink: 0;
    background: linear-gradient(
        90deg,
        transparent 0%,
        theme("colors.ms-gold") 20%,
        theme("colors.ms-gold-dim") 50%,
        theme("colors.ms-gold") 80%,
        transparent 100%
    );
}

/* ═══ 内容区 ═══ */
.flow-body {
    flex: 1;
    overflow-y: auto;
    padding: 32px 40px;
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
}

.flow-progress__bar {
    height: 100%;
    background: linear-gradient(90deg, theme("colors.xuepo.DEFAULT"), theme("colors.ms-gold"));
    transition: width 0.3s ease;
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
