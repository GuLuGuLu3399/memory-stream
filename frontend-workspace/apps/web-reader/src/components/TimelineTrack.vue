<script setup lang="ts">
/**
 * 🌟 TimelineTrack — 右侧极简时间轴导航
 *
 * 从卡片数据中提取 year/month 分组，渲染为垂直虚线 + 时间标记。
 * 点击月份可跳转到对应位置，当前可见月份高亮为 neon。
 */

import { computed } from "vue";

interface TimelineGroup {
    label: string; // "2026" or "Mar"
    year: number;
    month: number; // 0 for year-only entries
    index: number; // first card index in this group
    isYear: boolean;
}

const props = defineProps<{
    groups: TimelineGroup[];
    activeLabel: string;
}>();

const emit = defineEmits<{
    jump: [index: number];
}>();

const monthNames = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

const enriched = computed(() =>
    props.groups.map((g) => ({
        ...g,
        display: g.isYear ? String(g.year) : monthNames[g.month - 1] || "",
        active: g.label === props.activeLabel,
    })),
);
</script>

<template>
    <div v-if="enriched.length > 0"
        class="absolute right-2 top-1/2 -translate-y-1/2 z-20 flex flex-col items-end select-none"
        style="pointer-events: auto;">
        <!-- Vertical dashed line -->
        <div class="absolute right-[3px] top-4 bottom-4 w-px border-r border-dashed border-gray-800" />

        <div v-for="g in enriched" :key="g.label"
            class="relative flex items-center justify-end group/tl cursor-pointer transition-all duration-200"
            :class="g.isYear ? 'mt-3 mb-1' : 'mb-0.5'" :style="{ minHeight: g.isYear ? '20px' : '16px' }"
            @click="emit('jump', g.index)">
            <!-- Dot on the line -->
            <div class="absolute right-0 w-[7px] h-[7px] rounded-full border transition-all duration-200" :class="g.active
                ? 'bg-neon border-neon shadow-[0_0_8px_rgba(0,229,255,0.5)]'
                : 'bg-ms-deep border-gray-700 group-hover/tl:border-gray-500'" />

            <!-- Label -->
            <span class="pr-4 transition-all duration-200 whitespace-nowrap" :class="[
                g.isYear ? 'text-[11px] font-mono font-bold' : 'text-[10px] font-mono',
                g.active ? 'text-neon' : 'text-gray-600 group-hover/tl:text-gray-400',
            ]">
                {{ g.display }}
            </span>
        </div>
    </div>
</template>