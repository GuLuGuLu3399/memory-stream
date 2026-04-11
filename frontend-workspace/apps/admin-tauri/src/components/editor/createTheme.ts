import { EditorView } from '@codemirror/view'

/**
 * Mechanical Altar (机械祭坛) theme for CodeMirror 6
 *
 * Design tokens:
 * - ms-void: #050505, ms-deep: #0d0d0d, ms-carbon: #141414
 * - neon: #00e5ff, brass: #b8860b, brass-light: #d4a853
 * - ms-border: #1e1e1e
 */
export function createTheme() {
  return EditorView.theme({
    '&': {
      height: '100%',
      fontSize: '14px',
      backgroundColor: 'transparent',
    },
    '.cm-scroller': {
      fontFamily: '"JetBrains Mono", "Fira Code", Consolas, Monaco, monospace',
      lineHeight: '1.625',
      overflow: 'auto',
    },
    '.cm-content': {
      padding: '24px',
      caretColor: '#00e5ff',
      color: '#94a3b8', // slate-400
    },
    '.cm-cursor': {
      borderLeftColor: '#00e5ff',
      boxShadow: '0 0 4px rgba(0, 229, 255, 0.4)',
    },
    '&.cm-focused .cm-cursor': {
      borderLeftColor: '#00e5ff',
    },
    '.cm-gutters': {
      backgroundColor: '#050505', // ms-void
      color: '#b8860b80', // brass-dim
      border: 'none',
      paddingRight: '8px',
      paddingLeft: '4px',
    },
    '.cm-lineNumbers': {
      color: '#b8860b80', // brass-dim
    },
    '.cm-activeLineGutter': {
      color: '#d4a853', // brass-light
      textShadow: '0 0 8px rgba(212, 168, 83, 0.6)',
      backgroundColor: 'transparent',
    },
    '.cm-activeLine': {
      backgroundColor: 'transparent',
      borderBottom: '1px solid #b8860b', // brass
    },
    '.cm-line': {
      padding: '0 4px',
    },
    '.cm-placeholder': {
      color: '#475569', // slate-600
      fontStyle: 'italic',
      padding: '0 4px',
    },
    // Selection: brass tint instead of blue
    '.cm-selectionBackground': {
      backgroundColor: 'rgba(184, 134, 11, 0.15) !important', // brass tint
    },
    '&.cm-focused .cm-selectionBackground': {
      backgroundColor: 'rgba(184, 134, 11, 0.25) !important', // stronger brass tint
    },
    // Search highlight
    '.cm-searchMatch': {
      backgroundColor: 'rgba(0, 229, 255, 0.2)',
      outline: '1px solid rgba(0, 229, 255, 0.4)',
    },
    '.cm-searchMatch-selected': {
      backgroundColor: 'rgba(0, 229, 255, 0.35)',
    },
  })
}
