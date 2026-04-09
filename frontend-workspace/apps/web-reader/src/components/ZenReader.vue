<script setup lang="ts">
/**
 * 🌟 ZenReader — 全屏禅模式阅读器
 *
 * z-fullscreen 覆盖一切，沉浸式阅读体验。
 * 100% 宽度 Markdown 渲染（max-width: 72ch）
 * 右下角 FloatingCompass 悬浮目录导航
 * Escape 退出
 */

import { ref, watch, nextTick, onUnmounted } from "vue";

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

// ── 真实滚动进度追踪 ──
const readProgress = ref(0);

const calculateProgress = () => {
    if (!proseRef.value) return;
    const { scrollTop, scrollHeight, clientHeight } = proseRef.value;
    const maxScroll = scrollHeight - clientHeight;
    readProgress.value = maxScroll <= 0 ? 100 : Math.round((scrollTop / maxScroll) * 100);
};

// ── IntersectionObserver 驱动标题高亮 ──
const { activeSlug, delayedRefresh } = useActiveHeading(proseRef);

// ── 加载卡片 + TOC（直接使用后端持久化的 toc_data） ──
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

        // 直接使用保存时预计算的 TOC（无需 WASM 运行时重算）
        tocItems.value = result.tocData ?? [];

        // DOM 更新后延迟刷新 observer（等待 MarkdownViewer 异步渲染完成）
        await nextTick();
        delayedRefresh();
    } catch (err) {
        console.error("[ZenReader] load failed:", err);
        detail.value = null;
    }
}, { immediate: true });

// ── 锁定外部滚动 ──
watch(zenMode, (zen) => {
    document.body.style.overflow = zen ? "hidden" : "";
});

onUnmounted(() => {
    document.body.style.overflow = "";
});
</script>

<template>
    <Teleport to="body">
        <Transition name="ms-scale">
            <div v-if="zenMode && detail" class="fixed inset-0 z-fullscreen bg-ms-deep flex zen-container">
                <!-- 霓虹顶线装饰 -->
                <div class="zen-neon-topline"></div>

                <!-- 全屏 Markdown 阅读 -->
                <div class="flex-1 overflow-y-auto scrollbar-thin" ref="proseRef" @scroll="calculateProgress">
                    <div class="max-w-[72ch] mx-auto px-8 py-16">
                        <!-- 标题 -->
                        <h1 class="text-3xl font-bold text-white mb-8 leading-tight">
                            {{ detail.title }}
                        </h1>
                        <!-- 正文 -->
                        <MarkdownViewer :html-content="detail.html" />
                    </div>
                </div>

                <!-- 悬浮阅读罗盘 -->
                <div v-if="tocItems.length > 0" class="fixed bottom-6 right-6 z-[1]">
                    <FloatingCompass :toc-items="tocItems" :active-slug="activeSlug" :container-el="proseRef"
                        :read-progress="readProgress" />
                </div>

                <!-- 退出提示 -->
                <div class="fixed top-4 right-4 z-[1]">
                    <button
                        class="px-3 py-1.5 text-[11px] font-mono text-gray-600 hover:text-gray-300 bg-ms-panel/80 backdrop-blur border border-ms-border rounded-lg transition-all"
                        @click="store.toggleZenMode()">
                        ESC 退出
                    </button>
                </div>
            </div>
        </Transition>
    </Teleport>
</template>

<style scoped>
/* ── 霓虹顶线装饰：入殿感标志 ── */
.zen-neon-topline {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(90deg,
            transparent 0%,
            #00e5ff 15%,
            #00e5ff 50%,
            #00e5ff 85%,
            transparent 100%);
    box-shadow:
        0 0 8px rgba(0, 229, 255, 0.6),
        0 0 24px rgba(0, 229, 255, 0.3),
        0 0 48px rgba(0, 229, 255, 0.15);
    z-index: 2;
    animation: zen-neon-pulse 3s ease-in-out infinite;
}

@keyframes zen-neon-pulse {

    0%,
    100% {
        opacity: 0.7;
    }

    50% {
        opacity: 1;
    }
}

.scrollbar-thin::-webkit-scrollbar {
    width: 4px;
}

.scrollbar-thin::-webkit-scrollbar-track {
    background: transparent;
}

.scrollbar-thin::-webkit-scrollbar-thumb {
    background: #333;
    border-radius: 2px;
}
</style>
