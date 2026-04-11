import { ref, type Ref } from 'vue'

/**
 * useFloatingPosition — Position floating elements relative to trigger
 *
 * Provides viewport-clamped positioning for tooltips, popovers, context menus.
 * Automatically flips when near viewport edges.
 *
 * @example
 * const { style, update } = useFloatingPosition(triggerRef)
 * function show() {
 *   update()
 *   visible.value = true
 * }
 */

export function useFloatingPosition(
  triggerRef: Ref<HTMLElement | null>,
  options?: {
    placement?: 'top' | 'bottom' | 'left' | 'right'
    offset?: number
    strategy?: 'absolute' | 'fixed'
  }
) {
  const placement = options?.placement ?? 'bottom'
  const offset = options?.offset ?? 8
  const strategy = options?.strategy ?? 'fixed'

  const style = ref<Record<string, string>>({})
  const isFlipped = ref(false)

  function update() {
    if (!triggerRef.value) return

    const trigger = triggerRef.value.getBoundingClientRect()
    const vw = window.innerWidth
    const vh = window.innerHeight

    const pos = calculatePosition(trigger, vw, vh)
    style.value = pos.style
    isFlipped.value = pos.flipped
  }

  function calculatePosition(trigger: DOMRect, vw: number, vh: number) {
    let flipped = false
    let top = 0
    let left = 0

    switch (placement) {
      case 'bottom':
        top = trigger.bottom + offset
        left = trigger.left + trigger.width / 2
        if (top + 200 > vh) {
          top = trigger.top - offset
          flipped = true
        }
        break
      case 'top':
        top = trigger.top - offset
        left = trigger.left + trigger.width / 2
        if (top < 0) {
          top = trigger.bottom + offset
          flipped = true
        }
        break
      case 'right':
        top = trigger.top + trigger.height / 2
        left = trigger.right + offset
        if (left + 250 > vw) {
          left = trigger.left - offset
          flipped = true
        }
        break
      case 'left':
        top = trigger.top + trigger.height / 2
        left = trigger.left - offset
        if (left < 0) {
          left = trigger.right + offset
          flipped = true
        }
        break
    }

    // Clamp to viewport
    top = Math.max(8, Math.min(top, vh - 8))
    left = Math.max(8, Math.min(left, vw - 8))

    const baseStyle: Record<string, string> = {
      position: strategy,
    }

    if (placement === 'top' || placement === 'bottom') {
      baseStyle.top = flipped ? 'auto' : `${top}px`
      baseStyle.bottom = flipped ? `${vh - top}px` : 'auto'
      baseStyle.left = `${left}px`
      baseStyle.transform = 'translateX(-50%)'
    } else {
      baseStyle.left = flipped ? 'auto' : `${left}px`
      baseStyle.right = flipped ? `${vw - left}px` : 'auto'
      baseStyle.top = `${top}px`
      baseStyle.transform = 'translateY(-50%)'
    }

    return { style: baseStyle, flipped }
  }

  return { style, isFlipped, update }
}
