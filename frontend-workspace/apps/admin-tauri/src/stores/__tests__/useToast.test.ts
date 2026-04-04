// @vitest-environment jsdom
import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useToast } from '../useToast'

describe('useToast', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  describe('initial state', () => {
    it('should have empty toasts array initially', () => {
      const store = useToast()

      expect(store.toasts).toEqual([])
    })
  })

  describe('addToast', () => {
    it('should add a toast with default type info', () => {
      const store = useToast()

      store.addToast('Test message')

      expect(store.toasts).toHaveLength(1)
      expect(store.toasts[0].text).toBe('Test message')
      expect(store.toasts[0].type).toBe('info')
    })

    it('should add a success toast', () => {
      const store = useToast()

      store.addToast('Success!', 'success')

      expect(store.toasts[0].type).toBe('success')
    })

    it('should add an error toast', () => {
      const store = useToast()

      store.addToast('Error!', 'error')

      expect(store.toasts[0].type).toBe('error')
    })

    it('should add an info toast explicitly', () => {
      const store = useToast()

      store.addToast('Info!', 'info')

      expect(store.toasts[0].type).toBe('info')
    })

    it('should generate unique IDs for each toast', () => {
      const store = useToast()

      store.addToast('First')
      store.addToast('Second')
      store.addToast('Third')

      const ids = store.toasts.map((t) => t.id)
      expect(new Set(ids).size).toBe(3)
    })

    it('should increment IDs sequentially', () => {
      const store = useToast()

      store.addToast('First')
      store.addToast('Second')
      store.addToast('Third')

      expect(store.toasts[0].id).toBeLessThan(store.toasts[1].id)
      expect(store.toasts[1].id).toBeLessThan(store.toasts[2].id)
    })

    it('should auto-remove toast after 3 seconds', async () => {
      const store = useToast()

      store.addToast('Will disappear')
      expect(store.toasts).toHaveLength(1)

      vi.advanceTimersByTime(2999)
      expect(store.toasts).toHaveLength(1)

      vi.advanceTimersByTime(1)
      expect(store.toasts).toHaveLength(0)
    })

    it('should remove only the correct toast after timeout', () => {
      const store = useToast()

      store.addToast('First')
      vi.advanceTimersByTime(1000)
      store.addToast('Second')
      vi.advanceTimersByTime(2000)

      expect(store.toasts).toHaveLength(1)
      expect(store.toasts[0].text).toBe('Second')
    })

    it('should handle multiple toasts with independent timeouts', () => {
      const store = useToast()

      store.addToast('First')
      store.addToast('Second')
      store.addToast('Third')

      expect(store.toasts).toHaveLength(3)

      vi.advanceTimersByTime(3000)
      expect(store.toasts).toHaveLength(0)
    })
  })

  describe('multiple toasts', () => {
    it('should stack multiple toasts', () => {
      const store = useToast()

      store.addToast('First', 'info')
      store.addToast('Second', 'success')
      store.addToast('Third', 'error')

      expect(store.toasts).toHaveLength(3)
      expect(store.toasts[0].text).toBe('First')
      expect(store.toasts[1].text).toBe('Second')
      expect(store.toasts[2].text).toBe('Third')
    })

    it('should maintain correct types for stacked toasts', () => {
      const store = useToast()

      store.addToast('First', 'info')
      store.addToast('Second', 'success')
      store.addToast('Third', 'error')

      expect(store.toasts[0].type).toBe('info')
      expect(store.toasts[1].type).toBe('success')
      expect(store.toasts[2].type).toBe('error')
    })
  })

  describe('edge cases', () => {
    it('should handle empty message', () => {
      const store = useToast()

      store.addToast('')

      expect(store.toasts).toHaveLength(1)
      expect(store.toasts[0].text).toBe('')
    })

    it('should handle long messages', () => {
      const store = useToast()
      const longMessage = 'A'.repeat(1000)

      store.addToast(longMessage)

      expect(store.toasts[0].text).toBe(longMessage)
    })

    it('should handle special characters in message', () => {
      const store = useToast()
      const specialMessage = '<script>alert("xss")</script> & "quotes"'

      store.addToast(specialMessage)

      expect(store.toasts[0].text).toBe(specialMessage)
    })

    it('should handle unicode characters', () => {
      const store = useToast()
      const unicodeMessage = '你好世界 🎉 ñ é ü'

      store.addToast(unicodeMessage)

      expect(store.toasts[0].text).toBe(unicodeMessage)
    })
  })

  describe('reactivity', () => {
    it('should maintain reactivity when adding toasts', () => {
      const store = useToast()

      store.addToast('First')
      const initialLength = store.toasts.length

      store.addToast('Second')

      expect(store.toasts.length).toBe(initialLength + 1)
    })

    it('should maintain reactivity when removing toasts', () => {
      const store = useToast()

      store.addToast('Test')
      vi.advanceTimersByTime(3000)

      expect(store.toasts.length).toBe(0)
    })
  })
})
