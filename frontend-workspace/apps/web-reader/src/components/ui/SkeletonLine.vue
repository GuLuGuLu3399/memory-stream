<script setup lang="ts">
/**
 * SkeletonLine — 血肉神殿骨架屏原子组件
 *
 * 暗色 shimmer 动画模拟加载态。
 */

defineProps<{
    width?: string;
    height?: string;
    rounded?: boolean;
}>();
</script>

<template>
    <div class="skeleton-line" :class="{ 'skeleton-rounded': rounded }"
        :style="{ width: width || '100%', height: height || '14px' }" />
</template>

<style scoped>
.skeleton-line {
    background: linear-gradient(90deg, theme('colors.ms-mo') 25%, rgb(166, 38, 38, 0.08) 50%, theme('colors.ms-mo') 75%);
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
