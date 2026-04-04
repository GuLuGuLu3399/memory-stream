<script setup lang="ts">
/**
 * 🌟 SkeletonLine — 深色微光骨架屏原子组件
 *
 * 使用 shimmer 动画模拟加载态。
 * 配合 v-for 渲染出不同形状的骨架占位。
 */

defineProps<{
    /** 宽度，支持 CSS 值如 "60%"、"200px" */
    width?: string;
    /** 高度，支持 CSS 值如 "14px"、"1rem" */
    height?: string;
    /** 是否为圆形（用于头像占位） */
    rounded?: boolean;
}>();
</script>

<template>
    <div class="skeleton-line" :class="{ 'skeleton-rounded': rounded }"
        :style="{ width: width || '100%', height: height || '14px' }" />
</template>

<style scoped>
.skeleton-line {
    background: linear-gradient(90deg, #222 25%, #2a2a2a 50%, #222 75%);
    background-size: 200% 100%;
    animation: shimmer 1.5s ease-in-out infinite;
    border-radius: 4px;
}

.skeleton-rounded {
    border-radius: 50%;
}

@keyframes shimmer {
    0% {
        background-position: 200% 0;
    }

    100% {
        background-position: -200% 0;
    }
}
</style>