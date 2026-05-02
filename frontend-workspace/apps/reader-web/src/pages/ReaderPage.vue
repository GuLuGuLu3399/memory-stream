<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch, computed, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, List, Info, Share2 } from 'lucide-vue-next'
import { AstRenderer, GraphView, TermPopup } from '@memory-stream/ui-shared'
import { useReaderStore } from '@/stores/reader'
import { useLayoutStore } from '@/stores/layout'
import { useGlossaryStore } from '@/stores/glossary'
import { resolveCardByTitle } from '@/api/cards'
import type { TocNode } from '@memory-stream/types'

const route = useRoute()
const router = useRouter()
const store = useReaderStore()
const layout = useLayoutStore()
const glossary = useGlossaryStore()

const articleRef = ref<HTMLElement | null>(null)
const activeHeading = ref('')
const readingProgress = ref(0)

// ── Auto-hide topbar ──────────────────────────
const topbarVisible = ref(true)
const outlineOpen = ref(false)
let hideTimer: ReturnType<typeof setTimeout> | null = null
let outlineHoverTimer: ReturnType<typeof setTimeout> | null = null
let lastScrollY = 0
let mouseY = 0

function showTopbar() {
  topbarVisible.value = true
  if (hideTimer) clearTimeout(hideTimer)
}

function scheduleHide() {
  if (hideTimer) clearTimeout(hideTimer)
  hideTimer = setTimeout(() => {
    if (mouseY > 50) topbarVisible.value = false
  }, 2000)
}

function handleMouseMove(e: MouseEvent) {
  mouseY = e.clientY

  // Outline: open on left edge hover (desktop, 300ms delay)
  if (e.clientX < 20 && !outlineOpen.value && flatToc.value.length > 0 && window.innerWidth >= 769) {
    if (!outlineHoverTimer) {
      outlineHoverTimer = setTimeout(() => {
        outlineOpen.value = true
        outlineHoverTimer = null
      }, 300)
    }
  } else if (outlineHoverTimer) {
    clearTimeout(outlineHoverTimer)
    outlineHoverTimer = null
  }

  // Topbar
  if (e.clientY < 50) {
    showTopbar()
  } else {
    scheduleHide()
  }
}

function handlePageScroll() {
  const scrollY = window.scrollY

  // Reading progress
  const el = articleRef.value
  if (el) {
    const docHeight = el.scrollHeight - window.innerHeight
    readingProgress.value = docHeight > 0 ? Math.min((scrollY / docHeight) * 100, 100) : 0
  }

  // Auto-hide: show on scroll up, hide on scroll down
  if (scrollY < lastScrollY - 5) {
    showTopbar()
    scheduleHide()
  } else if (scrollY > lastScrollY + 20) {
    topbarVisible.value = false
  }
  lastScrollY = scrollY
}

function handleReaderKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'o') {
    e.preventDefault()
    outlineOpen.value = !outlineOpen.value
  }
  if (e.key === 'Escape' && outlineOpen.value) {
    outlineOpen.value = false
  }
}

// ── Card loading ──────────────────────────────
const uuid = computed(() => route.params.uuid as string)

function saveRecent() {
  if (!store.currentCard) return
  try {
    const raw = localStorage.getItem('reader_recent')
    const list: { uuid: string; title: string; updated_at: string }[] = raw ? JSON.parse(raw) : []
    const filtered = list.filter(i => i.uuid !== store.currentCard!.uuid)
    filtered.unshift({
      uuid: store.currentCard.uuid,
      title: store.currentCard.title,
      updated_at: store.currentCard.updated_at,
    })
    localStorage.setItem('reader_recent', JSON.stringify(filtered.slice(0, 20)))
  } catch (e) { console.warn('Failed to save recent:', e) }
}

async function loadCard() {
  if (!uuid.value) return
  await store.loadCard(uuid.value)
  store.loadGraph(uuid.value).catch(e => console.warn('Failed to load graph:', e))
  store.loadBacklinks(uuid.value).catch(e => console.warn('Failed to load backlinks:', e))
  saveRecent()
  nextTick(() => observeHeadings())
}

onMounted(() => {
  loadCard()
  scheduleHide()
  window.addEventListener('mousemove', handleMouseMove)
  window.addEventListener('scroll', handlePageScroll, { passive: true })
  window.addEventListener('keydown', handleReaderKeydown)
})

watch(uuid, loadCard)

onUnmounted(() => {
  observer?.disconnect()
  window.removeEventListener('mousemove', handleMouseMove)
  window.removeEventListener('scroll', handlePageScroll)
  window.removeEventListener('keydown', handleReaderKeydown)
  if (hideTimer) clearTimeout(hideTimer)
  if (outlineHoverTimer) clearTimeout(outlineHoverTimer)
})

// ── Wiki navigation ───────────────────────────
const wikiNotFoundMsg = ref('')

async function handleWikiNavigate(target: string) {
  const resolvedUuid = await resolveCardByTitle(target)
  if (resolvedUuid) {
    router.push({ name: 'card', params: { uuid: resolvedUuid } })
  } else {
    wikiNotFoundMsg.value = `未找到「${target}」`
    setTimeout(() => { wikiNotFoundMsg.value = '' }, 2500)
  }
}

// ── Concept popup ─────────────────────────────
const termPopup = ref<{ term: string; definition: string; x: number; y: number } | null>(null)

function handleConceptRefHover(payload: { term: string; x: number; y: number }) {
  const definition = glossary.getDefinition(payload.term)
  if (definition) {
    termPopup.value = { term: payload.term, definition, x: payload.x, y: payload.y }
  }
}

function handleConceptRefLeave() {
  termPopup.value = null
}

// ── Heading tracking ──────────────────────────
let observer: IntersectionObserver | null = new IntersectionObserver(
  (entries) => {
    for (const entry of entries) {
      if (entry.isIntersecting) {
        activeHeading.value = entry.target.id
      }
    }
  },
  { rootMargin: '-56px 0px -60% 0px' },
)

function observeHeadings() {
  if (!observer || !articleRef.value) return
  observer.disconnect()
  articleRef.value.querySelectorAll('h1[id], h2[id], h3[id], h4[id]').forEach(h => {
    observer!.observe(h)
  })
}

function scrollToHeading(slug: string) {
  const el = document.getElementById(slug)
  if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

function formatDate(d: string): string {
  return new Date(d).toLocaleDateString('zh-CN', { year: 'numeric', month: 'short', day: 'numeric' })
}

const flatToc = computed(() => {
  const items: { level: number; text: string; slug: string }[] = []
  function walk(nodes: TocNode[]) {
    for (const n of nodes) {
      items.push({ level: n.level, text: n.text, slug: n.slug })
      walk(n.children)
    }
  }
  if (store.currentToc) walk(store.currentToc)
  return items
})
</script>

<template>
  <div class="reader-page">
    <!-- Reading progress (always visible, 1px zen line) -->
    <div class="reading-progress" :style="{ width: readingProgress + '%', opacity: readingProgress > 2 ? 1 : 0 }" />

    <!-- Auto-hiding topbar -->
    <header :class="['reader-topbar', { hidden: !topbarVisible }]">
      <button class="icon-btn" @click="router.push('/')">
        <ArrowLeft :size="18" />
      </button>
      <div class="topbar-center">
        <h1 class="topbar-title">{{ store.currentCard?.title ?? '' }}</h1>
        <span v-if="store.currentCard" class="topbar-date">
          {{ formatDate(store.currentCard.updated_at) }}
        </span>
      </div>
      <div class="topbar-actions">
        <button v-if="flatToc.length" class="icon-btn" title="目录 (Ctrl+O)" @click="outlineOpen = !outlineOpen">
          <List :size="18" />
        </button>
        <button class="icon-btn" title="信息" @click="layout.toggleSheet('meta')">
          <Info :size="18" />
        </button>
        <button class="icon-btn" title="关系图" @click="layout.toggleSheet('graph')">
          <Share2 :size="18" />
        </button>
      </div>
    </header>

    <!-- Main content — centered, generous margins -->
    <main ref="articleRef" class="reader-article fade-up">
      <Suspense>
        <AstRenderer v-if="store.currentAst" :node="store.currentAst" @navigate="handleWikiNavigate"
          @concept-ref-hover="handleConceptRefHover" @concept-ref-leave="handleConceptRefLeave" />
      </Suspense>

      <!-- Backlinks -->
      <section v-if="store.currentBacklinks.length" class="backlinks-section">
        <h3 class="backlinks-title">反向链接</h3>
        <ul class="backlinks-list">
          <li v-for="bl in store.currentBacklinks" :key="bl.uuid">
            <RouterLink :to="{ name: 'card', params: { uuid: bl.uuid } }" class="backlink-link">
              <span class="backlink-badge" :class="bl.relation_type">{{ bl.relation_type === 'trunk' ? '主干' : '链接' }}</span>
              <span class="backlink-title">{{ bl.title }}</span>
            </RouterLink>
          </li>
        </ul>
      </section>
    </main>

    <!-- Desktop outline panel (Ctrl+O / left edge hover) -->
    <Teleport to="body">
      <Transition name="fade">
        <div v-if="outlineOpen" class="outline-backdrop" @click="outlineOpen = false" />
      </Transition>
      <Transition name="outline-slide">
        <aside v-if="outlineOpen" class="outline-panel">
          <div class="outline-header">目录</div>
          <nav class="outline-nav">
            <button v-for="item in flatToc" :key="item.slug"
              :class="['outline-item', `outline-h${item.level}`, { active: activeHeading === item.slug }]"
              @click="scrollToHeading(item.slug); outlineOpen = false">
              <span v-if="activeHeading === item.slug" class="outline-dot" />
              {{ item.text }}
            </button>
          </nav>
        </aside>
      </Transition>
    </Teleport>

    <!-- Mobile bottom sheet panels -->
    <Teleport to="body">
      <Transition name="sheet-backdrop">
        <div v-if="layout.activeSheet" class="sheet-overlay" @click="layout.closeSheet()" />
      </Transition>
      <div :class="['sheet-panel', { open: !!layout.activeSheet }]">
        <div class="sheet-handle" aria-hidden="true" />

        <template v-if="layout.activeSheet === 'toc'">
          <h3 class="sheet-title">目录</h3>
          <button v-for="item in flatToc" :key="item.slug"
            :class="['outline-item', `outline-h${item.level}`, { active: activeHeading === item.slug }]"
            @click="scrollToHeading(item.slug); layout.closeSheet()">
            <span v-if="activeHeading === item.slug" class="outline-dot" />
            {{ item.text }}
          </button>
        </template>

        <template v-if="layout.activeSheet === 'meta' && store.currentCard">
          <h3 class="sheet-title">元数据</h3>
          <div class="meta-row">
            <span class="meta-label">分类</span>
            <RouterLink v-if="store.currentCard.category"
              :to="{ name: 'category', params: { category: store.currentCard.category } }" class="meta-link"
              @click="layout.closeSheet()">
              {{ store.currentCard.category }}
            </RouterLink>
            <span v-else class="meta-value">未分类</span>
          </div>
          <div class="meta-row">
            <span class="meta-label">创建</span>
            <span class="meta-value">{{ formatDate(store.currentCard.created_at) }}</span>
          </div>
          <div class="meta-row">
            <span class="meta-label">更新</span>
            <span class="meta-value">{{ formatDate(store.currentCard.updated_at) }}</span>
          </div>
          <div class="meta-row">
            <span class="meta-label">UUID</span>
            <span class="meta-value mono">{{ store.currentCard.uuid }}</span>
          </div>
        </template>

        <template v-if="layout.activeSheet === 'graph' && store.currentGraph">
          <h3 class="sheet-title">局部图谱</h3>
          <div class="sheet-graph">
            <GraphView :nodes="store.currentGraph.nodes" :edges="store.currentGraph.edges" :read-only="true"
              @node-click="(id: string) => { router.push({ name: 'card', params: { uuid: id } }); layout.closeSheet() }" />
          </div>
        </template>
      </div>
    </Teleport>

    <!-- Concept popup -->
    <Teleport to="body">
      <TermPopup v-if="termPopup" :term="termPopup.term" :definition="termPopup.definition" :x="termPopup.x"
        :y="termPopup.y" @close="termPopup = null" />
    </Teleport>

    <!-- Wiki not found toast -->
    <Transition name="fade">
      <div v-if="wikiNotFoundMsg" class="wiki-toast">{{ wikiNotFoundMsg }}</div>
    </Transition>

    <!-- Mobile floating zen button -->
    <Teleport to="body">
      <button class="zen-fab" @click="layout.toggleSheet('toc')">
        <span class="zen-fab-dot" />
      </button>
    </Teleport>
  </div>
</template>

<style scoped>
.reader-page {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

/* ── Auto-hiding topbar ─────────────────────── */
.reader-topbar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: color-mix(in oklch, var(--ms-deep) 88%, transparent);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border-bottom: 1px solid var(--ms-border);
  z-index: 200;
  transform: translateY(0);
  opacity: 1;
  transition: transform var(--duration-normal) var(--ease-emerge),
    opacity var(--duration-normal) var(--ease-gentle);
}

.reader-topbar.hidden {
  transform: translateY(-100%);
  opacity: 0;
  pointer-events: none;
}

.topbar-center {
  flex: 1;
  min-width: 0;
}

.topbar-title {
  font-family: var(--font-display);
  font-size: 0.95rem;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  letter-spacing: 0.02em;
}

.topbar-date {
  font-size: 0.68rem;
  color: var(--text-muted);
  font-family: var(--font-mono);
  letter-spacing: 0.03em;
}

.topbar-actions {
  display: flex;
  gap: 2px;
}

/* ── Centered reading body ──────────────────── */
.reader-article {
  max-width: var(--reading-width);
  margin: 0 auto;
  padding: 56px 24px 80px;
  font-size: 1rem;
  line-height: 1.8;
  width: 100%;
}

/* ── Desktop outline panel ──────────────────── */
.outline-backdrop {
  position: fixed;
  inset: 0;
  z-index: 299;
  background: oklch(0 0 0 / 0.2);
}

.outline-panel {
  position: fixed;
  left: 0;
  top: 0;
  bottom: 0;
  width: 260px;
  background: color-mix(in oklch, var(--ms-deep) 94%, transparent);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border-right: 1px solid var(--ms-border);
  z-index: 300;
  padding: 56px 16px 32px;
  overflow-y: auto;
}

.outline-header {
  font-family: var(--font-mono);
  font-size: 0.68rem;
  font-weight: 500;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-muted);
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--ms-border);
}

.outline-nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.outline-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.8rem;
  font-family: var(--font-serif);
  color: var(--text-muted);
  cursor: pointer;
  padding: 6px 8px;
  background: none;
  border: none;
  border-radius: 4px;
  text-align: left;
  transition: color var(--duration-fast), background var(--duration-fast);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.outline-item:hover {
  color: var(--text-primary);
  background: var(--ms-surface);
}

.outline-item.active {
  color: var(--accent);
}

.outline-dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--accent);
  flex-shrink: 0;
  animation: breathe 3s ease-in-out infinite;
}

.outline-h2 { padding-left: 8px; }
.outline-h3 { padding-left: 20px; }
.outline-h4 { padding-left: 32px; }

/* ── Outline slide transition ───────────────── */
.outline-slide-enter-active {
  transition: transform var(--duration-normal) var(--ease-emerge);
}

.outline-slide-leave-active {
  transition: transform 200ms var(--ease-gentle);
}

.outline-slide-enter-from,
.outline-slide-leave-to {
  transform: translateX(-100%);
}

/* ── Mobile bottom sheets ───────────────────── */
.sheet-overlay {
  position: fixed;
  inset: 0;
  background: oklch(0 0 0 / 0.4);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  z-index: 90;
}

.sheet-backdrop-enter-active { transition: opacity 0.25s; }
.sheet-backdrop-leave-active { transition: opacity 0.15s; }
.sheet-backdrop-enter-from,
.sheet-backdrop-leave-to { opacity: 0; }

.sheet-panel {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  background: var(--ms-deep);
  border-top: 1px solid var(--ms-border);
  border-radius: 20px 20px 0 0;
  padding: 12px 20px 32px;
  max-height: 70vh;
  overflow-y: auto;
  z-index: 100;
  transform: translateY(100%);
  transition: transform 0.35s cubic-bezier(0.16, 1, 0.3, 1);
}

.sheet-panel.open {
  transform: translateY(0);
}

.sheet-handle {
  width: 36px;
  height: 4px;
  border-radius: 2px;
  background: var(--ms-border-light);
  margin: 0 auto 16px;
}

.sheet-title {
  font-family: var(--font-mono);
  font-size: 0.68rem;
  font-weight: 500;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-muted);
  margin-bottom: 14px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--ms-border);
}

.sheet-graph {
  height: 280px;
  border-radius: 10px;
  overflow: hidden;
  background: var(--ms-void);
  border: 1px solid var(--ms-border);
}

/* ── Metadata ───────────────────────────────── */
.meta-row {
  display: flex;
  justify-content: space-between;
  padding: 10px 0;
  border-bottom: 1px solid var(--ms-border);
}

.meta-label {
  color: var(--text-muted);
  font-size: 0.8rem;
  font-family: var(--font-mono);
}

.meta-value {
  font-size: 0.8rem;
  color: var(--text-secondary);
}

.meta-link {
  font-size: 0.8rem;
  color: var(--accent);
  text-decoration: none;
}

.meta-link:hover {
  text-decoration: underline;
}

.mono {
  font-family: var(--font-mono);
  font-size: 0.7rem;
}

/* ── Backlinks ──────────────────────────────── */
.backlinks-section {
  margin-top: 48px;
  padding-top: 24px;
  border-top: 1px solid var(--ms-border);
}

.backlinks-title {
  font-family: var(--font-mono);
  font-size: 0.68rem;
  font-weight: 500;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--accent-dim);
  margin: 0 0 14px;
}

.backlinks-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.backlink-link {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  text-decoration: none;
  color: var(--text-primary);
  background: var(--ms-surface);
  border: 1px solid var(--ms-border);
  border-radius: 6px;
  border-left: 2px solid transparent;
  transition: border-color var(--duration-fast), background var(--duration-fast);
}

.backlink-link:hover {
  border-left-color: var(--accent);
  background: var(--ms-surface-elevated);
  text-decoration: none;
}

.backlink-badge {
  font-size: 0.62rem;
  font-family: var(--font-mono);
  font-weight: 600;
  letter-spacing: 0.06em;
  padding: 2px 6px;
  border-radius: 2px;
  flex-shrink: 0;
}

.backlink-badge.trunk {
  background: var(--accent-glow);
  color: var(--accent);
}

.backlink-badge.link {
  background: rgba(255, 255, 255, 0.05);
  color: var(--ms-smoke);
}

.backlink-title {
  font-size: 0.88rem;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ── Wiki toast ─────────────────────────────── */
.wiki-toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--ms-surface-elevated);
  color: var(--text-secondary);
  font-size: 0.82rem;
  font-family: var(--font-serif);
  padding: 10px 20px;
  border-radius: 8px;
  border: 1px solid var(--ms-border);
  z-index: 200;
  pointer-events: none;
}

/* ── Mobile FAB ─────────────────────────────── */
.zen-fab {
  position: fixed;
  bottom: 24px;
  right: 24px;
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background: var(--accent);
  border: none;
  cursor: pointer;
  display: none;
  align-items: center;
  justify-content: center;
  z-index: 200;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.3);
  transition: transform var(--duration-normal) var(--ease-gentle);
}

.zen-fab:hover {
  transform: scale(1.05);
}

.zen-fab-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--ms-deep);
}

@media (max-width: 768px) {
  .zen-fab {
    display: flex;
  }
}

/* ── Transitions ────────────────────────────── */
.fade-enter-active,
.fade-leave-active {
  transition: opacity var(--duration-normal);
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
