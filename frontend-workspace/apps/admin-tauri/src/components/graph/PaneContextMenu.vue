<script setup lang="ts">
// 用途：图谱面板右键菜单，支持新建卡片、召唤连接和断开所有连接
import { onMounted, onUnmounted } from 'vue'

withDefaults(defineProps<{
  x: number
  y: number
  mode?: 'local' | 'global'
}>(), {
  mode: 'local',
})

const emit = defineEmits<{
  close: []
  addNode: []
  summonLink: []
  detachAll: []
  refresh: []
}>()

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

function handleClickOutside(e: MouseEvent) {
  if (!(e.target as HTMLElement).closest('.pane-context-menu')) {
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
  <div class="pane-context-menu" :style="{ left: `${x}px`, top: `${y}px` }">
    <button class="pane-menu-item" @click="emit('addNode')">
      <span class="pane-menu-icon">+</span>
      <span>{{ mode === 'local' ? '原地新建' : '独立新建卡片' }}</span>
    </button>
    <button v-if="mode === 'local'" class="pane-menu-item" @click="emit('summonLink')">
      <span class="pane-menu-icon">&circlearrowright;</span>
      <span>召唤连接</span>
    </button>
    <button v-if="mode === 'local'" class="pane-menu-item" @click="emit('detachAll')">
      <span class="pane-menu-icon">&times;</span>
      <span>断开所有连接</span>
    </button>
    <button v-if="mode === 'global'" class="pane-menu-item" @click="emit('refresh')">
      <span class="pane-menu-icon">&#x27F2;</span>
      <span>强制刷新</span>
    </button>
  </div>
</template>

<style scoped>
.pane-context-menu {
  position: fixed;
  min-width: 170px;
  background: color-mix(in oklch, var(--ms-void) 95%, transparent);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  backdrop-filter: blur(8px);
  z-index: 100;
  animation: pane-menu-in var(--duration-fast) var(--ease-emerge);
}

@keyframes pane-menu-in {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.pane-menu-item {
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

.pane-menu-item:hover {
  background: var(--ms-surface);
  color: var(--text-primary);
}

.pane-menu-item:first-child {
  border-radius: 2px 2px 0 0;
}

.pane-menu-item:last-child {
  border-radius: 0 0 2px 2px;
}

.pane-menu-item + .pane-menu-item {
  border-top: 1px solid var(--ms-border);
}

.pane-menu-icon {
  width: 16px;
  text-align: center;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
}
</style>
