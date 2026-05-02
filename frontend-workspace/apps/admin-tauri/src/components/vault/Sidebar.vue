// 用途：侧边栏面板，包含分类树、搜索和卡片列表
<script setup lang="ts">
import { ref, computed, watch, inject } from 'vue'
import { Search, Plus, FolderPlus, ChevronRight, Folder, FileText } from 'lucide-vue-next'
import { useTreeStore } from '@/stores/tree'
import { useEditorStore } from '@/stores/editor'
import * as searchService from '@/services/search'
import * as cardService from '@/services/card'
import * as vaultService from '@/services/vault'
import { useToast } from '@/composables/core/useToast'
import { findInTree } from '@memory-stream/core'
import type { TreeNode } from '@memory-stream/core'
import type { FtsHit } from '@memory-stream/types'
import ContextMenu from './ContextMenu.vue'
import ConfirmDialog from '@/components/base/ConfirmDialog.vue'
import RenameDialog from '@/components/base/RenameDialog.vue'
import MoveCategoryDialog from '@/components/base/MoveCategoryDialog.vue'

interface FlatItem {
  id: string
  name: string
  depth: number
  isCategory: boolean
  childCount: number
}

function flattenTree(nodes: TreeNode[], depth: number, result: FlatItem[], expanded: Set<string>, maxDepth = 1) {
  for (const node of nodes) {
    const isCategory = node.is_dir
    result.push({ id: node.id, name: node.name, depth, isCategory, childCount: node.children.length })
    if (isCategory && expanded.has(node.id) && depth < maxDepth) {
      flattenTree(node.children, depth + 1, result, expanded, maxDepth)
    }
  }
}

const treeStore = useTreeStore()
const editorStore = useEditorStore()
const toast = useToast()
const refreshGraph = inject<() => void>('refreshGraph', () => {})

const sortedCategories = computed(() =>
  [...treeStore.categories].sort((a, b) => {
    const aIsDir = a.is_dir ? 0 : 1
    const bIsDir = b.is_dir ? 0 : 1
    return aIsDir - bIsDir || a.name.localeCompare(b.name)
  })
)

const flatList = computed(() => {
  const result: FlatItem[] = []
  flattenTree(sortedCategories.value, 0, result, treeStore.expandedIds)
  return result
})

const searchQuery = ref('')
const searchResults = ref<FtsHit[]>([])
const isSearching = ref(false)
const isCreating = ref(false)
const addMenuOpen = ref(false)

const ctxOpen = ref(false)
const ctxPos = ref({ x: 0, y: 0 })
const ctxTarget = ref<string | null>(null)
const ctxIsCategory = ref(false)
const confirmDeleteOpen = ref(false)
const pendingDeleteUuid = ref<string | null>(null)

const renameOpen = ref(false)
const renameTarget = ref<string | null>(null)
const renameCurrentTitle = ref('')

const moveOpen = ref(false)
const moveUuid = ref<string | null>(null)
const moveCurrentCategory = ref('')

const createCategoryOpen = ref(false)

const renameCatOpen = ref(false)
const renameCatTarget = ref('')
const renameCatCurrentName = ref('')

const confirmDeleteCatOpen = ref(false)
const pendingDeleteCategory = ref<string | null>(null)

const createInCatOpen = ref(false)
const createInCatTarget = ref('')

const dragUuid = ref<string | null>(null)
const dragOverId = ref<string | null>(null)

function handleDragStart(uuid: string, e: DragEvent) {
  dragUuid.value = uuid
  e.dataTransfer!.effectAllowed = 'move'
  e.dataTransfer!.setData('text/plain', uuid)
}

function handleDragOver(e: DragEvent) {
  e.preventDefault()
  e.dataTransfer!.dropEffect = 'move'
}

function handleDragEnter(id: string) {
  dragOverId.value = id
}

function handleDragLeave() {
  dragOverId.value = null
}

async function handleDrop(categoryId: string) {
  const uuid = dragUuid.value
  dragUuid.value = null
  dragOverId.value = null
  if (!uuid || uuid === categoryId) return
  try {
    await vaultService.moveCard(uuid, categoryId)
    await treeStore.loadTree()
    toast.success('已移动')
  } catch {
    toast.error('移动失败')
  }
}

function handleDragEnd() {
  dragUuid.value = null
  dragOverId.value = null
}

let searchTimer: ReturnType<typeof setTimeout> | null = null

watch(searchQuery, (q) => {
  if (searchTimer) clearTimeout(searchTimer)
  if (!q.trim()) {
    searchResults.value = []
    isSearching.value = false
    return
  }
  searchTimer = setTimeout(async () => {
    isSearching.value = true
    try {
      searchResults.value = await searchService.search(q, 20)
    } catch {
      searchResults.value = []
    }
  }, 200)
})

async function handleCreateCard() {
  addMenuOpen.value = false
  isCreating.value = true
  try {
    const card = await cardService.createCard('未命名卡片')
    treeStore.setActive(card.uuid)
    await treeStore.loadTree()
    toast.success('已创建')
  } catch {
    toast.error('创建失败')
  } finally {
    isCreating.value = false
  }
}

function handleCardClick(uuid: string) {
  treeStore.setActive(uuid)
  searchQuery.value = ''
}

function handleContextMenu(uuid: string, e: MouseEvent, isCategory = false) {
  ctxTarget.value = uuid
  ctxIsCategory.value = isCategory
  ctxPos.value = { x: e.clientX, y: e.clientY }
  ctxOpen.value = true
}

function closeContextMenu() {
  ctxOpen.value = false
  ctxTarget.value = null
  ctxIsCategory.value = false
}

async function handleContextAction(action: string) {
  const uuid = ctxTarget.value
  const wasCategory = ctxIsCategory.value
  closeContextMenu()
  if (!uuid) return

  switch (action) {
    case 'rename': {
      if (wasCategory) {
        renameCatTarget.value = uuid
        renameCatCurrentName.value = uuid
        renameCatOpen.value = true
      } else {
        const node = findInTree(treeStore.categories, (n) => n.id === uuid)
        renameTarget.value = uuid
        renameCurrentTitle.value = node?.name ?? ''
        renameOpen.value = true
      }
      break
    }
    case 'move': {
      const parent = findInTree(treeStore.categories, (n) =>
        n.children.some(c => c.id === uuid),
      )
      moveUuid.value = uuid
      moveCurrentCategory.value = parent?.name ?? ''
      moveOpen.value = true
      break
    }
    case 'delete': {
      if (wasCategory) {
        pendingDeleteCategory.value = uuid
        confirmDeleteCatOpen.value = true
      } else {
        pendingDeleteUuid.value = uuid
        confirmDeleteOpen.value = true
      }
      break
    }
  }
}

async function confirmDeleteCard() {
  const uuid = pendingDeleteUuid.value
  confirmDeleteOpen.value = false
  pendingDeleteUuid.value = null
  if (!uuid) return

  try {
    await cardService.deleteCard(uuid, false)
    if (treeStore.activeCardUuid === uuid) {
      treeStore.setActive(null)
    }
    await treeStore.loadTree()
    refreshGraph()
    toast.success('已删除')
  } catch {
    toast.error('删除失败')
  }
}

function handleCreateInCategory(category: string) {
  createInCatTarget.value = category
  createInCatOpen.value = true
}

async function confirmCreateInCategory(title: string) {
  const category = createInCatTarget.value
  createInCatOpen.value = false
  createInCatTarget.value = ''
  if (!title.trim()) return
  try {
    const card = await cardService.createCard(title, category)
    treeStore.setActive(card.uuid)
    await treeStore.loadTree()
    toast.success('已创建到分类')
  } catch {
    toast.error('分类内新建失败')
  }
}

async function confirmRename(newTitle: string) {
  const uuid = renameTarget.value
  renameOpen.value = false
  renameTarget.value = null
  if (!uuid) return
  try {
    const newMeta = await cardService.renameCard(uuid, newTitle)
    if (treeStore.activeCardUuid === uuid) {
      editorStore.currentMeta = newMeta
    }
    await treeStore.loadTree()
    toast.success('已重命名')
  } catch {
    toast.error('重命名失败')
  }
}

async function confirmMove(category: string) {
  const uuid = moveUuid.value
  moveOpen.value = false
  moveUuid.value = null
  if (!uuid) return
  try {
    await vaultService.moveCard(uuid, category)
    await treeStore.loadTree()
    toast.success('已移动')
  } catch {
    toast.error('移动失败')
  }
}

function handleSearchContextmenu(hit: FtsHit, e: MouseEvent) {
  handleContextMenu(hit.uuid, e)
}

async function confirmCreateCategory(name: string) {
  createCategoryOpen.value = false
  if (!name.trim()) return
  try {
    await vaultService.createCategory(name.trim())
    await treeStore.loadTree()
    toast.success('分类已创建')
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : String(e))
  }
}

async function confirmRenameCategory(newName: string) {
  const oldName = renameCatTarget.value
  renameCatOpen.value = false
  renameCatTarget.value = ''
  if (!oldName || !newName.trim()) return
  try {
    await vaultService.renameCategory(oldName, newName.trim())
    await treeStore.loadTree()
    toast.success('分类已重命名')
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : String(e))
  }
}

async function confirmDeleteCategory() {
  const category = pendingDeleteCategory.value
  confirmDeleteCatOpen.value = false
  pendingDeleteCategory.value = null
  if (!category) return
  try {
    await vaultService.deleteCategory(category)
    await treeStore.loadTree()
    refreshGraph()
    toast.success('分类已删除')
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : String(e))
  }
}

defineExpose({ treeStore })
</script>

<template>
  <div class="sidebar-content" @click="addMenuOpen = false">
    <div class="sidebar-header">
      <div class="sidebar-search">
        <Search :size="13" :stroke-width="1.5" class="search-icon" />
        <input v-model="searchQuery" type="text" placeholder="搜索..." @click.stop />
      </div>
      <div class="sidebar-add" @click.stop>
        <button class="add-trigger" :disabled="isCreating" @click="addMenuOpen = !addMenuOpen">
          <Plus :size="14" :stroke-width="1.5" />
        </button>
        <Transition name="dropdown">
          <div v-if="addMenuOpen" class="add-dropdown">
            <button class="add-item" @click="handleCreateCard">
              <Plus :size="12" :stroke-width="1.5" />
              <span>新建卡片</span>
            </button>
            <button class="add-item" @click="addMenuOpen = false; createCategoryOpen = true">
              <FolderPlus :size="12" :stroke-width="1.5" />
              <span>新建分类</span>
            </button>
          </div>
        </Transition>
      </div>
    </div>

    <div v-if="isSearching && searchResults.length > 0" class="sidebar-tree stagger-children">
      <button v-for="hit in searchResults" :key="hit.uuid" class="search-hit"
        :class="{ active: treeStore.activeCardUuid === hit.uuid }" @click="handleCardClick(hit.uuid)"
        @contextmenu.prevent="handleSearchContextmenu(hit, $event)">
        <span class="search-hit-title">{{ hit.title }}</span>
        <span class="search-hit-excerpt">{{ hit.excerpt }}</span>
      </button>
    </div>

    <div v-else-if="isSearching && searchQuery && searchResults.length === 0" class="sidebar-empty">
      <span>未找到相关卡片</span>
    </div>

    <div v-else class="sidebar-tree">
      <div v-for="item in flatList" :key="item.id"
        class="tree-row"
        :class="[
          item.isCategory ? 'tree-category' : 'tree-card',
          { active: !item.isCategory && treeStore.activeCardUuid === item.id },
          { 'drag-over': item.isCategory && dragOverId === item.id },
        ]"
        :style="{ paddingLeft: `${item.depth * 16 + 12}px` }"
        :draggable="!item.isCategory"
        @click="item.isCategory ? treeStore.toggleExpand(item.id) : handleCardClick(item.id)"
        @contextmenu.prevent="handleContextMenu(item.id, $event, item.isCategory)"
        @dragstart="!item.isCategory && handleDragStart(item.id, $event)"
        @dragend="handleDragEnd"
        @dragover="item.isCategory && handleDragOver($event)"
        @dragenter="item.isCategory && handleDragEnter(item.id)"
        @dragleave="item.isCategory && handleDragLeave()"
        @drop="item.isCategory && handleDrop(item.id)">
        <span v-for="i in item.depth" :key="i" class="guide-line"
          :style="{ left: `${(i - 1) * 16 + 10}px` }" />
        <ChevronRight v-if="item.isCategory" :size="12" :stroke-width="1.5"
          class="tree-chevron" :class="{ expanded: treeStore.expandedIds.has(item.id) }" />
        <Folder v-if="item.isCategory" :size="13" :stroke-width="1.5" class="tree-icon" />
        <FileText v-else :size="13" :stroke-width="1.5" class="tree-icon" />
        <span class="tree-label">{{ item.name }}</span>
        <span v-if="item.isCategory && item.childCount === 0" class="tree-empty-hint">(空)</span>
        <span v-if="item.isCategory" class="tree-count">{{ item.childCount }}</span>
        <button v-if="item.isCategory" class="tree-cat-add" title="在分类内新建"
          @click.stop="handleCreateInCategory(item.id)">
          <Plus :size="11" :stroke-width="1.7" />
        </button>
      </div>
      <div v-if="flatList.length === 0 && !treeStore.loading" class="sidebar-empty">
        <span>暂无卡片</span>
      </div>
      <div v-if="treeStore.loading" class="sidebar-loading">
        <span class="animate-neon-pulse">加载中...</span>
      </div>
    </div>

    <ContextMenu v-if="ctxOpen" :x="ctxPos.x" :y="ctxPos.y" :is-category="ctxIsCategory"
      @close="closeContextMenu" @action="handleContextAction" />

    <ConfirmDialog
      v-if="confirmDeleteOpen"
      title="删除卡片"
      message="确定删除此卡片？"
      confirm-label="删除"
      :destructive="true"
      @confirm="confirmDeleteCard"
      @cancel="confirmDeleteOpen = false; pendingDeleteUuid = null"
    />

    <RenameDialog
      v-if="renameOpen"
      title="重命名"
      :value="renameCurrentTitle"
      @confirm="confirmRename"
      @cancel="renameOpen = false; renameTarget = null"
    />

    <MoveCategoryDialog
      v-if="moveOpen"
      :current-category="moveCurrentCategory"
      @confirm="confirmMove"
      @cancel="moveOpen = false; moveUuid = null"
    />

    <RenameDialog
      v-if="createCategoryOpen"
      title="新建分类"
      value=""
      @confirm="confirmCreateCategory"
      @cancel="createCategoryOpen = false"
    />

    <RenameDialog
      v-if="renameCatOpen"
      title="重命名分类"
      :value="renameCatCurrentName"
      @confirm="confirmRenameCategory"
      @cancel="renameCatOpen = false; renameCatTarget = ''"
    />

    <ConfirmDialog
      v-if="confirmDeleteCatOpen"
      title="删除分类"
      message="将删除分类及其中的所有卡片，确定继续？"
      confirm-label="删除"
      :destructive="true"
      @confirm="confirmDeleteCategory"
      @cancel="confirmDeleteCatOpen = false; pendingDeleteCategory = null"
    />

    <RenameDialog
      v-if="createInCatOpen"
      title="分类内新建卡片"
      value="未命名卡片"
      @confirm="confirmCreateInCategory"
      @cancel="createInCatOpen = false; createInCatTarget = ''"
    />
  </div>
</template>

<style scoped>
.sidebar-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* Header */
.sidebar-header {
  display: flex;
  align-items: center;
  gap: 0;
  padding: 0;
  border-bottom: 1px solid var(--ms-border);
  flex-shrink: 0;
}

.sidebar-search {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  padding: 7px 10px;
  min-width: 0;
}

.search-icon {
  flex-shrink: 0;
  color: var(--text-muted);
}

.sidebar-search input {
  flex: 1;
  min-width: 0;
  background: transparent;
  border: none;
  outline: none;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--text-primary);
}

.sidebar-search input::placeholder {
  color: var(--text-muted);
}

.sidebar-add {
  position: relative;
  flex-shrink: 0;
}

.add-trigger {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: color var(--duration-fast) var(--ease-hydraulic),
    background var(--duration-fast) var(--ease-hydraulic);
}

.add-trigger:hover {
  color: var(--neon);
  background: var(--ms-surface);
}

.add-trigger:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.add-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  min-width: 140px;
  background: var(--ms-panel);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  box-shadow: var(--shadow-raised);
  padding: 4px 0;
  z-index: var(--z-float);
}

.add-item {
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

.add-item:hover {
  background: var(--ms-surface);
  color: var(--text-primary);
}

/* Dropdown transition */
.dropdown-enter-active {
  transition: opacity 100ms var(--ease-emerge), transform 100ms var(--ease-emerge);
}

.dropdown-leave-active {
  transition: opacity 80ms var(--ease-hydraulic), transform 80ms var(--ease-hydraulic);
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.97);
}

/* Tree area */
.sidebar-tree {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
  -webkit-mask-image: linear-gradient(to bottom, transparent 0, black 12px, black calc(100% - 12px), transparent 100%);
  mask-image: linear-gradient(to bottom, transparent 0, black 12px, black calc(100% - 12px), transparent 100%);
}

/* Flat tree rows */
.tree-row {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding-top: 5px;
  padding-bottom: 5px;
  padding-right: 10px;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
  position: relative;
  user-select: none;
  transition: background var(--duration-fast) var(--ease-hydraulic);
}

.tree-row:hover {
  background: var(--ms-carbon);
}

/* Guide lines */
.guide-line {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 1px;
  background: var(--ms-border-light);
  pointer-events: none;
}

/* Category row */
.tree-category {
  color: var(--text-secondary);
}

.tree-category:hover {
  color: var(--text-primary);
}

.tree-chevron {
  flex-shrink: 0;
  transition: transform var(--duration-fast) var(--ease-hydraulic);
}

.tree-chevron.expanded {
  transform: rotate(90deg);
}

.tree-count {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  min-width: 16px;
  text-align: center;
}

.tree-empty-hint {
  font-size: 10px;
  color: var(--text-muted);
  font-style: italic;
}

.tree-cat-add {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  margin-left: auto;
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

.tree-row:hover .tree-cat-add {
  opacity: 1;
}

.tree-cat-add:hover {
  color: var(--neon);
  border-color: var(--ms-border-light);
}

/* Card row */
.tree-card {
  color: var(--text-secondary);
}

.tree-card::before {
  content: '';
  position: absolute;
  left: 0;
  top: 4px;
  bottom: 4px;
  width: 2px;
  background: transparent;
  border-radius: 1px;
  transition: background var(--duration-fast) var(--ease-hydraulic);
}

.tree-card:hover::before {
  background: var(--neon-dim);
}

.tree-card.active {
  background: var(--ms-surface);
}

.tree-card.active::before {
  background: var(--neon);
}

/* Shared label */
.tree-label {
  flex: 1;
  font-family: var(--font-sans);
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tree-icon {
  flex-shrink: 0;
  opacity: 0.5;
}

.tree-category .tree-icon {
  color: var(--brass);
  opacity: 0.7;
}

.tree-card .tree-icon {
  color: var(--text-muted);
}

.tree-card:hover .tree-icon,
.tree-card.active .tree-icon {
  color: var(--neon);
  opacity: 0.8;
}

.tree-row.drag-over {
  background: oklch(0.78 0.17 200 / 0.08);
  outline: 1px dashed var(--neon);
  outline-offset: -1px;
}

.tree-card[draggable="true"] {
  cursor: grab;
}

.tree-card[draggable="true"]:active {
  cursor: grabbing;
}

.tree-category .tree-label {
  font-weight: 600;
  font-size: 12px;
  letter-spacing: 0.02em;
}

.tree-card.active .tree-label {
  color: var(--neon);
}

/* Search results */
.search-hit {
  display: flex;
  flex-direction: column;
  gap: 2px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
  position: relative;
  transition: background var(--duration-fast) var(--ease-hydraulic);
}

.search-hit::before {
  content: '';
  position: absolute;
  left: 0;
  top: 4px;
  bottom: 4px;
  width: 2px;
  background: transparent;
  border-radius: 1px;
  transition: background var(--duration-fast) var(--ease-hydraulic);
}

.search-hit:hover {
  background: var(--ms-carbon);
}

.search-hit:hover::before {
  background: var(--neon-dim);
}

.search-hit.active {
  background: var(--ms-surface);
}

.search-hit.active::before {
  background: var(--neon);
}

.search-hit-title {
  font-family: var(--font-sans);
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.search-hit.active .search-hit-title {
  color: var(--neon);
}

.search-hit-excerpt {
  font-family: var(--font-sans);
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Empty / Loading */
.sidebar-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 120px;
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--text-muted);
}

.sidebar-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 120px;
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--text-muted);
}
</style>
