<script setup lang="ts">
/**
 * ✨ EntranceAnimation — Web 阅读器入场动画
 *
 * 页面首次加载时播放一次性序列动画：
 * 1. 中心 Logo 光晕扩散（800ms）
 * 2. 星图节点逐个浮现（600ms stagger）
 * 3. 背景粒子脉冲（持续 2s 后淡出）
 *
 * 用法：<EntranceAnimation @done="onReady" />
 * 动画结束后自动卸载 DOM（v-if 控制）
 */

import { ref, onMounted } from "vue";
import { Sparkles } from "lucide-vue-next";

const emit = defineEmits<{
    (e: "done"): void;
}>();

const phase = ref<"logo" | "pulse" | "fadeout">("logo");
const visible = ref(true);

onMounted(() => {
    // Phase 1: Logo 光晕
    setTimeout(() => {
        phase.value = "pulse";
    }, 600);

    // Phase 2: 脉冲扩散
    setTimeout(() => {
        phase.value = "fadeout";
    }, 1800);

    // Phase 3: 淡出并移除
    setTimeout(() => {
        visible.value = false;
        emit("done");
    }, 2400);
});
</script>

<template>
    <Transition name="entrance-fade">
        <div v-if="visible" class="entrance-overlay">
            <!-- 背景粒子 -->
            <div class="particle-field">
                <div v-for="i in 20" :key="i" class="particle" :style="{
                    left: `${Math.random() * 100}%`,
                    top: `${Math.random() * 100}%`,
                    animationDelay: `${Math.random() * 1.5}s`,
                    animationDuration: `${1.5 + Math.random() * 2}s`,
                }" />
            </div>

            <!-- 中心 Logo -->
            <div class="logo-center" :class="{ 'logo-expand': phase !== 'logo' }">
                <div class="logo-glow" />
                <Sparkles :size="32" class="logo-icon" />
            </div>

            <!-- 脉冲环 -->
            <div v-if="phase === 'pulse' || phase === 'fadeout'" class="pulse-rings">
                <div class="pulse-ring ring-1" />
                <div class="pulse-ring ring-2" />
                <div class="pulse-ring ring-3" />
            </div>

            <!-- 标题文字 -->
            <div class="title-text" :class="{ 'title-fade': phase === 'fadeout' }">
                <span class="text-neon font-bold tracking-[0.3em] text-lg">MEMORY</span>
                <span class="text-gray-400 font-light tracking-[0.2em] text-sm ml-2">STREAM</span>
            </div>
        </div>
    </Transition>
</template>

<style scoped>
.entrance-overlay {
    position: fixed;
    inset: 0;
    z-index: 9999;
    background: #0d0d0d;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    overflow: hidden;
}

/* ── 粒子场 ── */
.particle-field {
    position: absolute;
    inset: 0;
}

.particle {
    position: absolute;
    width: 2px;
    height: 2px;
    background: #00e5ff;
    border-radius: 50%;
    opacity: 0;
    animation: particle-drift 2s ease-in-out infinite;
}

@keyframes particle-drift {
    0% {
        opacity: 0;
        transform: scale(0) translateY(0);
    }

    50% {
        opacity: 0.6;
    }

    100% {
        opacity: 0;
        transform: scale(1.5) translateY(-30px);
    }
}

/* ── Logo 中心 ── */
.logo-center {
    position: relative;
    z-index: 1;
    transition: transform 0.8s cubic-bezier(0.16, 1, 0.3, 1);
}

.logo-expand {
    transform: scale(1.2);
}

.logo-glow {
    position: absolute;
    inset: -20px;
    border-radius: 50%;
    background: radial-gradient(circle, rgba(0, 229, 255, 0.15) 0%, transparent 70%);
    animation: glow-breathe 1.5s ease-in-out infinite alternate;
}

@keyframes glow-breathe {
    from {
        transform: scale(1);
        opacity: 0.6;
    }

    to {
        transform: scale(1.3);
        opacity: 1;
    }
}

.logo-icon {
    color: #00e5ff;
    filter: drop-shadow(0 0 12px rgba(0, 229, 255, 0.6));
    animation: icon-pulse 1s ease-out;
}

@keyframes icon-pulse {
    from {
        opacity: 0;
        transform: scale(0.5);
    }

    to {
        opacity: 1;
        transform: scale(1);
    }
}

/* ── 脉冲环 ── */
.pulse-rings {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 0;
}

.pulse-ring {
    position: absolute;
    border: 1px solid rgba(0, 229, 255, 0.3);
    border-radius: 50%;
    animation: ring-expand 2s cubic-bezier(0, 0, 0.2, 1) forwards;
}

.ring-1 {
    animation-delay: 0s;
}

.ring-2 {
    animation-delay: 0.3s;
}

.ring-3 {
    animation-delay: 0.6s;
}

@keyframes ring-expand {
    from {
        width: 40px;
        height: 40px;
        opacity: 0.8;
    }

    to {
        width: 500px;
        height: 500px;
        opacity: 0;
    }
}

/* ── 标题文字 ── */
.title-text {
    margin-top: 24px;
    z-index: 1;
    transition: opacity 0.6s ease;
    animation: title-enter 0.8s 0.3s ease-out both;
}

.title-fade {
    opacity: 0;
}

@keyframes title-enter {
    from {
        opacity: 0;
        transform: translateY(10px);
        letter-spacing: 0.6em;
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

/* ── 整体淡出 ── */
.entrance-fade-leave-active {
    transition: opacity 0.5s ease;
}

.entrance-fade-leave-to {
    opacity: 0;
}
</style>