// ────────────────────────────────────────────────────────────────
// cards.ts — Read-only card, category, graph, and backlink API
// ────────────────────────────────────────────────────────────────

import type { AstNode, FullGraph, TocNode } from '@memory-stream/types'
import { getClient } from './client'

export interface CardListItem {
  uuid: string
  title: string
  excerpt: string
  category: string
  version: number
  updated_at: string
}

export interface CardDetail {
  uuid: string
  title: string
  content: string
  excerpt: string
  ast_data: AstNode
  toc_data: TocNode[]
  category: string
  version: number
  hash: string
  created_at: string
  updated_at: string
}

export interface BacklinkItem {
  uuid: string
  title: string
  relation_type: string
}

export interface CategoryItem {
  name: string
  count: number
}

export async function getCard(uuid: string): Promise<CardDetail> {
  return getClient().get<CardDetail>(`/cards/${uuid}`)
}

export async function listCards(
  category?: string,
  limit = 20,
  offset = 0,
): Promise<CardListItem[]> {
  const params: Record<string, string> = {
    limit: String(limit),
    offset: String(offset),
  }
  if (category) params.category = category
  const res = await getClient().get<{ cards: CardListItem[] }>('/cards', params)
  return res.cards
}

export async function getRandomCards(count = 5): Promise<CardListItem[]> {
  const res = await getClient().get<{ cards: CardListItem[] }>('/cards/random', {
    count: String(count),
  })
  return res.cards
}

export async function listCategories(): Promise<CategoryItem[]> {
  const res = await getClient().get<{ categories: CategoryItem[] }>('/categories')
  return res.categories
}

export async function resolveCardByTitle(title: string): Promise<string | null> {
  try {
    const res = await getClient().get<{ uuid: string }>('/cards/resolve', { title })
    return res.uuid
  } catch {
    return null
  }
}

export async function getBacklinks(uuid: string): Promise<BacklinkItem[]> {
  const res = await getClient().get<{ backlinks: BacklinkItem[] }>(`/cards/${uuid}/backlinks`)
  return res.backlinks
}

export async function getCardGraph(
  uuid: string,
  depth = 2,
): Promise<FullGraph> {
  return getClient().get<FullGraph>(`/graph/neighborhood/${uuid}`, {
    depth: String(depth),
  })
}

export async function getFullGraph(): Promise<FullGraph> {
  return getClient().get<FullGraph>('/graph/all')
}
