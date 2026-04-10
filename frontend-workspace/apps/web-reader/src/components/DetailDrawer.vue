<script setup lang="ts">
/**
 * 🌟 DetailDrawer — 沉浸式阅读器 (The Infinite Drawer)
 *
 * 右侧滑出，占 45% 宽度。
 * 毛玻璃背景 + WASM Markdown 渲染。
 * 监听 selectedId 变化，按需加载卡片详情。
 *
 * 色板：ms-panel/ms-border/neon 统一极客工业风
 * Z-Index：z-drawer（遮罩与抽屉主体）
 */

import { ref, watch } from "vue";
import { storeToRefs } from "pinia";
import MarkdownViewer from "@memory-stream/ui-shared/components/MarkdownViewer.vue";
import { useGraphStore } from "../store/useGraphStore";
import { useCards } from "../composables/useCards";
import type { CardDetail } from "../composables/useCards";
import SkeletonLine from "./ui/SkeletonLine.vue";
import { useBreakpoints } from "../composables/useBreakpoints";
import { useSwipeClose } from "../composables/useSwipeClose";
import { Maximize2, Minimize2, ArrowLeft, Link2 } from "lucide-vue-next";
import { api } from "../api";

// ── Backlink 类型 ──
interface BacklinkItem {
    source_id: string;
    source_title: string;
    relation_type: string;
}

const store = useGraphStore();
const { selectedId } = storeToRefs(store);
const { loadDetail } = useCards();
const { isMobile } = useBreakpoints();

const detail = ref<CardDetail | null>(null);
const loading = ref(false);

// ── 反向引用（Backlinks） ──
const backlinks = ref<BacklinkItem[]>([]);
const backlinksLoading = ref(false);
const backlinksOpen = ref(false);

// ── 移动端右滑关闭 ──
const { offsetX: swipeOffset } = useSwipeClose({
    onClose: () => store.selectNode(null),
});

// 监听 selectedId → 加载详情 + 反向引用
watch(selectedId, async (newId) => {
    if (!newId) {
        detail.value = null;
        backlinks.value = [];
        return;
    }

    loading.value = true;
    detail.value = null;
    backlinks.value = [];

    try {
        const result = await loadDetail(newId);
        detail.value = result;

        // 并行加载反向引用
        backlinksLoading.value = true;
        try {
            const res = await api.getBacklinks(newId);
            backlinks.value = res.backlinks || [];
        } catch {
            backlinks.value = [];
        } finally {
            backlinksLoading.value = false;
        }
    } finally {
        loading.value = false;
    }
});

function close() {
    store.selectNode(null);
}

// Escape 键关闭
function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") close();
}

// ── 双向悬停：wikilink hover → 图谱节点高亮 ──
function onProseMouseOver(e: MouseEvent) {
    const target = (e.target as HTMLElement).closest("a.wikilink, a[data-card-id]");
    if (target) {
        const cardId = (target as HTMLAnchorElement).dataset.cardId || (target as HTMLAnchorElement).getAttribute("href")?.replace("/card/", "");
        if (cardId) store.highlightNode(cardId);
    }
}

function onProseMouseOut(e: MouseEvent) {
    const target = (e.target as HTMLElement).closest("a.wikilink, a[data-card-id]");
    if (target) store.highlightNode(null);
}
</script>

<template>
    <Teleport to="body">
        <!-- 遮罩层 -->
        <Transition name="ms-fade">
            <div v-if="selectedId" class="fixed inset-0 bg-black/30 z-drawer backdrop-blur-sm" @click="close"
                @keydown.escape="close" />
        </Transition>

        <!-- 抽屉主体 -->
        <Transition name="ms-slide-right">
            <div v-if="selectedId"
                class="fixed top-0 right-0 h-full z-drawer flex flex-col backdrop-blur-md bg-ms-panel/95 border-l border-ms-border shadow-2xl shadow-black/40"
                :style="{
                    width: isMobile ? '100%' : '45%',
                    minWidth: isMobile ? '0' : '400px',
                    maxWidth: isMobile ? '100%' : '680px',
                    transform: swipeOffset ? `translateX(${swipeOffset}px)` : undefined,
                    transition: swipeOffset ? 'none' : undefined,
                }" tabindex="0" @keydown="onKeydown">
                <!-- 抽屉头部 -->
                <div class="flex items-center justify-between px-6 py-4 border-b border-ms-border flex-shrink-0">
                    <h2 class="text-sm font-bold text-gray-200 truncate">
                        {{ detail?.title || "加载中..." }}
                    </h2>
                    <!-- 禅模式按钮（唯一保留的操作按钮） -->
                    <button v-if="detail"
                        class="w-7 h-7 flex items-center justify-center rounded-md text-gray-500 hover:text-neon hover:bg-neon/10 transition-all duration-150"
                        :title="store.zenMode ? '退出专注' : '专注模式'" @click="store.toggleZenMode()">
                        <Maximize2 v-if="!store.zenMode" :size="14" />
                        <Minimize2 v-else :size="14" />
                    </button>
                </div>

                <!-- 加载态：骨架屏 -->
                <div v-if="loading" class="flex-1 overflow-y-auto px-6 py-5 space-y-6">
                    <div class="space-y-3">
                        <SkeletonLine width="40%" height="20px" />
                        <SkeletonLine width="100%" height="12px" />
                        <SkeletonLine width="90%" height="12px" />
                    </div>
                    <div class="space-y-3">
                        <SkeletonLine width="60%" height="16px" />
                        <SkeletonLine width="100%" height="12px" />
                        <SkeletonLine width="85%" height="12px" />
                        <SkeletonLine width="75%" height="12px" />
                    </div>
                    <div class="space-y-3">
                        <SkeletonLine width="50%" height="16px" />
                        <SkeletonLine width="100%" height="12px" />
                        <SkeletonLine width="80%" height="12px" />
                    </div>
                    <div class="space-y-2">
                        <SkeletonLine width="100%" height="80px" />
                    </div>
                    <div class="space-y-3">
                        <SkeletonLine width="45%" height="16px" />
                        <SkeletonLine width="100%" height="12px" />
                        <SkeletonLine width="95%" height="12px" />
                        <SkeletonLine width="70%" height="12px" />
                    </div>
                </div>

                <!-- 内容区 + Backlinks -->
                <div v-else-if="detail"
                    class="flex-1 overflow-y-auto px-6 py-5 prose-container scrollbar-thin"
                    @mouseover="onProseMouseOver" @mouseout="onProseMouseOut">
                    <Transition name="crossfade" mode="out-in">
                        <div :key="detail.id">
                            <MarkdownViewer :html-content="detail.html" />

                            <!-- 反向引用区域 -->
                            <div v-if="backlinks.length > 0 || backlinksLoading"
                                class="mt-8 pt-5 border-t border-ms-border/50">
                                <button
                                    class="flex items-center gap-2 text-xs text-gray-400 hover:text-neon transition-colors duration-200 mb-3"
                                    @click="backlinksOpen = !backlinksOpen">
                                    <Link2 :size="12" />
                                    <span class="font-medium">被引用</span>
                                    <span v-if="backlinksLoading"
                                        class="inline-block w-3 h-3 border border-gray-500 border-t-neon rounded-full animate-spin" />
                                    <span v-else class="text-gray-600">({{ backlinks.length }})</span>
                                    <svg class="w-3 h-3 transition-transform duration-200"
                                        :class="{ 'rotate-180': backlinksOpen }" viewBox="0 0 12 12" fill="none">
                                        <path d="M3 4.5L6 7.5L9 4.5" stroke="currentColor" stroke-width="1.5"
                                            stroke-linecap="round" stroke-linejoin="round" />
                                    </svg>
                                </button>
                                <Transition name="slide-down">
                                    <div v-if="backlinksOpen" class="space-y-1.5">
                                        <button v-for="bl in backlinks" :key="bl.source_id"
                                            class="w-full flex items-center gap-2 px-3 py-2 rounded-md text-left text-xs text-gray-400 hover:text-neon hover:bg-neon/5 transition-all duration-200 group"
                                            @click="store.selectNode(bl.source_id)">
                                            <ArrowLeft :size="10"
                                                class="flex-shrink-0 text-gray-600 group-hover:text-neon transition-colors" />
                                            <span class="truncate">{{ bl.source_title }}</span>
                                            <span class="ml-auto flex-shrink-0 text-2xs text-gray-600 font-mono">
                                                {{ bl.relation_type }}
                                            </span>
                                        </button>
                                    </div>
                                </Transition>
                            </div>
                        </div>
                    </Transition>
                </div>

                <!-- 错误/空态 -->
                <div v-else class="flex-1 flex items-center justify-center">
                    <span class="text-gray-600 text-sm">无法加载卡片内容</span>
                </div>

                <!-- 底部信息栏 -->
                <div v-if="detail"
                    class="px-6 py-3 border-t border-ms-border flex items-center justify-between flex-shrink-0">
                    <span class="text-2xs text-gray-600 font-mono">
                        {{ detail.id.slice(0, 8) }}
                    </span>
                    <span class="text-2xs text-gray-600 font-mono">
                        {{ detail.updatedAt }}
                    </span>
                </div>

            </div>
        </Transition>
    </Teleport>
</template>

<style scoped>
/* 淡入淡出遮罩 */
.fade-enter-active,
.fade-leave-active {
    transition: opacity 200ms ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}

/* 右侧滑入滑出 — Expo-Out 缓动 */
.slide-enter-active,
.slide-leave-active {
    transition: transform 300ms cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-enter-from,
.slide-leave-to {
    transform: translateX(100%);
}

/* 内容 Crossfade */
.crossfade-enter-active,
.crossfade-leave-active {
    transition: opacity 200ms ease;
}

.crossfade-enter-from,
.crossfade-leave-to {
    opacity: 0;
}

/* Backlinks 展开/收起 */
.slide-down-enter-active,
.slide-down-leave-active {
    transition: all 200ms ease;
    overflow: hidden;
}

.slide-down-enter-from,
.slide-down-leave-to {
    opacity: 0;
    max-height: 0;
}

.slide-down-enter-to,
.slide-down-leave-from {
    opacity: 1;
    max-height: 500px;
}

/* 阅读宽幅限制 */
.prose-container {
    max-width: 72ch;
    margin: 0 auto;
}
</style>