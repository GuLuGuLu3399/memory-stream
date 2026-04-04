import { Decoration, DecorationSet, EditorView, ViewPlugin, ViewUpdate } from '@codemirror/view'
import { Extension, RangeSetBuilder } from '@codemirror/state'

const wikilinkDecoration = Decoration.mark({ class: 'cm-wikilink' })

const WIKILINK_REGEX = /\[\[([^\]]+)\]\]/g

function buildWikilinkDecorations(view: EditorView): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>()
  const doc = view.state.doc.toString()

  for (const { from, to } of view.visibleRanges) {
    const text = doc.slice(from, to)
    let match: RegExpExecArray | null

    WIKILINK_REGEX.lastIndex = 0

    while ((match = WIKILINK_REGEX.exec(text)) !== null) {
      const start = from + match.index
      const end = start + match[0].length
      builder.add(start, end, wikilinkDecoration)
    }
  }

  return builder.finish()
}

const wikilinkHighlightPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet

    constructor(view: EditorView) {
      this.decorations = buildWikilinkDecorations(view)
    }

    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = buildWikilinkDecorations(update.view)
      }
    }
  },
  {
    decorations: (v) => v.decorations,
  }
)

const wikilinkTheme = EditorView.baseTheme({
  '.cm-wikilink': {
    color: '#00e5ff',
    fontWeight: 'bold',
    backgroundColor: 'rgba(0, 229, 255, 0.08)',
    borderRadius: '2px',
    padding: '0 2px',
  },
  '.cm-wikilink:hover': {
    backgroundColor: 'rgba(0, 229, 255, 0.15)',
    cursor: 'pointer',
  },
})

export function wikilinkHighlight(): Extension {
  return [wikilinkHighlightPlugin, wikilinkTheme]
}
