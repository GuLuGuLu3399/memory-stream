import { ref, computed, onMounted, onUnmounted } from 'vue'

/**
 * 🎬 useTransitions — Animation System with prefers-reduced-motion Support
 * 
 * Provides reactive transition strings and CSS class names for Vue Transition components.
 * Respects user's reduced motion preferences by setting all durations to 0ms when enabled.
 * 
 * @example
 * ```vue
 * <script setup>
 * import { useTransitions } from '@memory-stream/ui-shared'
 * 
 * const { fade, slideRight, transitionName } = useTransitions()
 * </script>
 * 
 * <template>
 *   <Transition :name="transitionName">
 *     <div v-if="show" class="ms-fade">Content</div>
 *   </Transition>
 * </template>
 * ```
 * 
 * Design Tokens:
 * - Max duration: 250ms (industrial console aesthetic)
 * - Drawer duration: 300ms Expo-Out
 * - All durations → 0ms when prefers-reduced-motion
 */
export function useTransitions() {
  const prefersReducedMotion = ref(false)
  let mediaQuery: MediaQueryList | null = null

  const isBrowserEnvironment = typeof window !== 'undefined' && window.matchMedia

  onMounted(() => {
    if (!isBrowserEnvironment) return

    mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
    prefersReducedMotion.value = mediaQuery.matches

    const handleMediaQueryChange = (e: MediaQueryListEvent) => {
      prefersReducedMotion.value = e.matches
    }

    if (mediaQuery.addEventListener) {
      mediaQuery.addEventListener('change', handleMediaQueryChange)
    } else {
      mediaQuery.addListener(handleMediaQueryChange)
    }

    onUnmounted(() => {
      if (!mediaQuery) return
      
      if (mediaQuery.removeEventListener) {
        mediaQuery.removeEventListener('change', handleMediaQueryChange)
      } else {
        mediaQuery.removeListener(handleMediaQueryChange)
      }
    })
  })

  const standardDuration = computed(() => prefersReducedMotion.value ? '0ms' : '250ms')
  const drawerDuration = computed(() => prefersReducedMotion.value ? '0ms' : '300ms')

  const fade = computed(() => `opacity ${standardDuration.value} ease`)
  const slideRight = computed(() => 
    `transform ${drawerDuration.value} cubic-bezier(0.16, 1, 0.3, 1)`
  )
  const slideUp = computed(() => 
    `transform ${standardDuration.value} ease-out`
  )
  const scale = computed(() =>
    `transform ${standardDuration.value} ease, opacity ${standardDuration.value} ease`
  )

  // --- New transitions ---
  const slideLeft = computed(() =>
    `transform ${drawerDuration.value} cubic-bezier(0.16, 1, 0.3, 1)`
  )
  const slideDown = computed(() =>
    `transform ${standardDuration.value} ease-out`
  )
  const morph = computed(() =>
    `transform ${standardDuration.value} cubic-bezier(0.16, 1, 0.3, 1), opacity ${standardDuration.value} ease`
  )
  const spring = computed(() =>
    `transform ${prefersReducedMotion.value ? '0ms' : '300ms'} cubic-bezier(0.68, -0.3, 0.32, 1.3)`
  )

  const transitionName = computed(() =>
    prefersReducedMotion.value ? 'none' : 'default'
  )

  return {
    fade,
    slideRight,
    slideUp,
    scale,
    slideLeft,
    slideDown,
    morph,
    spring,
    transitionName,
    maxDuration: 250,
    drawerDurationMs: 300,
    prefersReducedMotion,
  }
}
