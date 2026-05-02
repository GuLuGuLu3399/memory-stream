// 编辑器文本选区浮动工具栏，提供加粗、斜体、代码和链接格式化操作
<script setup lang="ts">
import { ref, inject, onMounted, onUnmounted } from 'vue'
import { Bold, Italic, Link, Code } from 'lucide-vue-next'
import { EditorViewKey } from './cm-injection'

const viewRef = inject(EditorViewKey)
const visible = ref(false)
const position = ref({ top: 0, left: 0 })

function wrapSelection(before: string, after: string) {
  const view = viewRef?.value
  if (!view) return

  const { from, to } = view.state.selection.main
  const selected = view.state.sliceDoc(from, to)
  const replacement = `${before}${selected || 'text'}${after}`

  view.dispatch({
    changes: { from, to, insert: replacement },
    selection: { anchor: from + before.length, head: from + before.length + (selected || 'text').length },
  })
  view.focus()
}

function onSelectionChange() {
  const view = viewRef?.value
  if (!view || !view.hasFocus) {
    visible.value = false
    return
  }

  const { from, to } = view.state.selection.main
  if (from === to) {
    visible.value = false
    return
  }

  const startCoords = view.coordsAtPos(from)
  const endCoords = view.coordsAtPos(to)
  if (!startCoords || !endCoords) {
    visible.value = false
    return
  }

  position.value = {
    top: Math.min(startCoords.top, endCoords.top) - 40,
    left: (startCoords.left + endCoords.left) / 2,
  }
  visible.value = true
}

onMounted(() => document.addEventListener('selectionchange', onSelectionChange))
onUnmounted(() => document.removeEventListener('selectionchange', onSelectionChange))

defineExpose({ visible })
</script>

<template>
  <Teleport to="body">
    <Transition name="float">
      <div v-if="visible" class="floating-toolbar" :style="{ top: `${position.top}px`, left: `${position.left}px` }">
        <button class="ft-btn" title="加粗 (Ctrl+B)" @click="wrapSelection('**', '**')">
          <Bold :size="14" :stroke-width="1.5" />
        </button>
        <button class="ft-btn" title="斜体 (Ctrl+I)" @click="wrapSelection('*', '*')">
          <Italic :size="14" :stroke-width="1.5" />
        </button>
        <button class="ft-btn" title="代码 (Ctrl+`)" @click="wrapSelection('`', '`')">
          <Code :size="14" :stroke-width="1.5" />
        </button>
        <button class="ft-btn" title="链接 (Ctrl+K)" @click="wrapSelection('[', '](url)')">
          <Link :size="14" :stroke-width="1.5" />
        </button>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.floating-toolbar {
  position: fixed;
  transform: translateX(-50%);
  display: flex;
  gap: 2px;
  padding: 4px 6px;
  background: var(--ms-panel);
  border: 1px solid var(--ms-border-light);
  border-radius: 2px;
  box-shadow: 0 2px 8px oklch(0 0 0 / 0.5);
  z-index: var(--z-float);
}

.ft-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 2px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background var(--duration-fast) var(--ease-hydraulic),
    color var(--duration-fast) var(--ease-hydraulic);
}

.ft-btn:hover {
  background: var(--ms-surface);
  color: var(--neon);
}

/* Float transition */
.float-enter-active {
  transition: opacity var(--duration-fast) var(--ease-emerge),
    transform var(--duration-fast) var(--ease-emerge);
}

.float-leave-active {
  transition: opacity 100ms var(--ease-hydraulic);
}

.float-enter-from {
  opacity: 0;
  transform: translateX(-50%) scale(0.9) translateY(4px);
}

.float-leave-to {
  opacity: 0;
}
</style>
