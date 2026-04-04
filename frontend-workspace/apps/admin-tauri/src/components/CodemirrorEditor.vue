<script setup lang="ts">
import { ref, shallowRef, watch, onMounted, onUnmounted } from 'vue'
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter } from '@codemirror/view'
import { EditorState, Extension } from '@codemirror/state'
import { markdown } from '@codemirror/lang-markdown'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { wikilinkHighlight } from '../composables/wikilinkHighlight'

// Props
interface Props {
  modelValue: string
  placeholder?: string
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: '',
})

// Emits
const emit = defineEmits<{
  'update:modelValue': [value: string]
  save: []
}>()

// Refs
const editorContainer = ref<HTMLDivElement | null>(null)
const editorView = shallowRef<EditorView | null>(null)


// Theme for dark mode matching ms-deep/ms-carbon palette
const editorTheme = EditorView.theme({
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
  },
  '&.cm-focused .cm-cursor': {
    borderLeftColor: '#00e5ff',
  },
  '.cm-gutters': {
    backgroundColor: 'transparent',
    color: '#475569', // slate-600
    border: 'none',
    paddingRight: '8px',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'rgba(0, 229, 255, 0.05)',
    color: '#00e5ff',
  },
  '.cm-activeLine': {
    backgroundColor: 'rgba(0, 229, 255, 0.03)',
  },
  '.cm-line': {
    padding: '0 4px',
  },
  '.cm-placeholder': {
    color: '#475569', // slate-600
    fontStyle: 'italic',
    padding: '0 4px',
  },
  // Selection
  '.cm-selectionBackground': {
    backgroundColor: 'rgba(0, 229, 255, 0.15) !important',
  },
  '&.cm-focused .cm-selectionBackground': {
    backgroundColor: 'rgba(0, 229, 255, 0.25) !important',
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

// Placeholder extension
function createPlaceholder(text: string): Extension {
  return EditorView.contentAttributes.of({ 'data-placeholder': text })
}

// Save keymap handler
const saveKeymap = keymap.of([
  {
    key: 'Mod-s',
    run: () => {
      emit('save')
      return true
    },
  },
  ...defaultKeymap,
  ...historyKeymap,
])

// Create editor extensions
function createExtensions(): Extension[] {
  return [
    lineNumbers(),
    highlightActiveLine(),
    highlightActiveLineGutter(),
    history(),
    markdown(),
    wikilinkHighlight(),
    editorTheme,
    saveKeymap,
    createPlaceholder(props.placeholder),
    EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        const newContent = update.state.doc.toString()
        emit('update:modelValue', newContent)
      }
    }),
    EditorView.lineWrapping,
  ]
}

// Initialize editor
onMounted(() => {
  if (!editorContainer.value) return

  const state = EditorState.create({
    doc: props.modelValue,
    extensions: createExtensions(),
  })

  editorView.value = new EditorView({
    state,
    parent: editorContainer.value,
  })
})

// Cleanup
onUnmounted(() => {
  if (editorView.value) {
    editorView.value.destroy()
    editorView.value = null
  }
})

// Watch for external modelValue changes
watch(
  () => props.modelValue,
  (newValue) => {
    if (!editorView.value) return

    const currentValue = editorView.value.state.doc.toString()
    if (newValue !== currentValue) {
      editorView.value.dispatch({
        changes: {
          from: 0,
          to: currentValue.length,
          insert: newValue,
        },
      })
    }
  }
)

// Expose editor view for parent component (image paste)
defineExpose({
  editorView,
})
</script>

<template>
  <div ref="editorContainer" class="codemirror-editor h-full w-full"></div>
</template>

<style scoped>
.codemirror-editor {
  background: transparent;
}

.codemirror-editor :deep(.cm-editor) {
  height: 100%;
  outline: none;
}

.codemirror-editor :deep(.cm-scroller) {
  overflow: auto;
}
</style>
