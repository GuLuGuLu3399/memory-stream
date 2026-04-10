<script setup lang="ts">
/**
 * 🌟 FloatingCompass — 悬浮阅读罗盘
 *
 * 双态交互：
 * - 收起态：右下角 36×36 圆形胶囊，显示阅读进度 + 霓虹进度环
 * - 展开态：向左展开 240px 毛玻璃面板，递归 TOC 树
 *
 * 点击 TOC 项 → scrollIntoView 平滑跳转
 * IntersectionObserver 驱动当前项高亮（由父组件传入 activeSlug）
 */

import { ref, computed } from "vue";
import { Compass } from "lucide-vue-next";
import type { TocItem } from "../composables/useCards";
import TocNode from "./toc/TocNode.vue";

const props = defineProps<{
    tocItems: TocItem[];
    activeSlug: string;
    containerEl: HTMLElement | undefined;
    readProgress: number; // 真实滚动进度（0~100），由父组件通过 @scroll 计算
}>();

const expanded = ref(false);

// ── SVG 进度环参数 ──
const circumference = 2 * Math.PI * 14; // r=14
const strokeDashoffset = computed(() => {
    return circumference - (props.readProgress / 100) * circumference;
});

// ── 点击跳转：手动 scrollTo，避免 scrollIntoView 在 fixed 遮罩层下乱跳 ──
function scrollToHeading(slug: string) {
    const container = props.containerEl;
    if (!container) return;

    function tryScroll() {
        const c = container!;
        if (!c) return;
        // 1. 精确匹配 id
        let el = c.querySelector(`[id="${slug}"]`) as HTMLElement | null;
        // 2. Fallback：slug 中可能含特殊字符，尝试转义
        if (!el) {
            const escaped = CSS.escape(slug);
            el = c.querySelector(`[id="${escaped}"]`) as HTMLElement | null;
        }
        // 3. Fallback：按 heading 文本内容模糊匹配
        if (!el) {
            const headings = c.querySelectorAll("h1, h2, h3, h4");
            for (const h of headings) {
                const hSlug = h.id || h.textContent?.trim().toLowerCase().replace(/\s+/g, "-") || "";
                if (hSlug === slug || h.textContent?.trim() === slug) {
                    el = h as HTMLElement;
                    break;
                }
            }
        }
        if (!el) return;

        // 🎯 关键修复：手动计算目标相对于滚动容器的偏移量
        // 避免使用 scrollIntoView（会意外触发 body 滚动）
        let targetOffset = 0;
        let current: HTMLElement | null = el;
        while (current && current !== c) {
            targetOffset += current.offsetTop;
            current = current.offsetParent as HTMLElement | null;
        }
        const paddingTop = 48; // 预留顶部间距，避免标题贴边
        c.scrollTo({
            top: targetOffset - paddingTop,
            behavior: "smooth",
        });
    }

    // 延迟一帧确保 DOM 就绪
    requestAnimationFrame(tryScroll);
}
</script>

<template>
    <div v-if="tocItems.length > 0" class="relative" @mouseenter="expanded = true" @mouseleave="expanded = false">
        <!-- ── 收起态：进度胶囊 ── -->
        <button
            class="relative w-9 h-9 flex items-center justify-center rounded-full bg-ms-carbon/80 backdrop-blur-xl border border-ms-border shadow-lg shadow-black/30 transition-all duration-300 hover:border-neon/40 hover:shadow-neon/10"
            :class="{ 'opacity-0 scale-75 pointer-events-none': expanded }" @click="expanded = true">
            <!-- 进度环 -->
            <svg class="absolute inset-0 w-full h-full -rotate-90" viewBox="0 0 36 36">
                <circle cx="18" cy="18" r="14" fill="none" stroke="currentColor" stroke-width="2"
                    class="text-gray-800" />
                <circle cx="18" cy="18" r="14" fill="none" stroke="currentColor" stroke-width="2"
                    :stroke-dasharray="circumference" :stroke-dashoffset="strokeDashoffset" stroke-linecap="round"
                    class="text-neon transition-all duration-500" />
            </svg>
            <Compass :size="14" class="text-gray-400" />
        </button>

        <!-- ── 展开态：玻璃 TOC 面板 ── -->
        <Transition name="compass-expand">
            <div v-if="expanded"
                class="absolute bottom-0 right-12 w-60 max-h-[60vh] bg-ms-carbon/85 backdrop-blur-xl border border-ms-border rounded-sm shadow-2xl shadow-black/40 overflow-hidden flex flex-col">
                <!-- 面板头 -->
                <div class="px-4 pt-3 pb-2 border-b border-ms-border flex-shrink-0">
                    <div class="flex items-center justify-between">
                        <span class="text-2xs text-gray-600 font-mono uppercase tracking-widest">目录</span>
                        <span class="text-2xs text-neon font-mono">{{ readProgress }}%</span>
                    </div>
                </div>

                <!-- TOC 树 -->
                <nav class="flex-1 overflow-y-auto scrollbar-thin py-2 px-2">
                    <ul class="space-y-0.5">
                        <TocNode v-for="item in tocItems" :key="item.slug" :item="item" :depth="0"
                            :active-slug="activeSlug" @jump="scrollToHeading" />
                    </ul>
                </nav>
            </div>
        </Transition>
    </div>
</template>

<style scoped>
/* ── 面板展开/收起动画 ── */
.compass-expand-enter-active,
.compass-expand-leave-active {
    transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1);
}

.compass-expand-enter-from,
.compass-expand-leave-to {
    opacity: 0;
    transform: translateX(12px) scale(0.95);
}

</style>
