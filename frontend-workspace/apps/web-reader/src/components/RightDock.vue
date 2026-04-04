<script setup lang="ts">
/**
 * 🌟 RightDock — 右侧控制坞
 *
 * 贴紧屏幕右侧垂直居中。默认收起仅 48px（只显示 🎛️ 图标），
 * hover 展开至 220px 显示排序/密度/深度/聚光灯等控制。
 * 上下文感知：根据 viewMode 动态切换控制项。
 */

import { storeToRefs } from "pinia";
import {
    SlidersHorizontal,
    Flame,
    Clock,
    LayoutGrid,
    AlignJustify,
    Spotlight,
    Minimize2,
} from "lucide-vue-next";
import { useGraphStore } from "../store/useGraphStore";

const store = useGraphStore();
const { viewMode, sortBy, density, graphDepth, spotlightMode } = storeToRefs(store);

const emit = defineEmits<{
    fitView: [];
}>();

const ctrlBtnClass = (active: boolean) =>
    `flex items-center gap-1.5 px-2.5 py-1.5 text-[11px] font-mono rounded-md transition-all duration-150 whitespace-nowrap ${active
        ? "bg-neon/10 text-neon border border-neon/30"
        : "text-gray-500 hover:text-gray-300 border border-transparent hover:border-ms-border"
    }`;
</script>

<template>
    <div
        class="right-dock fixed right-4 top-1/2 -translate-y-1/2 z-30 flex flex-col bg-ms-panel/80 backdrop-blur-xl border border-ms-border rounded-sm shadow-lg shadow-black/30 overflow-hidden transition-all duration-300 ease-[cubic-bezier(0.16,1,0.3,1)]">
        <!-- 收起态：仅图标 -->
        <div class="dock-collapsed p-2 flex flex-col items-center gap-2">
            <button
                class="w-10 h-10 rounded-sm flex items-center justify-center text-gray-500 hover:text-neon hover:bg-neon/10 transition-all"
                title="控制面板">
                <SlidersHorizontal :size="16" />
            </button>
        </div>

        <!-- 展开态：完整控制面板 -->
        <div class="dock-expanded p-3 flex flex-col gap-3">
            <!-- ── 列表视图控制 ── -->
            <template v-if="viewMode === 'list'">
                <div class="flex flex-col gap-1.5">
                    <span class="text-[10px] text-gray-600 font-mono">排序</span>
                    <button @click="store.setSortBy('updated')" :class="ctrlBtnClass(sortBy === 'updated')">
                        <Clock :size="12" /> 时间
                    </button>
                    <button @click="store.setSortBy('hot')" :class="ctrlBtnClass(sortBy === 'hot')">
                        <Flame :size="12" /> 热度
                    </button>
                </div>
                <div class="h-px bg-ms-border/40" />
                <div class="flex flex-col gap-1.5">
                    <span class="text-[10px] text-gray-600 font-mono">密度</span>
                    <button @click="store.setDensity('cozy')" :class="ctrlBtnClass(density === 'cozy')">
                        <LayoutGrid :size="12" /> 舒适
                    </button>
                    <button @click="store.setDensity('compact')" :class="ctrlBtnClass(density === 'compact')">
                        <AlignJustify :size="12" /> 紧凑
                    </button>
                </div>
            </template>

            <!-- ── 图谱视图控制 ── -->
            <template v-else>
                <div class="flex flex-col gap-1.5">
                    <span class="text-[10px] text-gray-600 font-mono">深度</span>
                    <div class="flex items-center gap-1">
                        <button v-for="d in 3" :key="d" @click="store.setGraphDepth(d)" :class="`w-8 h-8 rounded-md text-[11px] font-mono font-bold transition-all ${graphDepth >= d
                            ? 'bg-neon/15 text-neon border border-neon/30'
                            : 'text-gray-600 border border-ms-border/50 hover:border-ms-border'
                            }`">
                            {{ d }}
                        </button>
                    </div>
                </div>
                <div class="h-px bg-ms-border/40" />
                <button @click="store.toggleSpotlight()" :class="ctrlBtnClass(spotlightMode)">
                    <Spotlight :size="12" /> 聚光灯
                </button>
                <button @click="emit('fitView')"
                    class="flex items-center gap-1.5 px-2.5 py-1.5 text-[11px] font-mono rounded-md text-gray-500 hover:text-gray-300 border border-transparent hover:border-ms-border transition-all whitespace-nowrap">
                    <Minimize2 :size="12" /> 归位
                </button>
            </template>
        </div>
    </div>
</template>

<style scoped>
.right-dock {
    width: 56px;
}

.right-dock:hover {
    width: 200px;
}

/* 收起态始终显示 */
.dock-collapsed {
    display: flex;
}

/* 展开态默认隐藏，hover 时显示 */
.dock-expanded {
    display: none;
}

.right-dock:hover .dock-collapsed {
    display: none;
}

.right-dock:hover .dock-expanded {
    display: flex;
}
</style>
