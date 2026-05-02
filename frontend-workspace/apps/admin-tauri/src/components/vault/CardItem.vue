// 卡片列表项，显示标题、激活态高亮和同步状态指示点

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  uuid: string
  title: string
  isActive: boolean
  syncStatus?: string
}>()

const emit = defineEmits<{
  click: []
  contextmenu: [event: MouseEvent]
}>()

const syncDotColor = computed(() => {
  switch (props.syncStatus) {
    case 'PendingPush': return 'var(--brass)'
    case 'Conflict': return 'var(--destructive)'
    case 'PendingDelete': return 'var(--text-muted)'
    default: return 'transparent'
  }
})
</script>

<template>
  <button class="card-item" :class="{ active: isActive }" @click="emit('click')"
    @contextmenu.prevent="emit('contextmenu', $event)">
    <span class="card-item-title">{{ title }}</span>
    <span v-if="syncStatus && syncStatus !== 'Synced'" class="card-item-dot" :style="{ background: syncDotColor }" />
  </button>
</template>

<style scoped>
.card-item {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 5px 12px 5px 16px;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
  position: relative;
  transition: background var(--duration-fast) var(--ease-hydraulic);
}

.card-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 4px;
  bottom: 4px;
  width: 2px;
  background: transparent;
  border-radius: 1px;
  transition: background var(--duration-fast) var(--ease-hydraulic);
}

.card-item:hover {
  background: var(--ms-carbon);
}

.card-item:hover::before {
  background: var(--neon-dim);
}

.card-item.active {
  background: var(--ms-surface);
}

.card-item.active::before {
  background: var(--neon);
}

.card-item-title {
  flex: 1;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: color var(--duration-fast) var(--ease-hydraulic);
}

.card-item:hover .card-item-title {
  color: var(--text-primary);
}

.card-item.active .card-item-title {
  color: var(--neon);
}

.card-item-dot {
  flex-shrink: 0;
  width: 5px;
  height: 5px;
  border-radius: 50%;
  margin-left: 6px;
}
</style>
