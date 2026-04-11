<script setup lang="ts">
/**
 * SkeletonBlock — Versatile loading placeholder
 *
 * Replaces SkeletonLine with multi-variant support.
 * Uses theme-aware shimmer animation.
 *
 * @example
 * <SkeletonBlock variant="text" :lines="3" />
 * <SkeletonBlock variant="circle" :width="40" :height="40" />
 * <SkeletonBlock variant="code" />
 * <SkeletonBlock variant="rect" width="100%" :height="120" />
 */

withDefaults(defineProps<{
  variant?: 'text' | 'circle' | 'rect' | 'code'
  width?: string
  height?: string
  lines?: number
}>(), {
  variant: 'text',
  width: '100%',
  height: '12px',
  lines: 1,
})
</script>

<template>
  <div class="skeleton-block" :class="`skeleton-block--${variant}`">
    <template v-if="variant === 'text'">
      <div
        v-for="i in lines"
        :key="i"
        class="skeleton-shimmer skeleton-line"
        :style="{
          width: i === lines ? '60%' : '100%',
          height: '12px',
          marginBottom: i < lines ? '8px' : '0',
        }"
      />
    </template>

    <div
      v-else-if="variant === 'circle'"
      class="skeleton-shimmer"
      :style="{ width, height, borderRadius: '50%' }"
    />

    <div
      v-else-if="variant === 'rect'"
      class="skeleton-shimmer"
      :style="{ width, height, borderRadius: '3px' }"
    />

    <template v-else-if="variant === 'code'">
      <div
        v-for="i in 4"
        :key="i"
        class="skeleton-shimmer"
        :style="{
          width: i === 4 ? '45%' : `${70 + Math.random() * 25}%`,
          height: '14px',
          marginBottom: '6px',
          borderRadius: '2px',
        }"
      />
    </template>
  </div>
</template>

<style scoped>
.skeleton-block {
  width: 100%;
}

.skeleton-line {
  border-radius: 2px;
}

.skeleton-shimmer {
  background: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0.04) 25%,
    rgba(255, 255, 255, 0.08) 50%,
    rgba(255, 255, 255, 0.04) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s ease-in-out infinite;
}

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}
</style>
