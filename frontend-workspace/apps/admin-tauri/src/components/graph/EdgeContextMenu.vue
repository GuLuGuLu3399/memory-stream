<script setup lang="ts">
// 用途：边右键菜单，提供反转方向和断开连接操作
import { onMounted, onUnmounted } from 'vue'

defineProps<{
  x: number
  y: number
}>()

const emit = defineEmits<{
  close: []
  reverse: []
  disconnect: []
}>()

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

function handleClickOutside(e: MouseEvent) {
  if (!(e.target as HTMLElement).closest('.edge-context-menu')) {
    emit('close')
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  document.addEventListener('click', handleClickOutside, true)
})
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('click', handleClickOutside, true)
})
</script>

<template>
  <div class="edge-context-menu" :style="{ left: `${x}px`, top: `${y}px` }">
    <button class="edge-menu-item" @click="emit('reverse')">
      <span class="edge-menu-icon">&#x21C4;</span>
      <span>反转方向</span>
    </button>
    <button class="edge-menu-item" @click="emit('disconnect')">
      <span class="edge-menu-icon">&#x2702;</span>
      <span>断开连接</span>
    </button>
  </div>
</template>

<style scoped>
.edge-context-menu {
  position: fixed;
  min-width: 150px;
  background: color-mix(in oklch, var(--ms-void) 95%, transparent);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  backdrop-filter: blur(8px);
  z-index: 100;
  animation: edge-menu-in var(--duration-fast) var(--ease-emerge);
}

@keyframes edge-menu-in {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.edge-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 12px;
  border: none;
  background: transparent;
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--text-secondary);
  cursor: pointer;
  text-align: left;
  transition: background var(--duration-fast) var(--ease-hydraulic),
    color var(--duration-fast) var(--ease-hydraulic);
}

.edge-menu-item:hover {
  background: var(--ms-surface);
  color: var(--text-primary);
}

.edge-menu-item:first-child {
  border-radius: 2px 2px 0 0;
}

.edge-menu-item:last-child {
  border-radius: 0 0 2px 2px;
  border-top: 1px solid var(--ms-border);
}

.edge-menu-icon {
  width: 16px;
  text-align: center;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
}
</style>
