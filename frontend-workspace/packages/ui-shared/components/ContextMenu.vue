<script setup lang="ts">
/**
 * ContextMenu — Shared right-click context menu
 *
 * Keyboard-navigable, click-outside-dismissed context menu.
 * Positioned at click coordinates with viewport clamping.
 *
 * @example
 * <ContextMenu
 *   :visible="menuVisible"
 *   :position="{ x: 100, y: 200 }"
 *   :items="[
 *     { id: 'focus', label: 'Focus', icon: FocusIcon },
 *     { id: 'delete', label: 'Delete', danger: true },
 *   ]"
 *   @select="onSelect"
 *   @close="menuVisible = false"
 * />
 */

import { ref, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { LAYER_Z_INDEX } from '../styles/layers'

export interface ContextMenuItem {
  id: string
  label: string
  icon?: any
  danger?: boolean
  disabled?: boolean
  separator?: boolean
}

const props = defineProps<{
  visible: boolean
  position: { x: number; y: number }
  items: ContextMenuItem[]
}>()

const emit = defineEmits<{
  select: [id: string]
  close: []
}>()

const menuRef = ref<HTMLElement | null>(null)
const activeIndex = ref(0)
const adjustedPos = ref({ x: 0, y: 0 })

watch(() => props.visible, async (v) => {
  if (v) {
    activeIndex.value = 0
    await nextTick()
    clampPosition()
  }
})

function clampPosition() {
  if (!menuRef.value) return
  const menu = menuRef.value.getBoundingClientRect()
  const vw = window.innerWidth
  const vh = window.innerHeight
  adjustedPos.value = {
    x: props.position.x + menu.width > vw ? Math.max(8, props.position.x - menu.width) : props.position.x,
    y: props.position.y + menu.height > vh ? Math.max(8, props.position.y - menu.height) : props.position.y,
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (!props.visible) return
  const actionable = props.items.filter(i => !i.separator && !i.disabled)

  if (e.key === 'ArrowDown') {
    e.preventDefault()
    const idx = actionable.findIndex(i => i.id === props.items[activeIndex.value]?.id)
    activeIndex.value = props.items.indexOf(actionable[(idx + 1) % actionable.length])
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    const idx = actionable.findIndex(i => i.id === props.items[activeIndex.value]?.id)
    activeIndex.value = props.items.indexOf(actionable[(idx - 1 + actionable.length) % actionable.length])
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const item = props.items[activeIndex.value]
    if (item && !item.disabled && !item.separator) emit('select', item.id)
  } else if (e.key === 'Escape') {
    emit('close')
  }
}

function handleClickOutside(e: MouseEvent) {
  if (props.visible && menuRef.value && !menuRef.value.contains(e.target as Node)) {
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
  <Teleport to="body">
    <Transition name="ctx-fade">
      <div v-if="visible" ref="menuRef" class="ctx-menu"
        :style="{ left: `${adjustedPos.x}px`, top: `${adjustedPos.y}px`, zIndex: String(LAYER_Z_INDEX.dropdown) }"
        role="menu">
        <template v-for="(item, i) in items" :key="item.id">
          <div v-if="item.separator" class="ctx-menu__separator" />
          <button v-else class="ctx-menu__item" :class="{
            'ctx-menu__item--active': i === activeIndex,
            'ctx-menu__item--danger': item.danger,
            'ctx-menu__item--disabled': item.disabled,
          }" role="menuitem" :disabled="item.disabled" @click="!item.disabled && emit('select', item.id)"
            @mouseenter="activeIndex = i">
            <component :is="item.icon" v-if="item.icon" class="ctx-menu__icon" />
            <span>{{ item.label }}</span>
          </button>
        </template>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.ctx-menu {
  position: fixed;
  min-width: 160px;
  padding: 4px;
  background: #12100c;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 3px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5), 0 2px 6px rgba(0, 0, 0, 0.3);
}

.ctx-menu__item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 12px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.7);
  background: none;
  border: none;
  border-radius: 2px;
  cursor: pointer;
  text-align: left;
  transition: all 100ms ease;
}

.ctx-menu__item--active {
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.9);
}

.ctx-menu__item--danger {
  color: #d44040;
}

.ctx-menu__item--danger.ctx-menu__item--active {
  background: rgba(212, 64, 64, 0.12);
}

.ctx-menu__item--disabled {
  opacity: 0.35;
  cursor: default;
}

.ctx-menu__icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

.ctx-menu__separator {
  height: 1px;
  margin: 4px 8px;
  background: rgba(255, 255, 255, 0.06);
}

.ctx-fade-enter-active {
  transition: opacity 100ms ease;
}

.ctx-fade-leave-active {
  transition: opacity 80ms ease;
}

.ctx-fade-enter-from,
.ctx-fade-leave-to {
  opacity: 0;
}
</style>
