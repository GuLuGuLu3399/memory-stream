<script setup lang="ts">
/**
 * LeftDock — 殿门旌旗（血肉神殿）
 *
 * - 固定左侧垂直居中
 * - 自动收回：滚动时隐藏，1.5s 后回弹
 * - 边缘唤醒：鼠标靠近左边缘 20px 时滑入
 * - 殿柱雕纹按钮，血珀灯笼辉光，金箔分隔
 * - WebSocket 状态以火焰图标呈现
 * - 长按显示快捷键提示
 */
import { ref, onMounted, onUnmounted } from "vue";
import { storeToRefs } from "pinia";
import { List, Network, Flame as FlameIcon } from "lucide-vue-next";
import { useGraphStore } from "../store/useGraphStore";
import { wsConnected, wsAuthenticated, wsLatency } from "../composables/useGraphSync";

const store = useGraphStore();
const { viewMode } = storeToRefs(store);
const connected = wsConnected;
const authenticated = wsAuthenticated;
const latency = wsLatency;

// ── 呼吸式显隐 ──
const isVisible = ref(true);
let scrollTimer: ReturnType<typeof setTimeout> | null = null;

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

onMounted(() => {
    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("scroll", onScroll, true);
});

onUnmounted(() => {
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("scroll", onScroll, true);
    if (scrollTimer) clearTimeout(scrollTimer);
});

// ── 殿柱按钮雕纹样式 ──
const templeColumnClass = (active: boolean) =>
    `relative group overflow-hidden ${active
        ? 'text-xuepo'
        : 'text-ms-smoke hover:text-ms-bone'
    } transition-all duration-300`;

// 金箔分隔线
const goldDivider = "w-6 h-px bg-gradient-to-r from-transparent via-ms-gold/40 to-transparent my-1";
</script>

<template>
    <div
        class="fixed left-0 top-0 h-full z-chrome flex items-start transition-all duration-500 ease-out"
        :style="{
            opacity: isVisible ? 1 : 0,
            transform: isVisible ? 'translateX(0)' : 'translateX(-12px)',
        }">

        <!-- 殿柱主导航栏 -->
        <div class="flex flex-col items-center justify-between bg-ms-xuan/95 backdrop-blur-md border-r border-ms-copper/40 py-6 shadow-[2px_0_0_0_rgba(0,0,0,0.4)]"
            style="width: 64px;">
            <!-- 上部导航区 -->
            <div class="flex flex-col items-center gap-3">
                <!-- 神殿徽章 -->
                <button @click="store.toggleCommandPalette()"
                    class="relative w-12 h-12 flex items-center justify-center rounded-altar bg-ms-xiang/50 border border-ms-copper/30 temple-column-btn"
                    :class="templeColumnClass(false)"
                    title="搜索 (⌘K)">
                    <span class="text-lg font-bold font-serif tracking-wider">M</span>
                    <div class="absolute inset-0 rounded-altar bg-gradient-to-br from-ms-gold/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />
                </button>

                <!-- 金箔分隔 -->
                <div :class="goldDivider" />

                <!-- 列表视图 -->
                <button @click="store.setViewMode('list')"
                    class="relative w-12 h-12 flex items-center justify-center rounded-altar temple-column-btn"
                    :class="templeColumnClass(viewMode === 'list')"
                    title="列表视图 (L)">
                    <List :size="20" />
                    <!-- 激活态血珀辉光 -->
                    <div v-if="viewMode === 'list'" class="absolute inset-0 rounded-altar shadow-altar-glow-sm border border-xuepo/20" />
                    <!-- 雕纹背景 -->
                    <div class="absolute inset-0 rounded-altar bg-gradient-to-br from-ms-copper/10 to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />
                </button>

                <!-- 图谱视图 -->
                <button @click="store.setViewMode('graph')"
                    class="relative w-12 h-12 flex items-center justify-center rounded-altar temple-column-btn"
                    :class="templeColumnClass(viewMode === 'graph')"
                    title="图谱视图 (G)">
                    <Network :size="20" />
                    <div v-if="viewMode === 'graph'" class="absolute inset-0 rounded-altar shadow-altar-glow-sm border border-xuepo/20" />
                    <div class="absolute inset-0 rounded-altar bg-gradient-to-br from-ms-copper/10 to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />
                </button>
            </div>

            <!-- 下部控制区 -->
            <div class="flex flex-col items-center gap-3">
                <!-- 金箔分隔 -->
                <div :class="goldDivider" />

                <!-- WebSocket 火焰状态 -->
                <div class="flex flex-col items-center gap-1 mt-1"
                    :title="`WS: ${authenticated ? '已连接 ' + latency + 'ms' : connected ? '认证中...' : '离线'}`">
                    <div class="relative w-8 h-8 flex items-center justify-center">
                        <!-- 火焰图标 -->
                        <FlameIcon :size="18" :class="{
                            'text-ms-success/80 animate-pulse': authenticated,
                            'text-ms-gold/70 animate-bounce': connected && !authenticated,
                            'text-xuepo/60': !connected
                        }" />
                        <!-- 辉光效果 -->
                        <div v-if="authenticated" class="absolute inset-0 rounded-full bg-ms-success/20 blur-md animate-pulse" />
                    </div>
                    <span v-if="authenticated && latency > 0" class="text-3xs font-mono text-ms-ash">{{ latency }}ms</span>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
/* ── 殿柱按钮雕纹 — optical elevation ── */
.temple-column-btn {
    position: relative;
    background: linear-gradient(135deg, rgba(58, 50, 40, 0.15) 0%, rgba(42, 34, 24, 0.08) 50%, rgba(58, 50, 40, 0.15) 100%);
    border: 1px solid rgba(58, 50, 40, 0.3);
    box-shadow:
        inset 0 1px 0 rgba(232, 223, 208, 0.05),
        2px 2px 0 0 rgba(0, 0, 0, 0.4);
    transition: transform 0.15s ease, box-shadow 0.15s ease, border-color 0.2s ease, background 0.2s ease;
}

.temple-column-btn:hover {
    background: linear-gradient(135deg, rgba(58, 50, 40, 0.25) 0%, rgba(42, 34, 24, 0.15) 50%, rgba(58, 50, 40, 0.25) 100%);
    border-color: rgba(58, 50, 40, 0.5);
    transform: translate(-1px, -1px);
    box-shadow:
        inset 0 1px 0 rgba(232, 223, 208, 0.08),
        3px 3px 0 0 rgba(0, 0, 0, 0.4);
}

.temple-column-btn:active {
    transform: translate(1px, 1px);
    box-shadow:
        inset 0 1px 0 rgba(232, 223, 208, 0.03),
        0px 0px 0 0 rgba(0, 0, 0, 0.4);
}

/* ── 血珀辉光增强 ── */
.shadow-altar-glow-sm {
    box-shadow: 0 0 8px rgba(166, 38, 38, 0.2), 0 0 16px rgba(166, 38, 38, 0.1);
}
</style>
