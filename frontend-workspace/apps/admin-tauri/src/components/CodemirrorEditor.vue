<script setup lang="ts">
import { ref, shallowRef, watch, onMounted, onUnmounted } from 'vue'
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter } from '@codemirror/view'
import { EditorState, Extension } from '@codemirror/state'
import { markdown } from '@codemirror/lang-markdown'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { wikilinkHighlight } from '../composables/wikilinkHighlight'
import { createTheme } from './editor/createTheme'
import { useGutterDecorations } from './editor/useGutterDecorations'
import { formatMarkdown } from '../composables/useMdFormatter'

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

// Initialize gutter decorations composable
const { createGutterDecorationsPlugin } = useGutterDecorations()

// Create Mechanical Altar theme
const editorTheme = createTheme()

// Placeholder extension
function createPlaceholder(text: string): Extension {
  return EditorView.contentAttributes.of({ 'data-placeholder': text })
}

// Format handler — async, returns true to consume the key
async function handleFormat(): Promise<boolean> {
  const view = editorView.value
  if (!view) return false

  const raw = view.state.doc.toString()
  const formatted = await formatMarkdown(raw)

  if (formatted !== raw) {
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: formatted },
    })
  }
  return true
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
  {
    key: 'Mod-Shift-f',
    run: () => {
      handleFormat()
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
    createGutterDecorationsPlugin(),
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

// Expose editor view + format for parent component
defineExpose({
  editorView,
  format: handleFormat,
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
