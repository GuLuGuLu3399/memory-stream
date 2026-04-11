<script setup lang="ts">
/**
 * FloatingCompass — 罗盘导航（血肉神殿）
 *
 * 双态交互：
 * - 收起态：右下角圆形胶囊，双环进度 + 罗盘指针
 * - 展开态：向左展开 240px 面板，递归 TOC 树
 */

import { ref, computed } from "vue";
import { Compass } from "lucide-vue-next";
import type { TocItem } from "../composables/useCards";
import TocNode from "./toc/TocNode.vue";

const props = defineProps<{
    tocItems: TocItem[];
    activeSlug: string;
    containerEl: HTMLElement | undefined;
    readProgress: number;
}>();

const expanded = ref(false);

// 内环：阅读进度 (r=12)
const innerCircumference = 2 * Math.PI * 12;
const innerStrokeDashoffset = computed(() => {
    return innerCircumference - (props.readProgress / 100) * innerCircumference;
});

// 外环：章节进度 (r=16)
const chapterProgress = computed(() => {
    if (props.tocItems.length === 0) return 0;
    const activeIndex = props.tocItems.findIndex(item => item.slug === props.activeSlug);
    return activeIndex >= 0 ? ((activeIndex + 1) / props.tocItems.length) * 100 : 0;
});

const outerCircumference = 2 * Math.PI * 16;
const outerStrokeDashoffset = computed(() => {
    return outerCircumference - (chapterProgress.value / 100) * outerCircumference;
});

// 罗盘指针旋转
const needleRotation = computed(() => {
    return props.readProgress * 3.6;
});

function scrollToHeading(slug: string) {
    const container = props.containerEl;
    if (!container) return;

    function tryScroll() {
        const c = container!;
        if (!c) return;
        let el = c.querySelector(`[id="${slug}"]`) as HTMLElement | null;
        if (!el) {
            const escaped = CSS.escape(slug);
            el = c.querySelector(`[id="${escaped}"]`) as HTMLElement | null;
        }
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

        let targetOffset = 0;
        let current: HTMLElement | null = el;
        while (current && current !== c) {
            targetOffset += current.offsetTop;
            current = current.offsetParent as HTMLElement | null;
        }
        const paddingTop = 48;
        c.scrollTo({
            top: targetOffset - paddingTop,
            behavior: "smooth",
        });
    }

    requestAnimationFrame(tryScroll);
}

function onCompassClick() {
    if (expanded.value) {
        // 已展开时，点击滚动到顶部
        const container = props.containerEl;
        if (container) {
            container.scrollTo({ top: 0, behavior: "smooth" });
        }
    } else {
        expanded.value = true;
    }
}
</script>

<template>
    <div v-if="tocItems.length > 0" class="relative" @mouseenter="expanded = true" @mouseleave="expanded = false">
        <!-- ── 收起态：双环进度胶囊 ── -->
        <button
            class="relative w-9 h-9 flex items-center justify-center rounded-full bg-ms-xiang border border-ms-copper shadow-raised transition-all duration-300 hover:border-xuepo/60 hover:shadow-altar-glow"
            :class="{ 'opacity-0 scale-75 pointer-events-none': expanded }"
            @click="onCompassClick">
            <!-- 外环：章节进度 (xuepo/60) -->
            <svg class="absolute inset-0 w-full h-full -rotate-90" viewBox="0 0 36 36">
                <!-- 外环底色 -->
                <circle cx="18" cy="18" r="16" fill="none" stroke="currentColor" stroke-width="1.5"
                    class="text-ms-copper/30" />
                <!-- 外环进度 -->
                <circle cx="18" cy="18" r="16" fill="none" stroke="currentColor" stroke-width="1.5"
                    :stroke-dasharray="outerCircumference" :stroke-dashoffset="outerStrokeDashoffset" stroke-linecap="round"
                    class="text-xuepo/60 transition-all duration-500" />

                <!-- 内环：阅读进度 (ms-gold) -->
                <circle cx="18" cy="18" r="12" fill="none" stroke="currentColor" stroke-width="2"
                    class="text-ms-copper" />
                <circle cx="18" cy="18" r="12" fill="none" stroke="currentColor" stroke-width="2"
                    :stroke-dasharray="innerCircumference" :stroke-dashoffset="innerStrokeDashoffset" stroke-linecap="round"
                    class="text-ms-gold transition-all duration-500" />
            </svg>

            <!-- 罗盘指针 -->
            <svg class="absolute inset-0 w-full h-full" viewBox="0 0 36 36">
                <g :transform="`rotate(${needleRotation}, 18, 18)`" class="transition-transform duration-300 ease-out">
                    <polygon points="18,6 15,18 18,15 21,18" fill="currentColor" class="text-ms-gold/80" />
                </g>
            </svg>

            <Compass :size="14" class="text-ms-smoke opacity-20" />
        </button>

        <!-- ── 展开态：TOC 面板 ── -->
        <Transition name="compass-expand">
            <div v-if="expanded"
                class="absolute bottom-0 right-12 w-60 max-h-[60vh] bg-ms-xiang border border-ms-copper rounded-altar shadow-raised-lg overflow-hidden flex flex-col">
                <!-- 面板头 -->
                <div class="px-4 pt-3 pb-2 border-b border-ms-copper flex-shrink-0">
                    <div class="flex items-center justify-between">
                        <span class="text-2xs text-ms-ash font-mono uppercase tracking-widest">目录</span>
                        <span class="text-2xs text-ms-gold font-mono">{{ Math.round(readProgress) }}%</span>
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
.compass-expand-enter-active,
.compass-expand-leave-active {
    transition: all 250ms cubic-bezier(0.25, 0.1, 0.25, 1);
}

.compass-expand-enter-from,
.compass-expand-leave-to {
    opacity: 0;
    transform: translateX(12px) scale(0.95);
}
</style>
