<script setup lang="ts">
// 用途：重命名对话框，支持输入新名称并自动选中
import { ref, onMounted, onUnmounted, nextTick } from 'vue'

const props = defineProps<{
  title: string
  value: string
}>()

const emit = defineEmits<{
  confirm: [value: string]
  cancel: []
}>()

const draft = ref(props.value)
const inputRef = ref<HTMLInputElement | null>(null)

onMounted(async () => {
  await nextTick()
  inputRef.value?.focus()
  inputRef.value?.select()
})

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault()
    commit()
  } else if (e.key === 'Escape') {
    emit('cancel')
  }
}

function commit() {
  const v = draft.value.trim()
  if (v) emit('confirm', v)
  else emit('cancel')
}

onMounted(() => window.addEventListener('keydown', handleKeydown))
onUnmounted(() => window.removeEventListener('keydown', handleKeydown))
</script>

<template>
  <Teleport to="body">
    <div class="rename-overlay" @click.self="emit('cancel')">
      <div class="rename-dialog">
        <h3 class="rename-title">{{ title }}</h3>
        <input
          ref="inputRef"
          v-model="draft"
          type="text"
          class="rename-input"
          @keydown="handleKeydown"
        />
        <div class="rename-actions">
          <button class="rename-btn cancel" @click="emit('cancel')">取消</button>
          <button class="rename-btn ok" @click="commit">确认</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.rename-overlay {
  position: fixed;
  inset: 0;
  background: oklch(0 0 0 / 0.65);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
}

.rename-dialog {
  width: 360px;
  padding: 24px;
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

.rename-title {
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 14px;
}

.rename-input {
  width: 100%;
  padding: 8px 12px;
  background: var(--ms-carbon);
  border: 1px solid transparent;
  border-radius: 0;
  box-shadow: var(--shadow-inset);
  color: var(--text-primary);
  caret-color: var(--neon);
  font-family: var(--font-sans);
  font-size: 14px;
  outline: none;
  transition: border-color var(--duration-fast) var(--ease-hydraulic),
    box-shadow var(--duration-fast) var(--ease-hydraulic);
}

.rename-input:focus {
  border-color: var(--neon);
  box-shadow: var(--shadow-inset), 0 0 0 1px color-mix(in oklch, var(--neon) 40%, transparent);
}

.rename-input::selection {
  background: oklch(0.78 0.17 200 / 0.25);
}

.rename-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 18px;
}

.rename-btn {
  padding: 6px 16px;
  border-radius: 2px;
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-snap);
}

.rename-btn.cancel {
  border: 1px solid var(--ms-border-light);
  background: transparent;
  color: var(--ms-smoke);
}

.rename-btn.cancel:hover {
  border-color: var(--ms-smoke);
  color: var(--text-secondary);
}

.rename-btn.ok {
  border: 1px solid var(--neon);
  background: color-mix(in oklch, var(--neon) 14%, transparent);
  color: var(--neon);
  box-shadow: inset 0 1px 0 oklch(1 0 0 / 0.08);
}

.rename-btn.ok:hover {
  background: color-mix(in oklch, var(--neon) 22%, transparent);
}
</style>
