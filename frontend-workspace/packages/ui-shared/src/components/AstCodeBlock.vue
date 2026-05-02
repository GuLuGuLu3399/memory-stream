// ────────────────────────────────────────────────────────────────
// AstCodeBlock.vue — renders code blocks with syntax highlighting or Mermaid diagrams
// AstCodeBlock.vue — 使用语法高亮或 Mermaid 图表渲染代码
// ────────────────────────────────────────────────────────────────

<script setup lang="ts">
import { ref, computed, watch, nextTick, onBeforeUnmount } from 'vue'
import mermaid from 'mermaid'
import { codeToHtml } from 'shiki'
import DOMPurify from 'dompurify'

function debounce<T extends (...args: any[]) => void>(fn: T, ms: number) {
  let timer: ReturnType<typeof setTimeout> | null = null
  const wrapped = (...args: Parameters<T>) => {
    if (timer) clearTimeout(timer)
    timer = setTimeout(() => { timer = null; fn(...args) }, ms)
  }
  wrapped.cancel = () => { if (timer) { clearTimeout(timer); timer = null } }
  return wrapped
}

let mermaidInitialized = false

const props = defineProps<{
  language: string | null
  value: string
}>()

const isMermaid = computed(() => props.language?.toLowerCase() === 'mermaid')

const complexityBadge = computed(() => {
  const algoLangs = ['c', 'cpp', 'python', 'java', 'rust', 'go', 'js', 'ts', 'javascript', 'typescript']
  if (!props.language || !algoLangs.includes(props.language.toLowerCase())) return null
  const timeMatch = props.value.match(/T=O\([^)]+\)/)
  const spaceMatch = props.value.match(/S=O\([^)]+\)/)
  if (!timeMatch && !spaceMatch) return null
  return {
    time: timeMatch?.[0] ?? null,
    space: spaceMatch?.[0] ?? null,
  }
})

// Unique ID per component instance — prevents DOM ID collisions
const uid = `mermaid-${Math.random().toString(36).slice(2, 10)}-${Date.now().toString(36)}`
const mermaidSvg = ref('')
const mermaidError = ref('')

const mermaidIframeSrc = computed(() => {
  if (!mermaidSvg.value) return ''
  const style = [
    'body{margin:0;background:transparent}',
    'svg{max-width:100%;height:auto}',
    '.nodeLabel,.edgeLabel,text,tspan{fill:#e8dfd0!important;color:#e8dfd0!important}',
    '.edgeLabel{background-color:#141414!important}',
    '.label,foreignObject div,foreignObject span,foreignObject p{color:#e8dfd0!important;text-shadow:none!important}',
    '.title{fill:#00e5ff!important;color:#00e5ff!important}',
  ].join('')
  return `<!DOCTYPE html><html><head><style>${style}</style></head><body>${mermaidSvg.value}</body></html>`
})
const highlightedHtml = ref('')
const isRendering = ref(false)

function isMermaidViable(code: string): boolean {
  const trimmed = code.trim()
  if (trimmed.length < 10) return false
  if (!/^(graph|flowchart|sequenceDiagram|classDiagram|stateDiagram|erDiagram|gantt|pie|gitGraph|journey|mindmap|timeline|quadrantChart|sankey|xychart|block)/.test(trimmed)) return false
  const opens = (trimmed.match(/{/g) || []).length
  const closes = (trimmed.match(/}/g) || []).length
  if (opens > closes + 1) return false
  return true
}

// ── Copy button state ──
const copied = ref(false)
let copyTimer: ReturnType<typeof setTimeout> | null = null

// ── Lightbox state ──
const lightboxOpen = ref(false)
const zoomLevel = ref(1)

function ensureMermaidInit() {
  if (mermaidInitialized) return
  mermaid.initialize({
    startOnLoad: false,
    theme: 'base',
    securityLevel: 'loose',
    themeVariables: {
      background: 'transparent',
      primaryColor: '#1a1a1a',
      primaryBorderColor: '#00e5ff',
      lineColor: '#5a4f3e',
      secondaryColor: '#141414',
      tertiaryColor: '#0d0d0d',
      fontFamily: 'Space Grotesk, system-ui, sans-serif',
      fontSize: '13px',
    },
  })
  mermaidInitialized = true
}

function cleanupPhantom() {
  const el = document.getElementById(`d${uid}`)
  if (el) el.remove()
}

const debouncedMermaidRender = debounce(async (value: string) => {
  try {
    isRendering.value = true
    mermaidError.value = ''
    const { svg } = await mermaid.render(uid, value)
    cleanupPhantom()
    mermaidSvg.value = svg
  } catch (e) {
    if (!mermaidSvg.value) {
      mermaidError.value = e instanceof Error ? e.message : 'Mermaid 渲染失败'
    }
    cleanupPhantom()
  } finally {
    isRendering.value = false
  }
}, 800)

watch(
  [isMermaid, () => props.value],
  async ([isM, value]) => {
    if (isM) {
      ensureMermaidInit()
      if (!isMermaidViable(value)) return
      cleanupPhantom()
      await nextTick()
      debouncedMermaidRender(value)
    } else {
      try {
        highlightedHtml.value = await codeToHtml(value, {
          lang: props.language || 'text',
          theme: 'github-dark',
        })
      } catch {
        const escaped = DOMPurify.sanitize(value, { ALLOWED_TAGS: [] })
        highlightedHtml.value = `<pre style="background:#141414;color:#e8dfd0;padding:16px 20px;font-family:'JetBrains Mono','Fira Code',monospace"><code>${escaped}</code></pre>`
      }
    }
  },
  { immediate: true },
)

async function handleCopy() {
  await navigator.clipboard.writeText(props.value)
  copied.value = true
  if (copyTimer) clearTimeout(copyTimer)
  copyTimer = setTimeout(() => { copied.value = false }, 2000)
}

const panX = ref(0)
const panY = ref(0)

function zoomIn() {
  zoomLevel.value = Math.min(zoomLevel.value * 1.25, 5)
}

function zoomOut() {
  zoomLevel.value = Math.max(zoomLevel.value / 1.25, 0.25)
}

function onLightboxWheel(e: WheelEvent) {
  e.preventDefault()
  if (e.ctrlKey || e.metaKey) {
    const delta = e.deltaY > 0 ? 0.85 : 1.18
    zoomLevel.value = Math.min(Math.max(zoomLevel.value * delta, 0.25), 5)
  } else if (e.shiftKey) {
    panX.value -= e.deltaY
  } else {
    panY.value -= e.deltaY
  }
}

function closeLightbox() {
  lightboxOpen.value = false
  zoomLevel.value = 1
  panX.value = 0
  panY.value = 0
}

watch(lightboxOpen, (v) => { if (v) { zoomLevel.value = 1; panX.value = 0; panY.value = 0 } })

onBeforeUnmount(() => {
  debouncedMermaidRender.cancel()
  cleanupPhantom()
  if (copyTimer) clearTimeout(copyTimer)
})
</script>

<template>
  <div class="code-container">
    <!-- Mermaid: success -->
    <div v-if="isMermaid && mermaidSvg" class="mermaid" :class="{ 'is-rendering-graph': isRendering }" @dblclick="lightboxOpen = true">
      <iframe v-if="mermaidSvg" class="mermaid-iframe" :srcdoc="mermaidIframeSrc" sandbox="allow-scripts" />
      <button class="expand-btn" @click="lightboxOpen = true" title="展开图谱" aria-label="展开图谱">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15 3 21 3 21 9" /><polyline points="9 21 3 21 3 15" />
          <line x1="21" y1="3" x2="14" y2="10" /><line x1="3" y1="21" x2="10" y2="14" />
        </svg>
      </button>
    </div>

    <!-- Mermaid: error -->
    <div v-else-if="isMermaid && mermaidError" class="mermaid-error">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" /></svg>
      <span>{{ mermaidError }}</span>
    </div>

    <!-- Mermaid: loading -->
    <div v-else-if="isMermaid" class="mermaid-loading">
      <span class="animate-neon-pulse">Rendering...</span>
    </div>

    <!-- Code block with copy -->
    <div v-else class="code-inner">
      <div class="shiki-wrapper" v-html="highlightedHtml" />
      <span v-if="complexityBadge" class="complexity-badge">
        <span v-if="complexityBadge.time" class="complexity-tag">{{ complexityBadge.time }}</span>
        <span v-if="complexityBadge.space" class="complexity-tag">{{ complexityBadge.space }}</span>
      </span>
      <button class="copy-btn" :class="{ 'copy-ok': copied }" @click="handleCopy" title="复制代码" aria-label="复制代码">
        <!-- Copy icon -->
        <svg v-if="!copied" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2" /><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
        </svg>
        <!-- Check icon -->
        <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="20 6 9 17 4 12" />
        </svg>
      </button>
    </div>
  </div>

  <!-- Mermaid Lightbox -->
  <Teleport to="body">
    <div v-if="lightboxOpen" class="lightbox-overlay" @click.self="closeLightbox">
      <div class="lightbox-controls">
        <button @click="zoomIn" title="放大" aria-label="放大">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" /></svg>
        </button>
        <button @click="zoomOut" title="缩小" aria-label="缩小">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12" /></svg>
        </button>
        <button @click="closeLightbox" title="关闭" aria-label="关闭">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
        </button>
      </div>
      <div class="lightbox-canvas" :style="{ transform: `translate(${panX}px, ${panY}px) scale(${zoomLevel})` }" @wheel.prevent="onLightboxWheel">
        <iframe v-if="mermaidSvg" class="lightbox-iframe" :srcdoc="mermaidIframeSrc" sandbox="allow-scripts" />
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
/* ── Code container ── */
.code-container {
  margin: 1em 0;
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  overflow-x: auto;
  box-shadow: inset 0 2px 4px oklch(0 0 0 / 0.4), inset 0 -1px 0 oklch(1 0 0 / 0.03);
}

.code-container::-webkit-scrollbar {
  height: 4px;
}
.code-container::-webkit-scrollbar-thumb {
  background: var(--ms-border-light);
  border-radius: 2px;
}

/* ── Shiki code block ── */
.code-inner {
  position: relative;
}

.shiki-wrapper :deep(pre) {
  margin: 0;
  padding: 16px 20px;
  padding-right: 2.5em;
  background: var(--ms-carbon) !important;
  font-family: var(--font-mono);
  font-size: 0.88em;
  line-height: 1.6;
  overflow-x: auto;
}

.shiki-wrapper :deep(code) {
  font-family: var(--font-mono);
}

/* ── Copy button ── */
.complexity-badge {
  position: absolute;
  top: 6px;
  right: 36px;
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 150ms cubic-bezier(0.33, 0, 0.2, 1);
}

.code-inner:hover .complexity-badge {
  opacity: 1;
}

.complexity-tag {
  font-family: var(--font-mono, monospace);
  font-size: 0.72em;
  padding: 0.15em 0.4em;
  background: oklch(0.18 0.03 75);
  border: 1px solid oklch(0.32 0.05 75);
  border-radius: 2px;
  color: #d4a017;
  letter-spacing: 0.02em;
}

.copy-btn {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 24px;
  height: 24px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid transparent;
  border-radius: 2px;
  background: transparent;
  color: var(--ms-smoke, #5a4f3e);
  cursor: pointer;
  opacity: 0;
  transition:
    opacity 150ms cubic-bezier(0.33, 0, 0.2, 1),
    color 150ms cubic-bezier(0.33, 0, 0.2, 1),
    border-color 150ms cubic-bezier(0.33, 0, 0.2, 1),
    transform 100ms cubic-bezier(0.68, -0.3, 0.32, 1.3);
}

.code-inner:hover .copy-btn {
  opacity: 1;
}

.copy-btn:hover {
  color: var(--text-secondary, #888);
  border-color: var(--ms-border-light, #2a2a2a);
}

.copy-btn:active {
  transform: scale(0.85);
}

.copy-btn.copy-ok {
  opacity: 1;
  color: #00e5ff;
  border-color: #00e5ff;
}

/* ── Mermaid ── */
.mermaid {
  position: relative;
  display: flex;
  justify-content: center;
  background: transparent;
  padding: 16px 20px;
  cursor: default;
}

.mermaid-iframe {
  width: 100%;
  border: none;
  background: transparent;
  min-height: 200px;
  display: flex;
  justify-content: center;
}

.mermaid.is-rendering-graph::after {
  content: '';
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    to bottom,
    transparent 0,
    transparent 2px,
    oklch(0.55 0.12 75 / 0.08) 2px,
    oklch(0.55 0.12 75 / 0.08) 4px
  );
  animation: mermaid-scan 1.5s linear infinite;
  pointer-events: none;
}

@keyframes mermaid-scan {
  0% { opacity: 0.6; }
  50% { opacity: 0.2; }
  100% { opacity: 0.6; }
}

/* ── Expand button ── */
.expand-btn {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 24px;
  height: 24px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid transparent;
  border-radius: 2px;
  background: transparent;
  color: var(--ms-smoke, #5a4f3e);
  cursor: pointer;
  opacity: 0;
  transition:
    opacity 150ms cubic-bezier(0.33, 0, 0.2, 1),
    color 150ms cubic-bezier(0.33, 0, 0.2, 1),
    border-color 150ms cubic-bezier(0.33, 0, 0.2, 1);
}

.mermaid:hover .expand-btn {
  opacity: 1;
}

.expand-btn:hover {
  color: #00e5ff;
  border-color: var(--ms-border-light, #2a2a2a);
}

/* ── Mermaid error ── */
.mermaid-error {
  display: flex;
  align-items: center;
  gap: 0.5em;
  padding: 16px 20px;
  color: #e53935;
  font-family: var(--font-mono, monospace);
  font-size: 0.88em;
  white-space: pre-wrap;
  background: var(--ms-carbon, #141414);
}

/* ── Mermaid loading ── */
.mermaid-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2em;
  color: var(--text-muted, #555);
  font-family: var(--font-sans, sans-serif);
  font-size: 0.88em;
}

/* ── Lightbox ── */
.lightbox-overlay {
  position: fixed;
  inset: 0;
  z-index: 400;
  background: #050505;
  background-image: radial-gradient(circle, oklch(1 0 0 / 0.06) 1px, transparent 1px);
  background-size: 24px 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fade-in 250ms cubic-bezier(0.33, 0, 0.2, 1) both;
}

@keyframes fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

.lightbox-controls {
  position: absolute;
  top: 12px;
  right: 12px;
  display: flex;
  gap: 4px;
  z-index: 1;
}

.lightbox-controls button {
  width: 28px;
  height: 28px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid #1e1e1e;
  border-radius: 2px;
  background: #050505;
  color: #888;
  cursor: pointer;
  transition: border-color 150ms, color 150ms;
}

.lightbox-controls button:hover {
  border-color: #00e5ff;
  color: #00e5ff;
}

.lightbox-canvas {
  max-width: 95vw;
  max-height: 90vh;
  overflow: auto;
  transition: transform 200ms cubic-bezier(0.33, 0, 0.2, 1);
}

.lightbox-iframe {
  width: 100%;
  border: none;
  background: transparent;
  min-height: 300px;
}
</style>
