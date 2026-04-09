// @vitest-environment jsdom
import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { createApp, defineComponent } from 'vue'
import { useGlobalShortcuts } from '../useGlobalShortcuts'
import { useLayoutStore } from '../../stores/layout'

vi.mock('../../stores/layout', () => ({
  useLayoutStore: vi.fn(),
}))

const mockUseLayoutStore = vi.mocked(useLayoutStore)

function withSetup<T>(composable: () => T): { result: T; app: ReturnType<typeof createApp>; cleanup: () => void } {
  let result: T

  const App = defineComponent({
    setup() {
      result = composable()
      return () => null
    },
  })

  const app = createApp(App)
  const root = document.createElement('div')
  app.mount(root)

  return {
    result: result!,
    app,
    cleanup: () => {
      app.unmount()
    },
  }
}

describe('useGlobalShortcuts', () => {
  let mockStore: any
  let keydownHandler: ((e: KeyboardEvent) => void) | null = null

  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()

    mockStore = {
      toggleLeftDrawer: vi.fn(),
      toggleRightPanel: vi.fn(),
      toggleSidebarPin: vi.fn(),
      closeAll: vi.fn(),
      closeLeftDrawer: vi.fn(),
      closeRightPanel: vi.fn(),
      isLeftDrawerOpen: false,
      isRightPanelOpen: false,
      isLeftSidebarPinned: false,
      isCategoryPanelOpen: false,
      isSettingsOpen: false,
      isMergeConsoleOpen: false,
      isImportPanelOpen: false,
      openCategoryPanel: vi.fn(),
      closeCategoryPanel: vi.fn(),
      openSettings: vi.fn(),
      openMergeConsole: vi.fn(),
      closeMergeConsole: vi.fn(),
      openImportPanel: vi.fn(),
      closeImportPanel: vi.fn(),
    }
    mockUseLayoutStore.mockReturnValue(mockStore as any)

    vi.spyOn(window, 'addEventListener').mockImplementation((type, handler) => {
      if (type === 'keydown') {
        keydownHandler = handler as (e: KeyboardEvent) => void
      }
    })
  })

  afterEach(() => {
    keydownHandler = null
  })

  function fireKeydown(key: string, opts: { ctrlKey?: boolean; metaKey?: boolean; target?: HTMLElement } = {}) {
    if (!keydownHandler) throw new Error('keydownHandler not captured')
    const event = new KeyboardEvent('keydown', {
      key,
      ctrlKey: opts.ctrlKey ?? false,
      metaKey: opts.metaKey ?? false,
      bubbles: true,
      cancelable: true,
    })
    if (opts.target) {
      Object.defineProperty(event, 'target', { value: opts.target, writable: false })
    }
    keydownHandler(event)
  }

  describe('Ctrl+B shortcut', () => {
    it('should toggle left drawer on Ctrl+B', () => {
      withSetup(() => useGlobalShortcuts())
      fireKeydown('b', { ctrlKey: true })
      expect(mockStore.toggleLeftDrawer).toHaveBeenCalled()
    })

    it('should toggle left drawer on Meta+B (Mac)', () => {
      withSetup(() => useGlobalShortcuts())
      fireKeydown('b', { metaKey: true })
      expect(mockStore.toggleLeftDrawer).toHaveBeenCalled()
    })

    it('should NOT trigger when focus is on input field', () => {
      withSetup(() => useGlobalShortcuts())
      const input = document.createElement('input')
      fireKeydown('b', { ctrlKey: true, target: input })
      expect(mockStore.toggleLeftDrawer).not.toHaveBeenCalled()
    })

    it('should NOT trigger when focus is in textarea', () => {
      withSetup(() => useGlobalShortcuts())
      const textarea = document.createElement('textarea')
      fireKeydown('b', { ctrlKey: true, target: textarea })
      expect(mockStore.toggleLeftDrawer).not.toHaveBeenCalled()
    })

    it('should NOT trigger without modifier key', () => {
      withSetup(() => useGlobalShortcuts())
      fireKeydown('b')
      expect(mockStore.toggleLeftDrawer).not.toHaveBeenCalled()
    })
  })

  describe('Ctrl+G shortcut', () => {
    it('should toggle right panel on Ctrl+G', () => {
      withSetup(() => useGlobalShortcuts())
      fireKeydown('g', { ctrlKey: true })
      expect(mockStore.toggleRightPanel).toHaveBeenCalled()
    })

    it('should toggle right panel on Meta+G (Mac)', () => {
      withSetup(() => useGlobalShortcuts())
      fireKeydown('g', { metaKey: true })
      expect(mockStore.toggleRightPanel).toHaveBeenCalled()
    })

    it('should NOT trigger when focus is in input field', () => {
      withSetup(() => useGlobalShortcuts())
      const input = document.createElement('input')
      fireKeydown('g', { ctrlKey: true, target: input })
      expect(mockStore.toggleRightPanel).not.toHaveBeenCalled()
    })
  })

  describe('Ctrl+\\ shortcut', () => {
    it('should toggle right panel on Ctrl+\\', () => {
      withSetup(() => useGlobalShortcuts())
      fireKeydown('\\', { ctrlKey: true })
      expect(mockStore.toggleRightPanel).toHaveBeenCalled()
    })

    it('should toggle right panel on Meta+\\ (Mac)', () => {
      withSetup(() => useGlobalShortcuts())
      fireKeydown('\\', { metaKey: true })
      expect(mockStore.toggleRightPanel).toHaveBeenCalled()
    })
  })

  describe('preventDefault behavior', () => {
    it('should call preventDefault on Ctrl+B', () => {
      withSetup(() => useGlobalShortcuts())
      const event = new KeyboardEvent('keydown', { key: 'b', ctrlKey: true, cancelable: true })
      const preventDefaultSpy = vi.spyOn(event, 'preventDefault')
      if (keydownHandler) keydownHandler(event)
      expect(preventDefaultSpy).toHaveBeenCalled()
    })

    it('should call preventDefault on Ctrl+G', () => {
      withSetup(() => useGlobalShortcuts())
      const event = new KeyboardEvent('keydown', { key: 'g', ctrlKey: true, cancelable: true })
      const preventDefaultSpy = vi.spyOn(event, 'preventDefault')
      if (keydownHandler) keydownHandler(event)
      expect(preventDefaultSpy).toHaveBeenCalled()
    })

    it('should call preventDefault on Ctrl+\\', () => {
      withSetup(() => useGlobalShortcuts())
      const event = new KeyboardEvent('keydown', { key: '\\', ctrlKey: true, cancelable: true })
      const preventDefaultSpy = vi.spyOn(event, 'preventDefault')
      if (keydownHandler) keydownHandler(event)
      expect(preventDefaultSpy).toHaveBeenCalled()
    })
  })

  describe('unrelated keys', () => {
    it('should NOT trigger on unrelated key combinations', () => {
      withSetup(() => useGlobalShortcuts())
      fireKeydown('x', { ctrlKey: true })
      expect(mockStore.toggleLeftDrawer).not.toHaveBeenCalled()
      expect(mockStore.toggleRightPanel).not.toHaveBeenCalled()
    })
  })

  describe('cleanup', () => {
    it('should remove event listener on unmount', () => {
      const removeEventListenerSpy = vi.spyOn(window, 'removeEventListener')
      const { cleanup } = withSetup(() => useGlobalShortcuts())
      cleanup()
      expect(removeEventListenerSpy).toHaveBeenCalledWith('keydown', expect.any(Function))
    })
  })
})
