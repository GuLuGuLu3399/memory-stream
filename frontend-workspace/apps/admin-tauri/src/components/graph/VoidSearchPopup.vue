<script setup lang="ts">
// 用途：虚空搜索弹窗，模糊搜索卡片并选择连接目标
import { ref, watch, onMounted, nextTick } from 'vue'
import { fuzzySearch } from '@/services/search'

interface CardSummary {
  uuid: string
  title: string
}

const props = defineProps<{
  sourceId?: string
  x: number
  y: number
  excludeIds?: string[]
}>()

const emit = defineEmits<{
  select: [uuid: string]
  close: []
}>()

const query = ref('')
const results = ref<CardSummary[]>([])
const selectedIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)

watch(query, async (q) => {
  if (!q.trim()) {
    results.value = []
    selectedIndex.value = 0
    return
  }
  try {
    const exclude = [...(props.excludeIds ?? [])]
    if (props.sourceId) exclude.push(props.sourceId)
    results.value = await fuzzySearch(q, exclude)
    selectedIndex.value = 0
  } catch {
    results.value = []
  }
})

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    emit('close')
    return
  }
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, results.value.length - 1)
    return
  }
  if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
    return
  }
  if (e.key === 'Enter') {
    const hit = results.value[selectedIndex.value]
    if (hit) emit('select', hit.uuid)
    return
  }
}

onMounted(() => {
  nextTick(() => inputRef.value?.focus())
})
</script>

<template>
  <div class="void-search" :style="{ left: `${x}px`, top: `${y}px` }" @keydown="handleKeydown">
    <input
      ref="inputRef"
      v-model="query"
      class="void-search-input"
      placeholder="召唤卡片..."
      spellcheck="false"
    />
    <div v-if="results.length > 0" class="void-search-results">
      <div
        v-for="(hit, i) in results"
        :key="hit.uuid"
        class="void-search-item"
        :class="{ selected: i === selectedIndex }"
        @click="emit('select', hit.uuid)"
        @mouseenter="selectedIndex = i"
      >
        <span class="void-search-title">{{ hit.title }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.void-search {
  position: fixed;
  transform: translate(-50%, 8px);
  width: 280px;
  background: color-mix(in oklch, var(--ms-void) 92%, transparent);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  backdrop-filter: blur(8px);
  z-index: 100;
  animation: void-search-in var(--duration-fast) var(--ease-emerge);
}

@keyframes void-search-in {
  from {
    opacity: 0;
    transform: translate(-50%, 8px) scale(0.96);
  }
  to {
    opacity: 1;
    transform: translate(-50%, 8px) scale(1);
  }
}

.void-search-input {
  width: 100%;
  padding: 8px 10px;
  border: none;
  border-bottom: 1px solid var(--ms-border);
  border-radius: 2px 2px 0 0;
  background: transparent;
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-primary);
  outline: none;
}

.void-search-input::placeholder {
  color: var(--text-muted);
}

.void-search-input:focus {
  border-bottom-color: var(--neon);
}

.void-search-results {
  max-height: 200px;
  overflow-y: auto;
}

.void-search-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 6px 10px;
  cursor: pointer;
  border-left: 2px solid transparent;
  transition: background var(--duration-fast) var(--ease-hydraulic),
    border-color var(--duration-fast) var(--ease-hydraulic);
}

.void-search-item:hover,
.void-search-item.selected {
  background: var(--ms-surface);
  border-left-color: var(--neon);
}

.void-search-title {
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--text-primary);
}
</style>
