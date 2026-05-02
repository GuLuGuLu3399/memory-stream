// 用途：卡片右键菜单，提供编辑、移动分类和删除操作
<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { Pencil, FolderInput, Trash2 } from 'lucide-vue-next'

const props = defineProps<{
  x: number
  y: number
  isCategory?: boolean
}>()

const emit = defineEmits<{
  close: []
  action: [id: string]
}>()

const menuStyle = computed(() => {
  const w = 172
  const h = 120
  const vw = window.innerWidth
  const vh = window.innerHeight
  let x = props.x
  let y = props.y
  if (x + w > vw - 8) x = vw - w - 8
  if (y + h > vh - 8) y = vh - h - 8
  return { left: `${x}px`, top: `${y}px` }
})

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

onMounted(() => window.addEventListener('keydown', handleKeydown))
onUnmounted(() => window.removeEventListener('keydown', handleKeydown))
</script>

<template>
  <Teleport to="body">
    <div class="ctx-backdrop" @click="emit('close')" @contextmenu.prevent="emit('close')">
      <div class="ctx-menu" :style="menuStyle" @click.stop>
        <button class="ctx-item" @click="emit('action', 'rename')">
          <Pencil :size="13" :stroke-width="1.5" />
          <span>重命名</span>
        </button>
        <button v-if="!isCategory" class="ctx-item" @click="emit('action', 'move')">
          <FolderInput :size="13" :stroke-width="1.5" />
          <span>移动到分类</span>
        </button>
        <div class="ctx-separator" />
        <button class="ctx-item ctx-danger" @click="emit('action', 'delete')">
          <Trash2 :size="13" :stroke-width="1.5" />
          <span>{{ isCategory ? '删除分类' : '删除' }}</span>
        </button>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.ctx-backdrop {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal);
}

.ctx-menu {
  position: fixed;
  min-width: 160px;
  background: var(--ms-panel);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  box-shadow: var(--shadow-raised);
  padding: 4px 0;
  animation: ctx-emerge 120ms var(--ease-emerge);
}

@keyframes ctx-emerge {
  from {
    opacity: 0;
    transform: scale(0.96);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.ctx-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  height: 28px;
  padding: 0 12px;
  border: none;
  background: transparent;
  color: var(--ms-smoke);
  font-family: var(--font-sans);
  font-size: 12px;
  cursor: pointer;
  text-align: left;
  transition: background var(--duration-fast) var(--ease-hydraulic),
    color var(--duration-fast) var(--ease-hydraulic);
}

.ctx-item:hover {
  background: var(--ms-surface);
  color: var(--text-primary);
}

.ctx-item svg {
  flex-shrink: 0;
}

.ctx-danger {
  color: var(--brass);
}

.ctx-danger:hover {
  background: color-mix(in oklch, var(--brass) 10%, transparent);
  color: var(--brass);
}

.ctx-separator {
  height: 1px;
  margin: 4px 8px;
  background: var(--ms-border);
}
</style>
