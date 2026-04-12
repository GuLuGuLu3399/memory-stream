<script setup lang="ts">
/**
 * 🌟 FloatingCommandBar — 浮动指挥岛（血肉神殿）
 */

import { storeToRefs } from "pinia";
import { useRouter } from "vue-router";
import {
    List,
    Network,
    Flame,
    Clock,
    LayoutGrid,
    AlignJustify,
    Minimize2,
    SlidersHorizontal,
} from "lucide-vue-next";
import { useGraphStore } from "../store/useGraphStore";

const store = useGraphStore();
const router = useRouter();
const {
    viewMode,
    sortBy,
    density,
} = storeToRefs(store);

// ── 视图切换按钮样式 — 新粗野主义 + 底部指示 ──
const viewBtnClass = (active: boolean) =>
    `relative flex items-center gap-1.5 px-4 py-2 text-xs font-medium transition-all duration-150 ${active
        ? "bg-xuepo/15 text-xuepo font-bold border border-xuepo/40 shadow-[2px_2px_0_0_rgba(0,0,0,0.6)]"
        : "text-ms-smoke border border-transparent hover:text-ms-bone hover:border-ms-copper hover:shadow-[1px_1px_0_0_rgba(0,0,0,0.4)] active:translate-x-[1px] active:translate-y-[1px] active:shadow-none"
    }`;

// ── 控制面板小按钮样式 — 新粗野主义 + 激活态增强 ──
const ctrlBtnClass = (active: boolean) =>
    `flex items-center gap-1 px-2.5 py-1.5 text-1.5xs font-mono transition-all duration-150 ${active
        ? "bg-xuepo/12 text-xuepo border border-xuepo/35 shadow-[2px_2px_0_0_rgba(0,0,0,0.5)]"
        : "text-ms-smoke border border-ms-copper/40 shadow-[1px_1px_0_0_rgba(0,0,0,0.4)] hover:text-ms-bone hover:border-ms-copper hover:shadow-[2px_2px_0_0_rgba(0,0,0,0.5)] active:translate-x-[1px] active:translate-y-[1px] active:shadow-none"
    }`;
</script>

<template>
    <div class="fixed top-4 left-1/2 -translate-x-1/2 z-30 flex flex-col items-center gap-2">
        <!-- ── 主胶囊 — 新粗野主义面板 ── -->
        <div
            class="flex items-center bg-ms-xuan/95 border border-ms-copper px-2 py-1.5 shadow-[3px_3px_0_0_rgba(0,0,0,0.6)] gap-1 hover:shadow-[4px_4px_0_0_rgba(0,0,0,0.5)] transition-shadow duration-150">
            <!-- Logo — metal stamp -->
            <div
                class="w-7 h-7 bg-xuepo/15 border border-xuepo/30 flex items-center justify-center text-xuepo font-bold text-2xs mr-2 shrink-0 font-serif shadow-[1px_1px_0_0_rgba(0,0,0,0.4)]">
                M
            </div>

            <div class="w-px h-5 bg-ms-copper/60" />

            <!-- 视图切换 -->
            <button @click="router.push('/list')" :class="viewBtnClass(viewMode === 'list')">
                <List :size="13" /> 列表
                <div v-if="viewMode === 'list'" class="absolute -bottom-1.5 left-1/2 -translate-x-1/2 w-3 h-[3px] rounded-full bg-xuepo shadow-[0_0_6px_rgba(166,38,38,0.5)]" />
            </button>
            <button @click="router.push('/graph')" :class="viewBtnClass(viewMode === 'graph')">
                <Network :size="13" /> 图谱
                <div v-if="viewMode === 'graph'" class="absolute -bottom-1.5 left-1/2 -translate-x-1/2 w-3 h-[3px] rounded-full bg-xuepo shadow-[0_0_6px_rgba(166,38,38,0.5)]" />
            </button>

            <div class="w-px h-5 bg-ms-copper/60" />

            <button
                class="flex items-center gap-1 px-2.5 py-1.5 text-1.5xs text-ms-smoke border border-transparent hover:text-ms-bone hover:border-ms-copper/50 hover:shadow-[1px_1px_0_0_rgba(0,0,0,0.3)] active:translate-x-[1px] active:translate-y-[1px] active:shadow-none transition-all duration-150"
                title="展示控制">
                <SlidersHorizontal :size="12" />
            </button>
        </div>

        <!-- ── 上下文感知控制面板 ── -->
        <Transition name="slideDown">
            <div
                class="flex items-center bg-ms-xuan/90 border border-ms-copper px-3 py-2 shadow-[2px_2px_0_0_rgba(0,0,0,0.5)] gap-2 flex-wrap justify-center">
                <!-- 列表视图控制 -->
                <template v-if="viewMode === 'list'">
                    <div class="flex items-center gap-1">
                        <span class="text-2xs text-ms-ash font-mono mr-1">排序</span>
                        <button @click="store.setSortBy('updated')" :class="ctrlBtnClass(sortBy === 'updated')">
                            <Clock :size="10" /> 时间
                        </button>
                        <button @click="store.setSortBy('hot')" :class="ctrlBtnClass(sortBy === 'hot')">
                            <Flame :size="10" /> 热度
                        </button>
                    </div>

                    <div class="w-px h-4 bg-ms-copper/40" />

                    <div class="flex items-center gap-1">
                        <span class="text-2xs text-ms-ash font-mono mr-1">密度</span>
                        <button @click="store.setDensity('cozy')" :class="ctrlBtnClass(density === 'cozy')">
                            <LayoutGrid :size="10" /> 舒适
                        </button>
                        <button @click="store.setDensity('compact')" :class="ctrlBtnClass(density === 'compact')">
                            <AlignJustify :size="10" /> 紧凑
                        </button>
                    </div>
                </template>

                <!-- 图谱视图控制 -->
                <template v-else>
                    <button @click="$emit('fitView')"
                        class="flex items-center gap-1 px-2.5 py-1.5 text-1.5xs font-mono text-ms-smoke border border-ms-copper/40 shadow-[1px_1px_0_0_rgba(0,0,0,0.4)] hover:text-ms-bone hover:border-ms-copper hover:shadow-[2px_2px_0_0_rgba(0,0,0,0.5)] active:translate-x-[1px] active:translate-y-[1px] active:shadow-none transition-all duration-150">
                        <Minimize2 :size="10" /> 归位
                    </button>
                </template>
            </div>
        </Transition>
    </div>
</template>

<script lang="ts">
export default {
    emits: ["fitView"],
};
</script>

<style scoped>
.slideDown-enter-active,
.slideDown-leave-active {
    transition: all 200ms cubic-bezier(0.25, 0.1, 0.25, 1);
}

.slideDown-enter-from,
.slideDown-leave-to {
    opacity: 0;
    transform: translateY(-8px);
}
</style>
