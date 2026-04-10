<script setup lang="ts">
/**
 * 🌟 FloatingCommandBar — 浮动指挥岛
 *
 * 顶部中央胶囊组件，包含：
 * 1. Segmented View Switcher：列表 / 图谱 切换
 * 2. 上下文感知控制面板：根据 viewMode 动态切换
 *    - 列表模式：排序 / 密度 / 分类过滤
 *    - 图谱模式：深度滑块 / 聚光灯 / 布局重置
 */

import { storeToRefs } from "pinia";
import {
    List,
    Network,
    Flame,
    Clock,
    LayoutGrid,
    AlignJustify,
    Spotlight,
    Minimize2,
    SlidersHorizontal,
} from "lucide-vue-next";
import { useGraphStore } from "../store/useGraphStore";

const store = useGraphStore();
const {
    viewMode,
    sortBy,
    density,
    graphDepth,
    spotlightMode,
} = storeToRefs(store);

// ── 视图切换按钮样式 ──
const viewBtnClass = (active: boolean) =>
    `flex items-center gap-1.5 px-4 py-2 text-xs font-medium rounded-lg transition-all duration-200 ${active
        ? "bg-neon/15 text-neon font-bold shadow-[0_0_12px_rgba(0,229,255,0.15)]"
        : "text-gray-500 hover:text-gray-300"
    }`;

// ── 控制面板小按钮样式 ──
const ctrlBtnClass = (active: boolean) =>
    `flex items-center gap-1 px-2.5 py-1.5 text-1.5xs font-mono rounded-md transition-all duration-150 ${active
        ? "bg-neon/10 text-neon border border-neon/30"
        : "text-gray-500 hover:text-gray-300 border border-transparent hover:border-ms-border"
    }`;
</script>

<template>
    <div class="fixed top-4 left-1/2 -translate-x-1/2 z-30 flex flex-col items-center gap-2">
        <!-- ── 主胶囊：Logo + 视图切换 ── -->
        <div
            class="flex items-center bg-ms-panel/80 backdrop-blur-xl border border-ms-border rounded-sm px-2 py-1.5 shadow-lg shadow-black/30 gap-1">
            <!-- Logo -->
            <div
                class="w-7 h-7 bg-neon/10 rounded-sm flex items-center justify-center text-neon font-bold text-2xs mr-2 shrink-0">
                M
            </div>

            <!-- 分隔线 -->
            <div class="w-px h-5 bg-ms-border/60" />

            <!-- 视图切换 -->
            <button @click="store.setViewMode('list')" :class="viewBtnClass(viewMode === 'list')">
                <List :size="13" /> 列表
            </button>
            <button @click="store.setViewMode('graph')" :class="viewBtnClass(viewMode === 'graph')">
                <Network :size="13" /> 图谱
            </button>

            <!-- 分隔线 -->
            <div class="w-px h-5 bg-ms-border/60" />

            <!-- 上下文感知控制面板入口 -->
            <button
                class="flex items-center gap-1 px-2.5 py-1.5 text-1.5xs text-gray-400 hover:text-gray-200 rounded-lg transition-all"
                @click="" title="展示控制">
                <SlidersHorizontal :size="12" />
            </button>
        </div>

        <!-- ── 上下文感知控制面板 ── -->
        <Transition name="slideDown">
            <div
                class="flex items-center bg-ms-panel/70 backdrop-blur-xl border border-ms-border rounded-sm px-3 py-2 shadow-lg shadow-black/20 gap-2 flex-wrap justify-center">
                <!-- ===== 列表视图控制 ===== -->
                <template v-if="viewMode === 'list'">
                    <!-- 排序拨片 -->
                    <div class="flex items-center gap-1">
                        <span class="text-2xs text-gray-600 font-mono mr-1">排序</span>
                        <button @click="store.setSortBy('updated')" :class="ctrlBtnClass(sortBy === 'updated')">
                            <Clock :size="10" /> 时间
                        </button>
                        <button @click="store.setSortBy('hot')" :class="ctrlBtnClass(sortBy === 'hot')">
                            <Flame :size="10" /> 热度
                        </button>
                    </div>

                    <div class="w-px h-4 bg-ms-border/40" />

                    <!-- 密度切换 -->
                    <div class="flex items-center gap-1">
                        <span class="text-2xs text-gray-600 font-mono mr-1">密度</span>
                        <button @click="store.setDensity('cozy')" :class="ctrlBtnClass(density === 'cozy')">
                            <LayoutGrid :size="10" /> 舒适
                        </button>
                        <button @click="store.setDensity('compact')" :class="ctrlBtnClass(density === 'compact')">
                            <AlignJustify :size="10" /> 紧凑
                        </button>
                    </div>
                </template>

                <!-- ===== 图谱视图控制 ===== -->
                <template v-else>
                    <!-- 层级深度滑块 -->
                    <div class="flex items-center gap-2">
                        <span class="text-2xs text-gray-600 font-mono">深度</span>
                        <div class="flex items-center gap-1">
                            <button v-for="d in 3" :key="d" @click="store.setGraphDepth(d)" :class="`w-6 h-6 rounded-md text-1.5xs font-mono font-bold transition-all ${graphDepth >= d
                                ? 'bg-neon/15 text-neon border border-neon/30'
                                : 'text-gray-600 border border-ms-border/50 hover:border-ms-border'
                                }`">
                                {{ d }}
                            </button>
                        </div>
                    </div>

                    <div class="w-px h-4 bg-ms-border/40" />

                    <!-- 聚光灯模式 -->
                    <button @click="store.toggleSpotlight()" :class="ctrlBtnClass(spotlightMode)">
                        <Spotlight :size="10" /> 聚光灯
                    </button>

                    <div class="w-px h-4 bg-ms-border/40" />

                    <!-- 布局重置（emit 给 GraphView） -->
                    <button @click="$emit('fitView')"
                        class="flex items-center gap-1 px-2.5 py-1.5 text-1.5xs font-mono rounded-md text-gray-500 hover:text-gray-300 border border-transparent hover:border-ms-border transition-all">
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
/* 控制面板滑入 */
.slideDown-enter-active,
.slideDown-leave-active {
    transition: all 200ms cubic-bezier(0.16, 1, 0.3, 1);
}

.slideDown-enter-from,
.slideDown-leave-to {
    opacity: 0;
    transform: translateY(-8px);
}
</style>
