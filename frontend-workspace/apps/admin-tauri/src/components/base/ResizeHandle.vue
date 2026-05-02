// 可拖拽的宽度调节条，支持设置最小/最大宽度限制
<script setup lang="ts">
import { ref, onBeforeUnmount } from 'vue'

const props = defineProps<{
  side: 'left' | 'right'
  minWidth?: number
  maxWidth?: number
  modelValue: number
}>()

const emit = defineEmits<{
  'update:modelValue': [width: number]
}>()

const isDragging = ref(false)
const isHovered = ref(false)

let activeMouseMove: ((e: MouseEvent) => void) | null = null
let activeMouseUp: (() => void) | null = null

function cleanup() {
  if (activeMouseMove) document.removeEventListener('mousemove', activeMouseMove)
  if (activeMouseUp) document.removeEventListener('mouseup', activeMouseUp)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  activeMouseMove = null
  activeMouseUp = null
}

function onMouseDown(e: MouseEvent) {
  e.preventDefault()
  isDragging.value = true

  const startX = e.clientX
  const startWidth = props.modelValue

  function onMouseMove(e: MouseEvent) {
    const delta = props.side === 'left'
      ? e.clientX - startX
      : startX - e.clientX
    const next = Math.min(
      props.maxWidth ?? 400,
      Math.max(props.minWidth ?? 160, startWidth + delta)
    )
    emit('update:modelValue', next)
  }

  function onMouseUp() {
    isDragging.value = false
    cleanup()
  }

  activeMouseMove = onMouseMove
  activeMouseUp = onMouseUp
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}

onBeforeUnmount(() => {
  if (isDragging.value) cleanup()
})
</script>

<template>
  <div class="resize-handle" :class="{ active: isDragging }" @mousedown="onMouseDown" @mouseenter="isHovered = true"
    @mouseleave="isHovered = false" />
</template>

<style scoped>
.resize-handle {
  width: 2px;
  flex-shrink: 0;
  background: var(--ms-border);
  cursor: col-resize;
  transition: background var(--duration-fast) var(--ease-hydraulic),
    box-shadow var(--duration-fast) var(--ease-hydraulic);
}

.resize-handle:hover,
.resize-handle.active {
  background: var(--neon);
  box-shadow: 0 0 8px oklch(0.78 0.17 200 / 0.3);
}
</style>
