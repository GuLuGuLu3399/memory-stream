// 命令面板中的选项条目，支持图标、快捷键和激活态高亮
<script setup lang="ts">
defineProps<{
  label: string
  type?: 'command' | 'card'
  shortcut?: string
  active: boolean
}>()

defineEmits<{
  select: []
}>()
</script>

<template>
  <button class="palette-item" :class="{ active }" @click="$emit('select')">
    <span class="palette-item-icon">
      <slot name="icon" />
    </span>
    <span class="palette-item-label">{{ label }}</span>
    <span v-if="shortcut" class="palette-item-shortcut">{{ shortcut }}</span>
    <span class="palette-item-type">{{ type === 'card' ? '卡片' : '' }}</span>
  </button>
</template>

<style scoped>
.palette-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
  position: relative;
  transition: background var(--duration-fast) var(--ease-hydraulic);
}

.palette-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 4px;
  bottom: 4px;
  width: 2px;
  background: transparent;
  border-radius: 0;
}

.palette-item:hover {
  background: var(--ms-carbon);
}

.palette-item.active {
  background: var(--ms-surface);
}

.palette-item.active::before {
  background: var(--neon);
}

.palette-item-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  color: var(--text-muted);
}

.palette-item.active .palette-item-icon {
  color: var(--neon);
}

.palette-item-label {
  flex: 1;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.palette-item.active .palette-item-label {
  color: var(--text-primary);
}

.palette-item-shortcut {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  padding: 1px 5px;
  border: 1px solid var(--ms-border);
  border-bottom: 2px solid var(--ms-smoke);
  border-radius: 0;
}

.palette-item-type {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  flex-shrink: 0;
}
</style>
