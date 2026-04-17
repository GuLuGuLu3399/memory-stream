<script setup lang="ts">
/**
 * ChamberHeader — 祭坛舱室统一标题栏
 *
 * 所有舱室（分类档案库、概念坍缩、系统配置）共享此头部。
 * 结构：◆ ENGLISH TITLE — 中文副标题 | [close]
 */
defineProps<{
  title: string
  subtitle: string
  /** Accent color for the diamond marker, default 'text-neon' */
  accent?: string
}>()

defineEmits<{
  (e: 'close'): void
}>()
</script>

<template>
  <div class="chamber-header">
    <div class="flex items-center gap-3">
      <span class="text-sm select-none" :class="accent || 'text-neon'">◆</span>
      <span class="chamber-header__title">{{ title }}</span>
      <span class="chamber-header__subtitle">{{ subtitle }}</span>
    </div>
    <button
      @click="$emit('close')"
      class="chamber-header__close"
      title="关闭 (ESC)"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.chamber-header {
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  border-bottom: 1px solid theme('colors.ms-border');
  background: theme('colors.ms-carbon');
  flex-shrink: 0;
}

.chamber-header__title {
  font-weight: bold;
  color: theme('colors.slate.300');
  font-size: 12px;
  font-family: ui-monospace, monospace;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.chamber-header__subtitle {
  font-size: 10px;
  color: theme('colors.slate.600');
  font-family: ui-monospace, monospace;
}

.chamber-header__close {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: theme('colors.slate.500');
  transition: color 150ms ease, background-color 150ms ease;
}

.chamber-header__close:hover {
  color: theme('colors.slate.300');
  background: theme('colors.ms-panel');
}
</style>
