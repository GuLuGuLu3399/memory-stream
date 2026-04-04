// @vitest-environment jsdom
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useLocalGraphStore, type LocalGraphNode, type LocalGraphEdge } from '../useLocalGraphStore'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'

const mockInvoke = vi.mocked(invoke)

const mockGraphData = {
  nodes: [
    { id: 'node-1', title: 'First Node' },
    { id: 'node-2', title: 'Second Node' },
    { id: 'node-3', title: 'Third Node' },
  ],
  edges: [
    { source: 'node-1', target: 'node-2', relation: 'reference' },
    { source: 'node-2', target: 'node-3', relation: 'sequence' },
  ],
}

describe('useLocalGraphStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('should have empty nodes array initially', () => {
      const store = useLocalGraphStore()

      expect(store.localNodes).toEqual([])
    })

    it('should have empty edges array initially', () => {
      const store = useLocalGraphStore()

      expect(store.localEdges).toEqual([])
    })
  })

  describe('loadLocalGraph', () => {
    it('should load graph data successfully', async () => {
      mockInvoke.mockResolvedValueOnce(mockGraphData)

      const store = useLocalGraphStore()
      await store.loadLocalGraph('card-123')

      expect(mockInvoke).toHaveBeenCalledWith('api_request', {
        method: 'GET',
        endpoint: '/graph/detail/card-123',
      })
    })

    it('should populate nodes with correct structure', async () => {
      mockInvoke.mockResolvedValueOnce(mockGraphData)

      const store = useLocalGraphStore()
      await store.loadLocalGraph('card-123')

      expect(store.localNodes).toHaveLength(3)
      expect(store.localNodes[0].id).toBe('node-1')
      expect(store.localNodes[0].title).toBe('First Node')
    })

    it('should populate edges with correct structure', async () => {
      mockInvoke.mockResolvedValueOnce(mockGraphData)

      const store = useLocalGraphStore()
      await store.loadLocalGraph('card-123')

      expect(store.localEdges).toHaveLength(2)
      expect(store.localEdges[0].source).toBe('node-1')
      expect(store.localEdges[0].target).toBe('node-2')
      expect(store.localEdges[0].relation).toBe('reference')
    })

    it('should initialize node coordinates to 0,0', async () => {
      mockInvoke.mockResolvedValueOnce(mockGraphData)

      const store = useLocalGraphStore()
      await store.loadLocalGraph('card-123')

      store.localNodes.forEach((node) => {
        expect(node.x).toBe(0)
        expect(node.y).toBe(0)
      })
    })

    it('should not load if cardId is empty', async () => {
      const store = useLocalGraphStore()
      await store.loadLocalGraph('')

      expect(mockInvoke).not.toHaveBeenCalled()
    })

    it('should handle missing node title by using id', async () => {
      mockInvoke.mockResolvedValueOnce({
        nodes: [{ id: 'node-no-title' }],
        edges: [],
      })

      const store = useLocalGraphStore()
      await store.loadLocalGraph('card-123')

      expect(store.localNodes[0].title).toBe('node-no-title')
    })

    it('should handle missing edge relation by defaulting to reference', async () => {
      mockInvoke.mockResolvedValueOnce({
        nodes: [{ id: 'node-1', title: 'Node' }],
        edges: [{ source: 'node-1', target: 'node-2' }],
      })

      const store = useLocalGraphStore()
      await store.loadLocalGraph('card-123')

      expect(store.localEdges[0].relation).toBe('reference')
    })

    it('should handle empty nodes array', async () => {
      mockInvoke.mockResolvedValueOnce({ nodes: [], edges: [] })

      const store = useLocalGraphStore()
      await store.loadLocalGraph('card-123')

      expect(store.localNodes).toEqual([])
      expect(store.localEdges).toEqual([])
    })

    it('should handle missing nodes/edges in response', async () => {
      mockInvoke.mockResolvedValueOnce({})

      const store = useLocalGraphStore()
      await store.loadLocalGraph('card-123')

      expect(store.localNodes).toEqual([])
      expect(store.localEdges).toEqual([])
    })

    it('should handle invoke error gracefully', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))

      const store = useLocalGraphStore()
      await store.loadLocalGraph('card-123')

      expect(consoleSpy).toHaveBeenCalled()
      expect(store.localNodes).toEqual([])
      expect(store.localEdges).toEqual([])

      consoleSpy.mockRestore()
    })

    it('should replace existing graph data on subsequent loads', async () => {
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      mockInvoke.mockResolvedValueOnce({
        nodes: [{ id: 'new-node', title: 'New Node' }],
        edges: [],
      })

      const store = useLocalGraphStore()
      await store.loadLocalGraph('card-123')
      expect(store.localNodes).toHaveLength(3)

      await store.loadLocalGraph('card-456')
      expect(store.localNodes).toHaveLength(1)
      expect(store.localNodes[0].id).toBe('new-node')
    })
  })

  describe('clearLocalGraph', () => {
    it('should clear all nodes', async () => {
      mockInvoke.mockResolvedValueOnce(mockGraphData)

      const store = useLocalGraphStore()
      await store.loadLocalGraph('card-123')
      expect(store.localNodes).toHaveLength(3)

      store.clearLocalGraph()
      expect(store.localNodes).toEqual([])
    })

    it('should clear all edges', async () => {
      mockInvoke.mockResolvedValueOnce(mockGraphData)

      const store = useLocalGraphStore()
      await store.loadLocalGraph('card-123')
      expect(store.localEdges).toHaveLength(2)

      store.clearLocalGraph()
      expect(store.localEdges).toEqual([])
    })

    it('should be safe to call when already empty', () => {
      const store = useLocalGraphStore()

      expect(() => store.clearLocalGraph()).not.toThrow()
      expect(store.localNodes).toEqual([])
      expect(store.localEdges).toEqual([])
    })
  })

  describe('graph data types', () => {
    it('should produce valid LocalGraphNode structures', async () => {
      mockInvoke.mockResolvedValueOnce(mockGraphData)

      const store = useLocalGraphStore()
      await store.loadLocalGraph('card-123')

      const node: LocalGraphNode = store.localNodes[0]
      expect(typeof node.id).toBe('string')
      expect(typeof node.title).toBe('string')
      expect(typeof node.x).toBe('number')
      expect(typeof node.y).toBe('number')
    })

    it('should produce valid LocalGraphEdge structures', async () => {
      mockInvoke.mockResolvedValueOnce(mockGraphData)

      const store = useLocalGraphStore()
      await store.loadLocalGraph('card-123')

      const edge: LocalGraphEdge = store.localEdges[0]
      expect(typeof edge.source).toBe('string')
      expect(typeof edge.target).toBe('string')
      expect(typeof edge.relation).toBe('string')
    })
  })
})
