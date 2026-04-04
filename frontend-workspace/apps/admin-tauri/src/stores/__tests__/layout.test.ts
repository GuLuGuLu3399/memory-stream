// @vitest-environment jsdom
import { describe, it, expect, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useLayoutStore } from '../layout'

describe('useLayoutStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  describe('initial state', () => {
    it('should have all drawers/panels closed by default', () => {
      const store = useLayoutStore()

      expect(store.isLeftDrawerOpen).toBe(false)
      expect(store.isRightPanelOpen).toBe(false)
      expect(store.isLeftSidebarPinned).toBe(false)
      expect(store.isCategoryPanelOpen).toBe(false)
      expect(store.isSettingsOpen).toBe(false)
      expect(store.isMergeConsoleOpen).toBe(false)
      expect(store.isImportPanelOpen).toBe(false)
    })
  })

  describe('toggleLeftDrawer', () => {
    it('should toggle left drawer open', () => {
      const store = useLayoutStore()

      store.toggleLeftDrawer()
      expect(store.isLeftDrawerOpen).toBe(true)
    })

    it('should toggle left drawer closed', () => {
      const store = useLayoutStore()

      store.toggleLeftDrawer()
      store.toggleLeftDrawer()
      expect(store.isLeftDrawerOpen).toBe(false)
    })

    it('should close right panel when left drawer opens (mutual exclusion)', () => {
      const store = useLayoutStore()

      store.toggleRightPanel()
      expect(store.isRightPanelOpen).toBe(true)

      store.toggleLeftDrawer()
      expect(store.isLeftDrawerOpen).toBe(true)
      expect(store.isRightPanelOpen).toBe(false)
    })
  })

  describe('toggleSidebarPin', () => {
    it('should toggle sidebar pin state', () => {
      const store = useLayoutStore()

      store.toggleSidebarPin()
      expect(store.isLeftSidebarPinned).toBe(true)

      store.toggleSidebarPin()
      expect(store.isLeftSidebarPinned).toBe(false)
    })

    it('should open left drawer when pinning', () => {
      const store = useLayoutStore()

      store.toggleSidebarPin()
      expect(store.isLeftSidebarPinned).toBe(true)
      expect(store.isLeftDrawerOpen).toBe(true)
    })
  })

  describe('toggleRightPanel', () => {
    it('should toggle right panel open', () => {
      const store = useLayoutStore()

      store.toggleRightPanel()
      expect(store.isRightPanelOpen).toBe(true)
    })

    it('should toggle right panel closed', () => {
      const store = useLayoutStore()

      store.toggleRightPanel()
      store.toggleRightPanel()
      expect(store.isRightPanelOpen).toBe(false)
    })

    it('should close left drawer when right panel opens (mutual exclusion)', () => {
      const store = useLayoutStore()

      store.toggleLeftDrawer()
      expect(store.isLeftDrawerOpen).toBe(true)

      store.toggleRightPanel()
      expect(store.isRightPanelOpen).toBe(true)
      expect(store.isLeftDrawerOpen).toBe(false)
    })
  })

  describe('closeLeftDrawer', () => {
    it('should close left drawer when not pinned', () => {
      const store = useLayoutStore()

      store.toggleLeftDrawer()
      expect(store.isLeftDrawerOpen).toBe(true)

      store.closeLeftDrawer()
      expect(store.isLeftDrawerOpen).toBe(false)
    })

    it('should NOT close left drawer when pinned', () => {
      const store = useLayoutStore()

      store.toggleSidebarPin()
      expect(store.isLeftSidebarPinned).toBe(true)
      expect(store.isLeftDrawerOpen).toBe(true)

      store.closeLeftDrawer()
      expect(store.isLeftDrawerOpen).toBe(true) // Still open due to pin
    })
  })

  describe('closeRightPanel', () => {
    it('should close right panel', () => {
      const store = useLayoutStore()

      store.toggleRightPanel()
      expect(store.isRightPanelOpen).toBe(true)

      store.closeRightPanel()
      expect(store.isRightPanelOpen).toBe(false)
    })
  })

  describe('closeAll', () => {
    it('should close all drawers and panels', () => {
      const store = useLayoutStore()

      store.toggleLeftDrawer()
      store.toggleRightPanel()
      expect(store.isRightPanelOpen).toBe(true)

      store.closeAll()
      expect(store.isLeftDrawerOpen).toBe(false)
      expect(store.isRightPanelOpen).toBe(false)
    })
  })

  describe('category panel', () => {
    it('should open category panel', () => {
      const store = useLayoutStore()

      store.openCategoryPanel()
      expect(store.isCategoryPanelOpen).toBe(true)
    })

    it('should close category panel', () => {
      const store = useLayoutStore()

      store.openCategoryPanel()
      store.closeCategoryPanel()
      expect(store.isCategoryPanelOpen).toBe(false)
    })
  })

  describe('settings', () => {
    it('should open settings', () => {
      const store = useLayoutStore()

      store.openSettings()
      expect(store.isSettingsOpen).toBe(true)
    })
  })

  describe('merge console', () => {
    it('should open merge console', () => {
      const store = useLayoutStore()

      store.openMergeConsole()
      expect(store.isMergeConsoleOpen).toBe(true)
    })

    it('should close merge console', () => {
      const store = useLayoutStore()

      store.openMergeConsole()
      store.closeMergeConsole()
      expect(store.isMergeConsoleOpen).toBe(false)
    })
  })

  describe('import panel', () => {
    it('should open import panel', () => {
      const store = useLayoutStore()

      store.openImportPanel()
      expect(store.isImportPanelOpen).toBe(true)
    })

    it('should close import panel', () => {
      const store = useLayoutStore()

      store.openImportPanel()
      store.closeImportPanel()
      expect(store.isImportPanelOpen).toBe(false)
    })
  })

  describe('complex interactions', () => {
    it('should handle pin then unpin correctly', () => {
      const store = useLayoutStore()

      // Pin sidebar
      store.toggleSidebarPin()
      expect(store.isLeftSidebarPinned).toBe(true)
      expect(store.isLeftDrawerOpen).toBe(true)

      // Try to close (should not work due to pin)
      store.closeLeftDrawer()
      expect(store.isLeftDrawerOpen).toBe(true)

      // Unpin
      store.toggleSidebarPin()
      expect(store.isLeftSidebarPinned).toBe(false)

      // Now close should work
      store.closeLeftDrawer()
      expect(store.isLeftDrawerOpen).toBe(false)
    })

    it('should allow independent panel operations', () => {
      const store = useLayoutStore()

      store.openCategoryPanel()
      store.openSettings()
      store.openMergeConsole()

      expect(store.isCategoryPanelOpen).toBe(true)
      expect(store.isSettingsOpen).toBe(true)
      expect(store.isMergeConsoleOpen).toBe(true)
    })
  })
})
