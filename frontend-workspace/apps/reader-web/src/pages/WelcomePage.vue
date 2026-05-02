<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Search, Shuffle, Clock, Compass } from 'lucide-vue-next'
import { useReaderStore } from '@/stores/reader'

const router = useRouter()
const store = useReaderStore()
const query = ref('')

interface RecentItem {
  uuid: string
  title: string
  updated_at: string
}

const recentItems = ref<RecentItem[]>([])

function handleSearch() {
  const q = query.value.trim()
  if (q) router.push({ name: 'search', query: { q } })
}

function formatRelative(dateStr: string): string {
  const diff = Date.now() - new Date(dateStr).getTime()
  const mins = Math.floor(diff / 60000)
  if (mins < 60) return `${mins} 分钟前`
  const hours = Math.floor(mins / 60)
  if (hours < 24) return `${hours} 小时前`
  const days = Math.floor(hours / 24)
  return `${days} 天前`
}

function loadRecent() {
  try {
    const raw = localStorage.getItem('reader_recent')
    if (raw) recentItems.value = JSON.parse(raw).slice(0, 10)
  } catch (e) { console.warn('Failed to load recent:', e) }
}

onMounted(() => {
  store.loadRandomCards(5)
  loadRecent()
})
</script>

<template>
  <div class="welcome">
    <div class="welcome-inner">
      <!-- Title with breathing circle -->
      <div class="hero fade-up">
        <div class="breath-circle" aria-hidden="true" />
        <h1 class="welcome-title">Memory Stream</h1>
        <p class="welcome-sub">思绪如溪，知识成河</p>
      </div>

      <!-- Search -->
      <form class="search-box fade-up" style="animation-delay: 0.15s" @submit.prevent="handleSearch">
        <Search :size="18" class="search-icon" />
        <input
          v-model="query"
          type="text"
          placeholder="搜索笔记..."
          class="search-input"
        />
        <kbd class="search-kbd">Ctrl+K</kbd>
      </form>

      <!-- Recent -->
      <div v-if="recentItems.length" class="section fade-up" style="animation-delay: 0.3s">
        <h2 class="section-title">
          <Clock :size="14" />
          最近阅读
        </h2>
        <div class="card-list">
          <RouterLink
            v-for="item in recentItems"
            :key="item.uuid"
            :to="{ name: 'card', params: { uuid: item.uuid } }"
            class="list-card"
          >
            <span class="list-card-title">{{ item.title }}</span>
            <span class="list-card-time">{{ formatRelative(item.updated_at) }}</span>
          </RouterLink>
        </div>
      </div>

      <!-- Random walk -->
      <div v-if="store.randomCards.length" class="section fade-up" style="animation-delay: 0.45s">
        <h2 class="section-title">
          <Shuffle :size="14" />
          随机漫步
        </h2>
        <div class="card-list">
          <RouterLink
            v-for="card in store.randomCards"
            :key="card.uuid"
            :to="{ name: 'card', params: { uuid: card.uuid } }"
            class="list-card"
          >
            <span class="list-card-title">{{ card.title }}</span>
            <span v-if="card.category" class="list-card-cat">{{ card.category }}</span>
          </RouterLink>
        </div>
      </div>

      <!-- Graph link -->
      <RouterLink to="/graph" class="graph-link fade-up" style="animation-delay: 0.6s">
        <Compass :size="16" />
        <span>全局图谱</span>
      </RouterLink>
    </div>
  </div>
</template>

<style scoped>
.welcome {
  display: flex;
  justify-content: center;
  min-height: 100vh;
  padding: 0 24px;
}

.welcome-inner {
  max-width: 520px;
  width: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 60px 0;
}

/* ── Hero ────────────────────────────────────── */
.hero {
  text-align: center;
  margin-bottom: 48px;
  position: relative;
}

.breath-circle {
  width: 120px;
  height: 120px;
  border-radius: 50%;
  border: 1px solid var(--accent-dim);
  margin: 0 auto 28px;
  position: relative;
  animation: breathe 4s ease-in-out infinite;
  opacity: 0.4;
}

.breath-circle::after {
  content: '';
  position: absolute;
  inset: 12px;
  border-radius: 50%;
  border: 1px solid var(--accent);
  opacity: 0.3;
}

.welcome-title {
  font-family: var(--font-display);
  font-size: 2.8rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  color: var(--text-primary);
  margin: 0 0 12px;
  line-height: 1.1;
}

.welcome-sub {
  font-size: 0.9rem;
  color: var(--text-muted);
  margin: 0;
  letter-spacing: 0.1em;
}

/* ── Search ──────────────────────────────────── */
.search-box {
  display: flex;
  align-items: center;
  gap: 10px;
  background: var(--ms-surface);
  border: 1px solid var(--ms-border);
  border-radius: 8px;
  padding: 14px 16px;
  margin-bottom: 48px;
  transition: border-color var(--duration-normal), box-shadow var(--duration-normal);
}

.search-box:focus-within {
  border-color: var(--accent-dim);
  box-shadow: 0 0 0 3px var(--accent-glow);
}

.search-icon {
  color: var(--accent-dim);
  flex-shrink: 0;
  transition: color var(--duration-fast);
}

.search-box:focus-within .search-icon {
  color: var(--accent);
}

.search-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  font-size: 1rem;
  color: var(--text-primary);
  font-family: var(--font-serif);
}

.search-input::placeholder {
  color: var(--text-muted);
}

.search-kbd {
  font-family: var(--font-mono);
  font-size: 0.6rem;
  color: var(--text-muted);
  padding: 2px 6px;
  border: 1px solid var(--ms-border);
  border-radius: 4px;
  flex-shrink: 0;
}

/* ── Sections ────────────────────────────────── */
.section {
  margin-bottom: 36px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: var(--font-mono);
  font-size: 0.68rem;
  font-weight: 500;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-muted);
  margin: 0 0 14px;
}

/* ── Card list ───────────────────────────────── */
.card-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.list-card {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  padding: 10px 14px;
  background: var(--ms-surface);
  border: 1px solid var(--ms-border);
  border-radius: 6px;
  text-decoration: none;
  color: var(--text-primary);
  gap: 12px;
  transition: border-color var(--duration-fast) var(--ease-gentle);
}

.list-card:hover {
  border-color: var(--accent-dim);
  color: var(--text-primary);
  text-decoration: none;
}

.list-card-title {
  font-size: 0.9rem;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.list-card-time,
.list-card-cat {
  font-size: 0.7rem;
  color: var(--text-muted);
  font-family: var(--font-mono);
  letter-spacing: 0.03em;
  flex-shrink: 0;
}

.list-card-cat {
  color: var(--accent-dim);
}

/* ── Graph link ──────────────────────────────── */
.graph-link {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin-top: 16px;
  padding: 12px;
  color: var(--accent-dim);
  font-family: var(--font-mono);
  font-size: 0.8rem;
  letter-spacing: 0.06em;
  text-decoration: none;
  border: 1px solid transparent;
  border-radius: 6px;
  transition: color var(--duration-fast), border-color var(--duration-fast);
}

.graph-link:hover {
  color: var(--accent);
  border-color: var(--ms-border);
  text-decoration: none;
}
</style>
