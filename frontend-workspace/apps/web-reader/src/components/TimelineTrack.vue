<script setup lang="ts">
/**
 * 🌟 TimelineTrack — 右侧极简时间轴导航（血肉神殿）
 *
 * 从卡片数据中提取 year/month 分组，渲染为垂直血珀流光 + 神殿门年份标记。
 * 点击月份可跳转到对应位置，当前可见月份高亮为血珀色。
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
        <!-- 垂直血珀流光线 -->
        <div class="absolute right-[3px] top-4 bottom-4 w-px blood-flow-line" />

        <div v-for="g in enriched" :key="g.label"
            class="relative flex items-center justify-end group/tl cursor-pointer transition-all duration-200"
            :class="g.isYear ? 'mt-3 mb-1' : 'mb-0.5'" :style="{ minHeight: g.isYear ? '20px' : '16px' }"
            @click="emit('jump', g.index)">
            <!-- 线上的点 -->
            <div class="absolute right-0 w-[7px] h-[7px] rounded-full border transition-all duration-200"
                :class="g.active
                    ? 'bg-xuepo border-xuepo active-dot-glow'
                    : 'bg-ms-xiang border-ms-copper/30 group-hover/tl:border-ms-copper'" />

            <!-- 年份：神殿门标记 -->
            <template v-if="g.isYear">
                <div class="pr-3 flex items-center temple-gate" :class="{ 'temple-gate-active': g.active }">
                    <svg class="w-4 h-4 text-ms-smoke transition-colors" :class="g.active ? 'text-ms-gold gate-glow' : ''" viewBox="0 0 16 16">
                        <!-- 神殿门形状 (门) -->
                        <line x1="3" y1="3" x2="3" y2="13" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"/>
                        <line x1="13" y1="3" x2="13" y2="13" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"/>
                        <line x1="3" y1="3" x2="13" y2="3" stroke="currentColor" stroke-width="1.5" stroke-linecap="square"/>
                    </svg>
                    <span class="ml-1 text-[10px] font-mono font-bold transition-colors" :class="g.active ? 'text-ms-gold' : 'text-ms-smoke group-hover/tl:text-ms-bone'">
                        {{ g.display }}
                    </span>
                </div>
            </template>

            <!-- 月份：普通标签 -->
            <template v-else>
                <span class="pr-4 transition-all duration-200 whitespace-nowrap text-[10px] font-mono"
                    :class="g.active ? 'text-xuepo' : 'text-ms-smoke group-hover/tl:text-ms-bone'">
                    {{ g.display }}
                </span>
            </template>
        </div>
    </div>
</template>

<style scoped>
/* ── 血珀流光连接线 ── */
.blood-flow-line {
    background: linear-gradient(180deg,
        transparent 0%,
        rgba(166, 38, 38, 0.1) 20%,
        rgba(166, 38, 38, 0.15) 50%,
        rgba(166, 38, 38, 0.1) 80%,
        transparent 100%
    );
    background-size: 100% 200%;
    animation: blood-flow 4s ease-in-out infinite;
}

@keyframes blood-flow {
    0%, 100% {
        background-position: 0% 0%;
    }

    50% {
        background-position: 0% 100%;
    }
}

/* ── 活跃点的血珀光晕 ── */
.active-dot-glow {
    box-shadow: 0 0 8px rgba(166, 38, 38, 0.6),
                0 0 16px rgba(166, 38, 38, 0.3);
    animation: active-pulse 2s ease-in-out infinite;
}

@keyframes active-pulse {
    0%, 100% {
        transform: scale(1);
        box-shadow: 0 0 8px rgba(166, 38, 38, 0.6),
                    0 0 16px rgba(166, 38, 38, 0.3);
    }

    50% {
        transform: scale(1.15);
        box-shadow: 0 0 12px rgba(166, 38, 38, 0.8),
                    0 0 20px rgba(166, 38, 38, 0.4);
    }
}

/* ── 神殿门标记 ── */
.temple-gate {
    transition: all 0.2s ease;
}

.temple-gate-active.gate-glow {
    filter: drop-shadow(0 0 6px rgba(201, 168, 76, 0.5));
}

/* 子元素脉冲 */
.temple-gate-active svg {
    animation: gate-pulse 2s ease-in-out infinite;
}

@keyframes gate-pulse {
    0%, 100% {
        opacity: 1;
    }

    50% {
        opacity: 0.8;
    }
}
</style>
