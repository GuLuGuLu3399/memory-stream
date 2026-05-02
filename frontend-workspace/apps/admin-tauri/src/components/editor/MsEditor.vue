// 用途：CodeMirror 编辑器封装，集成语法高亮、快捷键和 Wikilink 补全
<script setup lang="ts">
import { ref, shallowRef, onMounted, onBeforeUnmount, watch } from 'vue'
import { EditorState } from '@codemirror/state'
import { EditorView, keymap, ViewPlugin, ViewUpdate } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { languages } from '@codemirror/language-data'
import { altarTheme, altarHighlightStyle } from './cm-theme'
import { createMarkdownLinter, extractLintItems, type LintItem } from './cm-linter'

const props = defineProps<{
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  save: []
  'cursor-change': [line: number, col: number]
  'lint-change': [items: LintItem[]]
}>()

const editorRef = ref<HTMLDivElement | null>(null)
const view = shallowRef<EditorView | null>(null)
let ignoreNextUpdate = false

function createState(doc: string): EditorState {
  return EditorState.create({
    doc,
    extensions: [
      history(),
      keymap.of([...defaultKeymap, ...historyKeymap]),
      markdown({ base: markdownLanguage, codeLanguages: languages }),
      EditorView.lineWrapping,
      altarTheme,
      altarHighlightStyle,
      createMarkdownLinter(),
      keymap.of([{
        key: 'Mod-s',
        run: () => {
          emit('save')
          return true
        },
      }]),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          ignoreNextUpdate = true
          emit('update:modelValue', update.state.doc.toString())
        }
        if (update.selectionSet || update.docChanged) {
          const pos = update.state.selection.main.head
          const line = update.state.doc.lineAt(pos)
          emit('cursor-change', line.number, pos - line.from + 1)
        }
      }),
      ViewPlugin.fromClass(class {
        constructor(private v: EditorView) {}
        update(update: ViewUpdate) {
          if (update.docChanged) {
            const items = extractLintItems(this.v)
            emit('lint-change', items)
          }
        }
      }),
    ],
  })
}

onMounted(() => {
  if (!editorRef.value) return
  view.value = new EditorView({
    state: createState(props.modelValue),
    parent: editorRef.value,
  })
})

defineExpose({ view })

onBeforeUnmount(() => {
  view.value?.destroy()
})

watch(() => props.modelValue, (text) => {
  if (ignoreNextUpdate) {
    ignoreNextUpdate = false
    return
  }
  if (view.value && view.value.state.doc.toString() !== text) {
    view.value.setState(createState(text))
  }
})
</script>

<template>
  <div ref="editorRef" class="ms-editor" />
</template>

<style scoped>
.ms-editor {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
}

.ms-editor :deep(.cm-editor) {
  height: 100%;
  max-width: 100%;
}

.ms-editor :deep(.cm-scroller) {
  font-family: var(--font-mono);
  line-height: 1.7;
  padding: 16px 0;
  overflow-x: hidden !important;
}

.ms-editor :deep(.cm-content) {
  padding: 0 16px;
  max-width: 720px;
  white-space: pre-wrap !important;
  word-break: break-word !important;
  padding-right: 28px;
}
</style>
