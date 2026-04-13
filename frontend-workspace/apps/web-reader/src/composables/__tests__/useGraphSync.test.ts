// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from "vitest"
import { ref, nextTick, type Ref } from "vue"
import { useGraphSync, wsConnected, wsAuthenticated, wsLatency } from "../useGraphSync"

vi.mock("../../api", () => ({
  api: { getFullGraph: vi.fn() },
  getAuthToken: vi.fn(),
}))

import { api as _api } from "../../api"
import { getAuthToken as _getAuthToken } from "../../api"

describe("useGraphSync", () => {
  let capturedWs: any
  let mockNodes: Ref<any[], any[]>
  let mockEdges: Ref<any[], any[]>

  class MockWS {
    static CONNECTING = 0
    static OPEN = 1
    static CLOSING = 2
    static CLOSED = 3
    readyState = 0
    onopen: (() => void) | null = null
    onclose: (() => void) | null = null
    onmessage: ((e: { data: string }) => void) | null = null
    onerror: ((e: Error) => void) | null = null
    send = vi.fn()
    close = vi.fn()
    url: string
    constructor(url: string) {
      this.url = url
      capturedWs = this
    }
  }

  beforeEach(() => {
    vi.clearAllMocks()
    capturedWs = null
    mockNodes = ref<any[]>([]) as Ref<any[], any[]>
    mockEdges = ref<any[]>([]) as Ref<any[], any[]>

    wsConnected.value = false
    wsAuthenticated.value = false
    wsLatency.value = 0

    vi.mocked(_getAuthToken).mockReturnValue("test-token")
    vi.mocked(_api.getFullGraph).mockResolvedValue({ nodes: [], edges: [] })

    vi.stubGlobal("WebSocket", MockWS)
  })

  /** Connect and fire onopen so AUTH is sent */
  async function connectAndOpen() {
    const sync = useGraphSync(mockNodes, mockEdges)
    sync.connect()
    await nextTick()
    capturedWs.readyState = 1
    capturedWs.onopen(new Event("open"))
    await nextTick()
    return sync
  }

  it("should initialize with disconnected state", () => {
    const { connected, authenticated, latency } = useGraphSync(mockNodes, mockEdges)
    expect(connected.value).toBe(false)
    expect(authenticated.value).toBe(false)
    expect(latency.value).toBe(0)
  })

  it("should connect, send AUTH, and set global wsConnected", async () => {
    const { connected } = await connectAndOpen()

    expect(connected.value).toBe(true)
    expect(wsConnected.value).toBe(true)
    expect(capturedWs.send).toHaveBeenCalledWith(
      JSON.stringify({ action: "AUTH", payload: { token: "test-token" } }),
    )
  })

  it("should handle CARD_CREATED and CARD_DELETED events", async () => {
    const { connected: _connected, authenticated } = await connectAndOpen()

    capturedWs.onmessage({ data: JSON.stringify({ event: "AUTH_OK", payload: {} }) })
    await nextTick()
    expect(authenticated.value).toBe(true)

    capturedWs.onmessage({ data: JSON.stringify({
      event: "CARD_CREATED",
      payload: { card_id: "new-card", title: "New Card" },
    }) })
    await nextTick()
    expect(mockNodes.value).toHaveLength(1)
    expect(mockNodes.value?.[0]?.id).toBe("new-card")

    mockEdges.value = [
      { id: "e-new-card-other", source: "new-card", target: "other" } as any,
    ]

    capturedWs.onmessage({ data: JSON.stringify({
      event: "CARD_DELETED",
      payload: { card_id: "new-card" },
    }) })
    await nextTick()
    expect(mockNodes.value).toHaveLength(0)
    expect(mockEdges.value).toHaveLength(0)
  })

  it("should disconnect cleanly", async () => {
    const { connected, disconnect } = await connectAndOpen()

    disconnect()

    expect(connected.value).toBe(false)
    expect(wsConnected.value).toBe(false)
    expect(wsAuthenticated.value).toBe(false)
  })
})
