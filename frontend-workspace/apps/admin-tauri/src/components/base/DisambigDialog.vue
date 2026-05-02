// 用途：消歧对话框，当多个卡片匹配同一链接时供用户选择
<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'

export interface DisambigOption {
  uuid: string
  category: string
}

defineProps<{
  title: string
  options: DisambigOption[]
}>()

const emit = defineEmits<{
  select: [uuid: string]
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
          <h3 class="confirm-title">选择目标卡片</h3>
        </div>
        <p class="confirm-message">「{{ title }}」存在多个匹配，请选择要跳转的卡片</p>
        <div class="disambig-list">
          <button v-for="opt in options" :key="opt.uuid" class="disambig-item"
                  @click="emit('select', opt.uuid)">
            <span class="disambig-cat">{{ opt.category || '根目录' }}</span>
            <span class="disambig-sep">/</span>
            <span class="disambig-name">{{ title }}</span>
          </button>
        </div>
        <div class="confirm-actions">
          <button class="confirm-btn cancel" @click="emit('cancel')">取消</button>
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
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

.confirm-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
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
  margin: 0 0 16px;
  line-height: 1.5;
}

.disambig-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: 20px;
}

.disambig-item {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 8px 12px;
  border: 1px solid transparent;
  border-radius: 2px;
  background: transparent;
  color: var(--ms-smoke);
  font-family: var(--font-sans);
  font-size: 13px;
  cursor: pointer;
  text-align: left;
  transition: all var(--duration-fast) var(--ease-hydraulic);
}

.disambig-item:hover {
  background: var(--ms-surface);
  border-color: var(--ms-border-light);
  color: var(--text-primary);
}

.disambig-cat {
  color: var(--brass);
  font-size: 11px;
  font-weight: 600;
}

.disambig-sep {
  color: var(--text-muted);
  font-size: 11px;
}

.disambig-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
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
</style>
