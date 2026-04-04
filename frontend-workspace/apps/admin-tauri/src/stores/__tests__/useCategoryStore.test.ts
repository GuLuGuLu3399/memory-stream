// @vitest-environment jsdom
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useCategoryStore } from '../useCategoryStore'
import { useToast } from '../useToast'
import { z } from 'zod'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'

const mockInvoke = vi.mocked(invoke)

const mockCategoriesResponse = {
  categories: [
    { id: 1, name: 'Category 1', description: 'Desc 1', parent_id: null, created_at: '2024-01-01T00:00:00Z', updated_at: '2024-01-01T00:00:00Z', theme_color: 'cyan' },
    { id: 2, name: 'Category 2', description: 'Desc 2', parent_id: 1, created_at: '2024-01-02T00:00:00Z', updated_at: '2024-01-02T00:00:00Z', theme_color: null },
    { id: 3, name: 'Category 3', description: '', parent_id: null, created_at: '2024-01-03T00:00:00Z', updated_at: '', theme_color: 'orange' },
  ],
}

describe('useCategoryStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('should have empty categories array initially', () => {
      const store = useCategoryStore()

      expect(store.categories).toEqual([])
    })
  })

  describe('loadCategories', () => {
    it('should load categories from API', async () => {
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()
      await store.loadCategories()

      expect(mockInvoke).toHaveBeenCalledWith('api_request', {
        method: 'GET',
        endpoint: '/categories',
      })
    })

    it('should populate categories with correct data', async () => {
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()
      await store.loadCategories()

      expect(store.categories).toHaveLength(3)
      expect(store.categories[0].id).toBe(1)
      expect(store.categories[0].name).toBe('Category 1')
      expect(store.categories[0].description).toBe('Desc 1')
    })

    it('should handle null parent_id correctly', async () => {
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()
      await store.loadCategories()

      expect(store.categories[0].parent_id).toBeNull()
      expect(store.categories[1].parent_id).toBe(1)
    })

    it('should handle missing description with default', async () => {
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()
      await store.loadCategories()

      expect(store.categories[2].description).toBe('')
    })

    it('should handle theme_color correctly', async () => {
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()
      await store.loadCategories()

      expect(store.categories[0].theme_color).toBe('cyan')
      expect(store.categories[1].theme_color).toBeNull()
    })

    it('should handle Zod validation error gracefully', async () => {
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      mockInvoke.mockResolvedValueOnce({
        categories: [{ invalid_field: 'bad data' }],
      })

      const store = useCategoryStore()
      await store.loadCategories()

      expect(consoleSpy).toHaveBeenCalled()
      expect(addToastSpy).toHaveBeenCalledWith('分类数据格式异常', 'error')

      consoleSpy.mockRestore()
    })

    it('should handle generic invoke error', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      mockInvoke.mockRejectedValueOnce(new Error('Network error'))

      const store = useCategoryStore()
      await store.loadCategories()

      expect(consoleSpy).toHaveBeenCalled()
      expect(store.categories).toEqual([])

      consoleSpy.mockRestore()
    })
  })

  describe('createCategory', () => {
    it('should create category with name only', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')

      await store.createCategory('New Category')

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'api_request', {
        method: 'POST',
        endpoint: '/categories',
        body: { name: 'New Category', description: '' },
      })
      expect(addToastSpy).toHaveBeenCalledWith('分类已创建 ✓', 'success')
    })

    it('should create category with name and description', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()

      await store.createCategory('New Category', 'Description here')

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'api_request', {
        method: 'POST',
        endpoint: '/categories',
        body: { name: 'New Category', description: 'Description here' },
      })
    })

    it('should reload categories after create', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()
      await store.createCategory('New Category')

      expect(mockInvoke).toHaveBeenCalledTimes(2)
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'api_request', {
        method: 'GET',
        endpoint: '/categories',
      })
    })

    it('should show error toast on create failure', async () => {
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')

      mockInvoke.mockRejectedValueOnce(new Error('Create failed'))

      const store = useCategoryStore()
      await store.createCategory('New Category')

      expect(addToastSpy).toHaveBeenCalledWith('创建分类失败: Error: Create failed', 'error')
    })
  })

  describe('updateCategory', () => {
    it('should update category with all fields', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()

      await store.updateCategory(1, 'Updated Name', 'Updated desc', 'blue')

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'api_request', {
        method: 'PUT',
        endpoint: '/categories/1',
        body: { name: 'Updated Name', description: 'Updated desc', theme_color: 'blue' },
      })
    })

    it('should update category with null theme_color', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()

      await store.updateCategory(1, 'Name', 'Desc', null)

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'api_request', expect.objectContaining({
        body: expect.objectContaining({ theme_color: null }),
      }))
    })

    it('should show success toast on update', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')

      await store.updateCategory(1, 'Name')

      expect(addToastSpy).toHaveBeenCalledWith('分类已更新 ✓', 'success')
    })

    it('should reload categories after update', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()
      await store.updateCategory(1, 'Name')

      expect(mockInvoke).toHaveBeenCalledTimes(2)
    })

    it('should show error toast on update failure', async () => {
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')

      mockInvoke.mockRejectedValueOnce(new Error('Update failed'))

      const store = useCategoryStore()
      await store.updateCategory(1, 'Name')

      expect(addToastSpy).toHaveBeenCalledWith('更新分类失败: Error: Update failed', 'error')
    })
  })

  describe('deleteCategory', () => {
    it('should delete category via API', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()

      await store.deleteCategory(1)

      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'api_request', {
        method: 'DELETE',
        endpoint: '/categories/1',
      })
    })

    it('should show success toast on delete', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')

      await store.deleteCategory(1)

      expect(addToastSpy).toHaveBeenCalledWith('分类已删除 ✓', 'success')
    })

    it('should reload categories after delete', async () => {
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()
      await store.deleteCategory(1)

      expect(mockInvoke).toHaveBeenCalledTimes(2)
    })

    it('should show error toast on delete failure', async () => {
      const toastStore = useToast()
      const addToastSpy = vi.spyOn(toastStore, 'addToast')

      mockInvoke.mockRejectedValueOnce(new Error('Delete failed'))

      const store = useCategoryStore()
      await store.deleteCategory(1)

      expect(addToastSpy).toHaveBeenCalledWith('删除分类失败: Error: Delete failed', 'error')
    })
  })

  describe('CRUD workflow', () => {
    it('should handle full CRUD lifecycle', async () => {
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)
      mockInvoke.mockResolvedValueOnce(undefined)
      mockInvoke.mockResolvedValueOnce(mockCategoriesResponse)

      const store = useCategoryStore()

      await store.loadCategories()
      expect(store.categories).toHaveLength(3)

      await store.createCategory('New', 'Desc')
      await store.updateCategory(1, 'Updated', 'Updated desc')
      await store.deleteCategory(1)

      expect(mockInvoke).toHaveBeenCalledTimes(7)
    })
  })
})
