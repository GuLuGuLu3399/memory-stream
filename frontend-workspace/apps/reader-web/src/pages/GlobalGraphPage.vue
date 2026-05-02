<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowLeft, Search } from 'lucide-vue-next'
import { GraphView } from '@memory-stream/ui-shared'
import { useReaderStore } from '@/stores/reader'

const router = useRouter()
const store = useReaderStore()
const highlightQuery = ref('')

const nodeCount = computed(() => store.fullGraph?.nodes.length ?? 0)
const edgeCount = computed(() => store.fullGraph?.edges.length ?? 0)

onMounted(() => {
  if (!store.fullGraph) store.loadFullGraph()
})
</script>

<template>
  <div class="graph-page fade-up">
    <header class="graph-header">
      <button class="icon-btn" @click="router.push('/')">
        <ArrowLeft :size="18" />
      </button>
      <div class="header-search">
        <Search :size="14" class="search-icon" />
        <input
          v-model="highlightQuery"
          type="text"
          placeholder="高亮节点..."
          class="search-input"
        />
      </div>
    </header>

    <div class="graph-viewport">
      <span v-if="store.fullGraph" class="graph-watermark">GLOBAL STARMAP</span>

      <div v-if="store.fullGraph" class="graph-canvas">
        <GraphView
          :nodes="store.fullGraph.nodes"
          :edges="store.fullGraph.edges"
          :read-only="true"
          @node-click="(id: string) => router.push({ name: 'card', params: { uuid: id } })"
        />
      </div>
      <div v-else class="graph-loading">
        <div class="breath-loader" aria-hidden="true" />
        <span class="loading-text">正在绘制图谱...</span>
      </div>

      <!-- HUD -->
      <div v-if="store.fullGraph" class="graph-hud">
        <div class="hud-item">{{ nodeCount }} 节点</div>
        <div class="hud-item">{{ edgeCount }} 连接</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.graph-page {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--ms-void);
  position: relative;
}

.graph-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--ms-border);
  background: color-mix(in oklch, var(--ms-deep) 85%, transparent);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  z-index: 10;
}

.header-search {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--ms-surface);
  border: 1px solid var(--ms-border);
  border-radius: 6px;
  padding: 6px 10px;
  max-width: 300px;
  transition: border-color var(--duration-normal), box-shadow var(--duration-normal);
}

.header-search:focus-within {
  border-color: var(--neon-dim);
  box-shadow: 0 0 0 2px var(--neon-dim);
}

.search-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  font-size: 0.85rem;
  color: var(--text-primary);
  font-family: var(--font-serif);
}

.search-input::placeholder {
  color: var(--text-muted);
}

/* ── Graph viewport ──────────────────────────── */
.graph-viewport {
  flex: 1;
  position: relative;
  overflow: hidden;
}

.graph-canvas {
  position: absolute;
  inset: 0;
}

.graph-watermark {
  position: absolute;
  top: 7px;
  left: 8px;
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.1em;
  color: var(--text-muted);
  opacity: 0.2;
  pointer-events: none;
  z-index: 1;
}

/* ── Tauri-style graph overrides ─────────── */

.graph-canvas :deep(.graph-view-outer),
.graph-canvas :deep(.graph-view-inner),
.graph-canvas :deep(.vue-flow) {
  background: transparent;
}

/* Nodes — carbon dark with border-light */
.graph-canvas :deep(.vue-flow__node) {
  background: var(--ms-carbon);
  border: 1px solid var(--ms-border-light);
  border-radius: 2px;
  font-family: var(--font-serif);
  color: var(--text-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);
  transition: border-color 200ms cubic-bezier(0.25, 1, 0.5, 1),
    box-shadow 200ms cubic-bezier(0.25, 1, 0.5, 1);
}

.graph-canvas :deep(.vue-flow__node:hover) {
  border-color: var(--neon);
  box-shadow: 0 0 12px oklch(0.78 0.17 200 / 0.12);
}

.graph-canvas :deep(.vue-flow__node.selected) {
  border-color: var(--neon);
  box-shadow: 0 0 16px oklch(0.78 0.17 200 / 0.3);
}

/* Dimmed / highlighted nodes */
.graph-canvas :deep(.vue-flow__node.node-dimmed) {
  opacity: 0.3;
  transition: opacity var(--duration-fast) var(--ease-gentle);
}

.graph-canvas :deep(.vue-flow__node.node-highlighted) {
  border-color: var(--neon);
  box-shadow: 0 0 20px oklch(0.78 0.17 200 / 0.3);
  transition: border-color var(--duration-fast) var(--ease-gentle),
    box-shadow var(--duration-fast) var(--ease-gentle);
}

/* Trunk edges — neon steel blue */
.graph-canvas :deep(.vue-flow__edge:not(.link-breathing) .vue-flow__edge-path) {
  stroke: var(--neon) !important;
}

/* Link edges — breathing between muted and brass */
.graph-canvas :deep(.vue-flow__edge.link-breathing .vue-flow__edge-path) {
  animation: link-breathe 3s ease-in-out infinite;
  will-change: stroke;
}

@keyframes link-breathe {
  0%, 100% { stroke: var(--text-muted); }
  50%      { stroke: var(--brass); }
}

/* Background — transparent */
.graph-canvas :deep(.vue-flow__background) {
  background: transparent;
}

/* Controls — dark themed with neon accents */
.graph-canvas :deep(.vue-flow__panel) {
  background: var(--ms-void);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  color: var(--text-secondary);
}

.graph-canvas :deep(.vue-flow__controls-button) {
  background: var(--ms-void);
  border-bottom: 1px solid var(--ms-border);
  fill: var(--text-secondary);
}

.graph-canvas :deep(.vue-flow__controls-button:hover) {
  background: var(--ms-surface);
  fill: var(--neon);
}

.graph-canvas :deep(.vue-flow__attribution) {
  display: none;
}

/* Subtle vignette */
.graph-viewport::after {
  content: '';
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 1;
  background: radial-gradient(ellipse at center, transparent 40%, var(--ms-void) 100%);
}

/* ── Loading ─────────────────────────────────── */
.graph-loading {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 20px;
}

.breath-loader {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  border: 1px solid var(--neon-dim);
  animation: breathe 2s ease-in-out infinite;
}

.loading-text {
  color: var(--text-muted);
  font-size: 0.82rem;
  font-family: var(--font-mono);
  letter-spacing: 0.06em;
}

/* ── HUD ─────────────────────────────────────── */
.graph-hud {
  position: absolute;
  top: 12px;
  right: 12px;
  z-index: 2;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 12px;
  background: color-mix(in oklch, var(--ms-void) 85%, transparent);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
}

.hud-item {
  font-family: var(--font-mono);
  font-size: 9px;
  color: var(--neon);
  letter-spacing: 0.06em;
  white-space: nowrap;
  opacity: 0.6;
}
</style>
