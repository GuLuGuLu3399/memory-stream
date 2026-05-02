// 单条通知，HUD 战术铭牌风格 — 4px 左色条 + ease-snap 动画
<script setup lang="ts">
import { ref } from 'vue'
import type { Toast, ToastType } from '@/composables/core/useToast'

const props = defineProps<{
  toast: Toast
}>()

const emit = defineEmits<{
  dismiss: [id: number]
}>()

const isLeaving = ref(false)

const colorMap: Record<ToastType, string> = {
  success: 'var(--neon)',
  warning: 'var(--brass)',
  error: 'var(--destructive)',
}

function handleClose() {
  isLeaving.value = true
  setTimeout(() => emit('dismiss', props.toast.id), 200)
}
</script>

<template>
  <Transition name="toast">
    <div v-if="!isLeaving" class="toast-item" @click="handleClose">
      <div class="toast-accent" :style="{ background: colorMap[toast.type] }" />
      <span class="toast-message">{{ toast.message }}</span>
    </div>
  </Transition>
</template>

<style scoped>
.toast-item {
  display: flex;
  align-items: center;
  gap: 0;
  padding: 0;
  background: var(--ms-panel);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  box-shadow: 0 4px 16px oklch(0 0 0 / 0.4);
  overflow: hidden;
  min-width: 240px;
  max-width: 360px;
  cursor: pointer;
}

.toast-accent {
  width: 4px;
  align-self: stretch;
  flex-shrink: 0;
}

.toast-message {
  flex: 1;
  padding: 10px 14px;
  font-family: var(--font-mono);
  font-size: 12px;
  letter-spacing: 0.02em;
  color: var(--text-primary);
}

/* Transition */
.toast-enter-active {
  transition: opacity var(--duration-fast) var(--ease-snap),
    transform var(--duration-fast) var(--ease-snap);
}

.toast-leave-active {
  transition: opacity 150ms var(--ease-hydraulic),
    transform 150ms var(--ease-hydraulic);
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(24px) scaleY(0.9);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(16px);
}
</style>
