// @vitest-environment jsdom
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'
import { useKnowledgeStore } from '../knowledge'
import { useToast } from '../useToast'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'

const mockInvoke = vi.mocked(invoke)
const mockSave = vi.mocked(save)

const mockCardDetail = {
  id: 'card-1',
  title: 'Test Card',
  raw_md: '# Test Content',
  x: 100,
  y: 200,
  updated_at: '2024-01-01T00:00:00Z',
  category_id: 1,
}

 const mockProcessResult = {
  html: '<h1>Test</h1>',
  ast_json: '{"type":"root"}',
  excerpt: 'Test excerpt',
  extracted_links: [],
}

 const mockCategoriesResponse = {
  categories: [
    { id: 1, name: 'Category 1', description: '', parent_id: null, created_at: '2024-01-01T00:00:00Z' },
  ],
}

 const mockRecentCards = {
  data: [
    { id: 'recent-1', title: 'Recent Card', excerpt: 'Excerpt', x: 0, y: 0, category_id: null },
  ],
}
 const mockOrphanCards = {
  data: [
    { id: 'orphan-1', title: 'Orphan Card', excerpt: 'Excerpt', x: 0, y: 0, category_id: null },
  ],
}
 const mockGraphData = {
  nodes: [{ id: 'node-1', title: 'Node' }],
  edges: [],
}

 describe('useKnowledgeStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('should have null activeCard initially', () => {
      const store = useKnowledgeStore()
      expect(store.activeCard).toBeNull()
    })

    it('should have isLoading false initially', () => {
      const store = useKnowledgeStore()
      expect(store.isLoading).toBe(false)
    })
    it('should have isDirty false initially', () => {
      const store = useKnowledgeStore()
      expect(store.isDirty).toBe(false)
    })
    it('should have justSaved false initially', () => {
      const store = useKnowledgeStore()
      expect(store.justSaved).toBe(false)
    })
    it('should have searchFocused false initially', () => {
      const store = useKnowledgeStore()
      expect(store.searchFocused).toBe(false)
    })
    it('should have empty backlinks initially', () => {
      const store = useKnowledgeStore()
      expect(store.backlinks).toEqual([])
    })
  })

  describe('newCard', () => {
    it('should create a new empty card', () => {
      const store = useKnowledgeStore()
      store.newCard()
      expect(store.activeCard).toEqual({ id: '', title: '', content: '', x: 0, y: 0 })
      expect(store.isDirty).toBe(false)
    })
    it('should clear local graph', () => {
      const store = useKnowledgeStore()
      store.newCard()
      expect(store.localNodes).toEqual([])
      expect(store.localEdges).toEqual([])
    })
  })

  describe('setActiveCard', () => {
    it('should set active card without loading content', async () => {
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      const card = { id: 'card-1', title: 'Test', content: 'Content', x: 0, y: 0 }
      await store.setActiveCard(card)
      expect(store.activeCard).toEqual(card)
    })
    it('should capture snapshot after setting', async () => {
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      const card = { id: 'card-1', title: 'Test', content: 'Content', x: 0, y: 0 }
      await store.setActiveCard(card)
      store.checkDirty()
      expect(store.isDirty).toBe(false)
    })
    it('should NOT set card if dirty discard is cancelled', async () => {
      const confirmDialog = await import('../../composables/useConfirmDialog')
      const mockConfirm = vi.fn().mockResolvedValue(false)
      vi.spyOn(confirmDialog, 'useConfirmDialog').mockReturnValue({
        confirm: mockConfirm,
        dialogState: { value: { visible: false } } as ReturnType<typeof confirmDialog.useConfirmDialog>['dialogState'],
        handleConfirm: vi.fn(),
        handleCancel: vi.fn(),
      } as any)
      const store = useKnowledgeStore()
      store.activeCard = { id: 'existing', title: 'Existing', content: 'Old', x: 0, y: 0 }
      store.isDirty = true
      await store.setActiveCard({ id: 'new', title: 'New', content: 'New', x: 0, y: 0 })
      expect(store.activeCard?.id).toBe('existing')
    })
  })

  describe('loadAndActivateCard', () => {
    it('should load card detail and activate', async () => {
      mockInvoke.mockResolvedValueOnce(mockCardDetail)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      mockInvoke.mockResolvedValueOnce({ backlinks: [] })
      const store = useKnowledgeStore()
      await store.loadAndActivateCard('card-1')
      expect(mockInvoke).toHaveBeenCalledWith('get_card_detail', { id: 'card-1' })
      expect(store.activeCard?.id).toBe('card-1')
      expect(store.activeCard?.title).toBe('Test Card')
    })
    it('should not load if cardId is empty', async () => {
      const store = useKnowledgeStore()
      await store.loadAndActivateCard('')
      expect(mockInvoke).not.toHaveBeenCalled()
    })
    it('should show toast on error', async () => {
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')
      mockInvoke.mockRejectedValueOnce(new Error('Load failed'))
      const store = useKnowledgeStore()
      await store.loadAndActivateCard('card-1')
      expect(addToastSpy).toHaveBeenCalledWith('加载卡片失败: Error: Load failed', 'error')
    })
    it('should set isLoading correctly', async () => {
      mockInvoke.mockResolvedValueOnce(mockCardDetail)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      mockInvoke.mockResolvedValueOnce({ backlinks: [] })
      const store = useKnowledgeStore()
      const promise = store.loadAndActivateCard('card-1')
      await nextTick()
      expect(store.isLoading).toBe(true)
      await promise
      expect(store.isLoading).toBe(false)
    })
  })

  describe('loadAndActivateCardByTitle', () => {
    it('should find and load card by title', async () => {
      mockInvoke.mockResolvedValueOnce(mockCardDetail)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      mockInvoke.mockResolvedValueOnce({ backlinks: [] })
      const store = useKnowledgeStore()
      store.recentCards = [{ id: 'card-1', title: 'Find Me', content: '', x: 0, y: 0 }]
      await store.loadAndActivateCardByTitle('Find Me')
      expect(mockInvoke).toHaveBeenCalledWith('get_card_detail', { id: 'card-1' })
    })
    it('should create empty card if not found', async () => {
      const store = useKnowledgeStore()
      store.recentCards = []
      store.loadAndActivateCardByTitle('Nonexistent')
      expect(store.activeCard?.title).toBe('Nonexistent')
      expect(store.activeCard?.id).toBe('')
      expect(store.localNodes).toEqual([])
    })
  })

  describe('saveCard - update existing', () => {
    it('should update existing card via API', async () => {
      mockInvoke.mockResolvedValueOnce(mockProcessResult)
      mockInvoke.mockResolvedValueOnce(null)
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      store.activeCard = { id: 'card-1', title: 'Updated', content: 'Content', x: 0, y: 0 }
      await store.saveCard()
      expect(mockInvoke).toHaveBeenCalledWith('process_markdown', { content: 'Content' })
      expect(mockInvoke).toHaveBeenCalledWith('api_request', expect.objectContaining({
        method: 'PUT',
        endpoint: '/cards/card-1',
      }))
    })
    it('should show success toast on save', async () => {
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')
      mockInvoke.mockResolvedValueOnce(mockProcessResult)
      mockInvoke.mockResolvedValueOnce(null)
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      store.activeCard = { id: 'card-1', title: 'Test', content: 'Test', x: 0, y: 0 }
      await store.saveCard()
      expect(addToastSpy).toHaveBeenCalledWith('卡片已保存 ✓', 'success')
    })
    it('should set justSaved flag temporarily', async () => {
      vi.useFakeTimers()
      mockInvoke.mockResolvedValueOnce(mockProcessResult)
      mockInvoke.mockResolvedValueOnce(null)
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      store.activeCard = { id: 'card-1', title: 'Test', content: 'Test', x: 0, y: 0 }
      await store.saveCard()
      expect(store.justSaved).toBe(true)
      await vi.advanceTimersByTimeAsync(1500)
      expect(store.justSaved).toBe(false)
      vi.useRealTimers()
    })
    it('should include category_id in update body if set', async () => {
      mockInvoke.mockResolvedValueOnce(mockProcessResult)
      mockInvoke.mockResolvedValueOnce(null)
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      store.activeCard = { id: 'card-1', title: 'Test', content: 'Test', x: 0, y: 0, category_id: 5 }
      await store.saveCard()
      const apiCall = mockInvoke.mock.calls.find(c => c[0] === 'api_request')
      expect(apiCall![1]).toHaveProperty('body.category_id', 5)
    })
  })

  describe('saveCard - create new', () => {
    it('should create new card without id', async () => {
      mockInvoke.mockResolvedValueOnce(mockProcessResult)
      mockInvoke.mockResolvedValueOnce(null)
      mockInvoke.mockResolvedValueOnce('new-card-id')
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      store.activeCard = { id: '', title: 'New Card', content: 'Content', x: 0, y: 0 }
      await store.saveCard()
      expect(mockInvoke).toHaveBeenCalledWith('create_card_with_relation', expect.objectContaining({
        title: 'New Card',
        content: 'Content',
      }))
    })
    it('should update activeCard.id after creation', async () => {
      mockInvoke.mockResolvedValueOnce(mockProcessResult)
      mockInvoke.mockResolvedValueOnce(null)
      mockInvoke.mockResolvedValueOnce('new-card-id')
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      store.activeCard = { id: '', title: 'New', content: 'Test', x: 0, y: 0 }
      await store.saveCard()
      expect(store.activeCard?.id).toBe('new-card-id')
    })
  })

  describe('deleteCard', () => {
    it('should delete card via API', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      const store = useKnowledgeStore()
      await store.deleteCard('card-1')
      expect(mockInvoke).toHaveBeenCalledWith('delete_card', { id: 'card-1' })
    })
    it('should clear activeCard if it was the deleted card', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      const store = useKnowledgeStore()
      store.activeCard = { id: 'card-1', title: 'Test', content: 'Test', x: 0, y: 0 }
      await store.deleteCard('card-1')
      expect(store.activeCard).toBeNull()
    })
    it('should NOT clear activeCard if it was a different card', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      store.activeCard = { id: 'card-2', title: 'Other', content: 'Test', x: 0, y: 0 }
      await store.deleteCard('card-1')
      expect(store.activeCard?.id).toBe('card-2')
    })
    it('should not delete if cardId is empty', async () => {
      const store = useKnowledgeStore()
      await store.deleteCard('')
      expect(mockInvoke).not.toHaveBeenCalled()
    })
  })

  describe('saveDraft / loadDraft', () => {
    it('should save draft to local storage', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      const store = useKnowledgeStore()
      store.activeCard = { id: 'card-1', title: 'Test', content: 'Draft content', x: 0, y: 0 }
      await store.saveDraft()
      expect(mockInvoke).toHaveBeenCalledWith('save_draft', {
        cardId: 'card-1',
        rawMd: 'Draft content',
        astData: null,
      })
    })
    it('should not save draft if no activeCard id', async () => {
      const store = useKnowledgeStore()
      store.activeCard = { id: '', title: 'Test', content: 'Draft', x: 0, y: 0 }
      await store.saveDraft()
      expect(mockInvoke).not.toHaveBeenCalled()
    })
    it('should load draft from local storage', async () => {
      mockInvoke.mockResolvedValueOnce({ card_id: 'card-1', raw_md: 'Draft content', ast_data: null, updated_at: 123 })
      const store = useKnowledgeStore()
      const draft = await store.loadDraft('card-1')
      expect(draft).toBe('Draft content')
    })
    it('should return null if no draft found', async () => {
      mockInvoke.mockResolvedValueOnce(null)
      const store = useKnowledgeStore()
      const draft = await store.loadDraft('card-1')
      expect(draft).toBeNull()
    })
    it('should return null on error', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      mockInvoke.mockRejectedValueOnce(new Error('Load draft failed'))
      const store = useKnowledgeStore()
      const draft = await store.loadDraft('card-1')
      expect(draft).toBeNull()
      consoleSpy.mockRestore()
    })
  })

  describe('listDrafts / deleteDraft', () => {
    it('should list all drafts', async () => {
      const mockDrafts = [
        { card_id: 'card-1', raw_md: 'Draft 1', updated_at: 123 },
        { card_id: 'card-2', raw_md: 'Draft 2', updated_at: 456 },
      ]
      mockInvoke.mockResolvedValueOnce(mockDrafts)
      const store = useKnowledgeStore()
      const drafts = await store.listDrafts()
      expect(drafts).toEqual(mockDrafts)
    })
    it('should return empty array on error', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      mockInvoke.mockRejectedValueOnce(new Error('List failed'))
      const store = useKnowledgeStore()
      const drafts = await store.listDrafts()
      expect(drafts).toEqual([])
      consoleSpy.mockRestore()
    })
    it('should delete draft', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      const store = useKnowledgeStore()
      await store.deleteDraft('card-1')
      expect(mockInvoke).toHaveBeenCalledWith('delete_draft', { cardId: 'card-1' })
    })
  })

  describe('exportKb', () => {
    it('should export knowledge base to ZIP', async () => {
      mockSave.mockResolvedValueOnce('/path/to/export.zip')
      mockInvoke.mockResolvedValueOnce({ total_cards: 5, total_images: 2, zip_size_bytes: 10240 })
      const store = useKnowledgeStore()
      store.recentCards = [{ id: '1', title: 'Card', content: 'Content', x: 0, y: 0 }]
      const result = await store.exportKb()
      expect(result).toBe(true)
      expect(mockInvoke).toHaveBeenCalledWith('export_knowledge_base', expect.objectContaining({
        destPath: '/path/to/export.zip',
      }))
    })
    it('should return false if user cancels save dialog', async () => {
      mockSave.mockResolvedValueOnce(null)
      const store = useKnowledgeStore()
      const result = await store.exportKb()
      expect(result).toBe(false)
      expect(mockInvoke).not.toHaveBeenCalledWith('export_knowledge_base', expect.anything())
    })
    it('should show error toast on export failure', async () => {
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')
      mockSave.mockResolvedValueOnce('/path/to/export.zip')
      mockInvoke.mockRejectedValueOnce(new Error('Export failed'))
      const store = useKnowledgeStore()
      const result = await store.exportKb()
      expect(result).toBe(false)
      expect(addToastSpy).toHaveBeenCalledWith('导出失败: Error: Export failed', 'error')
    })
  })

  describe('checkDirty / captureSnapshot', () => {
    it('should detect changes to title', async () => {
      mockInvoke.mockResolvedValueOnce(mockCardDetail)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      mockInvoke.mockResolvedValueOnce({ backlinks: [] })
      const store = useKnowledgeStore()
      await store.loadAndActivateCard('card-1')
      store.activeCard!.title = 'Modified Title'
      store.checkDirty()
      expect(store.isDirty).toBe(true)
    })
    it('should detect changes to content', async () => {
      mockInvoke.mockResolvedValueOnce(mockCardDetail)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      mockInvoke.mockResolvedValueOnce({ backlinks: [] })
      const store = useKnowledgeStore()
      await store.loadAndActivateCard('card-1')
      store.activeCard!.content = 'Modified content'
      store.checkDirty()
      expect(store.isDirty).toBe(true)
    })
    it('should not detect changes if nothing modified', async () => {
      mockInvoke.mockResolvedValueOnce(mockCardDetail)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      mockInvoke.mockResolvedValueOnce({ backlinks: [] })
      const store = useKnowledgeStore()
      await store.loadAndActivateCard('card-1')
      store.checkDirty()
      expect(store.isDirty).toBe(false)
    })
  })

  describe('silentRefresh', () => {
    it('should debounce refresh calls', async () => {
      vi.useFakeTimers()
      mockInvoke.mockResolvedValue(mockRecentCards)
      const store = useKnowledgeStore()
      store.silentRefresh()
      store.silentRefresh()
      store.silentRefresh()
      expect(mockInvoke).not.toHaveBeenCalled()
      await vi.advanceTimersByTimeAsync(500)
      expect(mockInvoke).toHaveBeenCalled()
      vi.useRealTimers()
    })
  })

  describe('updateLayouts', () => {
    it('should update node positions in localNodes', async () => {
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      await store.loadLocalGraph('card-1')
      store.localNodes = [
        { id: 'node-1', title: 'Node 1', x: 0, y: 0 },
        { id: 'node-2', title: 'Node 2', x: 0, y: 0 },
      ]
      store.updateLayouts([
        { id: 'node-1', x: 100, y: 200 },
        { id: 'node-2', x: 300, y: 400 },
      ])
      expect(store.localNodes[0].x).toBe(100)
      expect(store.localNodes[0].y).toBe(200)
      expect(store.localNodes[1].x).toBe(300)
      expect(store.localNodes[1].y).toBe(400)
    })
    it('should not modify nodes not in layouts array', () => {
      const store = useKnowledgeStore()
      store.localNodes = [
        { id: 'node-1', title: 'Node 1', x: 10, y: 20 },
        { id: 'node-2', title: 'Node 2', x: 30, y: 40 },
      ]
      store.updateLayouts([{ id: 'node-1', x: 100, y: 200 }])
      expect(store.localNodes[1].x).toBe(30)
      expect(store.localNodes[1].y).toBe(40)
    })
  })

  describe('edge operations wrappers', () => {
    it('should call edgeStore.createEdgeHttp and refresh', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      await store.createEdgeHttp('source-1', 'target-1')
      expect(mockInvoke).toHaveBeenCalledWith('api_request', expect.objectContaining({
        method: 'POST',
        endpoint: '/edges',
      }))
    })
    it('should call edgeStore.deleteEdgeHttp and refresh', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      await store.deleteEdgeHttp('source-1', 'target-1')
      expect(mockInvoke).toHaveBeenCalledWith('api_request', expect.objectContaining({
        method: 'DELETE',
        endpoint: '/edges',
      }))
    })
    it('should call edgeStore.updateEdgeType and refresh', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      await store.updateEdgeType('source-1', 'target-1', 'sequence')
      expect(mockInvoke).toHaveBeenCalledWith('api_request', expect.objectContaining({
        method: 'PATCH',
        endpoint: '/edges',
      }))
    })
  })

  describe('unlinkCardFromCategory', () => {
    it('should unlink card from category', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      await store.unlinkCardFromCategory('card-1')
      expect(mockInvoke).toHaveBeenCalledWith('api_request', expect.objectContaining({
        method: 'PUT',
        endpoint: '/cards/card-1',
        body: { category_id: null },
      }))
    })
    it('should show success toast', async () => {
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      await store.unlinkCardFromCategory('card-1')
      expect(addToastSpy).toHaveBeenCalledWith('已从分类中移除', 'success')
    })
  })

  describe('updateCardCategory', () => {
    it('should update card category', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      await store.updateCardCategory('card-1', 5)
      expect(mockInvoke).toHaveBeenCalledWith('api_request', expect.objectContaining({
        method: 'PUT',
        endpoint: '/cards/card-1',
        body: { category_id: 5 },
      }))
    })
    it('should show success toast', async () => {
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockRecentCards)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(mockOrphanCards)
      mockInvoke.mockResolvedValueOnce(mockGraphData)
      const store = useKnowledgeStore()
      await store.updateCardCategory('card-1', 5)
      expect(addToastSpy).toHaveBeenCalledWith('卡片已迁移', 'success')
    })
  })


})
