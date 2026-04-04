// @vitest-environment jsdom
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { withSetup } from './test-utils'
import { useTransitions } from '../useTransitions'

describe('useTransitions', () => {
  beforeEach(() => {
    vi.stubGlobal('window', {
      matchMedia: vi.fn(),
    })
  })

  describe('prefers-reduced-motion: reduce', () => {
    it('should return 0ms durations when user prefers reduced motion', () => {
      vi.mocked(window.matchMedia).mockReturnValue({
        matches: true,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      } as unknown as MediaQueryList)

      const { fade, slideRight, slideUp, scale } = withSetup(() => useTransitions())

      expect(fade.value).toBe('opacity 0ms ease')
      expect(slideRight.value).toBe('transform 0ms cubic-bezier(0.16, 1, 0.3, 1)')
      expect(slideUp.value).toBe('transform 0ms ease-out')
      expect(scale.value).toBe('transform 0ms ease, opacity 0ms ease')
    })

    it('should set transitionName to "none" when user prefers reduced motion', () => {
      vi.mocked(window.matchMedia).mockReturnValue({
        matches: true,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      } as unknown as MediaQueryList)

      const { transitionName } = withSetup(() => useTransitions())

      expect(transitionName.value).toBe('none')
    })
  })

  describe('standard motion', () => {
    it('should return standard durations when user does not prefer reduced motion', () => {
      vi.mocked(window.matchMedia).mockReturnValue({
        matches: false,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      } as unknown as MediaQueryList)

      const { fade, slideRight, slideUp, scale } = withSetup(() => useTransitions())

      expect(fade.value).toBe('opacity 250ms ease')
      expect(slideRight.value).toBe('transform 300ms cubic-bezier(0.16, 1, 0.3, 1)')
      expect(slideUp.value).toBe('transform 250ms ease-out')
      expect(scale.value).toBe('transform 250ms ease, opacity 250ms ease')
    })

    it('should set transitionName to "default" when user does not prefer reduced motion', () => {
      vi.mocked(window.matchMedia).mockReturnValue({
        matches: false,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      } as unknown as MediaQueryList)

      const { transitionName } = withSetup(() => useTransitions())

      expect(transitionName.value).toBe('default')
    })
  })

  describe('design token compliance', () => {
    it('should expose maxDuration as 250ms', () => {
      const { maxDuration } = useTransitions()

      expect(maxDuration).toBe(250)
    })

    it('should expose drawerDurationMs as 300ms', () => {
      const { drawerDurationMs } = useTransitions()

      expect(drawerDurationMs).toBe(300)
    })

    it('should comply with industrial console aesthetic timing rules', () => {
      vi.mocked(window.matchMedia).mockReturnValue({
        matches: false,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      } as unknown as MediaQueryList)

      const { maxDuration, drawerDurationMs } = withSetup(() => useTransitions())

      expect(maxDuration).toBeLessThanOrEqual(250)
      expect(drawerDurationMs).toBeLessThanOrEqual(300)
    })
  })

  describe('reactive media query', () => {
    it('should register event listener for media query changes', () => {
      const addEventListener = vi.fn()
      vi.mocked(window.matchMedia).mockReturnValue({
        matches: false,
        addEventListener,
        removeEventListener: vi.fn(),
      } as unknown as MediaQueryList)

      withSetup(() => useTransitions())

      expect(addEventListener).toHaveBeenCalledWith('change', expect.any(Function))
    })

    it('should fallback to addListener if addEventListener is not available', () => {
      const addListener = vi.fn()
      vi.mocked(window.matchMedia).mockReturnValue({
        matches: false,
        addListener,
        removeListener: vi.fn(),
      } as unknown as MediaQueryList)

      withSetup(() => useTransitions())

      expect(addListener).toHaveBeenCalledWith(expect.any(Function))
    })
  })

  describe('SSR compatibility', () => {
    it('should not crash when window is undefined', () => {
      vi.stubGlobal('window', undefined)

      expect(() => useTransitions()).not.toThrow()
    })
  })
})
