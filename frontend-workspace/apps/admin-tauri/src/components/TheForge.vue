<script setup lang="ts">
/**
 * TheForge — Card editor with preview (redesigned with sub-components)
 *
 * Features:
 * - Title input with brass styling
 * - CodeMirror editor with Markdown support
 * - Split view with draggable resizer
 * - Live preview with debounced rendering
 * - Backlinks radar panel
 * - Image paste handling
 * - Keyboard shortcuts (Ctrl+S)
 */

import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useKnowledgeStore } from '../stores/knowledge'
import { useLayoutStore } from '../stores/layout'
import { storeToRefs } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { EditorView } from '@codemirror/view'
import CodemirrorEditor from './CodemirrorEditor.vue'
import ForgeHeader from './forge/ForgeHeader.vue'
import ForgePreview from './forge/ForgePreview.vue'
import BacklinksRadar, { type BacklinkItem } from './forge/BacklinksRadar.vue'
import ForgeEmptyState from './forge/ForgeEmptyState.vue'
import { useForgeRender } from './forge/useForgeRender'

type ViewMode = 'edit' | 'split' | 'preview'

// Store integration
const store = useKnowledgeStore()
const layoutStore = useLayoutStore()
const {
  activeCard,
  isLoading,
  isSaving,
  isDirty,
  justSaved,
  categories,
  backlinks,
} = storeToRefs(store)
const { isRightPanelOpen, isMergeConsoleOpen } = storeToRefs(layoutStore)

// Editor ref for image paste handling
const codemirrorRef = ref<{ editorView: EditorView | null } | null>(null)

// View mode state
const viewMode = ref<ViewMode>('split')

// Split ratio for split view (50% default)
const splitRatio = ref(50)
const isDragging = ref(false)

// Render preview using composable
const { html, isRendering, triggerRender } = useForgeRender(
  () => activeCard.value?.content ?? ''
)

// Validation state
const validationError = ref('')

// ============================================================================
// Image Paste Handler
// ============================================================================

async function handleImagePaste(e: ClipboardEvent) {
  const items = e.clipboardData?.items
  if (!items) return

  for (const item of items) {
    if (item.type.indexOf('image') !== -1) {
      e.preventDefault()
      const blob = item.getAsFile()
      if (!blob) continue

      const view = codemirrorRef.value?.editorView
      if (!view || !activeCard.value) return

      const { from, to } = view.state.selection.main
      const placeholder = '\n![上传中...]()\n'

      view.dispatch({
        changes: { from, to, insert: placeholder },
        selection: { anchor: from + placeholder.length },
      })

      const reader = new FileReader()
      reader.onload = async () => {
        const result = reader.result as string
        const base64Data = result.split(',')[1]

        try {
          const uploadResult = await invoke<{ url: string; key: string }>(
            'upload_clipboard_image',
            { base64Data }
          )
          const imageMd = `\n![image](${uploadResult.url})\n`
          const doc = view.state.doc.toString()
          const pos = doc.indexOf(placeholder)
          if (pos !== -1) {
            view.dispatch({
              changes: { from: pos, to: pos + placeholder.length, insert: imageMd },
              selection: { anchor: pos + imageMd.length },
            })
          }
        } catch (error) {
          console.error('[Forge] 图片上传失败:', error)
          const doc = view.state.doc.toString()
          const pos = doc.indexOf(placeholder)
          if (pos !== -1) {
            view.dispatch({
              changes: {
                from: pos,
                to: pos + placeholder.length,
                insert: `\n> ❌ 图片上传失败: ${error}\n`,
              },
            })
          }
        }
      }
      reader.readAsDataURL(blob)
      break
    }
  }
}

// ============================================================================
// Split View Resizer
// ============================================================================

function startDrag(_e: MouseEvent) {
  isDragging.value = true
  document.addEventListener('mousemove', onDrag)
  document.addEventListener('mouseup', stopDrag)
}

function onDrag(e: MouseEvent) {
  if (!isDragging.value) return

  const container = document.querySelector('.forge-editor-container') as HTMLElement
  if (!container) return

  const rect = container.getBoundingClientRect()
  const x = e.clientX - rect.left
  const ratio = (x / rect.width) * 100

  // Clamp between 20% and 80%
  splitRatio.value = Math.max(20, Math.min(80, ratio))
}

function stopDrag() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
}

// ============================================================================
// Save & Validation
// ============================================================================

const canSave = computed(() => {
  if (!activeCard.value) return false
  if (isSaving.value || isLoading.value || !isDirty.value) return false
  if (!activeCard.value.title?.trim()) return false
  if (!activeCard.value.content?.trim()) return false
  return true
})

function validateBeforeSave(): boolean {
  if (!activeCard.value) return false
  if (!activeCard.value.title?.trim()) {
    validationError.value = '标题不能为空'
    return false
  }
  if (!activeCard.value.content?.trim()) {
    validationError.value = '内容不能为空 — 请输入 Markdown 正文'
    return false
  }
  validationError.value = ''
  return true
}

function handleSave() {
  if (!validateBeforeSave()) return
  store.saveCard()
}

function handleFormat() {
  codemirrorRef.value?.format()
}

// ============================================================================
// Wikilink Navigation
// ============================================================================

function handleWikilinkClick(title: string) {
  store.loadAndActivateCardByTitle(title)
}

// ============================================================================
// Category Update
// ============================================================================

function handleCategoryUpdate(categoryId: number | null) {
  if (activeCard.value) {
    activeCard.value.category_id = categoryId
    store.checkDirty()
  }
}

// ============================================================================
// Title Update
// ============================================================================

function handleSetTitle(_title: string) {
  if (activeCard.value) {
    activeCard.value.title = _title
    store.checkDirty()
  }
}

// ============================================================================
// Keyboard Shortcuts
// ============================================================================

function handleKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === 's') {
    e.preventDefault()
    if (canSave.value) handleSave()
  }
}

// ============================================================================
// Lifecycle & Watchers
// ============================================================================

// Watch for dirty state changes
watch(
  () => activeCard.value?.content,
  () => {
    store.checkDirty()
    if (validationError.value && activeCard.value?.content?.trim()) {
      validationError.value = ''
    }
  }
)

watch(
  () => activeCard.value?.title,
  () => {
    store.checkDirty()
    if (validationError.value && activeCard.value?.title?.trim()) {
      validationError.value = ''
    }
  }
)

// Trigger initial render when card changes
watch(
  () => activeCard.value?.id,
  () => {
    triggerRender()
  }
)

onMounted(() => {
  triggerRender()
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

// ============================================================================
// Computed Styles
// ============================================================================

const editorStyle = computed(() => {
  if (viewMode.value === 'edit') return { flex: 1 }
  if (viewMode.value === 'split') return { flex: `0 0 ${splitRatio.value}%` }
  return { display: 'none' }
})

const previewStyle = computed(() => {
  if (viewMode.value === 'preview') return { flex: 1 }
  if (viewMode.value === 'split') return { flex: `0 0 ${100 - splitRatio.value}%` }
  return { display: 'none' }
})

const showResizer = computed(() => viewMode.value === 'split')

const shouldDimForMerge = computed(() => isMergeConsoleOpen.value)
</script>

<template>
  <main
    class="the-forge"
    :class="{ 'the-forge--dim': shouldDimForMerge }"
    @keydown="handleKeydown"
  >
    <!-- Header -->
    <ForgeHeader
      :active-card="activeCard"
      :is-dirty="isDirty"
      :is-saving="isSaving"
      :just-saved="justSaved"
      :view-mode="viewMode"
      :categories="categories"
      :is-right-panel-open="isRightPanelOpen"
      :validation-error="validationError"
      @save="handleSave"
      @format="handleFormat"
      @toggle-view="(mode) => (viewMode = mode)"
      @update-category="handleCategoryUpdate"
      @set-title="handleSetTitle"
      @toggle-right-panel="layoutStore.toggleRightPanel()"
    />

    <!-- Content Area -->
    <Transition name="forge-content" mode="out-in">
      <div v-if="activeCard" key="editor" class="forge-main">
        <div class="forge-editor-container">
          <!-- Editor Pane -->
          <div class="forge-pane forge-pane--editor" :style="editorStyle">
            <input
              :value="activeCard.title"
              @input="handleSetTitle(($event.target as HTMLInputElement).value)"
              placeholder="无标题..."
              class="forge-title-input"
            />
            <div class="forge-editor-wrapper" @paste="handleImagePaste">
              <CodemirrorEditor
                ref="codemirrorRef"
                v-model="activeCard.content"
                placeholder="开始锻造知识... (支持 Markdown，可粘贴图片)"
                @save="handleSave"
              />
            </div>
          </div>

          <!-- Draggable Resizer -->
          <div
            v-if="showResizer"
            class="forge-resizer"
            :class="{ 'forge-resizer--dragging': isDragging }"
            @mousedown="startDrag"
          >
            <div class="forge-resizer__handle">
              <div class="forge-resizer__rivet"></div>
            </div>
          </div>

          <!-- Preview Pane -->
          <div class="forge-pane forge-pane--preview" :style="previewStyle">
            <ForgePreview
              :html="html"
              :loading="isRendering"
              :title="activeCard.title"
              :show-title="viewMode === 'preview'"
              @link-click="handleWikilinkClick"
            />
          </div>
        </div>

        <!-- Loading Overlay -->
        <Transition name="fade">
          <div v-if="isLoading" class="forge-loading">
            <div class="forge-loading__spinner">
              <svg class="forge-loading__icon" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
              </svg>
              <span class="forge-loading__text">加载中...</span>
            </div>
          </div>
        </Transition>

        <!-- Backlinks Radar -->
        <BacklinksRadar
          v-if="activeCard.id && backlinks.length > 0"
          :backlinks="backlinks as BacklinkItem[]"
          :loading="false"
          @navigate="store.loadAndActivateCard"
        />
      </div>

      <!-- Empty State -->
      <ForgeEmptyState v-else key="empty" />
    </Transition>
  </main>
</template>

<style scoped>
.the-forge {
  height: 100%;
  background: #141414;
  display: flex;
  flex-direction: column;
  position: relative;
  min-width: 0;
}

.the-forge--dim {
  filter: brightness(0.4);
  pointer-events: none;
}

/* Main content area */
.forge-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.forge-editor-container {
  flex: 1;
  display: flex;
  min-height: 0;
  position: relative;
}

/* Panes */
.forge-pane {
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 280px;
  overflow: hidden;
}

.forge-pane--editor {
  border-right: 1px solid #1e1e1e;
}

/* Title input */
.forge-title-input {
  width: 100%;
  padding: 32px 24px 8px;
  font-size: 24px;
  font-weight: 700;
  color: #f3f4f6;
  border: none;
  border-bottom: 1px solid #b8860b;
  background: transparent;
  outline: none;
  flex-shrink: 0;
}

.forge-title-input::placeholder {
  color: #4b5563;
}

.forge-title-input:focus {
  box-shadow: 0 0 8px rgba(184, 134, 11, 0.2);
}

/* Editor wrapper */
.forge-editor-wrapper {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

/* Resizer */
.forge-resizer {
  width: 4px;
  background: #1e1e1e;
  cursor: col-resize;
  flex-shrink: 0;
  position: relative;
  transition: background 0.15s;
}

.forge-resizer:hover,
.forge-resizer--dragging {
  background: #b8860b;
}

.forge-resizer__handle {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.forge-resizer__rivet {
  width: 8px;
  height: 8px;
  background: #b8860b;
  border-radius: 50%;
  box-shadow: 0 0 4px rgba(184, 134, 11, 0.3);
}

/* Loading overlay */
.forge-loading {
  position: absolute;
  inset: 0;
  background: rgba(20, 20, 20, 0.8);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
}

.forge-loading__spinner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: #6b7280;
}

.forge-loading__icon {
  width: 20px;
  height: 20px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.forge-loading__text {
  font-size: 12px;
}

/* Transitions */
.forge-content-enter-active {
  transition: opacity 0.2s ease-out, transform 0.2s ease-out;
}

.forge-content-leave-active {
  transition: opacity 0.15s ease-in;
}

.forge-content-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.forge-content-leave-to {
  opacity: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
