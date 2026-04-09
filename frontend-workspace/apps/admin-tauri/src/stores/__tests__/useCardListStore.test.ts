// @vitest-environment jsdom
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useCardListStore } from '../useCardListStore'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'

const mockInvoke = vi.mocked(invoke)

const mockOrphanCards = {
  data: [
    { id: 'orphan-1', title: 'Orphan Card 1', excerpt: 'Excerpt 1', x: 100, y: 200, category_id: null },
    { id: 'orphan-2', title: 'Orphan Card 2', excerpt: 'Excerpt 2', x: 150, y: 250, category_id: 1 },
  ],
  has_more: false,
  total_count: 2,
}

const mockRecentCards = {
  data: [
    { id: 'recent-1', title: 'Recent Card 1', excerpt: 'Recent excerpt 1', x: 10, y: 20, category_id: 1, updated_at: '2024-01-01T00:00:00Z' },
    { id: 'recent-2', title: 'Recent Card 2', excerpt: 'Recent excerpt 2', x: 30, y: 40, category_id: null, updated_at: '2024-01-02T00:00:00Z' },
    { id: 'recent-3', title: 'Recent Card 3', excerpt: 'Recent excerpt 3', x: 50, y: 60, category_id: 2, updated_at: '2024-01-03T00:00:00Z' },
  ],
  has_more: false,
  next_cursor: null,
  total_count: 3,
}

describe('useCardListStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('should have empty orphan cards initially', () => {
      const store = useCardListStore()

      expect(store.orphanCards).toEqual([])
    })

    it('should have empty recent cards initially', () => {
      const store = useCardListStore()

      expect(store.recentCards).toEqual([])
    })

    it('should have empty search query initially', () => {
      const store = useCardListStore()

      expect(store.searchQuery).toBe('')
    })

    it('should have null selected category initially', () => {
      const store = useCardListStore()

      expect(store.selectedCategoryId).toBeNull()
    })
  })

  describe('loadOrphans', () => {
    it('should load orphan cards from API', async () => {
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)

      const store = useCardListStore()
      await store.loadOrphans()

      expect(mockInvoke).toHaveBeenCalledWith('api_request', {
        method: 'GET',
        endpoint: '/cards/discover',
      })
    })

    it('should populate orphanCards with correct data', async () => {
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)

      const store = useCardListStore()
      await store.loadOrphans()

      expect(store.orphanCards).toHaveLength(2)
      expect(store.orphanCards[0].id).toBe('orphan-1')
      expect(store.orphanCards[0].title).toBe('Orphan Card 1')
    })

    it('should handle cards array in response.cards format', async () => {
      mockInvoke.mockResolvedValueOnce({ cards: mockOrphanCards.data })

      const store = useCardListStore()
      await store.loadOrphans()

      expect(store.orphanCards).toHaveLength(2)
    })

    it('should default missing title to "无标题"', async () => {
      mockInvoke.mockResolvedValueOnce({
        data: [{ id: 'no-title', excerpt: 'Test' }],
      })

      const store = useCardListStore()
      await store.loadOrphans()

      expect(store.orphanCards[0].title).toBe('无标题')
    })

    it('should use raw_md as fallback for content', async () => {
      mockInvoke.mockResolvedValueOnce({
        data: [{ id: 'raw-md', title: 'Test', raw_md: 'Raw content' }],
      })

      const store = useCardListStore()
      await store.loadOrphans()

      expect(store.orphanCards[0].content).toBe('Raw content')
    })

    it('should default coordinates to 0,0 when missing', async () => {
      mockInvoke.mockResolvedValueOnce({
        data: [{ id: 'no-coords', title: 'Test' }],
      })

      const store = useCardListStore()
      await store.loadOrphans()

      expect(store.orphanCards[0].x).toBe(0)
      expect(store.orphanCards[0].y).toBe(0)
    })

    it('should handle invoke error gracefully', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))

      const store = useCardListStore()
      await store.loadOrphans()

      expect(consoleSpy).toHaveBeenCalled()
      expect(store.orphanCards).toEqual([])

      consoleSpy.mockRestore()
    })

    it('should handle null category_id correctly', async () => {
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)

      const store = useCardListStore()
      await store.loadOrphans()

      expect(store.orphanCards[0].category_id).toBeNull()
      expect(store.orphanCards[1].category_id).toBe(1)
    })
  })

  describe('loadRecent', () => {
    it('should load recent cards from API', async () => {
      mockInvoke.mockResolvedValueOnce(mockRecentCards)

      const store = useCardListStore()
      await store.loadRecent()

      expect(mockInvoke).toHaveBeenCalledWith('api_request', {
        method: 'GET',
        endpoint: '/cards',
      })
    })

    it('should populate recentCards with correct data', async () => {
      mockInvoke.mockResolvedValueOnce(mockRecentCards)

      const store = useCardListStore()
      await store.loadRecent()

      expect(store.recentCards).toHaveLength(3)
      expect(store.recentCards[0].id).toBe('recent-1')
      expect(store.recentCards[0].updated_at).toBe('2024-01-01T00:00:00Z')
    })

    it('should handle invoke error gracefully', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))

      const store = useCardListStore()
      await store.loadRecent()

      expect(consoleSpy).toHaveBeenCalled()
      expect(store.recentCards).toEqual([])

      consoleSpy.mockRestore()
    })
  })

  describe('filteredOrphans computed', () => {
    beforeEach(async () => {
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
    })

    it('should return all orphans when no search query', async () => {
      const store = useCardListStore()
      await store.loadOrphans()

      expect(store.filteredOrphans).toHaveLength(2)
    })

    it('should filter orphans by search query (case insensitive)', async () => {
      const store = useCardListStore()
      await store.loadOrphans()

      store.searchQuery = 'orphan card 1'

      expect(store.filteredOrphans).toHaveLength(1)
      expect(store.filteredOrphans[0].id).toBe('orphan-1')
    })

    it('should return empty array when no matches', async () => {
      const store = useCardListStore()
      await store.loadOrphans()

      store.searchQuery = 'nonexistent'

      expect(store.filteredOrphans).toHaveLength(0)
    })

    it('should filter by selected category', async () => {
      const store = useCardListStore()
      await store.loadOrphans()

      store.selectedCategoryId = 1

      expect(store.filteredOrphans).toHaveLength(1)
      expect(store.filteredOrphans[0].id).toBe('orphan-2')
    })

    it('should combine search query and category filter', async () => {
      const store = useCardListStore()
      await store.loadOrphans()

      store.searchQuery = 'orphan'
      store.selectedCategoryId = 1

      expect(store.filteredOrphans).toHaveLength(1)
      expect(store.filteredOrphans[0].id).toBe('orphan-2')
    })

    it('should show all when selectedCategoryId is null', async () => {
      const store = useCardListStore()
      await store.loadOrphans()

      store.selectedCategoryId = null

      expect(store.filteredOrphans).toHaveLength(2)
    })
  })

  describe('filteredRecent computed', () => {
    beforeEach(async () => {
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
    })

    it('should return all recent when no search query', async () => {
      const store = useCardListStore()
      await store.loadRecent()

      expect(store.filteredRecent).toHaveLength(3)
    })

    it('should filter recent cards by search query', async () => {
      const store = useCardListStore()
      await store.loadRecent()

      store.searchQuery = 'recent card 2'

      expect(store.filteredRecent).toHaveLength(1)
      expect(store.filteredRecent[0].id).toBe('recent-2')
    })

    it('should filter by selected category', async () => {
      const store = useCardListStore()
      await store.loadRecent()

      store.selectedCategoryId = 1

      expect(store.filteredRecent).toHaveLength(1)
      expect(store.filteredRecent[0].id).toBe('recent-1')
    })

    it('should combine search query and category filter', async () => {
      const store = useCardListStore()
      await store.loadRecent()

      store.searchQuery = 'card'
      store.selectedCategoryId = 2

      expect(store.filteredRecent).toHaveLength(1)
      expect(store.filteredRecent[0].id).toBe('recent-3')
    })
  })

  describe('searchQuery reactivity', () => {
    it('should update filtered results when searchQuery changes', async () => {
      mockInvoke.mockResolvedValueOnce(mockRecentCards)

      const store = useCardListStore()
      await store.loadRecent()

      expect(store.filteredRecent).toHaveLength(3)

      store.searchQuery = 'nonexistent'
      expect(store.filteredRecent).toHaveLength(0)

      store.searchQuery = 'recent'
      expect(store.filteredRecent).toHaveLength(3)
    })
  })

  describe('selectedCategoryId reactivity', () => {
    it('should update filtered results when category changes', async () => {
      mockInvoke.mockResolvedValueOnce(mockRecentCards)

      const store = useCardListStore()
      await store.loadRecent()

      store.selectedCategoryId = 1
      expect(store.filteredRecent).toHaveLength(1)

      store.selectedCategoryId = null
      expect(store.filteredRecent).toHaveLength(3)

      store.selectedCategoryId = 999
      expect(store.filteredRecent).toHaveLength(0)
    })
  })
})
