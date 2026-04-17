<script setup lang="ts">
/**
 * FloatingPanel — Unified floating panel shell (Zero-UI)
 *
 * No close buttons. Close via:
 *   - Backdrop click
 *   - Esc key
 *
 * @example
 * <FloatingPanel position="right" :open="show" @close="show = false">
 *   <template #header>Title</template>
 *   Content here
 * </FloatingPanel>
 */

import { computed, onMounted, onUnmounted } from 'vue'
import { LAYER_Z_INDEX } from '../styles/layers'

const props = withDefaults(defineProps<{
  open: boolean
  position?: 'center' | 'right' | 'bottom' | 'left'
  backdrop?: boolean
  zIndex?: number | string
  width?: string
}>(), {
  position: 'center',
  backdrop: true,
  zIndex: LAYER_Z_INDEX.modal,
  width: '',
})

const emit = defineEmits<{
  close: []
}>()

const transitionName = computed(() => {
  switch (props.position) {
    case 'right': return 'ms-slide-right'
    case 'left': return 'ms-slide-left'
    case 'bottom': return 'ms-slide-up'
    default: return 'ms-scale'
  }
})

const baseZIndex = computed(() => Number(props.zIndex))

function close() {
  emit('close')
}

function onBackdropClick() {
  close()
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.open) {
    e.preventDefault()
    close()
  }
}

onMounted(() => {
  document.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <Teleport to="body">
    <!-- Backdrop -->
    <Transition name="ms-fade">
      <div v-if="open && backdrop" class="floating-panel__backdrop" :style="{ zIndex: String(baseZIndex) }"
        @click="onBackdropClick" />
    </Transition>

    <!-- Panel -->
    <Transition :name="transitionName" appear>
      <div v-if="open" class="floating-panel" :class="[`floating-panel--${position}`]"
        :style="{ zIndex: String(baseZIndex + 1), maxWidth: width || undefined }" role="dialog" aria-modal="true">
        <header v-if="$slots.header" class="floating-panel__header">
          <slot name="header" />
        </header>

        <div class="floating-panel__body">
          <slot />
        </div>

        <footer v-if="$slots.footer" class="floating-panel__footer">
          <slot name="footer" />
        </footer>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.floating-panel__backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(2px);
}

.floating-panel {
  position: fixed;
  background: #0a0806;
  border: 1px solid rgba(255, 255, 255, 0.06);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Position variants */
.floating-panel--center {
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  border-radius: 3px;
  min-width: 360px;
  max-height: 80vh;
}

.floating-panel--right {
  top: 0;
  right: 0;
  bottom: 0;
  width: 45%;
  min-width: 320px;
  max-width: 640px;
  border-left: 1px solid rgba(255, 255, 255, 0.06);
}

.floating-panel--left {
  top: 0;
  left: 0;
  bottom: 0;
  width: 320px;
  border-right: 1px solid rgba(255, 255, 255, 0.06);
}

.floating-panel--bottom {
  bottom: 0;
  left: 0;
  right: 0;
  max-height: 50vh;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 3px 3px 0 0;
}

.floating-panel__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  min-height: 52px;
}

.floating-panel__body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.floating-panel__footer {
  padding: 12px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

/* Slide left transition */
.ms-slide-left-enter-active {
  transition: transform 300ms cubic-bezier(0.16, 1, 0.3, 1);
}

.ms-slide-left-leave-active {
  transition: transform 250ms cubic-bezier(0.16, 1, 0.3, 1);
}

.ms-slide-left-enter-from,
.ms-slide-left-leave-to {
  transform: translateX(-100%);
}
</style>