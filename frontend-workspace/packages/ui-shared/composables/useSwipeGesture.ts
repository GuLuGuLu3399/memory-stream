import { ref, onMounted, onUnmounted } from 'vue'

/**
 * useSwipeGesture — Enhanced swipe detection
 *
 * Replaces basic useSwipeClose with directional detection,
 * velocity threshold, and elastic overscroll support.
 *
 * @example
 * const { offsetX, isSwiping, direction } = useSwipeGesture(elementRef, {
 *   threshold: 80,
 *   onSwipe: (dir) => { if (dir === 'left') close() }
 * })
 */

export function useSwipeGesture(
  elementRef: { value: HTMLElement | null },
  options?: {
    direction?: 'left' | 'right' | 'up' | 'down'
    threshold?: number
    edgeWidth?: number
    onSwipe?: (direction: string) => void
    elastic?: boolean
  }
) {
  const direction = options?.direction ?? 'left'
  const threshold = options?.threshold ?? 80
  const edgeWidth = options?.edgeWidth ?? 0
  const elastic = options?.elastic ?? true
  const onSwipe = options?.onSwipe

  const offsetX = ref(0)
  const offsetY = ref(0)
  const isSwiping = ref(false)
  const detectedDirection = ref<string | null>(null)

  let startX = 0
  let startY = 0
  let startTime = 0

  function onTouchStart(e: TouchEvent) {
    if (!elementRef.value) return

    const touch = e.touches[0]

    // Edge detection: only start from specified edge
    if (edgeWidth > 0) {
      if (direction === 'right' && touch.clientX > edgeWidth) return
      if (direction === 'left' && touch.clientX < window.innerWidth - edgeWidth) return
    }

    startX = touch.clientX
    startY = touch.clientY
    startTime = Date.now()
    isSwiping.value = false
    detectedDirection.value = null
  }

  function onTouchMove(e: TouchEvent) {
    if (!elementRef.value) return

    const touch = e.touches[0]
    const dx = touch.clientX - startX
    const dy = touch.clientY - startY

    // Determine direction on first significant move
    if (!detectedDirection.value && (Math.abs(dx) > 5 || Math.abs(dy) > 5)) {
      detectedDirection.value = Math.abs(dx) > Math.abs(dy)
        ? (dx > 0 ? 'right' : 'left')
        : (dy > 0 ? 'down' : 'up')

      // Only activate if matching target direction
      if (detectedDirection.value !== direction) {
        return
      }
      isSwiping.value = true
    }

    if (!isSwiping.value) return

    // Apply elastic offset
    if (elastic) {
      const dampened = dampen(dx)
      offsetX.value = direction === 'left' || direction === 'right' ? dampened : 0
      offsetY.value = direction === 'up' || direction === 'down' ? dampen(dy) : 0
    }
  }

  function onTouchEnd() {
    if (!isSwiping.value) {
      offsetX.value = 0
      offsetY.value = 0
      return
    }

    const elapsed = Date.now() - startTime
    const velocity = Math.abs(offsetX.value) / elapsed

    // Trigger if past threshold OR fast swipe
    const distance = Math.abs(offsetX.value)
    if (distance > threshold || velocity > 0.5) {
      onSwipe?.(detectedDirection.value ?? direction)
    }

    // Reset with transition
    offsetX.value = 0
    offsetY.value = 0
    isSwiping.value = false
    detectedDirection.value = null
  }

  function dampen(delta: number): number {
    // Elastic dampening: reduces movement as it extends further
    const abs = Math.abs(delta)
    const sign = delta > 0 ? 1 : -1
    return sign * (abs * 0.6 / (1 + abs * 0.003))
  }

  onMounted(() => {
    if (!elementRef.value) return
    elementRef.value.addEventListener('touchstart', onTouchStart, { passive: true })
    elementRef.value.addEventListener('touchmove', onTouchMove, { passive: true })
    elementRef.value.addEventListener('touchend', onTouchEnd, { passive: true })
  })

  onUnmounted(() => {
    if (!elementRef.value) return
    elementRef.value.removeEventListener('touchstart', onTouchStart)
    elementRef.value.removeEventListener('touchmove', onTouchMove)
    elementRef.value.removeEventListener('touchend', onTouchEnd)
  })

  return {
    offsetX,
    offsetY,
    isSwiping,
    direction: detectedDirection,
  }
}
