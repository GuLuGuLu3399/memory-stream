<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Search, ArrowLeft } from 'lucide-vue-next'
import { useReaderStore } from '@/stores/reader'

const route = useRoute()
const router = useRouter()
const store = useReaderStore()
const query = ref(typeof route.query.q === 'string' ? route.query.q : '')

watch(() => route.query.q, (q) => {
  const s = typeof q === 'string' ? q : ''
  query.value = s
  if (s) store.doSearch(s)
}, { immediate: true })

function handleSearch() {
  const q = query.value.trim()
  if (q) router.push({ name: 'search', query: { q } })
}
</script>

<template>
  <div class="search-page fade-up">
    <header class="search-header">
      <button class="icon-btn" @click="router.push('/')">
        <ArrowLeft :size="18" />
      </button>
      <form class="search-box" @submit.prevent="handleSearch">
        <Search :size="16" class="search-icon" />
        <input
          v-model="query"
          type="text"
          placeholder="搜索笔记..."
          class="search-input"
        />
      </form>
    </header>

    <!-- Result count -->
    <p v-if="store.searchResults.length > 0 && query" class="result-count">
      找到 {{ store.searchResults.length }} 个结果
    </p>

    <div class="results">
      <div v-if="store.searchResults.length === 0 && query" class="empty">
        <div class="empty-line" aria-hidden="true" />
        <p>无匹配结果</p>
      </div>
      <div class="result-list">
        <RouterLink
          v-for="(item, i) in store.searchResults"
          :key="item.uuid"
          :to="{ name: 'card', params: { uuid: item.uuid } }"
          class="result-card fade-up"
          :style="{ animationDelay: `${i * 0.04}s` }"
        >
          <span class="result-title">{{ item.title }}</span>
          <span v-if="item.excerpt" class="result-excerpt">{{ item.excerpt }}</span>
        </RouterLink>
      </div>
    </div>
  </div>
</template>

<style scoped>
.search-page {
  max-width: 640px;
  margin: 0 auto;
  padding: 0 24px;
}

.search-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px 0;
  border-bottom: 1px solid var(--ms-border);
}

.search-box {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--ms-surface);
  border: 1px solid var(--ms-border);
  border-radius: 8px;
  padding: 12px 14px;
  transition: border-color var(--duration-normal), box-shadow var(--duration-normal);
}

.search-box:focus-within {
  border-color: var(--accent-dim);
  box-shadow: 0 0 0 3px var(--accent-glow);
}

.search-icon {
  color: var(--accent-dim);
  flex-shrink: 0;
}

.search-box:focus-within .search-icon {
  color: var(--accent);
}

.search-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  font-size: 0.95rem;
  color: var(--text-primary);
  font-family: var(--font-serif);
}

.search-input::placeholder {
  color: var(--text-muted);
}

/* ── Results ─────────────────────────────────── */
.result-count {
  font-family: var(--font-mono);
  font-size: 0.7rem;
  color: var(--accent-dim);
  letter-spacing: 0.06em;
  margin: 16px 0 12px;
  text-align: center;
}

.results {
  padding-bottom: 60px;
}

.empty {
  text-align: center;
  padding: 56px 0;
}

.empty p {
  color: var(--text-muted);
  font-size: 0.9rem;
  margin: 0;
}

.result-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.result-card {
  display: block;
  text-decoration: none;
  color: var(--text-primary);
  padding: 12px 16px;
  background: var(--ms-surface);
  border: 1px solid var(--ms-border);
  border-radius: 6px;
  border-left: 2px solid transparent;
  transition: border-color var(--duration-fast);
}

.result-card:hover {
  border-left-color: var(--accent);
  color: var(--text-primary);
  text-decoration: none;
}

.result-title {
  display: block;
  font-size: 0.95rem;
  font-weight: 500;
  margin-bottom: 4px;
}

.result-excerpt {
  display: block;
  font-size: 0.8rem;
  color: var(--text-secondary);
  line-height: 1.5;
}
</style>
