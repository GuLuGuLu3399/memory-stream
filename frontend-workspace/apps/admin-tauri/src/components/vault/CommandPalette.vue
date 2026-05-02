// 命令面板（Ctrl+K），快速执行命令或搜索并跳转到卡片
<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { Search, Plus, RefreshCw, Globe, Settings, PanelLeftClose, Upload } from 'lucide-vue-next'
import { useCommands } from '@/composables/core/useCommands'
import { useTreeStore } from '@/stores/tree'
import * as searchService from '@/services/search'
import * as cardService from '@/services/card'
import type { FtsHit } from '@memory-stream/types'
import PaletteItem from '../base/PaletteItem.vue'

const emit = defineEmits<{
  close: []
  'open-graph': []
  'open-settings': []
}>()

const { search: searchCommands, register, unregister } = useCommands()
const treeStore = useTreeStore()

const query = ref('')
const activeIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)
const searchResults = ref<FtsHit[]>([])
const isSearching = ref(false)

const builtInCommands = [
  {
    id: 'new-card',
    label: '新建卡片',
    icon: 'plus',
    shortcut: 'Ctrl+N',
    action: async () => {
      const card = await cardService.createCard('未命名卡片')
      treeStore.setActive(card.uuid)
      await treeStore.loadTree()
      emit('close')
    },
  },
  {
    id: 'sync-now',
    label: '立即同步',
    icon: 'refresh',
    shortcut: '',
    action: async () => {
      const { syncNow } = await import('@/services/sync')
      await syncNow()
      emit('close')
    },
  },
  {
    id: 'open-graph',
    label: '打开图谱',
    icon: 'globe',
    shortcut: 'Ctrl+G',
    action: () => {
      emit('open-graph')
      emit('close')
    },
  },
  {
    id: 'toggle-sidebar',
    label: '切换侧栏',
    icon: 'sidebar',
    shortcut: 'Ctrl+B',
    action: () => {
      emit('close')
    },
  },
  {
    id: 'settings',
    label: '设置',
    icon: 'settings',
    shortcut: 'Ctrl+,',
    action: () => {
      emit('open-settings')
      emit('close')
    },
  },
] as const

// Search logic: commands first, then cards
const matchedCommands = computed(() => searchCommands(query.value))

const allItems = computed(() => [
  ...matchedCommands.value.map(cmd => ({ type: 'command' as const, data: cmd })),
  ...searchResults.value.map(hit => ({ type: 'card' as const, data: hit })),
])

watch(query, async (q) => {
  activeIndex.value = 0
  if (!q.trim()) {
    searchResults.value = []
    isSearching.value = false
    return
  }
  isSearching.value = true
  try {
    searchResults.value = await searchService.search(q, 10)
  } catch {
    searchResults.value = []
  }
})

watch(allItems, () => {
  if (activeIndex.value >= allItems.value.length) {
    activeIndex.value = Math.max(0, allItems.value.length - 1)
  }
})

function selectItem(index: number) {
  const item = allItems.value[index]
  if (!item) return
  if (item.type === 'command') {
    item.data.action()
  } else {
    treeStore.setActive(item.data.uuid)
    emit('close')
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    activeIndex.value = (activeIndex.value + 1) % Math.max(1, allItems.value.length)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    activeIndex.value = (activeIndex.value - 1 + allItems.value.length) % Math.max(1, allItems.value.length)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    selectItem(activeIndex.value)
  } else if (e.key === 'Escape') {
    emit('close')
  }
}

onMounted(() => {
  builtInCommands.forEach((cmd) => {
    register({ ...cmd })
  })
  nextTick(() => inputRef.value?.focus())
})

onUnmounted(() => {
  builtInCommands.forEach((cmd) => {
    unregister(cmd.id)
  })
})
</script>

<template>
  <Teleport to="body">
    <Transition name="palette">
      <div class="palette-overlay" @click.self="emit('close')">
        <div class="palette-container">
          <!-- Search input -->
          <div class="palette-header">
            <Search :size="16" :stroke-width="1.5" class="palette-search-icon" />
            <input ref="inputRef" v-model="query" type="text" class="palette-input" placeholder="输入命令或搜索卡片..."
              @keydown="handleKeydown" />
            <kbd class="palette-esc">ESC</kbd>
          </div>

          <!-- Results -->
          <div class="palette-results">
            <!-- Commands section -->
            <template v-if="matchedCommands.length > 0">
              <div v-if="query" class="palette-section">命令</div>
              <PaletteItem v-for="(cmd, i) in matchedCommands" :key="cmd.id" :label="cmd.label" type="command"
                :shortcut="cmd.shortcut" :active="activeIndex === i" @select="selectItem(i)">
                <template #icon>
                  <Plus v-if="cmd.icon === 'plus'" :size="14" />
                  <Upload v-else-if="cmd.icon === 'upload'" :size="14" />
                  <RefreshCw v-else-if="cmd.icon === 'refresh'" :size="14" />
                  <Globe v-else-if="cmd.icon === 'globe'" :size="14" />
                  <PanelLeftClose v-else-if="cmd.icon === 'sidebar'" :size="14" />
                  <Settings v-else-if="cmd.icon === 'settings'" :size="14" />
                </template>
              </PaletteItem>
            </template>

            <!-- Cards section -->
            <template v-if="searchResults.length > 0">
              <div class="palette-section">卡片</div>
              <PaletteItem v-for="(hit, i) in searchResults" :key="hit.uuid" :label="hit.title" type="card"
                :active="activeIndex === matchedCommands.length + i" @select="selectItem(matchedCommands.length + i)" />
            </template>

            <!-- Empty -->
            <div v-if="query && matchedCommands.length === 0 && searchResults.length === 0 && !isSearching"
              class="palette-empty">
              未找到匹配项
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.palette-overlay {
  position: fixed;
  inset: 0;
  background: oklch(0 0 0 / 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  justify-content: center;
  padding-top: 15vh;
  z-index: var(--z-overlay);
}

.palette-container {
  width: 520px;
  max-height: 60vh;
  display: flex;
  flex-direction: column;
  background: var(--ms-void);
  border: 1px solid var(--ms-border-light);
  border-radius: 2px;
  box-shadow: 0 20px 40px oklch(0 0 0 / 0.8), 0 0 0 1px var(--ms-surface);
  overflow: hidden;
}

/* Header / Input */
.palette-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--ms-border);
}

.palette-search-icon {
  flex-shrink: 0;
  color: var(--text-muted);
}

.palette-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-family: var(--font-sans);
  font-size: 15px;
  color: var(--text-primary);
}

.palette-input::placeholder {
  color: var(--text-muted);
}

.palette-esc {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  padding: 2px 6px;
  border: 1px solid var(--ms-border);
  border-bottom: 2px solid var(--ms-smoke);
  border-radius: 0;
}

/* Results */
.palette-results {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
  -webkit-mask-image: linear-gradient(to bottom, transparent 0, black 10px, black calc(100% - 10px), transparent 100%);
  mask-image: linear-gradient(to bottom, transparent 0, black 10px, black calc(100% - 10px), transparent 100%);
}

.palette-section {
  padding: 8px 16px 4px;
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.08em;
  color: var(--text-muted);
  text-transform: uppercase;
}

.palette-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  font-family: var(--font-sans);
  font-size: 13px;
  color: var(--text-muted);
}

/* Transition */
.palette-enter-active {
  transition: opacity var(--duration-normal) var(--ease-emerge);
}

.palette-enter-active .palette-container {
  transition: opacity var(--duration-normal) var(--ease-emerge),
    transform var(--duration-normal) var(--ease-emerge);
}

.palette-leave-active {
  transition: opacity 150ms var(--ease-hydraulic);
}

.palette-enter-from {
  opacity: 0;
}

.palette-enter-from .palette-container {
  opacity: 0;
  transform: scale(0.97) translateY(-8px);
}

.palette-leave-to {
  opacity: 0;
}
</style>
