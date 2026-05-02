// 用途：图谱面板，在右侧面板中渲染当前卡片的子图谱
<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { Lock, Unlock } from 'lucide-vue-next'
import { GraphView } from '@memory-stream/ui-shared'
import GraphHud from './GraphHud.vue'
import PaneContextMenu from './PaneContextMenu.vue'
import EdgeContextMenu from './EdgeContextMenu.vue'
import VoidSearchPopup from './VoidSearchPopup.vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import * as protocol from '@/bridge/protocol'
import * as cardService from '@/services/card'
import { useToast } from '@/composables/core/useToast'
import { useLayoutStore } from '@/stores/layout'
import type { GraphNode, GraphEdge } from '@memory-stream/types'

const props = defineProps<{
  activeCardUuid: string | null
}>()

const emit = defineEmits<{
  navigate: [uuid: string]
}>()

const nodes = ref<GraphNode[]>([])
const edges = ref<GraphEdge[]>([])
const loading = ref(false)
const toast = useToast()
const layout = useLayoutStore()
const panelOpen = computed(() => layout.rightPanel === 'graph')

const showTrunk = ref(true)
const showLink = ref(true)
const showOrphan = ref(true)

const isLayoutLocked = ref(true)

const contextMenuPos = ref<{ x: number; y: number } | null>(null)
const edgeMenuPos = ref<{ x: number; y: number; source: string; target: string; relation: string } | null>(null)
const selectedEdgeEndpoints = ref<{ source: string; target: string } | null>(null)
const searchPopupPos = ref<{ x: number; y: number } | null>(null)
const mousePos = ref({ x: 0, y: 0 })
const graphReady = ref(false)

const graphViewRef = ref<InstanceType<typeof GraphView> | null>(null)

const excludeIds = computed(() => nodes.value.map((n) => n.id))

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

async function fetchGraph() {
  if (!props.activeCardUuid) {
    nodes.value = []
    edges.value = []
    return
  }
  loading.value = true
  try {
    const result = await protocol.fetchNeighborhood(props.activeCardUuid, 2)
    const cleanNodes = result.nodes.filter(n => n.title !== '未命名卡片')
    const validIds = new Set(cleanNodes.map(n => n.id))
    const cleanEdges = result.edges.filter(e => {
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
    await nextTick()
    graphViewRef.value?.runDagreLayout()
  } catch {
    nodes.value = []
    edges.value = []
  } finally {
    loading.value = false
  }
}

watch(() => props.activeCardUuid, fetchGraph, { immediate: true })

function refreshGraph() {
  void fetchGraph()
}

function fullCleanup() {
  contextMenuPos.value = null
  edgeMenuPos.value = null
  searchPopupPos.value = null
}

async function handleAddTrunk(payload: { source: string; target: string }) {
  try {
    await protocol.createTrunkEdge(payload.source, payload.target)
    await fetchGraph()
    toast.success('已连接主干')
  } catch {
    toast.error('创建 Trunk 失败')
  }
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

async function handleContextAddNode() {
  contextMenuPos.value = null
  if (!props.activeCardUuid) return
  try {
    const card = await cardService.createCard('未命名卡片')
    await handleAddTrunk({ source: props.activeCardUuid, target: card.uuid })
  } catch {
    toast.error('新建卡片失败')
  }
}

function handleSummonLink() {
  const pos = contextMenuPos.value
  contextMenuPos.value = null
  if (pos) searchPopupPos.value = { x: pos.x, y: pos.y }
}

async function handleSearchSelect(uuid: string) {
  searchPopupPos.value = null
  if (!props.activeCardUuid) return
  try {
    await protocol.createLinkEdge(props.activeCardUuid, uuid)
    await fetchGraph()
  } catch {
    toast.error('创建连接失败')
  }
}

async function handleDetachAll() {
  contextMenuPos.value = null
  if (!props.activeCardUuid) return
  try {
    const related = edges.value.filter(
      (e) => e.source === props.activeCardUuid || e.target === props.activeCardUuid,
    )
    for (const e of related) {
      await protocol.deleteTrunkEdge(e.source, e.target)
    }
    await fetchGraph()
    toast.success(`已断开 ${related.length} 条连接`)
  } catch {
    toast.error('断开连接失败')
  }
}

async function handleEdgeReverse() {
  const pos = edgeMenuPos.value
  edgeMenuPos.value = null
  if (!pos) return
  try {
    await protocol.reverseTrunkEdge(pos.source, pos.target)
    await fetchGraph()
    toast.success('已反转方向')
  } catch {
    toast.error('反转方向失败')
  }
}

async function handleEdgeDisconnect() {
  const pos = edgeMenuPos.value
  edgeMenuPos.value = null
  if (!pos) return
  try {
    await protocol.deleteTrunkEdge(pos.source, pos.target)
    await fetchGraph()
    toast.success('已断开连接')
  } catch {
    toast.error('断开连接失败')
  }
}

async function handleAutoLayout() {
  if (!props.activeCardUuid) return
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

function handleMousemove(e: MouseEvent) {
  mousePos.value = { x: e.clientX, y: e.clientY }
}

let unlistenFs: UnlistenFn | null = null

function handleShiftSpace(e: KeyboardEvent) {
  if (e.key === ' ' && e.shiftKey && panelOpen.value && props.activeCardUuid) {
    e.preventDefault()
    searchPopupPos.value = { x: mousePos.value.x, y: mousePos.value.y }
  }
}

function dismissOnEsc(e: KeyboardEvent) {
  if (e.key === 'Escape') fullCleanup()
}

watch(panelOpen, async (open) => {
  if (open) {
    await nextTick()
    setTimeout(() => { graphReady.value = true }, 80)
  } else {
    graphReady.value = false
  }
})

onMounted(async () => {
  window.addEventListener('keydown', dismissOnEsc)
  window.addEventListener('keydown', handleShiftSpace)
  unlistenFs = await listen<{ path: string; kind: string }>('fs:change', (event) => {
    if (event.payload.kind === 'delete') refreshGraph()
  })
  if (panelOpen.value) {
    await nextTick()
    setTimeout(() => { graphReady.value = true }, 80)
  }
})
onUnmounted(() => {
  window.removeEventListener('keydown', dismissOnEsc)
  window.removeEventListener('keydown', handleShiftSpace)
  unlistenFs?.()
})

defineExpose({ refreshGraph })
</script>

<template>
  <div class="graph-panel">
    <div class="graph-panel-header" />

    <div class="graph-panel-canvas" @mousemove="handleMousemove">
      <span v-if="nodes.length > 0" class="graph-panel-watermark">LOCAL RADAR</span>

      <GraphView
        v-if="graphReady && panelOpen && activeCardUuid && nodes.length > 0"
        ref="graphViewRef"
        :nodes="filteredNodes"
        :edges="filteredEdges"
        :readonly="false"
        :locked="isLayoutLocked"
        :highlighted-nodes="selectedEdgeEndpoints"
        @navigate="emit('navigate', $event)"
        @add-trunk="handleAddTrunk"
        @pane-click="handlePaneClick"
        @pane-context-menu="handlePaneContextMenu"
        @edge-context-menu="handleEdgeContextMenu"
        @edge-click="handleEdgeClick"
        @layout-done="handleLayoutDone"
        @pane-ready="handlePaneReady"
      />

      <GraphHud
        v-if="nodes.length > 0"
        v-model:show-trunk="showTrunk"
        v-model:show-link="showLink"
        v-model:show-orphan="showOrphan"
      />

      <div v-if="loading" class="graph-panel-loading">
        <span class="animate-neon-pulse">加载中...</span>
      </div>

      <div v-else-if="nodes.length === 0" class="graph-panel-status">
        <span>选择卡片以查看局部图谱</span>
      </div>

      <PaneContextMenu
        v-if="contextMenuPos"
        :x="contextMenuPos.x"
        :y="contextMenuPos.y"
        mode="local"
        @close="contextMenuPos = null"
        @add-node="handleContextAddNode"
        @summon-link="handleSummonLink"
        @detach-all="handleDetachAll"
      />

      <EdgeContextMenu
        v-if="edgeMenuPos"
        :x="edgeMenuPos.x"
        :y="edgeMenuPos.y"
        @close="edgeMenuPos = null"
        @reverse="handleEdgeReverse"
        @disconnect="handleEdgeDisconnect"
      />

      <VoidSearchPopup
        v-if="searchPopupPos"
        :x="searchPopupPos.x"
        :y="searchPopupPos.y"
        :source-id="activeCardUuid ?? undefined"
        :exclude-ids="excludeIds"
        @select="handleSearchSelect"
        @close="searchPopupPos = null"
      />

      <div class="graph-panel-controls">
        <button class="graph-ctrl-btn" :class="{ 'ctrl-btn-warning': !isLayoutLocked }"
          :title="isLayoutLocked ? '刚体模式（点击解锁拖拽）' : '自由拖拽（点击锁定）'"
          @click="handleToggleLock">
          <Lock v-if="isLayoutLocked" :size="14" :stroke-width="1.5" />
          <Unlock v-else :size="14" :stroke-width="1.5" />
        </button>
        <button class="graph-ctrl-btn" title="自动布阵" @click="handleAutoLayout">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M1 7h4M9 7h4M7 1v4M7 9v4" stroke="currentColor" stroke-width="1.2" />
            <circle cx="7" cy="7" r="2" stroke="currentColor" stroke-width="1" />
          </svg>
        </button>
        <button class="graph-ctrl-btn" title="战术重置" @click="handleTacticalReset">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="3" y="3" width="8" height="8" rx="1" stroke="currentColor" stroke-width="1.2" />
            <line x1="0" y1="7" x2="3" y2="7" stroke="currentColor" stroke-width="1.2" />
            <line x1="11" y1="7" x2="14" y2="7" stroke="currentColor" stroke-width="1.2" />
            <line x1="7" y1="0" x2="7" y2="3" stroke="currentColor" stroke-width="1.2" />
            <line x1="7" y1="11" x2="7" y2="14" stroke="currentColor" stroke-width="1.2" />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.graph-panel {
  position: relative;
  height: 100%;
  min-height: 300px;
  background: var(--ms-deep);
  overflow: hidden;
}

.graph-panel-header {
  height: 1px;
  background: var(--ms-border);
}

.graph-panel-watermark {
  position: absolute;
  top: 7px;
  left: 8px;
  font-family: var(--font-sans);
  font-size: 10px;
  letter-spacing: 0.1em;
  color: var(--text-muted);
  opacity: 0.2;
  pointer-events: none;
  z-index: 1;
}

.graph-panel-canvas {
  position: absolute;
  inset: 0;
  overflow: hidden;
}

.graph-panel-canvas :deep(.graph-view-outer) {
  background: transparent;
}

.graph-panel-canvas :deep(.graph-view-inner) {
  background: transparent;
}

.graph-panel-canvas :deep(.vue-flow) {
  background: transparent;
}

.graph-panel-canvas :deep(.vue-flow__node) {
  background: var(--ms-carbon);
  border: 1px solid var(--ms-border-light);
  border-radius: 2px;
  font-family: var(--font-sans);
  color: var(--text-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);
  transition: border-color 200ms cubic-bezier(0.25, 1, 0.5, 1),
    box-shadow 200ms cubic-bezier(0.25, 1, 0.5, 1);
}

.graph-panel-canvas :deep(.vue-flow__node:hover) {
  border-color: var(--neon);
  box-shadow: 0 0 12px oklch(0.78 0.17 200 / 0.12);
}

.graph-panel-canvas :deep(.vue-flow__node.selected) {
  border-color: var(--neon);
  box-shadow: 0 0 16px oklch(0.78 0.17 200 / 0.3);
}

.graph-panel-canvas :deep(.vue-flow__edge:not(.link-breathing) .vue-flow__edge-path) {
  stroke: var(--neon) !important;
}

.graph-panel-canvas :deep(.vue-flow__edge.link-breathing .vue-flow__edge-path) {
  animation: link-breathe 3s ease-in-out infinite;
  will-change: stroke;
}

@keyframes link-breathe {
  0%, 100% { stroke: var(--text-muted); }
  50%      { stroke: var(--brass); }
}

.graph-panel-canvas :deep(.vue-flow__node.node-dimmed) {
  opacity: 0.3;
  transition: opacity var(--duration-fast) var(--ease-hydraulic);
}

.graph-panel-canvas :deep(.vue-flow__node.node-highlighted) {
  border-color: var(--neon);
  box-shadow: 0 0 20px oklch(0.78 0.17 200 / 0.3);
  transition: border-color var(--duration-fast) var(--ease-hydraulic),
    box-shadow var(--duration-fast) var(--ease-hydraulic);
}

.graph-panel-canvas :deep(.vue-flow__background) {
  background: transparent;
}

.graph-panel-canvas :deep(.vue-flow__panel) {
  background: var(--ms-void);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  color: var(--text-secondary);
}

.graph-panel-canvas :deep(.vue-flow__controls-button) {
  background: var(--ms-void);
  border-bottom: 1px solid var(--ms-border);
  fill: var(--text-secondary);
}

.graph-panel-canvas :deep(.vue-flow__controls-button:hover) {
  background: var(--ms-surface);
  fill: var(--neon);
}

.graph-panel-status {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--text-muted);
}

.graph-panel-loading {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in oklch, var(--ms-deep) 80%, transparent);
  z-index: 20;
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--text-muted);
}

.graph-panel-controls {
  position: absolute;
  bottom: 8px;
  left: 8px;
  display: flex;
  gap: 0;
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  overflow: hidden;
  opacity: 0;
  transition: opacity var(--duration-normal) var(--ease-hydraulic);
  z-index: 10;
}

.graph-panel-canvas:hover .graph-panel-controls {
  opacity: 1;
}

.graph-ctrl-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-right: 1px solid var(--ms-border);
  border-radius: 0;
  background: var(--ms-void);
  color: var(--text-muted);
  cursor: pointer;
  transition: color var(--duration-fast) var(--ease-hydraulic);
}

.graph-ctrl-btn:last-child {
  border-right: none;
}

.graph-ctrl-btn:hover {
  background: var(--ms-carbon);
  color: var(--neon);
}

.graph-ctrl-btn.ctrl-btn-warning {
  color: var(--brass);
}

.graph-ctrl-btn.ctrl-btn-warning:hover {
  background: var(--ms-carbon);
  color: var(--text-primary);
}
</style>
