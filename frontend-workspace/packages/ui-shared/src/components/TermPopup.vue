// ────────────────────────────────────────────────────────────────
// TermPopup.vue — floating definition popup for ConceptRef terms
// ────────────────────────────────────────────────────────────────

<script setup lang="ts">
import { onMounted, onBeforeUnmount } from 'vue'

defineProps<{
  term: string
  definition: string
  x: number
  y: number
}>()

const emit = defineEmits<{ (e: 'close'): void }>()

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

function onClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.ms-term-popup')) emit('close')
}

onMounted(() => {
  document.addEventListener('keydown', onKey)
  document.addEventListener('click', onClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKey)
  document.removeEventListener('click', onClickOutside)
})
</script>

<template>
  <div class="ms-term-popup" :style="{ left: `${x + 12}px`, top: `${y + 12}px` }">
    <div class="ms-term-popup__header">{{ term }}</div>
    <div class="ms-term-popup__body">{{ definition }}</div>
  </div>
</template>

<style scoped>
.ms-term-popup {
  position: fixed;
  z-index: 500;
  max-width: 320px;
  max-height: 200px;
  overflow-y: auto;
  background: var(--ms-void, #080808);
  border: 1px solid var(--ms-border, #1e1e1e);
  border-radius: 3px;
  box-shadow: 0 4px 16px oklch(0 0 0 / 0.5);
  animation: term-popup-in 150ms cubic-bezier(0.33, 0, 0.2, 1) both;
  pointer-events: auto;
}

@keyframes term-popup-in {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

.ms-term-popup__header {
  font-weight: 600;
  color: #00e5ff;
  padding: 0.5em 0.75em 0.25em;
  font-size: 0.9em;
  border-bottom: 1px solid var(--ms-border-light, #1a1a1a);
}

.ms-term-popup__body {
  padding: 0.5em 0.75em 0.75em;
  font-size: 0.85em;
  color: #c8bfa8;
  line-height: 1.6;
}
</style>
