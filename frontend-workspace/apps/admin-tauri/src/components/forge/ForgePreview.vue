<script setup lang="ts">
/**
 * ForgePreview — Preview pane for rendered Markdown
 *
 * Features:
 * - Displays rendered HTML via MarkdownViewer (KaTeX / Mermaid / Shiki)
 * - Loading skeleton state
 * - Handles wikilink navigation
 */

import { computed } from 'vue'
import { SkeletonBlock, MarkdownViewer } from '@memory-stream/ui-shared'

interface Props {
  html: string
  loading: boolean
  title?: string
  showTitle?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showTitle: true,
})

const emit = defineEmits<{
  linkClick: [title: string]
}>()

const hasContent = computed(() => props.html && props.html.trim())

function handleClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  const link = target.closest('a.wikilink') as HTMLAnchorElement | null
  if (!link) return
  e.preventDefault()
  const href = link.getAttribute('href')
  const title = href ?? (link.textContent ?? '')
  if (title) {
    emit('linkClick', title)
  }
}
</script>

<template>
  <div class="forge-preview">
    <!-- Title in preview-only mode -->
    <div v-if="showTitle && title" class="forge-preview__title">
      <h1>{{ title }}</h1>
    </div>

    <!-- Loading skeleton — initial load only (no previous content) -->
    <div v-if="loading && !hasContent" class="forge-preview__loading">
      <SkeletonBlock v-for="i in 5" :key="i" class="forge-preview__skeleton" />
    </div>

    <!-- Empty State -->
    <div v-else-if="!hasContent" class="forge-preview__empty">
      <svg class="forge-preview__empty-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      <p>在编辑区输入 Markdown，这里会实时预览</p>
    </div>

    <!-- Rendered Content (MarkdownViewer persists during re-renders to avoid expensive re-init) -->
    <div
      v-else
      class="forge-preview__content"
      @click="handleClick"
    >
      <MarkdownViewer :html-content="html" />
    </div>
  </div>
</template>

<style scoped>
.forge-preview {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.forge-preview__title {
  flex-shrink: 0;
  padding: 32px 24px 8px;
}

.forge-preview__title h1 {
  font-size: 24px;
  font-weight: 700;
  color: #f3f4f6;
  margin: 0;
}

.forge-preview__loading {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.forge-preview__skeleton {
  height: 16px;
  border-radius: 2px;
}

.forge-preview__skeleton:nth-child(1) { width: 80%; }
.forge-preview__skeleton:nth-child(2) { width: 100%; }
.forge-preview__skeleton:nth-child(3) { width: 90%; }
.forge-preview__skeleton:nth-child(4) { width: 70%; }
.forge-preview__skeleton:nth-child(5) { width: 85%; }

.forge-preview__empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  color: #6b7280;
}

.forge-preview__empty-icon {
  width: 48px;
  height: 48px;
  opacity: 0.3;
}

.forge-preview__empty p {
  font-size: 13px;
  font-style: italic;
}

.forge-preview__content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 24px 24px;
}

/* Wikilink styling */
.forge-preview__content :deep(a.wikilink) {
  color: #00e5ff;
  border-bottom: 1px dashed rgba(0, 229, 255, 0.3);
  cursor: pointer;
  transition: border-color 0.15s;
}

.forge-preview__content :deep(a.wikilink:hover) {
  border-bottom-color: rgba(0, 229, 255, 0.8);
}
</style>
