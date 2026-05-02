// ────────────────────────────────────────────────────────────────
// reader.ts — Read-only reader state (current card, graph, search)
// ────────────────────────────────────────────────────────────────

import { defineStore } from 'pinia'
import { ref, shallowRef } from 'vue'
import type { AstNode, FullGraph, TocNode } from '@memory-stream/types'
import * as cardsApi from '@/api/cards'
import type { BacklinkItem, CardDetail, CardListItem, CategoryItem } from '@/api/cards'
import * as searchApi from '@/api/search'
import type { SearchResult } from '@/api/search'

interface CachedCard {
  card: CardDetail
  ast: AstNode
  toc: TocNode[]
}

export const useReaderStore = defineStore('reader', () => {
  // Card list
  const cardList = shallowRef<CardListItem[]>([])
  const categories = shallowRef<CategoryItem[]>([])
  const randomCards = shallowRef<CardListItem[]>([])

  // Current card
  const currentCard = shallowRef<CardDetail | null>(null)
  const currentAst = shallowRef<AstNode | null>(null)
  const currentToc = shallowRef<TocNode[]>([])
  const currentGraph = shallowRef<FullGraph | null>(null)
  const fullGraph = shallowRef<FullGraph | null>(null)

  // Search
  const searchResults = shallowRef<SearchResult[]>([])

  // Backlinks
  const currentBacklinks = shallowRef<BacklinkItem[]>([])

  // Loading
  const loading = ref(false)
  const graphLoading = ref(false)
  const searchLoading = ref(false)

  // AST cache
  const cache = new Map<string, CachedCard>()

  async function loadCardList(category?: string) {
    loading.value = true
    try {
      cardList.value = await cardsApi.listCards(category)
    } finally {
      loading.value = false
    }
  }

  async function loadCard(uuid: string) {
    loading.value = true
    try {
      const cached = cache.get(uuid)
      if (cached) {
        currentCard.value = cached.card
        currentAst.value = cached.ast
        currentToc.value = cached.toc
        return
      }
      const card = await cardsApi.getCard(uuid)
      currentCard.value = card
      currentAst.value = card.ast_data
      currentToc.value = card.toc_data
      cache.set(uuid, { card, ast: card.ast_data, toc: card.toc_data })
    } finally {
      loading.value = false
    }
  }

  async function loadGraph(uuid: string, depth = 2) {
    graphLoading.value = true
    try {
      currentGraph.value = await cardsApi.getCardGraph(uuid, depth)
    } finally {
      graphLoading.value = false
    }
  }

  async function loadBacklinks(uuid: string) {
    currentBacklinks.value = await cardsApi.getBacklinks(uuid)
  }

  async function loadFullGraph() {
    graphLoading.value = true
    try {
      fullGraph.value = await cardsApi.getFullGraph()
    } finally {
      graphLoading.value = false
    }
  }

  async function loadCategories() {
    categories.value = await cardsApi.listCategories()
  }

  async function loadRandomCards(count = 5) {
    randomCards.value = await cardsApi.getRandomCards(count)
  }

  async function doSearch(query: string) {
    searchLoading.value = true
    try {
      searchResults.value = await searchApi.search(query)
    } finally {
      searchLoading.value = false
    }
  }

  function clear() {
    currentCard.value = null
    currentAst.value = null
    currentToc.value = []
    currentGraph.value = null
    currentBacklinks.value = []
  }

  return {
    cardList,
    categories,
    randomCards,
    currentCard,
    currentAst,
    currentToc,
    currentGraph,
    currentBacklinks,
    fullGraph,
    searchResults,
    loading,
    graphLoading,
    searchLoading,
    loadCardList,
    loadCard,
    loadGraph,
    loadBacklinks,
    loadFullGraph,
    loadCategories,
    loadRandomCards,
    doSearch,
    clear,
  }
})
