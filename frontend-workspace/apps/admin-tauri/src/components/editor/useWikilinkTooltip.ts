import { Ref, shallowRef, onUnmounted } from 'vue'
import { EditorView, ViewPlugin, ViewUpdate } from '@codemirror/view'
import { Extension } from '@codemirror/state'

const WIKILINK_REGEX = /\[\[([^\]]+)\]\]/g

interface TooltipState {
  targetCard: string | null
  show: boolean
  x: number
  y: number
  isLoading: boolean
  cardPreview: CardPreview | null
}

interface CardPreview {
  title: string
  excerpt: string
}

/**
 * Composable for showing inline card preview tooltips on Ctrl+hover over [[wikilinks]]
 *
 * Features:
 * - Shows tooltip only when Ctrl key is held
 * - Fetches card preview from backend via Tauri IPC
 * - Displays card title and excerpt
 * - Closes on Ctrl release or mouse move away
 */
export function useWikilinkTooltip(
  editorView: Ref<EditorView | null>
) {
  const tooltipState = shallowRef<TooltipState>({
    targetCard: null,
    show: false,
    x: 0,
    y: 0,
    isLoading: false,
    cardPreview: null,
  })

  let ctrlPressed = false
  let hoveredWikilink: { text: string; pos: number } | null = null

  // Track Ctrl key state
  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Control') {
      ctrlPressed = true
      if (hoveredWikilink && !tooltipState.value.show) {
        showTooltip(hoveredWikilink)
      }
    }
  }

  function handleKeyUp(e: KeyboardEvent) {
    if (e.key === 'Control') {
      ctrlPressed = false
      hideTooltip()
    }
  }

  // Show tooltip at position
  async function showTooltip(wikilink: { text: string; pos: number }) {
    if (!editorView.value) return

    const view = editorView.value
    const coords = view.coordsAtPos(wikilink.pos)

    if (!coords) return

    tooltipState.value = {
      targetCard: wikilink.text,
      show: true,
      x: coords.left,
      y: coords.bottom + 8,
      isLoading: true,
      cardPreview: null,
    }

    // Fetch card preview from backend
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const result = await invoke<{ title: string; raw_md: string } | null>(
        'get_card_by_title',
        { title: wikilink.text }
      )

      if (result) {
        // Extract excerpt (first 150 chars)
        const excerpt = result.raw_md.slice(0, 150) + (result.raw_md.length > 150 ? '...' : '')

        tooltipState.value = {
          ...tooltipState.value,
          isLoading: false,
          cardPreview: {
            title: result.title,
            excerpt,
          },
        }
      } else {
        tooltipState.value = {
          ...tooltipState.value,
          isLoading: false,
          cardPreview: null,
        }
      }
    } catch (error) {
      console.error('[useWikilinkTooltip] Failed to fetch card:', error)
      tooltipState.value = {
        ...tooltipState.value,
        isLoading: false,
        cardPreview: null,
      }
    }
  }

  function hideTooltip() {
    tooltipState.value = {
      targetCard: null,
      show: false,
      x: 0,
      y: 0,
      isLoading: false,
      cardPreview: null,
    }
  }

  // Create tooltip ViewPlugin
  function createWikilinkTooltipPlugin(): Extension {
    return ViewPlugin.fromClass(
      class {
        private mouseMoveHandler = (e: MouseEvent) => this.onMouseMove(e)

        constructor(private view: EditorView) {
          view.dom.addEventListener('mousemove', this.mouseMoveHandler)
        }

        destroy() {
          this.view.dom.removeEventListener('mousemove', this.mouseMoveHandler)
        }

        onMouseMove(e: MouseEvent) {
          const pos = this.view.posAtCoords({ x: e.clientX, y: e.clientY })
          if (pos === null) {
            hoveredWikilink = null
            return
          }

          const line = this.view.state.doc.lineAt(pos)
          const lineText = line.text
          const lineStart = line.from

          // Check if cursor is over a wikilink
          WIKILINK_REGEX.lastIndex = 0
          let match: RegExpExecArray | null
          let found = false

          while ((match = WIKILINK_REGEX.exec(lineText)) !== null) {
            const start = lineStart + match.index
            const end = start + match[0].length

            if (pos >= start && pos <= end) {
              const linkText = match[1]
              hoveredWikilink = { text: linkText, pos: start }

              // Show tooltip if Ctrl is pressed
              if (ctrlPressed) {
                const currentTarget = tooltipState.value.targetCard
                if (currentTarget !== linkText) {
                  showTooltip(hoveredWikilink)
                }
              }
              found = true
              break
            }
          }

          if (!found) {
            hoveredWikilink = null
            if (tooltipState.value.show && !ctrlPressed) {
              hideTooltip()
            }
          }
        }

        update(_update: ViewUpdate) {
          // Hide tooltip on document changes
          if (_update.docChanged && tooltipState.value.show) {
            hideTooltip()
          }
        }
      }
    )
  }

  // Setup global keyboard listeners
  function setupKeyboardListeners() {
    window.addEventListener('keydown', handleKeyDown)
    window.addEventListener('keyup', handleKeyUp)
  }

  function cleanupKeyboardListeners() {
    window.removeEventListener('keydown', handleKeyDown)
    window.removeEventListener('keyup', handleKeyUp)
  }

  onUnmounted(() => {
    cleanupKeyboardListeners()
  })

  return {
    tooltipState,
    createWikilinkTooltipPlugin,
    setupKeyboardListeners,
  }
}
