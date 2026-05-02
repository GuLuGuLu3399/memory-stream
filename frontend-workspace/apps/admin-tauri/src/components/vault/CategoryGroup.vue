// 可折叠的分类树节点，递归渲染子分类和卡片，支持分类内新建

<script setup lang="ts">
import { computed } from 'vue'
import { ChevronRight, Plus } from 'lucide-vue-next'
import { useTreeStore } from '@/stores/tree'
import type { TreeNode } from '@memory-stream/core'
import CardItem from './CardItem.vue'

const props = defineProps<{
  node: TreeNode
  depth: number
}>()

const emit = defineEmits<{
  'select-card': [uuid: string]
  'create-in-category': [category: string]
  'card-contextmenu': [uuid: string, event: MouseEvent]
}>()

const treeStore = useTreeStore()
const isExpanded = computed(() => treeStore.expandedIds.has(props.node.id))

function toggle() {
  treeStore.toggleExpand(props.node.id)
}

function handleHeaderKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault()
    toggle()
  }
}

function handleCreateInCategory(e: MouseEvent) {
  e.stopPropagation()
  emit('create-in-category', props.node.id)
}

const indentStyle = computed(() => ({
  paddingLeft: `${props.depth * 12 + 10}px`,
}))
</script>

<template>
  <div class="category-group">
    <div class="category-header" :style="indentStyle" role="button" tabindex="0" @click="toggle"
      @keydown="handleHeaderKeydown">
      <ChevronRight :size="12" :stroke-width="1.5" class="category-chevron" :class="{ expanded: isExpanded }" />
      <span class="category-name">{{ node.name }}</span>
      <span class="category-count">{{ node.children.length }}</span>
      <button class="category-create-btn" type="button" title="在当前分类新建" @click="handleCreateInCategory">
        <Plus :size="11" :stroke-width="1.7" />
      </button>
    </div>

    <div v-if="isExpanded" class="category-children">
      <template v-for="child in node.children" :key="child.id">
        <!-- Sub-category -->
        <CategoryGroup v-if="child.children.length > 0" :node="child" :depth="depth + 1"
          @select-card="(uuid) => emit('select-card', uuid)"
          @create-in-category="(category) => emit('create-in-category', category)"
          @card-contextmenu="(uuid, e) => emit('card-contextmenu', uuid, e)" />
        <!-- Leaf card -->
        <CardItem v-else :uuid="child.id" :title="child.name" :is-active="treeStore.activeCardUuid === child.id"
          @click="emit('select-card', child.id)"
          @contextmenu="(e) => emit('card-contextmenu', child.id, e)" />
      </template>
    </div>
  </div>
</template>

<style scoped>
.category-group {
  user-select: none;
}

.category-header {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 6px 10px;
  border: none;
  background: transparent;
  cursor: pointer;
  color: var(--text-secondary);
  transition: background var(--duration-fast) var(--ease-hydraulic),
    color var(--duration-fast) var(--ease-hydraulic);
}

.category-header:hover {
  background: var(--ms-carbon);
  color: var(--text-primary);
}

.category-chevron {
  flex-shrink: 0;
  transition: transform var(--duration-fast) var(--ease-hydraulic);
}

.category-chevron.expanded {
  transform: rotate(90deg);
}

.category-name {
  flex: 1;
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.02em;
  text-align: left;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.category-count {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  min-width: 16px;
  text-align: center;
}

.category-create-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 2px;
  opacity: 0;
  transition: opacity var(--duration-fast) var(--ease-hydraulic),
    border-color var(--duration-fast) var(--ease-hydraulic),
    color var(--duration-fast) var(--ease-hydraulic);
}

.category-header:hover .category-create-btn {
  opacity: 1;
}

.category-create-btn:hover {
  color: var(--neon);
  border-color: var(--ms-border-light);
}

.category-children {
  overflow: hidden;
}
</style>
