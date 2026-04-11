<script setup lang="ts">
/**
 * RightDock — 右侧控制坞（血肉神殿）
 *
 * 贴紧屏幕右侧垂直居中。默认收起仅 48px（只显示 🎛️ 图标），
 * hover 展开至 220px 显示排序/密度/深度/聚光灯等控制。
 * 上下文感知：根据 viewMode 动态切换控制项。
 *
 * BUG FIX: 替换所有 admin-tauri 主题标记（neon, bg-ms-panel, text-gray-*）
 *         为 web-reader 血肉神殿标记（ms-xuan, ms-xiang, ms-copper, xuepo, ms-bone, ms-bone-dim, ms-ash）
 */

import { ref } from "vue";
import { storeToRefs } from "pinia";
import {
    SlidersHorizontal,
    Flame,
    Clock,
    LayoutGrid,
    AlignJustify,
    Minimize2,
} from "lucide-vue-next";
import { useGraphStore } from "../store/useGraphStore";

const store = useGraphStore();
const { viewMode, sortBy, density } = storeToRefs(store);

const emit = defineEmits<{
    fitView: [];
}>();

const isExpanded = ref(false);

// ── 血肉神殿控制按钮样式 — 新粗野主义 ──
const ctrlBtnClass = (active: boolean) =>
    `flex items-center gap-2 px-3 py-2 text-1.5xs font-mono rounded-altar transition-all duration-150 whitespace-nowrap ${active
        ? "text-xuepo bg-xuepo/10 border border-xuepo/30 shadow-[2px_2px_0_0_rgba(0,0,0,0.5)]"
        : "text-ms-bone-dim border border-ms-copper/30 shadow-[1px_1px_0_0_rgba(0,0,0,0.3)] hover:text-ms-bone hover:border-ms-copper hover:shadow-[2px_2px_0_0_rgba(0,0,0,0.4)] active:translate-x-[1px] active:translate-y-[1px] active:shadow-none"
    }`;

// 分隔线样式
const dividerClass = "h-px bg-gradient-to-r from-transparent via-ms-copper/40 to-transparent my-3";

// 收起态按钮 — mechanical stamp
const collapsedBtnClass = "w-12 h-12 rounded-altar flex items-center justify-center text-ms-smoke border border-ms-copper/30 shadow-[1px_1px_0_0_rgba(0,0,0,0.3)] hover:text-xuepo hover:bg-xuepo/10 hover:border-xuepo/30 hover:shadow-[2px_2px_0_0_rgba(0,0,0,0.4)] active:translate-x-[1px] active:translate-y-[1px] active:shadow-none transition-all duration-150";
</script>

<template>
    <div
        class="right-dock fixed right-4 top-1/2 -translate-y-1/2 z-chrome flex flex-col bg-ms-xuan/95 backdrop-blur-xl border border-ms-copper/40 rounded-altar shadow-[3px_3px_0_0_rgba(0,0,0,0.5)] overflow-hidden transition-all duration-400 ease-[cubic-bezier(0.16,1,0.3,1)]"
        @mouseenter="isExpanded = true"
        @mouseleave="isExpanded = false">

        <!-- 收起态：仅图标 -->
        <Transition name="fade-out">
            <div v-if="!isExpanded" class="dock-collapsed p-2.5 flex flex-col items-center gap-2">
                <button
                    :class="collapsedBtnClass"
                    title="控制面板">
                    <SlidersHorizontal :size="18" />
                </button>
            </div>
        </Transition>

        <!-- 展开态：完整控制面板 -->
        <Transition name="fade-in">
            <div v-if="isExpanded" class="dock-expanded p-4 flex flex-col gap-2">
                <!-- ── 列表视图控制 ── -->
                <template v-if="viewMode === 'list'">
                    <div class="flex flex-col gap-2">
                        <span class="text-2xs text-ms-smoke font-mono uppercase tracking-spine border-b border-ms-copper/30 pb-1.5">排序</span>
                        <button @click="store.setSortBy('updated')" :class="ctrlBtnClass(sortBy === 'updated')">
                            <Clock :size="14" /> 时间
                        </button>
                        <button @click="store.setSortBy('hot')" :class="ctrlBtnClass(sortBy === 'hot')">
                            <Flame :size="14" /> 热度
                        </button>
                    </div>
                    <div :class="dividerClass" />
                    <div class="flex flex-col gap-2">
                        <span class="text-2xs text-ms-smoke font-mono uppercase tracking-spine border-b border-ms-copper/30 pb-1.5">密度</span>
                        <button @click="store.setDensity('cozy')" :class="ctrlBtnClass(density === 'cozy')">
                            <LayoutGrid :size="14" /> 舒适
                        </button>
                        <button @click="store.setDensity('compact')" :class="ctrlBtnClass(density === 'compact')">
                            <AlignJustify :size="14" /> 紧凑
                        </button>
                    </div>
                </template>

                <!-- ── 图谱视图控制 ── -->
                <template v-else>
                    <button @click="emit('fitView')"
                        :class="ctrlBtnClass(false)">
                        <Minimize2 :size="14" /> 归位
                    </button>
                </template>
            </div>
        </Transition>
    </div>
</template>

<style scoped>
.right-dock {
    width: 64px;
}

.right-dock:hover {
    width: 200px;
}

/* ── 淡入淡出动画 ── */
.fade-in-enter-active,
.fade-out-leave-active {
    transition: opacity 200ms ease-out;
}

.fade-in-enter-from,
.fade-out-leave-to {
    opacity: 0;
}

.fade-in-enter-to,
.fade-out-leave-from {
    opacity: 1;
}

/* ── 收起态淡出 ── */
.fade-out-leave-active {
    transition: opacity 150ms ease-in;
}

/* ── 展开态淡入 ── */
.fade-in-enter-active {
    transition: opacity 200ms ease-out 50ms;
}

/* ── 血珀辉光增强 ── */
.shadow-altar-glow-sm {
    box-shadow: 0 0 8px rgba(166, 38, 38, 0.2), 0 0 16px rgba(166, 38, 38, 0.1);
}

/* ── 香烟拖尾背景（展开时） ── */
.right-dock:hover .dock-expanded {
    position: relative;
}

.right-dock:hover .dock-expanded::before {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(to bottom,
        transparent 0%,
        rgba(166, 38, 38, 0.02) 20%,
        rgba(166, 38, 38, 0.04) 50%,
        rgba(166, 38, 38, 0.02) 80%,
        transparent 100%);
    opacity: 0.5;
    pointer-events: none;
    z-index: 0;
}

.right-dock:hover .dock-expanded > * {
    position: relative;
    z-index: 1;
}
</style>
