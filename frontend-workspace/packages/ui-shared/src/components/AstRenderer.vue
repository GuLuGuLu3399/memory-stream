// ────────────────────────────────────────────────────────────────
// AstRenderer.vue — recursively renders an AST node as Vue components
// AstRenderer.vue — 递归地将 AST 节点渲染为 Vue组件
// ────────────────────────────────────────────────────────────────


<script setup lang="ts">
import type { AstNode, AstHeading } from '@memory-stream/types'
import AstCodeBlock from './AstCodeBlock.vue'
import AstMath from './AstMath.vue'
import AstCloze from './AstCloze.vue'
import AstAdmonition from './AstAdmonition.vue'
import { useIsTouchDevice } from '../composables/useIsTouchDevice'

withDefaults(defineProps<{
  node: AstNode
  inHead?: boolean
}>(), { inHead: false })

const emit = defineEmits<{
  (e: 'navigate', target: string): void
  (e: 'toggle-heading-status', payload: { level: number; headingText: string; currentStatus: string }): void
  (e: 'concept-ref-hover', payload: { term: string; x: number; y: number }): void
  (e: 'concept-ref-leave'): void
}>()

function forward(target: string) {
  emit('navigate', target)
}

function forwardHeadingToggle(p: { level: number; headingText: string; currentStatus: string }) {
  emit('toggle-heading-status', p)
}

function forwardConceptRefHover(p: { term: string; x: number; y: number }) {
  emit('concept-ref-hover', p)
}

function forwardConceptRefLeave() {
  emit('concept-ref-leave')
}

function extractHeadingText(node: AstHeading): string {
  const texts: string[] = []
  function walk(n: AstNode) {
    if (n.type === 'Text') texts.push(n.value)
    else if ('children' in n) (n as any).children?.forEach(walk)
  }
  node.children.forEach(walk)
  return texts.join('')
}

function handleStatusClick(node: AstHeading) {
  emit('toggle-heading-status', {
    level: node.level,
    headingText: extractHeadingText(node),
    currentStatus: node.status ?? 'none',
  })
}

const { isTouchDevice } = useIsTouchDevice()
let longPressTimer: ReturnType<typeof setTimeout> | null = null
const LONG_PRESS_MS = 500

function handleConceptRefEnter(term: string, event: MouseEvent) {
  if (isTouchDevice.value) return
  emit('concept-ref-hover', { term, x: event.clientX, y: event.clientY })
}

function handleConceptRefLeave() {
  if (isTouchDevice.value) return
  emit('concept-ref-leave')
}

function handleConceptRefTouchStart(term: string, event: TouchEvent) {
  const touch = event.touches[0]
  longPressTimer = setTimeout(() => {
    emit('concept-ref-hover', { term, x: touch.clientX, y: touch.clientY })
  }, LONG_PRESS_MS)
}

function handleConceptRefTouchEnd() {
  if (longPressTimer) {
    clearTimeout(longPressTimer)
    longPressTimer = null
  }
  emit('concept-ref-leave')
}

function slugify(node: AstHeading): string {
  const texts: string[] = []
  function walk(n: AstNode) {
    if (n.type === 'Text') texts.push(n.value)
    else if ('children' in n) n.children?.forEach(walk)
  }
  node.children.forEach(walk)
  return texts.join('').toLowerCase()
    .replace(/[^\p{L}\p{N}]/gu, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '')
    .substring(0, 64) || 'heading'
}
</script>

<template>
  <!-- Structural: Root -->
  <div v-if="node.type === 'Root'" class="ms-article-root">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </div>

  <!-- Headings (with optional status icon) -->
  <component :is="`h${node.level}`" v-else-if="node.type === 'Heading'" :id="slugify(node)" class="ms-h">
    <span v-if="node.status === 'Done'" class="ms-heading-status ms-heading-status--done" title="已掌握（点击切换）" @click.stop="handleStatusClick(node)">✓</span>
    <span v-else-if="node.status === 'Undone'" class="ms-heading-status ms-heading-status--undone" title="待复习（点击切换）" @click.stop="handleStatusClick(node)">○</span>
    <span v-else-if="node.status === 'Unclear'" class="ms-heading-status ms-heading-status--unclear" title="不确定（点击切换）" @click.stop="handleStatusClick(node)">~</span>
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </component>

  <!-- Paragraph -->
  <p v-else-if="node.type === 'Paragraph'" class="ms-p">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </p>

  <!-- Text -->
  <template v-else-if="node.type === 'Text'">{{ node.value }}</template>

  <!-- Inline styles -->
  <strong v-else-if="node.type === 'Strong'">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </strong>
  <em v-else-if="node.type === 'Emphasis'">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </em>
  <del v-else-if="node.type === 'Strikethrough'">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </del>

  <!-- Code -->
  <AstCodeBlock v-else-if="node.type === 'CodeBlock'" :language="node.language" :value="node.value" />
  <code v-else-if="node.type === 'InlineCode'" class="ms-inline-code">{{ node.value }}</code>

  <!-- Math -->
  <AstMath v-else-if="node.type === 'Math'" :value="node.value" :inline="node.inline" />

  <!-- Links & Images -->
  <a v-else-if="node.type === 'Link'" :href="node.url" target="_blank" rel="noopener" class="ms-ext-link">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </a>
  <img v-else-if="node.type === 'Image' && !/\.(mp4|webm|mov)$/i.test(node.url)" :src="node.url" :alt="node.alt" class="ms-img" />
  <video v-else-if="node.type === 'Image' && /\.(mp4|webm|mov)$/i.test(node.url)" :src="node.url" controls class="ms-video" />

  <!-- Wikilink -->
  <button v-else-if="node.type === 'Wikilink'" class="ms-wikilink" @click="emit('navigate', node.target)">
    [[{{ node.target }}{{ node.alias ? `|${node.alias}` : '' }}]]
  </button>

  <!-- Cloze (memory mask) -->
  <AstCloze v-else-if="node.type === 'Cloze'" :children="node.children" />

  <!-- ConceptRef (term reference) -->
  <span v-else-if="node.type === 'ConceptRef'" class="ms-concept-ref"
        @mouseenter="handleConceptRefEnter(node.term, $event)"
        @mouseleave="handleConceptRefLeave"
        @touchstart.passive="handleConceptRefTouchStart(node.term, $event)"
        @touchend="handleConceptRefTouchEnd"
        @click.stop>@{{ node.term }}</span>

  <!-- Blockquote -->
  <blockquote v-else-if="node.type === 'Blockquote'" class="ms-blockquote">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </blockquote>

  <!-- Admonition (warning / tip / question) -->
  <AstAdmonition v-else-if="node.type === 'Admonition'" :kind="node.kind" :children="node.children" />

  <!-- Lists -->
  <ul v-else-if="node.type === 'List' && !node.ordered" class="ms-ul">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </ul>
  <ol v-else-if="node.type === 'List' && node.ordered" class="ms-ol" :start="node.start ?? undefined">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </ol>
  <li v-else-if="node.type === 'ListItem'" class="ms-li">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </li>

  <!-- Task list marker -->
  <input v-else-if="node.type === 'TaskListMarker'" type="checkbox" :checked="node.checked" disabled
    class="ms-task-checkbox" />

  <!-- Thematic break -->
  <hr v-else-if="node.type === 'ThematicBreak'" class="ms-hr" />

  <!-- Table -->
  <div v-else-if="node.type === 'Table'" class="ms-table-scroll">
    <table class="ms-table">
      <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
    </table>
  </div>
  <thead v-else-if="node.type === 'TableHead'">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" :in-head="true" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </thead>
  <tr v-else-if="node.type === 'TableRow'">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" :in-head="inHead" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </tr>
  <td v-else-if="node.type === 'TableCell' && !inHead">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </td>
  <th v-else-if="node.type === 'TableCell' && inHead">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </th>

  <!-- Footnotes -->
  <aside v-else-if="node.type === 'FootnoteDefinition'" :id="`fn-${node.name}`" class="ms-fn-def">
    <sup><a :href="`#fnref-${node.name}`">^</a></sup>
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </aside>
  <sup v-else-if="node.type === 'FootnoteReference'" class="ms-fn-ref">
    <a :href="`#fn-${node.name}`" :id="`fnref-${node.name}`">[{{ node.name }}]</a>
  </sup>

  <!-- Definition list -->
  <dl v-else-if="node.type === 'DefinitionList'" class="ms-dl">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </dl>
  <dt v-else-if="node.type === 'DefinitionListTitle'" class="ms-dt">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </dt>
  <dd v-else-if="node.type === 'DefinitionListDefinition'" class="ms-dd">
    <AstRenderer v-for="(child, i) in node.children" :key="i" :node="child" @navigate="forward" @toggle-heading-status="forwardHeadingToggle" @concept-ref-hover="forwardConceptRefHover" @concept-ref-leave="forwardConceptRefLeave" />
  </dd>

  <!-- Fallback -->
  <span v-else class="ms-unknown">[{{ node.type }}]</span>
</template>

<style scoped>
/* ── Root ── */
.ms-article-root {
  line-height: 1.75;
  color: #c8bfa8;
  word-wrap: break-word;
  font-family: var(--font-sans);
  font-size: 15px;
}

/* ── Headings ── */
.ms-h {
  color: #f5ead0;
  font-family: var(--font-sans);
  font-weight: 600;
  letter-spacing: 0.01em;
  margin-top: 1.6em;
  margin-bottom: 0.5em;
}

h1.ms-h {
  font-size: 1.6em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid var(--ms-border-light);
}

h2.ms-h {
  font-size: 1.35em;
  padding-bottom: 0.25em;
  border-bottom: 1px solid var(--ms-border-light);
}

h3.ms-h { font-size: 1.15em; }
h4.ms-h { font-size: 1.05em; }
h5.ms-h, h6.ms-h { font-size: 1em; color: var(--text-secondary); }

/* ── Paragraph ── */
.ms-p {
  margin-bottom: 1em;
}

/* ── Blockquote — telemetry log ── */
.ms-blockquote {
  background: var(--ms-carbon);
  border-left: 4px solid var(--brass);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 0.92em;
  padding: 0.75em 1em;
  margin: 1em 0;
  border-radius: 0 2px 2px 0;
  box-shadow: inset 0 1px 0 oklch(1 0 0 / 0.03);
}

/* ── Lists ── */
.ms-ul,
.ms-ol {
  padding-left: 1.5em;
  margin-bottom: 1em;
}

.ms-li {
  margin-bottom: 0.3em;
}

.ms-li::marker {
  color: var(--text-muted);
}

.ms-task-checkbox {
  margin-right: 0.5em;
  vertical-align: middle;
  accent-color: var(--neon);
}

/* ── HR ── */
.ms-hr {
  border: none;
  border-top: 1px solid var(--ms-border-light);
  margin: 2em 0;
}

/* ── Image ── */
.ms-img {
  max-width: 100%;
  border-radius: 2px;
  border: 1px solid var(--ms-border);
}

/* ── Video ── */
.ms-video {
  max-width: 100%;
  border-radius: 2px;
  border: 1px solid var(--ms-border);
}

/* ── Heading status ── */
.ms-heading-status {
  margin-right: 0.35em;
  font-weight: 600;
  font-size: 0.85em;
  cursor: pointer;
  transition: transform 150ms cubic-bezier(0.33, 0, 0.2, 1);
}

.ms-heading-status:hover {
  transform: scale(1.3);
}
.ms-heading-status--done { color: #4caf50; }
.ms-heading-status--undone { color: var(--text-muted, #555); }
.ms-heading-status--unclear { color: #d4a017; }

/* ── ConceptRef ── */
.ms-concept-ref {
  text-decoration: underline;
  text-decoration-style: dotted;
  text-underline-offset: 0.15em;
  color: #e8dfd0;
  font-style: italic;
  cursor: help;
}

/* ── Inline code ── */
.ms-inline-code {
  background: var(--ms-carbon);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  padding: 0.1em 0.4em;
  font-family: var(--font-mono);
  font-size: 0.88em;
  color: #e8dfd0;
  box-shadow: inset 0 1px 0 oklch(1 0 0 / 0.03);
}

/* ── Links ── */
.ms-ext-link {
  color: var(--neon);
  text-decoration: none;
  transition: text-shadow var(--duration-fast) var(--ease-hydraulic);
}

.ms-ext-link:hover {
  text-shadow: 0 0 6px oklch(0.78 0.17 200 / 0.4);
}

/* ── Wikilinks ── */
.ms-wikilink {
  background: none;
  border: none;
  color: var(--neon);
  font-family: var(--font-mono);
  font-weight: 500;
  font-size: 0.92em;
  cursor: pointer;
  padding: 0;
  text-decoration: none;
  transition: text-shadow var(--duration-fast) var(--ease-hydraulic);
}

.ms-wikilink:hover {
  text-shadow: 0 0 8px oklch(0.78 0.17 200 / 0.5);
}

/* ── Table ── */
.ms-table-scroll {
  overflow-x: auto;
  margin: 1em 0;
}
.ms-table-scroll::-webkit-scrollbar {
  height: 4px;
}
.ms-table-scroll::-webkit-scrollbar-thumb {
  background: var(--ms-border-light);
  border-radius: 2px;
}

.ms-table {
  border-collapse: collapse;
  width: 100%;
  font-size: 0.92em;
  border: 1px solid var(--ms-border);
}

.ms-table :deep(tr) {
  border-bottom: 1px solid var(--ms-border-light);
}

.ms-table :deep(th) {
  padding: 0.5em 0.75em;
  text-align: left;
  background: var(--ms-carbon);
  color: #f5ead0;
  font-weight: 600;
}

.ms-table :deep(td) {
  padding: 0.5em 0.75em;
  text-align: left;
}

/* ── Footnotes ── */
.ms-fn-def {
  font-size: 0.88em;
  margin: 0.5em 0;
  color: var(--text-secondary);
}

.ms-fn-ref a {
  color: var(--brass);
  text-decoration: none;
  font-family: var(--font-mono);
  font-size: 0.85em;
}

.ms-fn-ref a:hover {
  color: #d4a017;
}

/* ── Definition list ── */
.ms-dl {
  margin: 1em 0;
}

.ms-dt {
  font-weight: 600;
  margin-top: 0.5em;
  color: var(--text-primary);
}

.ms-dd {
  margin-left: 1.5em;
  margin-bottom: 0.25em;
  color: var(--text-secondary);
}

/* ── Unknown ── */
.ms-unknown {
  color: #e53935;
  border: 1px dashed #e53935;
  padding: 0 4px;
  font-size: 0.85em;
  font-family: var(--font-mono);
}

/* ── Strong / Emphasis / Del ── */
strong {
  color: #f5ead0;
  font-weight: 600;
}

em {
  font-style: italic;
  color: #d4c9a8;
}

del {
  opacity: 0.5;
  text-decoration: line-through;
}
</style>
