// ────────────────────────────────────────────────────────────────
// AstMath.vue — renders LaTeX math expressions using KaTeX
// AstMath.vue — 使用 KaTeX 渲染 LaTeX 数学表达式
// ────────────────────────────────────────────────────────────────

<script setup lang="ts">
import { computed } from 'vue'
import katex from 'katex'
import DOMPurify from 'dompurify'
import 'katex/dist/katex.min.css'

const props = defineProps<{
  value: string
  inline?: boolean
}>()

const renderedHtml = computed(() => {
  try {
    const html = katex.renderToString(props.value, {
      displayMode: !props.inline,
      throwOnError: false,
      output: 'html',
      strict: 'warn',
    })
    return DOMPurify.sanitize(html, {
      ADD_TAGS: ['math', 'span', 'div', 'mrow', 'mi', 'mn', 'mo', 'mtext', 'mspace', 'annotation', 'semantics', 'mfrac', 'msqrt', 'mroot', 'msub', 'msup', 'munder', 'mover', 'mtable', 'mtr', 'mtd', 'mpadded', 'mphantom', 'mfenced', 'menclose', 'msubsup', 'munderover', 'mmultiscripts', 'none'],
      ADD_ATTR: ['class', 'style', 'xmlns', 'mathvariant', 'display', 'stretchy', 'lspace', 'rspace', 'width', 'height', 'depth', 'voffset', 'scriptlevel', 'displaystyle'],
    })
  } catch {
    return `<span style="color:red">${DOMPurify.sanitize(props.value)}</span>`
  }
})
</script>
<template>
  <component :is="inline ? 'span' : 'div'" :class="['math-render-box', { 'math-block': !inline }]"
    v-html="renderedHtml" />
</template>

<style scoped>
.math-render-box {
  overflow-x: auto;
  overflow-y: hidden;
}

.math-render-box :deep(.katex) {
  color: #e8dfd0;
}

.math-block {
  text-align: center;
  margin: 1em 0;
  background: var(--ms-void);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  padding: 1em;
  box-shadow: inset 0 1px 0 oklch(1 0 0 / 0.03);
}

.math-block :deep(.katex-display) {
  margin: 0;
}
</style>
