<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch, nextTick } from 'vue'
import { RouterView, useRouter } from 'vue-router'
import { Search, ArrowRight } from 'lucide-vue-next'
import { useAuthStore } from '@/stores/auth'
import { useGlossaryStore } from '@/stores/glossary'
import { useLayoutStore } from '@/stores/layout'
import { useReaderStore } from '@/stores/reader'

const authStore = useAuthStore()
const glossaryStore = useGlossaryStore()
const layout = useLayoutStore()
const reader = useReaderStore()
const router = useRouter()

const searchQuery = ref('')
const searchInputRef = ref<HTMLInputElement | null>(null)
let searchTimer: ReturnType<typeof setTimeout> | null = null

onMounted(async () => {
  await authStore.initAuth()
  await glossaryStore.loadGlossary()
  window.addEventListener('keydown', handleGlobalKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeydown)
  if (searchTimer) clearTimeout(searchTimer)
})

function handleGlobalKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault()
    layout.toggleSearch()
  }
  if (e.key === 'Escape' && layout.searchOpen) {
    e.preventDefault()
    layout.closeSearch()
  }
}

watch(() => layout.searchOpen, async (open) => {
  if (open) {
    searchQuery.value = ''
    await nextTick()
    searchInputRef.value?.focus()
  }
})

function onSearchInput() {
  if (searchTimer) clearTimeout(searchTimer)
  const q = searchQuery.value.trim()
  if (!q) return
  searchTimer = setTimeout(() => reader.doSearch(q), 250)
}

async function handleSearch() {
  if (searchTimer) clearTimeout(searchTimer)
  const q = searchQuery.value.trim()
  if (q) await reader.doSearch(q)
}

function goToCard(uuid: string) {
  layout.closeSearch()
  router.push({ name: 'card', params: { uuid } })
}
</script>

<template>
  <div class="app-root">
    <div class="zen-ambient" aria-hidden="true" />
    <div class="app-content">
      <RouterView v-slot="{ Component }">
        <transition name="page" mode="out-in">
          <component :is="Component" />
        </transition>
      </RouterView>
    </div>

    <!-- Global search overlay (Ctrl+K) -->
    <Teleport to="body">
      <Transition name="overlay">
        <div v-if="layout.searchOpen" class="search-backdrop" @click="layout.closeSearch()">
          <div class="search-panel" @click.stop>
            <div class="search-input-row">
              <Search :size="18" class="search-icon" />
              <input
                ref="searchInputRef"
                v-model="searchQuery"
                type="text"
                placeholder="搜索笔记..."
                class="search-field"
                @input="onSearchInput"
                @keydown.enter="handleSearch"
              />
              <kbd class="search-kbd">ESC</kbd>
            </div>
            <div v-if="searchQuery && reader.searchResults.length" class="search-results">
              <button
                v-for="item in reader.searchResults"
                :key="item.uuid"
                class="search-result"
                @click="goToCard(item.uuid)"
              >
                <span class="result-title">{{ item.title }}</span>
                <ArrowRight :size="14" class="result-arrow" />
              </button>
            </div>
            <div v-else-if="searchQuery && !reader.searchLoading && reader.searchResults.length === 0 && searchQuery.length > 1" class="search-empty">
              无匹配结果
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.app-root {
  min-height: 100vh;
  background: var(--ms-deep);
  color: var(--text-primary);
  font-family: var(--font-serif);
  position: relative;
  overflow-x: hidden;
}

.app-content {
  position: relative;
  z-index: 1;
}

/* Subtle warm vignette — candlelight from above */
.zen-ambient {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  background: radial-gradient(ellipse at 50% 0%, oklch(0.22 0.015 70 / 0.25), transparent 60%);
}

/* ── Page transitions — pure crossfade ── */
.page-enter-active {
  transition: opacity var(--duration-normal) var(--ease-gentle);
}

.page-leave-active {
  transition: opacity 200ms var(--ease-gentle);
}

.page-enter-from,
.page-leave-to {
  opacity: 0;
}

/* ── Search overlay ── */
.search-backdrop {
  position: fixed;
  inset: 0;
  background: oklch(0.1 0.01 70 / 0.7);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  z-index: 400;
  display: flex;
  justify-content: center;
  padding-top: 15vh;
}

.search-panel {
  width: 90%;
  max-width: 560px;
  background: var(--ms-surface);
  border: 1px solid var(--ms-border);
  border-radius: 12px;
  overflow: hidden;
  align-self: flex-start;
}

.search-input-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 20px;
  border-bottom: 1px solid var(--ms-border);
}

.search-icon {
  color: var(--accent-dim);
  flex-shrink: 0;
}

.search-field {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  font-size: 1rem;
  color: var(--text-primary);
  font-family: var(--font-serif);
}

.search-field::placeholder {
  color: var(--text-muted);
}

.search-kbd {
  font-family: var(--font-mono);
  font-size: 0.65rem;
  color: var(--text-muted);
  padding: 2px 6px;
  border: 1px solid var(--ms-border);
  border-radius: 4px;
  flex-shrink: 0;
}

.search-results {
  max-height: 40vh;
  overflow-y: auto;
}

.search-result {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 12px 20px;
  background: none;
  border: none;
  border-bottom: 1px solid var(--ms-border);
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
  font-family: var(--font-serif);
  transition: background var(--duration-fast) var(--ease-gentle);
}

.search-result:hover {
  background: var(--ms-surface-elevated);
}

.search-result:last-child {
  border-bottom: none;
}

.result-title {
  flex: 1;
  font-size: 0.95rem;
}

.result-arrow {
  color: var(--text-muted);
  flex-shrink: 0;
  transition: color var(--duration-fast);
}

.search-result:hover .result-arrow {
  color: var(--accent);
}

.search-empty {
  padding: 24px;
  text-align: center;
  color: var(--text-muted);
  font-size: 0.9rem;
}

/* ── Overlay transition ── */
.overlay-enter-active {
  transition: opacity var(--duration-normal) var(--ease-gentle);
}

.overlay-leave-active {
  transition: opacity 200ms var(--ease-gentle);
}

.overlay-enter-from,
.overlay-leave-to {
  opacity: 0;
}
</style>
