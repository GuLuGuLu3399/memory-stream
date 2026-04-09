<script setup lang="ts">
/**
 * 🌟 LeftDock — 呼吸式侧边导航栏
 *
 * - 固定左侧垂直居中
 * - 自动收回：滚动时隐藏，1.5s 后回弹
 * - 边缘唤醒：鼠标靠近左边缘 20px 时滑入
 */
import { ref, onMounted, onUnmounted } from "vue";
import { storeToRefs } from "pinia";
import { List, Network, SlidersHorizontal, Flame, Clock, Spotlight, Minimize2 } from "lucide-vue-next";
import { useGraphStore } from "../store/useGraphStore";
import { wsConnected, wsAuthenticated, wsLatency } from "../composables/useGraphSync";

const store = useGraphStore();
const { viewMode, sortBy, graphDepth, spotlightMode } = storeToRefs(store);
const connected = wsConnected;
const authenticated = wsAuthenticated;
const latency = wsLatency;

const panelOpen = ref(false);

// ── 呼吸式显隐 ──
const isVisible = ref(true);
let scrollTimer: ReturnType<typeof setTimeout> | null = null;
const dockRef = ref<HTMLElement>();

function onScroll() {
    isVisible.value = false;
    if (scrollTimer) clearTimeout(scrollTimer);
    scrollTimer = setTimeout(() => {
        isVisible.value = true;
    }, 1500);
}

function onMouseMove(e: MouseEvent) {
    if (e.clientX < 20) {
        isVisible.value = true;
        if (scrollTimer) { clearTimeout(scrollTimer); scrollTimer = null; }
    }
}

function handleClickOutside(e: MouseEvent) {
    if (panelOpen.value && dockRef.value && !dockRef.value.contains(e.target as Node)) {
        panelOpen.value = false;
    }
}

onMounted(() => {
    document.addEventListener("click", handleClickOutside);
    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("scroll", onScroll, true);
});

onUnmounted(() => {
    document.removeEventListener("click", handleClickOutside);
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("scroll", onScroll, true);
    if (scrollTimer) clearTimeout(scrollTimer);
});

const emit = defineEmits<{ fitView: [] }>();

const navBtnClass = (active: boolean) =>
    `w-12 h-12 flex items-center justify-center transition-all duration-500 ${active
        ? "text-white/80"
        : "text-white/20 hover:text-white/50"
    }`;

const ctrlBtnClass = (active: boolean) =>
    `flex items-center gap-2 px-3 py-2 text-sm font-mono transition-all duration-500 whitespace-nowrap ${active
        ? "text-white/70 border-b border-white/10"
        : "text-white/30 hover:text-white/50 border-b border-transparent hover:border-white/10"
    }`;
</script>

<template>
    <div ref="dockRef"
        class="fixed left-0 top-0 h-full z-chrome flex items-start transition-all duration-500 ease-out"
        :style="{
            opacity: isVisible ? 1 : 0,
        }">

        <!-- 主导航栏 -->
        <div class="flex flex-col items-center justify-between bg-ms-void border-r border-ms-border/30 py-6"
            style="width: 56px;">
            <div class="flex flex-col items-center gap-2.5">
                <button :class="navBtnClass(false)" @click="store.toggleCommandPalette()" title="搜索 (⌘K)">
                    <span class="text-sm font-bold font-mono">M</span>
                </button>
                <div class="w-7 h-px bg-ms-border/30" />
                <button :class="navBtnClass(viewMode === 'list')" @click="store.setViewMode('list')" title="列表视图">
                    <List :size="20" />
                </button>
                <button :class="navBtnClass(viewMode === 'graph')" @click="store.setViewMode('graph')" title="图谱视图">
                    <Network :size="20" />
                </button>
            </div>
            <div class="flex flex-col items-center gap-2.5">
                <div class="w-7 h-px bg-ms-border/30" />
                <button :class="navBtnClass(panelOpen)" @click.stop="panelOpen = !panelOpen" title="控制面板">
                    <SlidersHorizontal :size="20" />
                </button>
                <div class="flex flex-col items-center gap-0.5 mt-1"
                    :title="`WS: ${authenticated ? '已连接 ' + latency + 'ms' : connected ? '认证中...' : '离线'}`">
                    <div class="w-2.5 h-2.5 rounded-full transition-colors duration-300"
                        :class="authenticated ? 'bg-emerald-400/60' : connected ? 'bg-yellow-400/40' : 'bg-red-500/60'" />
                    <span v-if="authenticated && latency > 0" class="text-[9px] font-mono text-gray-500">{{ latency
                        }}ms</span>
                </div>
            </div>
        </div>

        <!-- 可展开控制面板 -->
        <Transition name="panel-slide">
            <div v-if="panelOpen"
                class="bg-ms-void border-r border-ms-border/30 p-4"
                style="width: 220px;">
                <template v-if="viewMode === 'list'">
                    <div class="flex flex-col gap-2">
                        <span class="text-xs text-gray-500 font-mono uppercase tracking-wider mb-1">排序</span>
                        <button @click="store.setSortBy('updated')" :class="ctrlBtnClass(sortBy === 'updated')">
                            <Clock :size="16" /> 时间
                        </button>
                        <button @click="store.setSortBy('hot')" :class="ctrlBtnClass(sortBy === 'hot')">
                            <Flame :size="16" /> 热度
                        </button>
                    </div>
                </template>
                <template v-else>
                    <div class="flex flex-col gap-2">
                        <span class="text-xs text-gray-500 font-mono uppercase tracking-wider mb-1">深度</span>
                        <div class="flex items-center gap-1.5">
                            <button v-for="d in 3" :key="d" @click="store.setGraphDepth(d)" :class="`w-9 h-9 rounded-lg text-sm font-mono font-bold transition-all ${graphDepth >= d
                                ? 'bg-neon/15 text-neon border border-neon/30'
                                : 'text-gray-600 border border-ms-border/50 hover:border-ms-border'
                                }`">
                                {{ d }}
                            </button>
                        </div>
                    </div>
                    <div class="h-px bg-ms-border/40 my-3" />
                    <button @click="store.toggleSpotlight()" :class="ctrlBtnClass(spotlightMode)">
                        <Spotlight :size="16" /> 聚光灯
                    </button>
                    <button @click="emit('fitView')"
                        class="flex items-center gap-2 px-3 py-2 text-sm font-mono rounded-lg text-gray-400 hover:text-gray-200 border border-transparent hover:border-ms-border transition-all whitespace-nowrap">
                        <Minimize2 :size="16" /> 归位
                    </button>
                </template>
            </div>
        </Transition>
    </div>
</template>

<style scoped>
.panel-slide-enter-active,
.panel-slide-leave-active {
    transition: opacity 500ms cubic-bezier(0.4, 0, 0.2, 1),
        transform 500ms cubic-bezier(0.4, 0, 0.2, 1);
}

.panel-slide-enter-from,
.panel-slide-leave-to {
    opacity: 0;
    transform: translateX(-12px);
}
</style>