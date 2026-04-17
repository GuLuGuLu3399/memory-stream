// @vitest-environment jsdom
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useEdgeStore } from '../useEdgeStore'
import { useToast } from '../useToast'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'

const mockInvoke = vi.mocked(invoke)

describe('useEdgeStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('createEdgeHttp', () => {
    it('should create edge with default relation type', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useEdgeStore()
      await store.createEdgeHttp('source-1', 'target-1')

      expect(mockInvoke).toHaveBeenCalledWith('api_request', {
        method: 'POST',
        endpoint: '/edges',
        body: {
          source_id: 'source-1',
          target_id: 'target-1',
          relation_type: 'reference',
        },
      })
    })

    it('should create edge with custom relation type', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useEdgeStore()
      await store.createEdgeHttp('source-1', 'target-1', 'sequence')

      expect(mockInvoke).toHaveBeenCalledWith('api_request', {
        method: 'POST',
        endpoint: '/edges',
        body: {
          source_id: 'source-1',
          target_id: 'target-1',
          relation_type: 'sequence',
        },
      })
    })

    it('should show success toast on create', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useEdgeStore()
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')

      await store.createEdgeHttp('source-1', 'target-1')

      expect(addToastSpy).toHaveBeenCalledWith('连线已创建 ✓', 'success')
    })

    it('should show error toast and rethrow on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))

      const store = useEdgeStore()
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')

      await expect(store.createEdgeHttp('source-1', 'target-1')).rejects.toThrow()

      expect(addToastSpy).toHaveBeenCalledWith('创建连线失败: Network error', 'error')
    })
  })

  describe('deleteEdgeHttp', () => {
    it('should delete edge via API', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useEdgeStore()
      await store.deleteEdgeHttp('source-1', 'target-1')

      expect(mockInvoke).toHaveBeenCalledWith('api_request', {
        method: 'DELETE',
        endpoint: '/edges',
        body: {
          source_id: 'source-1',
          target_id: 'target-1',
        },
      })
    })

    it('should show success toast on delete', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useEdgeStore()
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')

      await store.deleteEdgeHttp('source-1', 'target-1')

      expect(addToastSpy).toHaveBeenCalledWith('连线已断开 ✓', 'success')
    })

    it('should show error toast and rethrow on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Delete failed'))

      const store = useEdgeStore()
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')

      await expect(store.deleteEdgeHttp('source-1', 'target-1')).rejects.toThrow()

      expect(addToastSpy).toHaveBeenCalledWith('删除连线失败: Delete failed', 'error')
    })
  })

  describe('updateEdgeType', () => {
    it('should update edge type via PATCH request', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useEdgeStore()
      await store.updateEdgeType('source-1', 'target-1', 'sequence')

      expect(mockInvoke).toHaveBeenCalledWith('api_request', {
        method: 'PATCH',
        endpoint: '/edges',
        body: {
          source_id: 'source-1',
          target_id: 'target-1',
          relation_type: 'sequence',
        },
      })
    })

    it('should show success toast on update', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useEdgeStore()
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')

      await store.updateEdgeType('source-1', 'target-1', 'reference')

      expect(addToastSpy).toHaveBeenCalledWith('连线类型已更新 ✓', 'success')
    })

    it('should show error toast and rethrow on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Update failed'))

      const store = useEdgeStore()
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')

      await expect(store.updateEdgeType('source-1', 'target-1', 'sequence')).rejects.toThrow()

      expect(addToastSpy).toHaveBeenCalledWith('更新连线失败: Update failed', 'error')
    })
  })

  describe('createEdge (WebSocket)', () => {
    it('should create edge via WebSocket command', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useEdgeStore()
      await store.createEdge('source-1', 'target-1', 'reference')

      expect(mockInvoke).toHaveBeenCalledWith('create_edge_cmd', {
        source: 'source-1',
        target: 'target-1',
        rel: 'reference',
      })
    })

    it('should log error and rethrow on failure', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      mockInvoke.mockRejectedValueOnce(new Error('WS error'))

      const store = useEdgeStore()

      await expect(store.createEdge('source-1', 'target-1', 'sequence')).rejects.toThrow()

      expect(consoleSpy).toHaveBeenCalled()

      consoleSpy.mockRestore()
    })
  })

  describe('deleteEdge (WebSocket)', () => {
    it('should delete edge via WebSocket command', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useEdgeStore()
      await store.deleteEdge('source-1', 'target-1')

      expect(mockInvoke).toHaveBeenCalledWith('delete_edge_cmd', {
        source: 'source-1',
        target: 'target-1',
      })
    })

    it('should log error and rethrow on failure', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      mockInvoke.mockRejectedValueOnce(new Error('WS delete error'))

      const store = useEdgeStore()

      await expect(store.deleteEdge('source-1', 'target-1')).rejects.toThrow()

      expect(consoleSpy).toHaveBeenCalled()

      consoleSpy.mockRestore()
    })
  })

  describe('integration scenarios', () => {
    it('should handle sequential create then delete', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useEdgeStore()

      await store.createEdgeHttp('node-a', 'node-b')
      await store.deleteEdgeHttp('node-a', 'node-b')

      expect(mockInvoke).toHaveBeenCalledTimes(2)
    })

    it('should handle update after create', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(undefined)

      const store = useEdgeStore()

      await store.createEdgeHttp('node-a', 'node-b', 'reference')
      await store.updateEdgeType('node-a', 'node-b', 'sequence')

      expect(mockInvoke).toHaveBeenCalledTimes(2)
    })
  })
})
