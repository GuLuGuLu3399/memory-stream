// @vitest-environment jsdom
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useConfirmDialog } from '../useConfirmDialog'

describe('useConfirmDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('initial state', () => {
    it('should have dialog hidden initially', () => {
      const { dialogState } = useConfirmDialog()

      expect(dialogState.value.visible).toBe(false)
    })

    it('should have empty message initially', () => {
      const { dialogState } = useConfirmDialog()

      expect(dialogState.value.message).toBe('')
    })

    it('should have default title', () => {
      const { dialogState } = useConfirmDialog()

      expect(dialogState.value.title).toBe('确认操作')
    })

    it('should have default confirm text', () => {
      const { dialogState } = useConfirmDialog()

      expect(dialogState.value.confirmText).toBe('确认')
    })

    it('should have default cancel text', () => {
      const { dialogState } = useConfirmDialog()

      expect(dialogState.value.cancelText).toBe('取消')
    })

    it('should have danger false initially', () => {
      const { dialogState } = useConfirmDialog()

      expect(dialogState.value.danger).toBe(false)
    })

    it('should have null resolve initially', () => {
      const { dialogState } = useConfirmDialog()

      expect(dialogState.value.resolve).toBeNull()
    })
  })

  describe('confirm', () => {
    it('should return a promise', () => {
      const { confirm } = useConfirmDialog()

      const result = confirm('Test message')

      expect(result).toBeInstanceOf(Promise)
    })

    it('should show dialog with message', async () => {
      const { confirm, dialogState } = useConfirmDialog()

      const confirmPromise = confirm('Are you sure?')

      expect(dialogState.value.visible).toBe(true)
      expect(dialogState.value.message).toBe('Are you sure?')

      dialogState.value.resolve?.(false)
      await confirmPromise
    })

    it('should use custom title', async () => {
      const { confirm, dialogState } = useConfirmDialog()

      const confirmPromise = confirm('Message', { title: 'Custom Title' })

      expect(dialogState.value.title).toBe('Custom Title')

      dialogState.value.resolve?.(false)
      await confirmPromise
    })

    it('should use custom confirm text', async () => {
      const { confirm, dialogState } = useConfirmDialog()

      const confirmPromise = confirm('Message', { confirmText: 'Delete' })

      expect(dialogState.value.confirmText).toBe('Delete')

      dialogState.value.resolve?.(false)
      await confirmPromise
    })

    it('should use custom cancel text', async () => {
      const { confirm, dialogState } = useConfirmDialog()

      const confirmPromise = confirm('Message', { cancelText: 'Go back' })

      expect(dialogState.value.cancelText).toBe('Go back')

      dialogState.value.resolve?.(false)
      await confirmPromise
    })

    it('should set danger flag', async () => {
      const { confirm, dialogState } = useConfirmDialog()

      const confirmPromise = confirm('Message', { danger: true })

      expect(dialogState.value.danger).toBe(true)

      dialogState.value.resolve?.(false)
      await confirmPromise
    })

    it('should use default options when not provided', async () => {
      const { confirm, dialogState } = useConfirmDialog()

      const confirmPromise = confirm('Message')

      expect(dialogState.value.title).toBe('确认操作')
      expect(dialogState.value.confirmText).toBe('确认')
      expect(dialogState.value.cancelText).toBe('取消')
      expect(dialogState.value.danger).toBe(false)

      dialogState.value.resolve?.(false)
      await confirmPromise
    })
  })

  describe('handleConfirm', () => {
    it('should resolve promise with true', async () => {
      const { confirm, handleConfirm } = useConfirmDialog()

      const confirmPromise = confirm('Test')
      handleConfirm()

      const result = await confirmPromise
      expect(result).toBe(true)
    })

    it('should hide dialog', async () => {
      const { confirm, handleConfirm, dialogState } = useConfirmDialog()

      const confirmPromise = confirm('Test')
      expect(dialogState.value.visible).toBe(true)

      handleConfirm()

      expect(dialogState.value.visible).toBe(false)

      await confirmPromise
    })
  })

  describe('handleCancel', () => {
    it('should resolve promise with false', async () => {
      const { confirm, handleCancel } = useConfirmDialog()

      const confirmPromise = confirm('Test')
      handleCancel()

      const result = await confirmPromise
      expect(result).toBe(false)
    })

    it('should hide dialog', async () => {
      const { confirm, handleCancel, dialogState } = useConfirmDialog()

      const confirmPromise = confirm('Test')
      expect(dialogState.value.visible).toBe(true)

      handleCancel()

      expect(dialogState.value.visible).toBe(false)

      await confirmPromise
    })
  })

  describe('confirm dialog flow', () => {
    it('should complete full confirm flow', async () => {
      const { confirm, handleConfirm } = useConfirmDialog()

      const confirmPromise = confirm('Delete this item?', {
        title: 'Confirm Delete',
        confirmText: 'Delete',
        cancelText: 'Keep',
        danger: true,
      })

      handleConfirm()

      const result = await confirmPromise
      expect(result).toBe(true)
    })

    it('should complete full cancel flow', async () => {
      const { confirm, handleCancel } = useConfirmDialog()

      const confirmPromise = confirm('Cancel this action?', {
        title: 'Confirm Cancel',
      })

      handleCancel()

      const result = await confirmPromise
      expect(result).toBe(false)
    })
  })

  describe('shared state', () => {
    it('should share state across multiple useConfirmDialog calls', async () => {
      const dialog1 = useConfirmDialog()
      const dialog2 = useConfirmDialog()

      const confirmPromise = dialog1.confirm('Shared message')

      expect(dialog2.dialogState.value.visible).toBe(true)
      expect(dialog2.dialogState.value.message).toBe('Shared message')

      dialog2.handleCancel()
      await confirmPromise
    })

    it('should allow handleConfirm from different instance', async () => {
      const dialog1 = useConfirmDialog()
      const dialog2 = useConfirmDialog()

      const confirmPromise = dialog1.confirm('Test')

      dialog2.handleConfirm()

      const result = await confirmPromise
      expect(result).toBe(true)
    })
  })

  describe('readonly state', () => {
    it('should not allow direct mutation of dialogState', () => {
      const { dialogState } = useConfirmDialog()

      const originalVisible = dialogState.value.visible
      ;(dialogState.value as { visible: boolean }).visible = true
      expect(dialogState.value.visible).toBe(originalVisible)
    })
  })
})
