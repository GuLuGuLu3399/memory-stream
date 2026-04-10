<script setup lang="ts">
/**
 * 📊 StatsWidget — 右下角可折叠数据面板
 * 神殿控制台风格：折叠态小球，展开态精致面板 + Mini Sparkline
 */
import { ref, computed } from "vue";
import { storeToRefs } from "pinia";
import { useGraphStore } from "../store/useGraphStore";

const props = defineProps<{
    totalNodes: number;
    todayCount: number;
    avgHot: string;
    sortLabel: string;
    sparklineData: number[];
}>();

const store = useGraphStore();
const { spotlightMode } = storeToRefs(store);

const expanded = ref(false);

// ── Mini Sparkline SVG points ──
const sparklinePoints = computed(() => {
    const data = props.sparklineData;
    if (!data || data.length < 2) return "";
    const max = Math.max(...data, 1);
    const w = 200;
    const h = 32;
    const step = w / (data.length - 1);
    return data
        .map((v, i) => `${(i * step).toFixed(1)},${(h - (v / max) * h * 0.8 - h * 0.1).toFixed(1)}`)
        .join(" ");
});
</script>

<template>
    <div class="fixed right-8 bottom-8 z-30 select-none">
        <!-- 折叠态：霓虹小球 + 聚光灯旋转环 -->
        <button v-if="!expanded" @click="expanded = true"
            class="relative w-12 h-12 rounded-none flex items-center justify-center transition-all duration-300 hover:scale-110"
            :class="todayCount > 0
                ? 'bg-gradient-to-br from-neon/30 to-cyan-500/30 shadow-neon-glow-ball border border-neon/40'
                : 'bg-ms-panel/80 border border-ms-border shadow-lg shadow-black/30'">
            <!-- 聚光灯激活时的旋转光环 -->
            <div v-if="spotlightMode"
                class="spotlight-square absolute -inset-1 border-2 border-neon/50 pointer-events-none" />
            <span class="text-lg font-mono font-bold relative z-10" :class="todayCount > 0 ? 'text-neon' : 'text-gray-400'">
                {{ todayCount > 0 ? `+${todayCount}` : totalNodes }}
            </span>
        </button>

        <!-- 展开态：精致面板 -->
        <Transition name="widget-expand">
            <div v-if="expanded"
                class="rounded-sm bg-ms-panel/85 backdrop-blur-xl border border-ms-border shadow-2xl shadow-black/50 p-5 min-w-stats-panel origin-bottom-right">

                <!-- 头部 -->
                <div class="flex items-center justify-between mb-4">
                    <div class="flex items-center gap-2">
                        <span class="w-2 h-2 bg-neon animate-pulse" />
                        <span class="text-sm font-mono text-gray-400 uppercase tracking-wider">神殿数据</span>
                    </div>
                    <button @click="expanded = false"
                        class="w-6 h-6 rounded-md flex items-center justify-center text-gray-500 hover:text-gray-300 hover:bg-white/5 transition-colors">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                            stroke-width="2">
                            <path d="M18 6L6 18M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                <!-- 三列数据 -->
                <div class="flex items-center justify-between gap-4 mb-4">
                    <div class="flex flex-col items-center gap-1 flex-1">
                        <span class="text-2xl font-bold text-slate-200 font-mono leading-none">{{ totalNodes }}</span>
                        <span class="text-2xs text-gray-500 font-mono">节点</span>
                    </div>
                    <div class="flex flex-col items-center gap-1 flex-1">
                        <span class="text-2xl font-bold font-mono leading-none"
                            :class="todayCount > 0 ? 'text-neon' : 'text-slate-400'">
                            {{ todayCount > 0 ? `+${todayCount}` : '0' }}
                        </span>
                        <span class="text-2xs text-gray-500 font-mono">今日</span>
                    </div>
                    <div class="flex flex-col items-center gap-1 flex-1">
                        <span class="text-2xl font-bold text-orange-400 font-mono leading-none">{{ avgHot }}</span>
                        <span class="text-2xs text-gray-500 font-mono">热度</span>
                    </div>
                </div>

                <!-- Mini Sparkline -->
                <div v-if="sparklineData.length >= 2" class="mb-3">
                    <div class="flex items-center justify-between mb-1">
                        <span class="text-2xs text-gray-600 font-mono">热度趋势</span>
                        <span class="text-2xs text-gray-600 font-mono">最近 {{ sparklineData.length }} 条</span>
                    </div>
                    <svg class="w-full h-8 opacity-70" viewBox="0 0 200 32" preserveAspectRatio="none">
                        <defs>
                            <linearGradient id="sparkGrad" x1="0" y1="0" x2="0" y2="1">
                                <stop offset="0%" stop-color="rgba(0,229,255,0.3)" />
                                <stop offset="100%" stop-color="rgba(0,229,255,0)" />
                            </linearGradient>
                        </defs>
                        <polygon v-if="sparklinePoints" :points="`0,32 ${sparklinePoints} 200,32`"
                            fill="url(#sparkGrad)" />
                        <polyline v-if="sparklinePoints" :points="sparklinePoints" fill="none" stroke="currentColor"
                            stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="text-neon" />
                    </svg>
                </div>

                <!-- 底部操作栏：排序标签 + 聚光灯切换 -->
                <div class="flex items-center gap-2 pt-3 border-t border-white/[0.06]">
                    <span
                        class="text-xs font-mono text-gray-400 bg-ms-carbon/80 px-2.5 py-1 rounded-md border border-ms-border/50">
                        {{ sortLabel }}
                    </span>
                    <!-- 聚光灯模式切换 -->
                    <button @click="store.toggleSpotlight()"
                        class="flex items-center gap-1.5 text-xs font-mono px-2.5 py-1 rounded-md border transition-all duration-200"
                        :class="spotlightMode
                            ? 'bg-neon/15 border-neon/40 text-neon shadow-neon-glow-btn'
                            : 'bg-ms-carbon/80 border-ms-border/50 text-gray-400 hover:text-gray-300 hover:border-gray-500'">
                        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <rect x="9" y="9" width="6" height="6" fill="currentColor" />
                            <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
                        </svg>
                        <span>{{ spotlightMode ? '聚光灯 ON' : '聚光灯' }}</span>
                    </button>
                </div>
            </div>
        </Transition>
    </div>
</template>

<style scoped>
@keyframes spotlight-spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
}

@keyframes spotlight-pulse {
    0%, 100% { opacity: 0.5; box-shadow: 0 0 8px rgba(0, 229, 255, 0.3); }
    50% { opacity: 0.8; box-shadow: 0 0 16px rgba(0, 229, 255, 0.5); }
}

.spotlight-square {
    animation: spotlight-spin 3s linear infinite, spotlight-pulse 2s ease-in-out infinite;
    border: 2px solid rgba(0, 229, 255, 0.5);
    border-radius: 0;
    position: absolute;
    inset: -1px;
}

.widget-expand-enter-active {
    transition: all 300ms cubic-bezier(0.34, 1.56, 0.64, 1);
}

.widget-expand-leave-active {
    transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
}

.widget-expand-enter-from,
.widget-expand-leave-to {
    opacity: 0;
    transform: scale(0.8);
}
</style>
