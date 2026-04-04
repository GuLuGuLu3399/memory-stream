// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref } from 'vue'
import { useActiveHeading } from '../useActiveHeading'

describe('useActiveHeading', () => {
  let capturedCallback: IntersectionObserverCallback | null = null
  let mockObserve: ReturnType<typeof vi.fn>
  let mockDisconnect: ReturnType<typeof vi.fn>

  beforeEach(() => {
    vi.clearAllMocks()
    capturedCallback = null
    mockObserve = vi.fn()
    mockDisconnect = vi.fn()

    vi.stubGlobal('IntersectionObserver', vi.fn(function (this: any, callback: IntersectionObserverCallback) {
      capturedCallback = callback
      this.observe = mockObserve
      this.disconnect = mockDisconnect
    }) as any)
  })

  it('should initialize with empty activeSlug', () => {
    const { activeSlug } = useActiveHeading(ref(undefined))
    expect(activeSlug.value).toBe('')
  })

  it('should observe headings and track active slug', () => {
    const container = document.createElement('div')
    container.innerHTML = '<h1 id="s1">S1</h1><h2 id="s2">S2</h2>'
    const containerRef = ref(container)

    const { activeSlug, refreshObserver } = useActiveHeading(containerRef)
    refreshObserver()

    expect(capturedCallback).not.toBeNull()
    expect(mockObserve).toHaveBeenCalledTimes(2)

    const heading = container.querySelector('#s1')!
    capturedCallback!(
      [{
        target: heading,
        isIntersecting: true,
        boundingClientRect: { top: 100 } as DOMRectReadOnly,
        intersectionRatio: 1,
        intersectionRect: {} as DOMRectReadOnly,
        rootBounds: null,
        time: Date.now(),
      }],
      {} as IntersectionObserver,
    )

    expect(activeSlug.value).toBe('s1')
  })

  it('should select topmost visible heading', () => {
    const container = document.createElement('div')
    container.innerHTML = '<h1 id="s1">S1</h1><h1 id="s2">S2</h1>'
    const containerRef = ref(container)

    const { activeSlug, refreshObserver } = useActiveHeading(containerRef)
    refreshObserver()

    const h1 = container.querySelector('#s1')!
    const h2 = container.querySelector('#s2')!
    capturedCallback!(
      [
        { target: h1, isIntersecting: true, boundingClientRect: { top: 200 } as DOMRectReadOnly, intersectionRatio: 1, intersectionRect: {} as DOMRectReadOnly, rootBounds: null, time: Date.now() },
        { target: h2, isIntersecting: true, boundingClientRect: { top: 50 } as DOMRectReadOnly, intersectionRatio: 1, intersectionRect: {} as DOMRectReadOnly, rootBounds: null, time: Date.now() },
      ],
      {} as IntersectionObserver,
    )

    expect(activeSlug.value).toBe('s2')
  })

  it('should handle undefined container gracefully', () => {
    const { activeSlug, refreshObserver } = useActiveHeading(ref(undefined))
    expect(() => refreshObserver()).not.toThrow()
    expect(activeSlug.value).toBe('')
  })
})
