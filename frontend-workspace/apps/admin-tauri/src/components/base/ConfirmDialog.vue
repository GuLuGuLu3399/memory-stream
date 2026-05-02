<script setup lang="ts">
// 用途：确认对话框，支持确认/取消操作和危险警告样式
import { onMounted, onUnmounted } from 'vue'

defineProps<{
  title: string
  message: string
  confirmLabel?: string
  destructive?: boolean
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('cancel')
}

onMounted(() => window.addEventListener('keydown', handleKeydown))
onUnmounted(() => window.removeEventListener('keydown', handleKeydown))
</script>

<template>
  <Teleport to="body">
    <div class="confirm-overlay" @click.self="emit('cancel')">
      <div class="confirm-dialog">
        <div class="confirm-header">
          <svg v-if="destructive" class="confirm-hazard" width="20" height="20" viewBox="0 0 20 20" fill="none">
            <path d="M10 3L18 17H2L10 3Z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round" />
            <line x1="10" y1="8.5" x2="10" y2="12" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
            <circle cx="10" cy="14.5" r="0.7" fill="currentColor" />
          </svg>
          <h3 class="confirm-title">{{ title }}</h3>
        </div>
        <p class="confirm-message">{{ message }}</p>
        <div class="confirm-actions">
          <button class="confirm-btn cancel" @click="emit('cancel')">取消</button>
          <button class="confirm-btn ok" :class="{ destructive }" @click="emit('confirm')">
            {{ confirmLabel || '确认' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.confirm-overlay {
  position: fixed;
  inset: 0;
  background: oklch(0 0 0 / 0.65);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
}

.confirm-dialog {
  width: 380px;
  padding: 28px;
  background: var(--ms-panel);
  border: 1px solid var(--ms-border-light);
  border-radius: 2px;
  box-shadow: var(--shadow-raised);
  animation: dialog-lock 200ms var(--ease-snap);
}

@keyframes dialog-lock {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.confirm-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.confirm-hazard {
  flex-shrink: 0;
  color: var(--brass);
}

.confirm-title {
  font-family: var(--font-sans);
  font-size: 15px;
  font-weight: 700;
  letter-spacing: 0.02em;
  color: var(--text-primary);
  margin: 0;
}

.confirm-message {
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--ms-smoke);
  margin: 0 0 24px;
  line-height: 1.5;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.confirm-btn {
  padding: 7px 18px;
  border-radius: 2px;
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.03em;
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-snap);
}

.confirm-btn.cancel {
  border: 1px solid var(--ms-border-light);
  background: transparent;
  color: var(--ms-smoke);
}

.confirm-btn.cancel:hover {
  border-color: var(--ms-smoke);
  color: var(--text-secondary);
  background: var(--ms-surface);
}

.confirm-btn.ok {
  border: 1px solid var(--neon);
  background: color-mix(in oklch, var(--neon) 14%, transparent);
  color: var(--neon);
  box-shadow: inset 0 1px 0 oklch(1 0 0 / 0.08);
}

.confirm-btn.ok:hover {
  background: color-mix(in oklch, var(--neon) 22%, transparent);
}

.confirm-btn.ok.destructive {
  border: 1px solid var(--brass);
  background: var(--brass);
  color: var(--ms-void);
  box-shadow:
    inset 0 1px 0 oklch(1 0 0 / 0.15),
    inset 0 -1px 0 oklch(0 0 0 / 0.25),
    0 2px 0 oklch(0.5 0.12 65);
}

.confirm-btn.ok.destructive:hover {
  background: color-mix(in oklch, var(--brass) 85%, oklch(1 0 0));
  box-shadow:
    inset 0 1px 0 oklch(1 0 0 / 0.2),
    inset 0 -1px 0 oklch(0 0 0 / 0.2),
    0 2px 0 oklch(0.5 0.12 65);
}
</style>
