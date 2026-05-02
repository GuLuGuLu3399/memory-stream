// ────────────────────────────────────────────────────────────────
// search.ts — Search API
// ────────────────────────────────────────────────────────────────

import { getClient } from './client'

export interface SearchResult {
  uuid: string
  title: string
  excerpt: string
}

export async function search(
  query: string,
  limit = 20,
): Promise<SearchResult[]> {
  const res = await getClient().get<{ results: SearchResult[] }>('/search', {
    q: query,
    limit: String(limit),
  })
  return res.results
}
