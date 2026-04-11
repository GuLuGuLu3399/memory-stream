import { ref } from 'vue'

/**
 * useKeyboardListNavigation — Arrow-key navigation for list-like UI
 *
 * Eliminates duplicated keyboard navigation logic in SearchBar and CommandPalette.
 *
 * @example
 * const { selectedIndex, handleKeydown, reset } = useKeyboardListNavigation(
 *   computed(() => results.length),
 *   (index) => selectResult(results[index])
 * )
 */

export function useKeyboardListNavigation(
  itemCount: { value: number },
  onSelect: (index: number) => void,
  options?: {
    wrap?: boolean
    initialIndex?: number
  }
) {
  const wrap = options?.wrap ?? true
  const selectedIndex = ref(options?.initialIndex ?? -1)

  function handleKeydown(e: KeyboardEvent): boolean {
    if (itemCount.value === 0) return false

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault()
        if (selectedIndex.value < itemCount.value - 1) {
          selectedIndex.value++
        } else if (wrap) {
          selectedIndex.value = 0
        }
        return true

      case 'ArrowUp':
        e.preventDefault()
        if (selectedIndex.value > 0) {
          selectedIndex.value--
        } else if (wrap) {
          selectedIndex.value = itemCount.value - 1
        }
        return true

      case 'Enter':
        e.preventDefault()
        if (selectedIndex.value >= 0 && selectedIndex.value < itemCount.value) {
          onSelect(selectedIndex.value)
          return true
        }
        return false

      default:
        return false
    }
  }

  function reset() {
    selectedIndex.value = -1
  }

  function setIndex(index: number) {
    selectedIndex.value = index
  }

  return {
    selectedIndex,
    handleKeydown,
    reset,
    setIndex,
  }
}
