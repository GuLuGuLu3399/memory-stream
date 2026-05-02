// 用途：全屏星图视图，渲染完整知识图谱的力导向布局
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { Lock, Unlock } from 'lucide-vue-next'
import { GraphView } from '@memory-stream/ui-shared'
import GraphHud from './GraphHud.vue'
import PaneContextMenu from './PaneContextMenu.vue'
import EdgeContextMenu from './EdgeContextMenu.vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import * as protocol from '@/bridge/protocol'
import * as cardService from '@/services/card'
import { useToast } from '@/composables/core/useToast'
import type { GraphNode, GraphEdge } from '@memory-stream/types'

const emit = defineEmits<{
  navigate: [uuid: string]
}>()

const nodes = ref<GraphNode[]>([])
const edges = ref<GraphEdge[]>([])
const loading = ref(true)
const error = ref('')
const toast = useToast()

const showTrunk = ref(true)
const showLink = ref(true)
const showOrphan = ref(true)

const isLayoutLocked = ref(true)

const contextMenuPos = ref<{ x: number; y: number } | null>(null)
const edgeMenuPos = ref<{ x: number; y: number; source: string; target: string; relation: string } | null>(null)
const selectedEdgeEndpoints = ref<{ source: string; target: string } | null>(null)

const graphViewRef = ref<InstanceType<typeof GraphView> | null>(null)

const filteredEdges = computed(() =>
  edges.value.filter((e) => {
    if (e.relation === 'trunk' && !showTrunk.value) return false
    if (e.relation !== 'trunk' && !showLink.value) return false
    return true
  }),
)

const filteredNodes = computed(() => {
  if (showOrphan.value) return nodes.value
  const connected = new Set<string>()
  for (const e of filteredEdges.value) {
    connected.add(e.source)
    connected.add(e.target)
  }
  return nodes.value.filter((n) => connected.has(n.id))
})

async function loadGraph() {
  try {
    const graph = await protocol.fetchFullGraph()
    const validIds = new Set(graph.nodes.map(n => n.id))
    const cleanNodes = graph.nodes
    const cleanEdges = graph.edges.filter(e => {
      if (!validIds.has(e.source) || !validIds.has(e.target)) {
        if (e.relation === 'trunk') {
          console.warn(`[graph] ghost trunk: ${e.source} → ${e.target}`)
        }
        return false
      }
      return true
    })
    edges.value = []
    nodes.value = cleanNodes
    await nextTick()
    edges.value = cleanEdges
  } catch (e) {
    error.value = e instanceof Error ? e.message : '图谱加载失败'
  } finally {
    loading.value = false
  }
}

async function refreshGraph() {
  try {
    const graph = await protocol.fetchFullGraph()
    const validIds = new Set(graph.nodes.map(n => n.id))
    const cleanNodes = graph.nodes
    const cleanEdges = graph.edges.filter(e => {
      if (!validIds.has(e.source) || !validIds.has(e.target)) {
        if (e.relation === 'trunk') {
          console.warn(`[graph] ghost trunk: ${e.source} → ${e.target}`)
        }
        return false
      }
      return true
    })
    edges.value = []
    nodes.value = cleanNodes
    await nextTick()
    edges.value = cleanEdges
  } catch {
    toast.error('刷新图谱失败')
  }
}

function fullCleanup() {
  contextMenuPos.value = null
  edgeMenuPos.value = null
}

function handleNavigate(uuid: string) {
  emit('navigate', uuid)
}

function handlePaneClick() {
  fullCleanup()
  selectedEdgeEndpoints.value = null
}

function handlePaneContextMenu(event: MouseEvent) {
  event.preventDefault()
  contextMenuPos.value = { x: event.clientX, y: event.clientY }
}

function handleEdgeContextMenu(payload: { source: string; target: string; relation: string; event: MouseEvent }) {
  payload.event.preventDefault()
  edgeMenuPos.value = { x: payload.event.clientX, y: payload.event.clientY, source: payload.source, target: payload.target, relation: payload.relation }
}

async function handleAddTrunk(payload: { source: string; target: string }) {
  const savedPositions = new Map(nodes.value.map(n => [n.id, { x: n.x ?? 0, y: n.y ?? 0 }]))
  try {
    await protocol.createTrunkEdge(payload.source, payload.target)
    await refreshGraph()
    nodes.value = nodes.value.map(n => {
      const saved = savedPositions.get(n.id)
      return saved ? { ...n, x: saved.x, y: saved.y } : n
    })
    toast.success('已连接主干')
  } catch {
    toast.error('创建连接失败')
  }
}

async function handleContextAddNode() {
  contextMenuPos.value = null
  try {
    await cardService.createCard('未命名卡片')
    await refreshGraph()
  } catch {
    toast.error('新建卡片失败')
  }
}

async function handleContextRefresh() {
  contextMenuPos.value = null
  loading.value = true
  await refreshGraph()
  loading.value = false
}

async function handleEdgeReverse() {
  const pos = edgeMenuPos.value
  edgeMenuPos.value = null
  if (!pos) return
  const savedPositions = new Map(nodes.value.map(n => [n.id, { x: n.x ?? 0, y: n.y ?? 0 }]))
  try {
    await protocol.reverseTrunkEdge(pos.source, pos.target)
    await refreshGraph()
    nodes.value = nodes.value.map(n => {
      const saved = savedPositions.get(n.id)
      return saved ? { ...n, x: saved.x, y: saved.y } : n
    })
    toast.success('已反转方向')
  } catch {
    toast.error('反转方向失败')
  }
}

async function handleEdgeDisconnect() {
  const pos = edgeMenuPos.value
  edgeMenuPos.value = null
  if (!pos) return
  const savedPositions = new Map(nodes.value.map(n => [n.id, { x: n.x ?? 0, y: n.y ?? 0 }]))
  try {
    await protocol.deleteTrunkEdge(pos.source, pos.target)
    await refreshGraph()
    nodes.value = nodes.value.map(n => {
      const saved = savedPositions.get(n.id)
      return saved ? { ...n, x: saved.x, y: saved.y } : n
    })
    toast.success('已断开连接')
  } catch {
    toast.error('断开连接失败')
  }
}

async function handleAutoLayout() {
  try {
    graphViewRef.value?.runDagreLayout()
  } catch {
    toast.error('自动布阵失败')
  }
}

function handleLayoutDone(positions: { uuid: string; x: number; y: number }[]) {
  nodes.value = nodes.value.map((n) => {
    const pos = positions.find((p) => p.uuid === n.id)
    return pos ? { ...n, x: pos.x, y: pos.y } : n
  })
  graphViewRef.value?.fitView()
}

function handleEdgeClick(payload: { source: string; target: string; relation: string }) {
  selectedEdgeEndpoints.value = { source: payload.source, target: payload.target }
}

function handlePaneReady() {
  graphViewRef.value?.fitView({ padding: 0.2 })
}

function handleTacticalReset() {
  graphViewRef.value?.fitView({ padding: 0.2, duration: 800 })
}

function handleToggleLock() {
  if (isLayoutLocked.value) {
    isLayoutLocked.value = false
  } else {
    isLayoutLocked.value = true
    const positions = nodes.value.map(n => ({ id: n.id, x: n.x ?? 0, y: n.y ?? 0 }))
    graphViewRef.value?.animateToPositions(positions)
  }
}

let unlistenFs: UnlistenFn | null = null

function dismissOnEsc(e: KeyboardEvent) {
  if (e.key === 'Escape') fullCleanup()
}

onMounted(async () => {
  window.addEventListener('keydown', dismissOnEsc)
  unlistenFs = await listen<{ path: string; kind: string }>('fs:change', (event) => {
    if (event.payload.kind === 'delete') void refreshGraph()
  })
  await loadGraph()
  await nextTick()
  graphViewRef.value?.runDagreLayout()
})
onUnmounted(() => {
  window.removeEventListener('keydown', dismissOnEsc)
  unlistenFs?.()
})
</script>

<template>
  <div class="starmap-view">
    <GraphHud
      v-if="!loading && !error"
      v-model:show-trunk="showTrunk"
      v-model:show-link="showLink"
      v-model:show-orphan="showOrphan"
    />

    <GraphView
      v-if="!loading && !error"
      ref="graphViewRef"
      :nodes="filteredNodes"
      :edges="filteredEdges"
      :readonly="false"
      :locked="isLayoutLocked"
      :highlighted-nodes="selectedEdgeEndpoints"
      @navigate="handleNavigate"
      @add-trunk="handleAddTrunk"
      @pane-click="handlePaneClick"
      @pane-context-menu="handlePaneContextMenu"
      @edge-context-menu="handleEdgeContextMenu"
      @edge-click="handleEdgeClick"
      @layout-done="handleLayoutDone"
      @pane-ready="handlePaneReady"
    />

    <div v-else-if="loading" class="starmap-status">
      <span class="animate-neon-pulse">图谱加载中...</span>
    </div>

    <div v-else class="starmap-status">
      <span>{{ error }}</span>
    </div>

    <PaneContextMenu
      v-if="contextMenuPos"
      :x="contextMenuPos.x"
      :y="contextMenuPos.y"
      mode="global"
      @close="contextMenuPos = null"
      @add-node="handleContextAddNode"
      @refresh="handleContextRefresh"
    />

    <EdgeContextMenu
      v-if="edgeMenuPos"
      :x="edgeMenuPos.x"
      :y="edgeMenuPos.y"
      @close="edgeMenuPos = null"
      @reverse="handleEdgeReverse"
      @disconnect="handleEdgeDisconnect"
    />

    <div class="starmap-controls">
      <button class="starmap-ctrl-btn" :class="{ 'ctrl-btn-warning': !isLayoutLocked }"
        :title="isLayoutLocked ? '刚体模式（点击解锁拖拽）' : '自由拖拽（点击锁定）'"
        @click="handleToggleLock">
        <Lock v-if="isLayoutLocked" :size="16" :stroke-width="1.5" />
        <Unlock v-else :size="16" :stroke-width="1.5" />
      </button>
      <button class="starmap-ctrl-btn" title="自动布阵" @click="handleAutoLayout">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <path d="M1 8h5M10 8h5M8 1v5M8 10v5" stroke="currentColor" stroke-width="1.2" />
          <circle cx="8" cy="8" r="2.5" stroke="currentColor" stroke-width="1" />
        </svg>
      </button>
      <button class="starmap-ctrl-btn" title="战术重置" @click="handleTacticalReset">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <rect x="4" y="4" width="8" height="8" rx="1" stroke="currentColor" stroke-width="1.2" />
          <line x1="0" y1="8" x2="4" y2="8" stroke="currentColor" stroke-width="1.2" />
          <line x1="12" y1="8" x2="16" y2="8" stroke="currentColor" stroke-width="1.2" />
          <line x1="8" y1="0" x2="8" y2="4" stroke="currentColor" stroke-width="1.2" />
          <line x1="8" y1="12" x2="8" y2="16" stroke="currentColor" stroke-width="1.2" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.starmap-view {
  position: relative;
  width: 100%;
  height: 100%;
  background: var(--ms-deep);
}

.starmap-view :deep(.graph-view-outer) {
  background: transparent;
}

.starmap-view :deep(.graph-view-inner) {
  background: transparent;
}

.starmap-view :deep(.vue-flow) {
  background: transparent;
}

.starmap-view :deep(.vue-flow__node) {
  background: var(--ms-carbon);
  border: 1px solid var(--ms-border-light);
  border-radius: 2px;
  padding: 6px 12px;
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--text-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);
  transition: border-color 200ms cubic-bezier(0.25, 1, 0.5, 1),
    box-shadow 200ms cubic-bezier(0.25, 1, 0.5, 1);
}

.starmap-view :deep(.vue-flow__node:hover) {
  border-color: var(--neon);
  box-shadow: 0 0 12px oklch(0.78 0.17 200 / 0.12);
}

.starmap-view :deep(.vue-flow__node.selected) {
  border-color: var(--neon);
  box-shadow: 0 0 16px oklch(0.78 0.17 200 / 0.3);
}

.starmap-view :deep(.vue-flow__edge:not(.link-breathing) .vue-flow__edge-path) {
  stroke: var(--neon) !important;
}

.starmap-view :deep(.vue-flow__edge.link-breathing .vue-flow__edge-path) {
  animation: link-breathe 3s ease-in-out infinite;
  will-change: stroke;
}

@keyframes link-breathe {
  0%, 100% { stroke: var(--text-muted); }
  50%      { stroke: var(--brass); }
}

.starmap-view :deep(.vue-flow__edge.selected .vue-flow__edge-path) {
  filter: drop-shadow(0 0 6px oklch(0.78 0.17 200 / 0.6));
  stroke-width: 3;
}

.starmap-view :deep(.vue-flow__node.node-dimmed) {
  opacity: 0.3;
  transition: opacity var(--duration-fast) var(--ease-hydraulic);
}

.starmap-view :deep(.vue-flow__node.node-highlighted) {
  border-color: var(--neon);
  box-shadow: 0 0 20px oklch(0.78 0.17 200 / 0.3);
  transition: border-color var(--duration-fast) var(--ease-hydraulic),
    box-shadow var(--duration-fast) var(--ease-hydraulic);
}

.starmap-view :deep(.vue-flow__background) {
  background: transparent;
}

.starmap-view :deep(.vue-flow__panel) {
  background: var(--ms-void);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  color: var(--text-secondary);
}

.starmap-view :deep(.vue-flow__controls-button) {
  background: var(--ms-void);
  border-bottom: 1px solid var(--ms-border);
  fill: var(--text-secondary);
}

.starmap-view :deep(.vue-flow__controls-button:hover) {
  background: var(--ms-surface);
  fill: var(--neon);
}

.starmap-status {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  font-family: var(--font-sans);
  font-size: 14px;
  color: var(--text-muted);
}

.starmap-controls {
  position: absolute;
  bottom: 16px;
  right: 16px;
  display: flex;
  gap: 0;
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  overflow: hidden;
  opacity: 0;
  transition: opacity var(--duration-normal) var(--ease-hydraulic);
}

.starmap-view:hover .starmap-controls {
  opacity: 1;
}

.starmap-ctrl-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-right: 1px solid var(--ms-border);
  border-radius: 0;
  background: var(--ms-void);
  color: var(--text-muted);
  cursor: pointer;
  transition: color var(--duration-fast) var(--ease-hydraulic);
}

.starmap-ctrl-btn:last-child {
  border-right: none;
}

.starmap-ctrl-btn:hover {
  background: var(--ms-carbon);
  color: var(--neon);
}

.starmap-ctrl-btn.ctrl-btn-warning {
  color: var(--brass);
}

.starmap-ctrl-btn.ctrl-btn-warning:hover {
  background: var(--ms-carbon);
  color: var(--text-primary);
}
</style>
