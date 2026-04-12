<script setup lang="ts">
/**
 * LeftDock — 殿柱（血肉神殿）
 *
 * 左侧殿柱结构：
 * - 柱头斗拱：神殿徽章 + 金色冠顶
 * - 柱身壁龛：按钮嵌入石壁凹陷
 * - 金缮嵌线：铜质装饰分隔
 * - 柱基香炉：WS 状态以香炉 + 烟雾呈现
 * - 自动收回 + 边缘唤醒
 */
import { ref, onMounted, onUnmounted } from "vue";
import { storeToRefs } from "pinia";
import { useRouter } from "vue-router";
import { List, Network, Flame as FlameIcon } from "lucide-vue-next";
import { useGraphStore } from "../store/useGraphStore";
import { wsConnected, wsAuthenticated, wsLatency } from "../composables/useGraphSync";

const store = useGraphStore();
const router = useRouter();
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
</script>

<template>
    <div
        class="fixed left-0 top-0 h-full z-chrome flex items-stretch transition-all duration-500 ease-out"
        :style="{
            opacity: isVisible ? 1 : 0,
            transform: isVisible ? 'translateX(0)' : 'translateX(-12px)',
        }">

        <!-- 殿柱主体 -->
        <div class="pillar">
            <!-- 柱头斗拱 — 冠顶装饰 -->
            <div class="pillar__capital">
                <div class="pillar__capital-bracket" />
                <!-- 神殿徽章 -->
                <button @click="store.toggleCommandPalette()"
                    class="pillar__emblem group"
                    title="搜索 (⌘K)">
                    <span class="text-base font-bold font-serif tracking-wider text-ms-bone group-hover:text-ms-gold transition-colors duration-300">M</span>
                    <span class="absolute -bottom-1 text-[8px] font-mono text-ms-ash/40 opacity-0 group-hover:opacity-100 transition-opacity">⌘K</span>
                </button>
            </div>

            <!-- 柱身 — 石壁纹理 + 壁龛按钮 -->
            <div class="pillar__shaft">
                <!-- 金缮嵌线 -->
                <div class="pillar__inlay" />

                <!-- 列表视图壁龛 -->
                <button @click="router.push('/list')"
                    class="niche group"
                    :class="{ 'niche--lit': viewMode === 'list' }">
                    <div class="niche__recess">
                        <List :size="17" class="niche__icon" />
                        <span class="niche__label">列表</span>
                    </div>
                    <!-- 壁龛灯笼光 -->
                    <div v-if="viewMode === 'list'" class="niche__glow" />
                    <!-- 灯芯脉动 -->
                    <div v-if="viewMode === 'list'" class="niche__wick" />
                </button>

                <!-- 图谱视图壁龛 -->
                <button @click="router.push('/graph')"
                    class="niche group"
                    :class="{ 'niche--lit': viewMode === 'graph' }">
                    <div class="niche__recess">
                        <Network :size="17" class="niche__icon" />
                        <span class="niche__label">图谱</span>
                    </div>
                    <div v-if="viewMode === 'graph'" class="niche__glow" />
                    <div v-if="viewMode === 'graph'" class="niche__wick" />
                </button>

                <!-- 金缮嵌线 -->
                <div class="pillar__inlay" />
            </div>

            <!-- 柱基香炉 -->
            <div class="pillar__base">
                <!-- 香炉容器 -->
                <div class="brazier"
                    :class="{
                        'brazier--lit': authenticated,
                        'brazier--warm': connected && !authenticated,
                        'brazier--cold': !connected
                    }">
                    <!-- 炉体 -->
                    <div class="brazier__body">
                        <FlameIcon :size="13" class="brazier__flame" />
                    </div>
                    <!-- 烟雾 -->
                    <div v-if="connected" class="brazier__smoke">
                        <span class="smoke-wisp smoke-wisp--1" />
                        <span class="smoke-wisp smoke-wisp--2" />
                    </div>
                    <!-- 延迟环 -->
                    <svg v-if="authenticated" class="brazier__ring" viewBox="0 0 28 28">
                        <circle cx="14" cy="14" r="12" fill="none" stroke="currentColor" stroke-width="1"
                            class="text-ms-success/20" />
                        <circle cx="14" cy="14" r="12" fill="none" stroke="currentColor" stroke-width="1"
                            :stroke-dasharray="75" :stroke-dashoffset="75 - (latency > 0 ? Math.min(latency / 100, 1) * 75 : 38)"
                            stroke-linecap="round" class="text-ms-success/50 transition-all duration-700" />
                    </svg>
                </div>
                <span v-if="latency > 0" class="brazier__latency"
                    :class="latency < 50 ? 'text-ms-success/60' : latency < 150 ? 'text-ms-gold/60' : 'text-xuepo/50'">
                    {{ latency }}
                </span>
            </div>
        </div>
    </div>
</template>

<style scoped>
/* ═══ 殿柱主体 ═══ */
.pillar {
    width: 62px;
    display: flex;
    flex-direction: column;
    align-items: center;
    background:
        /* 石壁纹理 — 竖向微纹 */
        repeating-linear-gradient(
            0deg,
            transparent,
            transparent 3px,
            rgba(42, 34, 24, 0.04) 3px,
            rgba(42, 34, 24, 0.04) 4px
        ),
        /* 中心高光 — 圆柱光影 */
        radial-gradient(
            ellipse 40% 100% at 50% 50%,
            rgba(58, 50, 40, 0.06) 0%,
            transparent 70%
        ),
        /* 基底色 */
        linear-gradient(180deg, #16130f, #12100c, #0e0d0a);
    border-right: 1px solid rgba(58, 50, 40, 0.5);
    box-shadow:
        inset -1px 0 0 rgba(232, 223, 208, 0.02),
        3px 0 8px rgba(0, 0, 0, 0.4);
}

/* ── 柱头斗拱 ── */
.pillar__capital {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding-top: 10px;
    padding-bottom: 8px;
}

/* 斗拱横梁 */
.pillar__capital-bracket {
    width: 48px;
    height: 4px;
    background: linear-gradient(90deg, transparent, rgba(201, 168, 76, 0.2), rgba(201, 168, 76, 0.25), rgba(201, 168, 76, 0.2), transparent);
    border-radius: 0 0 1px 1px;
    margin-bottom: 8px;
}

/* 神殿徽章 */
.pillar__emblem {
    position: relative;
    width: 42px;
    height: 42px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(145deg, rgba(58, 50, 40, 0.3), rgba(28, 24, 20, 0.6));
    border: 1px solid rgba(201, 168, 76, 0.2);
    border-radius: 3px;
    cursor: pointer;
    transition: all 250ms ease;
    box-shadow:
        inset 0 1px 0 rgba(232, 223, 208, 0.05),
        0 2px 6px rgba(0, 0, 0, 0.3);
}

.pillar__emblem:hover {
    border-color: rgba(201, 168, 76, 0.4);
    box-shadow:
        inset 0 1px 0 rgba(232, 223, 208, 0.08),
        0 0 12px rgba(201, 168, 76, 0.1),
        0 2px 6px rgba(0, 0, 0, 0.3);
}

.pillar__emblem:active {
    transform: scale(0.95);
    box-shadow:
        inset 0 1px 0 rgba(232, 223, 208, 0.02),
        0 0 4px rgba(0, 0, 0, 0.3);
}

/* ── 柱身 ── */
.pillar__shaft {
    flex: 1;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 6px 0;
}

/* 金缮嵌线 */
.pillar__inlay {
    width: 32px;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(201, 168, 76, 0.25), transparent);
}

/* ═══ 壁龛按钮 ═══ */
.niche {
    position: relative;
    width: 46px;
    height: 54px;
    cursor: pointer;
    transition: all 200ms ease;
}

.niche__recess {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 3px;
    /* 石壁凹陷 */
    background: linear-gradient(145deg, rgba(10, 9, 7, 0.6), rgba(14, 13, 10, 0.4));
    border: 1px solid rgba(42, 34, 24, 0.4);
    border-radius: 2px;
    box-shadow:
        inset 1px 1px 3px rgba(0, 0, 0, 0.4),
        inset -1px -1px 2px rgba(232, 223, 208, 0.02);
    transition: all 250ms ease;
}

.niche:hover .niche__recess {
    border-color: rgba(58, 50, 40, 0.6);
    background: linear-gradient(145deg, rgba(14, 12, 9, 0.7), rgba(18, 16, 12, 0.5));
    box-shadow:
        inset 1px 1px 4px rgba(0, 0, 0, 0.5),
        inset -1px -1px 3px rgba(232, 223, 208, 0.03);
}

.niche:active .niche__recess {
    box-shadow:
        inset 2px 2px 6px rgba(0, 0, 0, 0.6),
        inset -1px -1px 2px rgba(232, 223, 208, 0.01);
    transition-duration: 80ms;
}

.niche__icon {
    color: #5a4f3e;
    transition: color 250ms ease;
}

.niche:hover .niche__icon {
    color: #8a7e6e;
}

.niche--lit .niche__icon {
    color: #c8a060;
}

.niche__label {
    font-size: 9px;
    font-weight: 500;
    color: rgba(90, 79, 62, 0.4);
    transition: color 250ms ease;
}

.niche:hover .niche__label {
    color: rgba(138, 126, 110, 0.6);
}

.niche--lit .niche__label {
    color: rgba(200, 160, 96, 0.7);
}

/* 壁龛灯笼光 — 暖光从内部溢出 */
.niche__glow {
    position: absolute;
    inset: -2px;
    border-radius: 4px;
    background: radial-gradient(
        ellipse 60% 80% at 50% 40%,
        rgba(201, 168, 76, 0.08) 0%,
        transparent 70%
    );
    pointer-events: none;
    animation: lanternPulse 3s ease-in-out infinite;
}

@keyframes lanternPulse {
    0%, 100% { opacity: 0.6; }
    50% { opacity: 1; }
}

/* 灯芯 — 壁龛中心微弱亮点 */
.niche__wick {
    position: absolute;
    top: 8px;
    left: 50%;
    transform: translateX(-50%);
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: rgba(201, 168, 76, 0.5);
    box-shadow: 0 0 6px rgba(201, 168, 76, 0.3);
    animation: wickFlicker 2s ease-in-out infinite;
    pointer-events: none;
}

@keyframes wickFlicker {
    0%, 100% { opacity: 0.5; transform: translateX(-50%) scale(1); }
    25% { opacity: 0.7; transform: translateX(-50%) scale(1.1); }
    50% { opacity: 0.9; transform: translateX(-50%) scale(1); }
    75% { opacity: 0.6; transform: translateX(-50%) scale(0.9); }
}

/* ── 柱基香炉 ── */
.pillar__base {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 8px 0 12px;
}

.brazier {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
}

.brazier__body {
    width: 24px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    /* 香炉形状 — 上宽下窄 */
    background: linear-gradient(180deg,
        rgba(42, 34, 24, 0.4) 0%,
        rgba(28, 24, 20, 0.6) 100%);
    border: 1px solid rgba(58, 50, 40, 0.35);
    border-radius: 3px 3px 2px 2px;
    box-shadow:
        inset 0 1px 0 rgba(232, 223, 208, 0.03),
        0 2px 4px rgba(0, 0, 0, 0.3);
    transition: all 300ms ease;
}

.brazier--lit .brazier__body {
    border-color: rgba(90, 156, 96, 0.25);
    box-shadow:
        0 0 8px rgba(90, 156, 96, 0.08),
        0 2px 4px rgba(0, 0, 0, 0.3);
}

.brazier--warm .brazier__body {
    border-color: rgba(201, 168, 76, 0.25);
    box-shadow:
        0 0 6px rgba(201, 168, 76, 0.06),
        0 2px 4px rgba(0, 0, 0, 0.3);
    animation: brazierWarm 2s ease-in-out infinite;
}

@keyframes brazierWarm {
    0%, 100% { box-shadow: 0 0 4px rgba(201, 168, 76, 0.04), 0 2px 4px rgba(0, 0, 0, 0.3); }
    50% { box-shadow: 0 0 10px rgba(201, 168, 76, 0.1), 0 2px 4px rgba(0, 0, 0, 0.3); }
}

.brazier__flame {
    transition: color 300ms ease;
}

.brazier--lit .brazier__flame {
    color: #5a9c60;
}

.brazier--warm .brazier__flame {
    color: #c9a84c;
    animation: flameDance 1.5s ease-in-out infinite;
}

.brazier--cold .brazier__flame {
    color: rgba(166, 38, 38, 0.4);
}

@keyframes flameDance {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.1) translateY(-0.5px); }
}

/* 香烟缭绕 */
.brazier__smoke {
    position: absolute;
    top: -4px;
    left: 50%;
    transform: translateX(-50%);
    width: 12px;
    height: 20px;
    pointer-events: none;
}

.smoke-wisp {
    position: absolute;
    bottom: 0;
    left: 50%;
    width: 2px;
    height: 8px;
    border-radius: 1px;
    opacity: 0;
    animation: smokeRise 3s ease-out infinite;
}

.smoke-wisp--1 {
    animation-delay: 0s;
    background: linear-gradient(to top, rgba(138, 126, 110, 0.15), transparent);
}

.smoke-wisp--2 {
    animation-delay: 1.5s;
    background: linear-gradient(to top, rgba(138, 126, 110, 0.1), transparent);
}

@keyframes smokeRise {
    0% {
        opacity: 0;
        transform: translateX(-50%) translateY(0) scaleX(1);
    }
    20% {
        opacity: 0.4;
    }
    60% {
        opacity: 0.15;
        transform: translateX(-30%) translateY(-12px) scaleX(1.5);
    }
    100% {
        opacity: 0;
        transform: translateX(10%) translateY(-20px) scaleX(2);
    }
}

/* 延迟环 */
.brazier__ring {
    position: absolute;
    inset: -4px;
    width: 28px;
    height: 28px;
    transform: rotate(-90deg);
}

.brazier__latency {
    font-size: 9px;
    font-family: 'JetBrains Mono', monospace;
}
</style>
