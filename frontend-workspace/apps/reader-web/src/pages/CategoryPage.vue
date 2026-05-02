<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import { useReaderStore } from '@/stores/reader'

const route = useRoute()
const router = useRouter()
const store = useReaderStore()

const category = computed(() => route.params.category as string)

function formatDate(d: string): string {
  return new Date(d).toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
}

onMounted(() => {
  store.loadCardList(category.value)
})
</script>

<template>
  <div class="category-page fade-up">
    <header class="category-header">
      <button class="icon-btn" @click="router.push('/')">
        <ArrowLeft :size="18" />
      </button>
      <div>
        <h1 class="category-title">{{ category }}</h1>
      </div>
      <span v-if="store.cardList.length" class="card-count">{{ store.cardList.length }} 篇</span>
    </header>

    <div v-if="store.cardList.length" class="card-list">
      <RouterLink
        v-for="(card, i) in store.cardList"
        :key="card.uuid"
        :to="{ name: 'card', params: { uuid: card.uuid } }"
        class="list-card fade-up"
        :style="{ animationDelay: `${i * 0.04}s` }"
      >
        <span class="card-title">{{ card.title }}</span>
        <span v-if="card.excerpt" class="card-excerpt">{{ card.excerpt }}</span>
        <span class="card-date">{{ formatDate(card.updated_at) }}</span>
      </RouterLink>
    </div>

    <div v-if="store.cardList.length === 0 && !store.loading" class="empty">
      <div class="empty-line" aria-hidden="true" />
      <p>该分类下暂无卡片</p>
    </div>

    <div class="archive-footer">
      <RouterLink to="/" class="archive-back">返回</RouterLink>
    </div>
  </div>
</template>

<style scoped>
.category-page {
  max-width: 640px;
  margin: 0 auto;
  padding: 0 24px 60px;
}

.category-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 20px 0 16px;
  border-bottom: 1px solid var(--ms-border);
  margin-bottom: 24px;
}

.category-title {
  font-family: var(--font-display);
  font-size: 1.2rem;
  font-weight: 700;
  letter-spacing: 0.02em;
  margin: 0;
}

.card-count {
  font-family: var(--font-mono);
  font-size: 0.7rem;
  color: var(--accent-dim);
  letter-spacing: 0.06em;
  margin-left: auto;
  padding: 2px 8px;
  border: 1px solid var(--ms-border);
  border-radius: 4px;
}

/* ── Card list ───────────────────────────────── */
.card-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.list-card {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  padding: 12px 16px;
  background: var(--ms-surface);
  border: 1px solid var(--ms-border);
  border-radius: 6px;
  border-left: 2px solid transparent;
  text-decoration: none;
  color: var(--text-primary);
  gap: 8px;
  transition: border-color var(--duration-fast);
}

.list-card:hover {
  border-left-color: var(--accent);
  color: var(--text-primary);
  text-decoration: none;
}

.card-title {
  display: block;
  width: 100%;
  font-size: 0.95rem;
  font-weight: 500;
}

.card-excerpt {
  display: block;
  width: 100%;
  font-size: 0.78rem;
  color: var(--text-secondary);
  line-height: 1.5;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-date {
  font-size: 0.7rem;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

/* ── Empty ───────────────────────────────────── */
.empty {
  text-align: center;
  padding: 56px 0;
}

.empty p {
  color: var(--text-muted);
  font-size: 0.9rem;
  margin: 0;
}

/* ── Footer ──────────────────────────────────── */
.archive-footer {
  margin-top: 32px;
  padding-top: 16px;
  border-top: 1px solid var(--ms-border);
}

.archive-back {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--text-muted);
  letter-spacing: 0.04em;
  text-decoration: none;
  transition: color var(--duration-fast);
}

.archive-back:hover {
  color: var(--accent);
  text-decoration: none;
}
</style>
