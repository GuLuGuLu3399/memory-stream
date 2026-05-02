<template>
  <div class="graph-view-outer" :class="{ 'gv-visible': visible }">
    <div class="graph-view-inner">
      <VueFlow ref="vueFlowRef"
        :nodes="vfNodes"
        :edges="vfEdges"
        :node-types="mergedNodeTypes"
        :nodes-draggable="!locked"
        :nodes-connectable="!readonly"
        :elements-selectable="true"
        :elevate-nodes-on-select="true"
        :connection-mode="ConnectionMode.Loose"
        @node-click="onNodeClick"
        @connect="onConnect"
        @pane-click="onPaneClick"
        @pane-context-menu="onPaneContextMenu"
        @node-context-menu="onNodeContextMenu"
        @edge-context-menu="onEdgeContextMenu"
        @edge-click="onEdgeClick"
        @pane-ready="onPaneReady"
      >
        <Background pattern-color="#3A3A3A" :gap="20" />
        <Controls />
      </VueFlow>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, markRaw, onBeforeUnmount, watch, nextTick } from 'vue'
import { VueFlow, MarkerType, ConnectionMode } from '@vue-flow/core'
import type { NodeTypesObject } from '@vue-flow/core'
import type { GraphEdge } from '@memory-stream/types'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import AltarNode from './AltarNode.vue'
import { layoutGraph } from '../composables/useDagreLayout'
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'

interface GraphNodeInput {
  id: string
  title: string
  x?: number | null
  y?: number | null
  type?: string
}

const props = withDefaults(defineProps<{
  nodes: GraphNodeInput[]
  edges: GraphEdge[]
  readonly?: boolean
  locked?: boolean
  customNodeTypes?: NodeTypesObject
  highlightedNodes?: { source: string; target: string } | null
}>(), {
  readonly: true,
  locked: true,
})

const mergedNodeTypes = computed<NodeTypesObject>(() => ({
  altar: markRaw(AltarNode) as any,
  ...props.customNodeTypes,
}))

const emit = defineEmits<{
  (e: 'navigate', uuid: string): void
  (e: 'add-trunk', payload: { source: string; target: string }): void
  (e: 'pane-click'): void
  (e: 'pane-context-menu', event: MouseEvent): void
  (e: 'node-context-menu', payload: { uuid: string; event: MouseEvent }): void
  (e: 'edge-context-menu', payload: { source: string; target: string; relation: string; event: MouseEvent }): void
  (e: 'edge-click', payload: { source: string; target: string; relation: string }): void
  (e: 'layout-done', positions: { uuid: string; x: number; y: number }[]): void
  (e: 'pane-ready'): void
}>()

const vueFlowRef = ref<InstanceType<typeof VueFlow> | null>(null)
const posOverrides = ref<Map<string, { x: number; y: number }>>(new Map())
const layoutPositions = ref<Map<string, { x: number; y: number }>>(new Map())
const visible = ref(false)

function onPaneReady() {
  visible.value = true
  emit('pane-ready')
  // Auto-fitView on initial mount if layout was already computed
  if (layoutPositions.value.size > 0) {
    nextTick(() => fitView({ padding: 0.2, duration: 800 }))
  }
}

function fitView(options?: { padding?: number; duration?: number }) {
  vueFlowRef.value?.fitView?.(options)
}

function project(clientPos: { x: number; y: number }) {
  return vueFlowRef.value?.project?.(clientPos) ?? { x: clientPos.x, y: clientPos.y }
}

let animFrameId: number | null = null

function animateToPositions(newPositions: { id: string; x: number; y: number }[]) {
  if (animFrameId !== null) cancelAnimationFrame(animFrameId)
  const duration = 300
  const startTime = performance.now()
  const endMap = new Map(newPositions.map((p) => [p.id, { x: p.x, y: p.y }]))
  const startMap = new Map<string, { x: number; y: number }>()
  for (const node of props.nodes) {
    const lp = layoutPositions.value.get(node.id)
    startMap.set(node.id, { x: lp?.x ?? node.x ?? 0, y: lp?.y ?? node.y ?? 0 })
  }

  function tick(now: number) {
    const elapsed = now - startTime
    const t = Math.min(elapsed / duration, 1)
    const eased = t < 0.5 ? 2 * t * t : 1 - Math.pow(-2 * t + 2, 2) / 2

    const overrides = new Map<string, { x: number; y: number }>()
    for (const [id, end] of endMap) {
      const start = startMap.get(id)
      if (start) {
        overrides.set(id, {
          x: start.x + (end.x - start.x) * eased,
          y: start.y + (end.y - start.y) * eased,
        })
      }
    }
    posOverrides.value = overrides

    if (t < 1) {
      animFrameId = requestAnimationFrame(tick)
    } else {
      animFrameId = null
      posOverrides.value = new Map()
      emit('layout-done', newPositions.map((p) => ({ uuid: p.id, x: p.x, y: p.y })))
    }
  }

  animFrameId = requestAnimationFrame(tick)
}

onBeforeUnmount(() => {
  if (animFrameId !== null) cancelAnimationFrame(animFrameId)
})

// Auto-layout: compute Dagre positions BEFORE Vue Flow renders
watch(() => [props.nodes, props.edges] as const, async ([nodes, edges]) => {
  if (nodes.length === 0) {
    layoutPositions.value = new Map()
    return
  }
  // Only compute if nodes lack explicit positions
  const needsLayout = nodes.some(n => n.x == null || n.y == null)
  if (!needsLayout) return

  const positions = layoutGraph(
    nodes.map(n => ({ id: n.id })),
    edges.map(e => ({
      source: e.source,
      target: e.target,
      relation: e.relation,
    })),
  )
  layoutPositions.value = positions

  await nextTick()
  fitView({ padding: 0.2, duration: 800 })
}, { immediate: true })

function onPaneClick() {
  emit('pane-click')
}

function onPaneContextMenu(event: MouseEvent) {
  emit('pane-context-menu', event)
}

function onNodeContextMenu(event: any) {
  emit('node-context-menu', { uuid: event.node.id, event: event.event })
}

function onEdgeContextMenu(event: any) {
  const id = event.edge.id as string
  const parts = id.split('-')
  const relation = parts.length > 2 ? parts.slice(2).join('-') : 'trunk'
  emit('edge-context-menu', {
    source: event.edge.source,
    target: event.edge.target,
    relation,
    event: event.event,
  })
}

function onEdgeClick(event: any) {
  const id = event.edge.id as string
  const parts = id.split('-')
  const relation = parts.length > 2 ? parts.slice(2).join('-') : 'trunk'
  emit('edge-click', {
    source: event.edge.source,
    target: event.edge.target,
    relation,
  })
}

function resetZoom() {
  vueFlowRef.value?.setViewport?.({ x: 0, y: 0, zoom: 1.0 })
}

function runDagreLayout() {
  const positions = layoutGraph(
    props.nodes.map(n => ({ id: n.id })),
    props.edges.map(e => ({
      source: e.source,
      target: e.target,
      relation: e.relation,
    })),
  )
  const posArray = Array.from(positions.entries()).map(([id, { x, y }]) => ({ id, x, y }))
  layoutPositions.value = positions
  animateToPositions(posArray)
}

defineExpose({ fitView, project, resetZoom, animateToPositions, runDagreLayout })

const vfNodes = computed(() =>
  props.nodes.map((node) => {
    const isHighlighted = props.highlightedNodes &&
      (node.id === props.highlightedNodes.source || node.id === props.highlightedNodes.target)
    const isDimmed = props.highlightedNodes && !isHighlighted
    const override = posOverrides.value.get(node.id)
    const layoutPos = layoutPositions.value.get(node.id)
    return {
      id: node.id,
      type: node.type || 'altar',
      position: {
        x: override?.x ?? layoutPos?.x ?? node.x ?? 0,
        y: override?.y ?? layoutPos?.y ?? node.y ?? 0,
      },
      data: { label: node.title },
      class: isDimmed ? 'node-dimmed' : isHighlighted ? 'node-highlighted' : '',
    }
  }),
)

const vfEdges = computed(() => {
  // Detect bidirectional pairs: A→B and B→A with same relation
  const edgeKeys = new Set(props.edges.map(e => `${e.source}->${e.target}->${e.relation}`))
  const seen = new Set<string>()

  return props.edges
    .filter((edge) => {
      const isTrunk = edge.relation === 'trunk'
      const reverseKey = `${edge.target}->${edge.source}->${edge.relation}`
      // For bidir links, only keep the one where source < target
      if (!isTrunk && edgeKeys.has(reverseKey)) {
        const canonical = [edge.source, edge.target].sort().join('->') + '->bidir'
        if (seen.has(canonical)) return false
        seen.add(canonical)
      }
      return true
    })
    .map((edge) => {
      const isTrunk = edge.relation === 'trunk'
      const reverseKey = `${edge.target}->${edge.source}->${edge.relation}`
      const isBidir = !isTrunk && edgeKeys.has(reverseKey)

      return {
        id: `${edge.source}-${edge.target}-${edge.relation}`,
        type: isTrunk ? 'step' : 'default',
        class: isTrunk ? undefined : 'link-breathing',
        interactionWidth: 10,
        source: edge.source,
        target: edge.target,
        sourceHandle: isTrunk ? 'trunk-source' : 'link-source',
        targetHandle: isTrunk ? 'trunk-target' : 'link-target',
        animated: isTrunk,
        style: isTrunk
          ? { stroke: '#6B8E7B', strokeWidth: 2, strokeDasharray: '8 8' }
          : { stroke: '#3A3A3A', strokeWidth: 1, strokeDasharray: '4 4' },
        markerEnd: isTrunk || isBidir ? MarkerType.ArrowClosed : undefined,
        markerStart: isBidir ? MarkerType.ArrowClosed : undefined,
      }
    })
})

function onNodeClick(event: any) {
  emit('navigate', event.node.id)
}

function onConnect(connection: any) {
  if (props.readonly) return
  emit('add-trunk', {
    source: connection.source,
    target: connection.target,
  })
}
</script>

<style scoped>
.graph-view-outer {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 300px;
  opacity: 0;
  transition: opacity 300ms cubic-bezier(0.25, 1, 0.5, 1);
}

.graph-view-outer.gv-visible {
  opacity: 1;
}

.graph-view-inner {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
}

.vue-flow__panel {
  z-index: 10;
}

.graph-view-inner :deep(.vue-flow__edges) {
  z-index: 0;
}

/* Dark theme overrides for VueFlow controls */
.graph-view-inner :deep(.vue-flow__controls) {
  background: var(--ms-surface, #222222);
  border: 1px solid var(--ms-border, #3A3A3A);
  border-radius: 4px;
  box-shadow: none;
}

.graph-view-inner :deep(.vue-flow__controls-button) {
  background: var(--ms-surface, #222222);
  border-bottom: 1px solid var(--ms-border, #3A3A3A);
  fill: var(--text-muted, #888);
}

.graph-view-inner :deep(.vue-flow__controls-button:hover) {
  background: var(--ms-surface-elevated, #2A2826);
}

.graph-view-inner :deep(.vue-flow__attribution) {
  display: none;
}
</style>
