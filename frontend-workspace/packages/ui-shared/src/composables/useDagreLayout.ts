import Dagre from '@dagrejs/dagre'

const FANOUT_RADIUS = 200

export function layoutGraph(
  nodes: { id: string; width?: number; height?: number }[],
  edges: { source: string; target: string; relation?: string }[],
  direction: 'TB' | 'LR' = 'LR',
): Map<string, { x: number; y: number }> {
  // ── Layer 1: Trunk-only Dagre (skeleton) ──

  const g = new Dagre.graphlib.Graph()
  g.setDefaultEdgeLabel(() => ({}))
  g.setGraph({
    rankdir: direction,
    nodesep: 60,
    ranksep: 200,
    marginx: 20,
    marginy: 20,
  })

  for (const node of nodes) {
    g.setNode(node.id, { width: node.width ?? 180, height: node.height ?? 50 })
  }

  // Only trunk edges for layout — link edges can create cycles that crash dagre
  for (const edge of edges) {
    if (edge.relation === 'trunk') {
      g.setEdge(edge.source, edge.target, { weight: 100, minlen: 1 })
    }
  }

  Dagre.layout(g)

  const result = new Map<string, { x: number; y: number }>()
  for (const node of nodes) {
    const pos = g.node(node.id)
    if (pos) result.set(node.id, { x: pos.x, y: pos.y })
  }

  // ── Layer 2: Link attachment (orphan placement) ──

  const nodeIds = new Set(nodes.map(n => n.id))

  // Classify: skeleton nodes (have at least one trunk edge) vs orphan nodes
  const skeletonIds = new Set<string>()
  for (const edge of edges) {
    if (edge.relation === 'trunk') {
      skeletonIds.add(edge.source)
      skeletonIds.add(edge.target)
    }
  }
  const orphanIds = new Set<string>()
  for (const id of nodeIds) {
    if (!skeletonIds.has(id)) orphanIds.add(id)
  }

  // No orphans → skeleton-only graph, skip attachment
  if (orphanIds.size === 0) return result

  // Build link adjacency: orphan → [skeleton neighbors via link edges]
  const orphanToAnchors = new Map<string, string[]>()
  for (const edge of edges) {
    if (edge.relation === 'link') {
      const srcOrphan = orphanIds.has(edge.source)
      const tgtOrphan = orphanIds.has(edge.target)
      const srcSkeleton = skeletonIds.has(edge.source)
      const tgtSkeleton = skeletonIds.has(edge.target)

      if (srcOrphan && tgtSkeleton) {
        const list = orphanToAnchors.get(edge.source) ?? []
        list.push(edge.target)
        orphanToAnchors.set(edge.source, list)
      }
      if (tgtOrphan && srcSkeleton) {
        const list = orphanToAnchors.get(edge.target) ?? []
        list.push(edge.source)
        orphanToAnchors.set(edge.target, list)
      }
    }
  }

  // Group orphans by their best anchor (first skeleton neighbor found)
  const anchorToOrphans = new Map<string, string[]>()
  const placedOrphans = new Set<string>()

  for (const [orphanId, anchors] of orphanToAnchors) {
    const anchor = anchors[0]
    if (!anchor) continue
    const list = anchorToOrphans.get(anchor) ?? []
    list.push(orphanId)
    anchorToOrphans.set(anchor, list)
    placedOrphans.add(orphanId)
  }

  // Rule B: Fan out orphan groups around their skeleton anchor
  for (const [anchorId, orphans] of anchorToOrphans) {
    const anchorPos = result.get(anchorId)
    if (!anchorPos) continue

    if (orphans.length === 1) {
      // Single orphan: offset to right with slight vertical jitter
      const jitter = (Math.random() - 0.5) * 40
      result.set(orphans[0], {
        x: anchorPos.x + FANOUT_RADIUS,
        y: anchorPos.y + jitter,
      })
    } else {
      // Multiple orphans: right-side semicircle fan-out
      const startAngle = -Math.PI / 2
      const angleRange = Math.PI
      for (let i = 0; i < orphans.length; i++) {
        const angle = startAngle + (i / orphans.length) * angleRange
        result.set(orphans[i], {
          x: anchorPos.x + FANOUT_RADIUS * Math.cos(angle),
          y: anchorPos.y + FANOUT_RADIUS * Math.sin(angle),
        })
      }
    }
  }

  // Rule C: Orphan pairs linked together (both orphan, neither has skeleton anchor)
  const linkedOrphanPairs: [string, string][] = []
  const linkedOrphanSet = new Set<string>()

  for (const edge of edges) {
    if (edge.relation !== 'link') continue
    if (orphanIds.has(edge.source) && orphanIds.has(edge.target)) {
      if (!linkedOrphanSet.has(edge.source) && !linkedOrphanSet.has(edge.target)) {
        linkedOrphanPairs.push([edge.source, edge.target])
        linkedOrphanSet.add(edge.source)
        linkedOrphanSet.add(edge.target)
      }
    }
  }

  // Place linked orphan pairs side-by-side using first orphan's Dagre position
  for (const [a, b] of linkedOrphanPairs) {
    const posA = result.get(a)
    if (posA) {
      result.set(b, { x: posA.x + FANOUT_RADIUS, y: posA.y })
    }
  }

  // Rule D: Truly isolated orphans — keep Dagre default position (already in result)

  return result
}
