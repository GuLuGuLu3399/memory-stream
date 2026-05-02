<script setup lang="ts">
// 用途：移动卡片到分类对话框，显示分类树并选择目标分类
import { computed, onMounted, onUnmounted } from 'vue'
import { useTreeStore } from '@/stores/tree'
import { FolderOpen } from 'lucide-vue-next'
import type { TreeNode } from '@memory-stream/core'

defineProps<{
  currentCategory: string
}>()

const emit = defineEmits<{
  confirm: [category: string]
  cancel: []
}>()

const treeStore = useTreeStore()

interface CategoryEntry {
  path: string
  name: string
  depth: number
}

function extractCategories(nodes: TreeNode[], prefix = '', depth = 0): CategoryEntry[] {
  const result: CategoryEntry[] = []
  for (const node of nodes) {
    if (node.children.length > 0) {
      const path = prefix ? `${prefix}/${node.name}` : node.name
      result.push({ path, name: node.name, depth })
      result.push(...extractCategories(node.children, path, depth + 1))
    }
  }
  return result
}

const categories = computed(() => extractCategories(treeStore.categories))

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('cancel')
}

onMounted(() => window.addEventListener('keydown', handleKeydown))
onUnmounted(() => window.removeEventListener('keydown', handleKeydown))
</script>

<template>
  <Teleport to="body">
    <div class="move-overlay" @click.self="emit('cancel')">
      <div class="move-dialog">
        <h3 class="move-title">移动到分类</h3>
        <div class="move-list">
          <button
            class="move-item"
            :class="{ active: currentCategory === '' }"
            @click="emit('confirm', '')"
          >
            <FolderOpen :size="14" :stroke-width="1.5" />
            <span>根目录</span>
          </button>
          <button
            v-for="cat in categories"
            :key="cat.path"
            class="move-item"
            :class="{ active: currentCategory === cat.path }"
            :style="{ paddingLeft: `${cat.depth * 14 + 12}px` }"
            @click="emit('confirm', cat.path)"
          >
            <FolderOpen :size="13" :stroke-width="1.5" />
            <span>{{ cat.name }}</span>
          </button>
        </div>
        <div class="move-footer">
          <button class="move-btn cancel" @click="emit('cancel')">取消</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.move-overlay {
  position: fixed;
  inset: 0;
  background: oklch(0 0 0 / 0.65);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
}

.move-dialog {
  width: 340px;
  max-height: 420px;
  display: flex;
  flex-direction: column;
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

.move-title {
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  padding: 18px 20px 14px;
  flex-shrink: 0;
}

.move-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 0 4px;
  border-top: 1px solid var(--ms-border);
  -webkit-mask-image: linear-gradient(to bottom, transparent 0, black 8px, black calc(100% - 8px), transparent 100%);
  mask-image: linear-gradient(to bottom, transparent 0, black 8px, black calc(100% - 8px), transparent 100%);
}

.move-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: transparent;
  color: var(--ms-smoke);
  font-family: var(--font-sans);
  font-size: 13px;
  cursor: pointer;
  text-align: left;
  transition: background var(--duration-fast) var(--ease-hydraulic),
    color var(--duration-fast) var(--ease-hydraulic);
}

.move-item:hover {
  background: var(--ms-surface);
  color: var(--text-primary);
}

.move-item.active {
  color: var(--neon);
  background: color-mix(in oklch, var(--neon) 8%, transparent);
}

.move-item svg {
  flex-shrink: 0;
}

.move-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 20px;
  border-top: 1px solid var(--ms-border);
  flex-shrink: 0;
}

.move-btn.cancel {
  padding: 6px 16px;
  border: 1px solid var(--ms-border-light);
  border-radius: 2px;
  background: transparent;
  color: var(--ms-smoke);
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-snap);
}

.move-btn.cancel:hover {
  border-color: var(--ms-smoke);
  color: var(--text-secondary);
}
</style>
