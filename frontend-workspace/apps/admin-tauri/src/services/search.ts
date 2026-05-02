// ────────────────────────────────────────────────────────────────
// search.ts — FTS5 full-text search + Fuse.js fuzzy search
// ────────────────────────────────────────────────────────────────

import Fuse from 'fuse.js'
import type { FtsHit } from '@memory-stream/types'
import * as bridge from '@/bridge/invoke'

export async function search(query: string, limit = 20): Promise<FtsHit[]> {
  return bridge.searchFts(query, limit)
}

// ── Fuzzy search (Fuse.js) ──────────────────────────────────────

interface CardSummary {
  uuid: string
  title: string
}

let fuseCache: Fuse<CardSummary> | null = null
let cacheSeed = 0

function invalidateFuseCache() {
  fuseCache = null
  cacheSeed++
}

async function ensureFuseIndex(): Promise<Fuse<CardSummary>> {
  if (fuseCache) return fuseCache

  const graph = await bridge.getFullGraph()
  const cards: CardSummary[] = graph.nodes
    .filter((n) => n.title && n.title !== '未命名卡片')
    .map((n) => ({ uuid: n.id, title: n.title }))

  fuseCache = new Fuse(cards, {
    keys: [{ name: 'title', weight: 0.7 }],
    threshold: 0.3,
    ignoreLocation: true,
    includeScore: true,
  })

  return fuseCache
}

export async function fuzzySearch(
  query: string,
  excludeIds: string[] = [],
): Promise<CardSummary[]> {
  if (!query.trim()) return []

  const fuse = await ensureFuseIndex()
  const excludeSet = new Set(excludeIds)

  return fuse
    .search(query)
    .map((r) => r.item)
    .filter((c) => !excludeSet.has(c.uuid))
}

export { invalidateFuseCache }
