// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { createApp, defineComponent, ref } from 'vue'
import { useWSListener } from '../useWSListener'
import { useKnowledgeStore } from '../../stores/knowledge'

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}))

vi.mock('../../stores/knowledge', () => ({
  useKnowledgeStore: vi.fn(),
}))

import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const mockListen = vi.mocked(listen)
const mockUseKnowledgeStore = vi.mocked(useKnowledgeStore)

function withSetup(composable: () => void) {
  let result: any
  const app = createApp(defineComponent({
    setup() {
      result = composable()
      return () => null
    },
  }))
  const root = document.createElement('div')
  app.mount(root)
  return { app, result }
}

function flushPromises(): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, 0))
}

describe('useWSListener', () => {
  let mockStore: any

  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    mockStore = {
      updateLayouts: vi.fn(),
      silentRefresh: vi.fn(),
      loadRecent: vi.fn(),
      loadOrphans: vi.fn(),
      loadAndActivateCard: vi.fn(),
      activeCard: ref(null),
    }
    mockUseKnowledgeStore.mockImplementation(() => mockStore)
  })

  it('should register all 6 event listeners on mount', async () => {
    mockListen.mockImplementation(async () => vi.fn() as unknown as UnlistenFn)

    withSetup(() => useWSListener())
    await flushPromises()

    expect(mockListen).toHaveBeenCalledTimes(6)
    const events = mockListen.mock.calls.map(c => c[0])
    expect(events).toContain('layout_synced')
    expect(events).toContain('graph_changed')
    expect(events).toContain('ws_card_created')
    expect(events).toContain('ws_card_updated')
    expect(events).toContain('ws_card_deleted')
    expect(events).toContain('ws_cards_merged')
  })

  it('should fire store methods when events arrive', async () => {
    mockListen.mockImplementation(async () => vi.fn() as unknown as UnlistenFn)

    withSetup(() => useWSListener())
    await flushPromises()

    const createdCall = mockListen.mock.calls.find(c => c[0] === 'ws_card_created')
    expect(createdCall).toBeDefined()
    const handler = createdCall![1] as Function
    handler({ payload: {} })

    expect(mockStore.loadRecent).toHaveBeenCalled()
    expect(mockStore.loadOrphans).toHaveBeenCalled()
  })

  it('should call all unlisteners on unmount', async () => {
    const unlisteners = Array.from({ length: 6 }, () => vi.fn())
    let idx = 0
    mockListen.mockImplementation(async () => unlisteners[idx++])

    const { app } = withSetup(() => useWSListener())
    await flushPromises()

    unlisteners.forEach(fn => expect(fn).not.toHaveBeenCalled())
    app.unmount()
    unlisteners.forEach(fn => expect(fn).toHaveBeenCalled())
  })
})
