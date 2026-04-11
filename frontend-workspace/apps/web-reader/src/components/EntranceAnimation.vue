<script setup lang="ts">
/**
 * EntranceAnimation — 入殿动画（血肉神殿）
 *
 * 页面首次加载时播放一次性序列动画：
 * 1. 粒子汇聚形成 Logo（0-600ms）
 * 2. Logo 脉冲 + 神殿大门开启（600-1400ms）
 * 3. 标题书法笔触揭示（1400-2200ms）
 * 4. 整体淡出（2200-2800ms）
 */

import { ref, onMounted } from "vue";
import { Sparkles } from "lucide-vue-next";

const emit = defineEmits<{
    (e: "done"): void;
}>();

const phase = ref<"logo" | "doors" | "calligraphy" | "fadeout">("logo");
const visible = ref(true);

// 粒子汇聚到 Logo 形状的固定位置（12个粒子围绕中心）
const particlePositions = [
    { x: -30, y: -20 }, { x: -20, y: -30 }, { x: 0, y: -35 },
    { x: 20, y: -30 }, { x: 30, y: -20 }, { x: 35, y: 0 },
    { x: 30, y: 20 }, { x: 20, y: 30 }, { x: 0, y: 35 },
    { x: -20, y: 30 }, { x: -30, y: 20 }, { x: -35, y: 0 }
];

const startPositions = Array.from({ length: 12 }, () => ({
    x: Math.random() * 200 - 100,
    y: Math.random() * 200 - 100
}));

onMounted(() => {
    setTimeout(() => {
        phase.value = "doors";
    }, 600);

    setTimeout(() => {
        phase.value = "calligraphy";
    }, 1400);

    setTimeout(() => {
        phase.value = "fadeout";
    }, 2200);

    setTimeout(() => {
        visible.value = false;
        emit("done");
    }, 2800);
});

const skipAnimation = () => {
    visible.value = false;
    emit("done");
};
</script>

<template>
    <Transition name="entrance-fade">
        <div v-if="visible" class="entrance-overlay z-entrance">
            <!-- 背景粒子汇聚 -->
            <div class="particle-field">
                <div v-for="(pos, i) in particlePositions" :key="i" class="particle" :style="{
                    '--start-x': `${startPositions[i].x}px`,
                    '--start-y': `${startPositions[i].y}px`,
                    '--end-x': `${pos.x}px`,
                    '--end-y': `${pos.y}px`,
                    animationDelay: `${i * 0.05}s`,
                }" />
            </div>

            <!-- 中心 Logo -->
            <div class="logo-center" :class="{
                'logo-expand': phase !== 'logo',
                'logo-glow-strong': phase === 'doors' || phase === 'calligraphy'
            }">
                <div class="logo-glow" />
                <Sparkles :size="32" class="logo-icon" />
            </div>

            <!-- 神殿大门 -->
            <Transition name="door-slide">
                <div v-if="phase === 'logo' || phase === 'doors'" class="temple-doors">
                    <div class="door door-left" />
                    <div class="door door-right" />
                </div>
            </Transition>

            <!-- 脉冲环 -->
            <Transition name="pulse-appear">
                <div v-if="phase === 'doors' || phase === 'calligraphy'" class="pulse-rings">
                    <div class="pulse-ring ring-1" />
                    <div class="pulse-ring ring-2" />
                    <div class="pulse-ring ring-3" />
                </div>
            </Transition>

            <!-- 标题书法笔触 -->
            <div class="title-text" :class="{ 'title-visible': phase === 'calligraphy' || phase === 'fadeout' }">
                <span class="text-xuepo font-bold tracking-[0.3em] text-lg calligraphy-text">MEMORY</span>
                <span class="text-ms-smoke font-light tracking-[0.2em] text-sm ml-2 calligraphy-text calligraphy-delay">STREAM</span>
            </div>

            <!-- SKIP 按钮 -->
            <button class="skip-btn" @click="skipAnimation">
                SKIP
            </button>
        </div>
    </Transition>
</template>

<style scoped>
.entrance-overlay {
    position: fixed;
    inset: 0;
    background: theme('colors.ms-xuan');
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    overflow: hidden;
}

/* ── 粒子汇聚 ── */
.particle-field {
    position: absolute;
    inset: 0;
    pointer-events: none;
}

.particle {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 3px;
    height: 3px;
    background: theme('colors.xuepo.DEFAULT');
    border-radius: 50%;
    opacity: 0;
    animation: particle-converge 1.2s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

@keyframes particle-converge {
    0% {
        opacity: 0;
        transform: translate(var(--start-x), var(--start-y)) scale(0);
    }

    30% {
        opacity: 0.8;
    }

    100% {
        opacity: 1;
        transform: translate(var(--end-x), var(--end-y)) scale(1);
    }
}

/* ── Logo 中心 ── */
.logo-center {
    position: relative;
    z-index: 2;
    transition: transform 0.6s cubic-bezier(0.16, 1, 0.3, 1);
}

.logo-expand {
    transform: scale(1.15);
}

.logo-glow {
    position: absolute;
    inset: -25px;
    border-radius: 50%;
    background: radial-gradient(circle, rgba(166, 38, 38, 0.2) 0%, transparent 70%);
    transition: all 0.6s ease;
    animation: glow-breathe 2s ease-in-out infinite alternate;
}

.logo-glow-strong {
    background: radial-gradient(circle, rgba(166, 38, 38, 0.35) 0%, transparent 70%);
}

@keyframes glow-breathe {
    from {
        transform: scale(1);
        opacity: 0.6;
    }

    to {
        transform: scale(1.2);
        opacity: 1;
    }
}

.logo-icon {
    color: theme('colors.xuepo.DEFAULT');
    filter: drop-shadow(0 0 16px rgba(166, 38, 38, 0.7));
    animation: icon-pulse 0.8s ease-out;
}

@keyframes icon-pulse {
    from {
        opacity: 0;
        transform: scale(0.3);
    }

    to {
        opacity: 1;
        transform: scale(1);
    }
}

/* ── 神殿大门 ── */
.temple-doors {
    position: absolute;
    inset: 0;
    z-index: 3;
    pointer-events: none;
}

.door {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 50%;
    background: theme('colors.ms-xuan');
    transition: transform 0.8s cubic-bezier(0.16, 1, 0.3, 1);
}

.door::after {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    width: 2px;
    background: theme('colors.ms-copper');
}

.door-left {
    left: 0;
    transform: translateX(0);
}

.door-left::after {
    right: 0;
}

.door-right {
    right: 0;
    transform: translateX(0);
}

.door-right::after {
    left: 0;
}

.door-slide-enter-active,
.door-slide-leave-active {
    transition: all 0.8s cubic-bezier(0.16, 1, 0.3, 1);
}

.door-slide-enter-from,
.door-slide-leave-to {
    opacity: 1;
}

.door-slide-enter-from .door-left,
.door-slide-leave-to .door-left {
    transform: translateX(-100%);
}

.door-slide-enter-from .door-right,
.door-slide-leave-to .door-right {
    transform: translateX(100%);
}

/* ── 脉冲环 ── */
.pulse-rings {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1;
    pointer-events: none;
}

.pulse-ring {
    position: absolute;
    border: 1px solid rgba(166, 38, 38, 0.4);
    animation: ring-expand 2.4s cubic-bezier(0, 0, 0.2, 1) forwards;
}

.ring-1 {
    animation-delay: 0s;
}

.ring-2 {
    animation-delay: 0.25s;
}

.ring-3 {
    animation-delay: 0.5s;
}

@keyframes ring-expand {
    from {
        width: 50px;
        height: 50px;
        opacity: 0.8;
    }

    to {
        width: 600px;
        height: 600px;
        opacity: 0;
    }
}

.pulse-appear-enter-active {
    transition: opacity 0.4s ease;
}

.pulse-appear-enter-from {
    opacity: 0;
}

/* ── 标题书法笔触 ── */
.title-text {
    margin-top: 32px;
    z-index: 2;
    display: flex;
    align-items: center;
}

.calligraphy-text {
    opacity: 0;
    background: linear-gradient(90deg, transparent 0%, theme('colors.ms-bone') 50%, transparent 100%);
    background-size: 200% 100%;
    background-position: 100% 0;
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    animation: calligraphy-wipe 0.9s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

.calligraphy-delay {
    animation-delay: 0.15s;
}

.title-visible .calligraphy-text {
    animation-play-state: running;
}

@keyframes calligraphy-wipe {
    0% {
        background-position: 100% 0;
        opacity: 0;
    }

    100% {
        background-position: 0 0;
        opacity: 1;
    }
}

/* ── SKIP 按钮 ── */
.skip-btn {
    position: absolute;
    bottom: 24px;
    right: 24px;
    padding: 8px 16px;
    font-size: 12px;
    font-weight: 500;
    letter-spacing: 0.1em;
    color: theme('colors.ms-ash');
    background: transparent;
    border: 1px solid transparent;
    cursor: pointer;
    transition: all 0.2s ease;
    z-index: 10;
}

.skip-btn:hover {
    color: theme('colors.ms-bone');
    border-color: theme('colors.ms-copper');
}

/* ── 整体淡出 ── */
.entrance-fade-leave-active {
    transition: opacity 0.6s ease;
}

.entrance-fade-leave-to {
    opacity: 0;
}
</style>
