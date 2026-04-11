<script setup lang="ts">
/**
 * ForgeHeader — Top bar for TheForge editor
 *
 * Features:
 * - Title input with brass border styling
 * - Save button with pulse animation on dirty state
 * - View mode toggle (edit/split/preview)
 * - Category selector
 * - Graph toggle button
 */

import { computed } from 'vue'
import { Wand } from 'lucide-vue-next'
import type { CardItem } from '../../stores/useCardListStore'

type ViewMode = 'edit' | 'split' | 'preview'

interface Category {
  id: number
  name: string
}

interface Props {
  activeCard: CardItem | null
  isDirty: boolean
  isSaving: boolean
  justSaved: boolean
  viewMode: ViewMode
  categories: Category[]
  isRightPanelOpen: boolean
  validationError: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  save: []
  format: []
  toggleView: [mode: ViewMode]
  updateCategory: [categoryId: number | null]
  setTitle: [title: string]
  toggleRightPanel: []
}>()

const viewModes: { key: ViewMode; label: string }[] = [
  { key: 'edit', label: '编辑' },
  { key: 'split', label: '分屏' },
  { key: 'preview', label: '预览' },
]

const canSave = computed(() => {
  if (!props.activeCard) return false
  if (props.isSaving || !props.isDirty) return false
  if (!props.activeCard.title?.trim()) return false
  if (!props.activeCard.content?.trim()) return false
  return true
})

function handleCategoryChange(e: Event) {
  const target = e.target as HTMLSelectElement
  const value = target.value
  emit('updateCategory', value ? Number(value) : null)
}
</script>

<template>
  <div class="forge-header">
    <template v-if="activeCard">
      <!-- Title Display + Dirty Indicator -->
      <div class="forge-header__title-group">
        <span class="forge-header__title">{{ activeCard.title || '无标题' }}</span>

        <!-- Status Indicator -->
        <span v-if="isDirty" class="forge-header__status forge-header__status--dirty">
          <span class="forge-header__pulse"></span>
          未保存
        </span>
        <span v-else-if="activeCard.id" class="forge-header__status forge-header__status--saved">
          已保存
        </span>
      </div>

      <!-- Controls Group -->
      <div class="forge-header__controls">
        <!-- Category Selector -->
        <select
          v-if="activeCard.id"
          :value="activeCard.category_id ?? ''"
          @change="handleCategoryChange"
          class="forge-header__select"
        >
          <option value="">未分类</option>
          <option v-for="cat in categories" :key="cat.id" :value="cat.id">
            {{ cat.name }}
          </option>
        </select>

        <!-- View Mode Toggle -->
        <div class="forge-header__view-toggle">
          <button
            v-for="mode in viewModes"
            :key="mode.key"
            @click="emit('toggleView', mode.key)"
            class="forge-header__view-btn"
            :class="{ 'forge-header__view-btn--active': viewMode === mode.key }"
          >
            {{ mode.label }}
          </button>
        </div>

        <!-- Graph Toggle -->
        <button
          @click="emit('toggleRightPanel')"
          title="切换图谱面板 (Ctrl+\\)"
          class="forge-header__icon-btn"
          :class="{ 'forge-header__icon-btn--active': isRightPanelOpen }"
        >
          <svg class="forge-header__icon" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path d="M12 2a9 9 0 0 0 0 18 9 9 0 0 0 0-18z"/>
            <path d="M12 7a4.5 4.5 0 0 0 0 9 4.5 4.5 0 0 0 0-9z"/>
            <circle cx="12" cy="12" r="1.5" fill="currentColor"/>
          </svg>
        </button>

        <!-- Validation Error -->
        <span v-if="validationError" class="forge-header__error">
          {{ validationError }}
        </span>

        <!-- Format Button -->
        <button
          @click="emit('format')"
          title="格式化 (Ctrl+Shift+F)"
          class="forge-header__format-btn"
        >
          <Wand :size="14" />
        </button>

        <!-- Save Button -->
        <div class="forge-header__save-wrapper">
          <button
            @click="emit('save')"
            :disabled="!canSave"
            class="forge-header__save-btn"
            :class="{
              'forge-header__save-btn--success': justSaved,
              'forge-header__save-btn--disabled': !canSave,
            }"
          >
            {{ justSaved ? '✓ 已保存' : isSaving ? '保存中...' : '保存 ⌘S' }}
          </button>
          <!-- Neon pulse animation on save -->
          <div v-if="justSaved" class="forge-header__pulse-ring"></div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.forge-header {
  height: 56px;
  display: flex;
  align-items: center;
  padding: 0 24px;
  border-bottom: 1px solid #1e1e1e;
  background: #1a1a1a;
  justify-content: space-between;
  flex-shrink: 0;
}

.forge-header__title-group {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.forge-header__title {
  font-size: 13px;
  font-weight: 500;
  color: #d1d5db;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
}

.forge-header__status {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  padding: 4px 10px;
  border-radius: 9999px;
  flex-shrink: 0;
}

.forge-header__status--dirty {
  color: #ffaa00;
  background: rgba(255, 170, 0, 0.1);
}

.forge-header__status--saved {
  color: #6b7280;
}

.forge-header__pulse {
  width: 6px;
  height: 6px;
  background: #f59e0b;
  border-radius: 50%;
  animation: pulse-dot 1.5s ease-in-out infinite;
}

@keyframes pulse-dot {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(0.9); }
}

.forge-header__controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.forge-header__select {
  font-size: 11px;
  background: #0d0d0d;
  color: #9ca3af;
  padding: 6px 10px;
  border: 1px solid #1e1e1e;
  box-shadow: 1px 1px 0 0 rgba(0,0,0,0.4);
  outline: none;
  transition: border-color 0.15s, box-shadow 0.15s;
  cursor: pointer;
}

.forge-header__select:focus {
  border-color: #b8860b;
}

.forge-header__view-toggle {
  display: flex;
  border: 1px solid #1e1e1e;
  font-size: 11px;
}

.forge-header__view-btn {
  padding: 6px 14px;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: #6b7280;
  cursor: pointer;
  transition: all 0.15s;
}

.forge-header__view-btn:hover:not(.forge-header__view-btn--active) {
  color: #d1d5db;
}

.forge-header__view-btn--active {
  color: #00e5ff;
  font-weight: 500;
  border-bottom-color: #00e5ff;
  background: rgba(0,229,255,0.05);
  box-shadow: inset 0 -1px 0 0 rgba(0,229,255,0.3);
}

.forge-header__icon-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid #1e1e1e;
  background: transparent;
  color: #4b5563;
  cursor: pointer;
  transition: all 0.15s;
}

.forge-header__icon-btn:hover:not(.forge-header__icon-btn--active) {
  color: #9ca3af;
  border-color: #6b7280;
}

.forge-header__icon-btn--active {
  color: #00e5ff;
  border-color: rgba(0, 229, 255, 0.3);
  background: rgba(0, 229, 255, 0.05);
  box-shadow: 1px 1px 0 0 rgba(0,229,255,0.2);
}

.forge-header__icon {
  width: 16px;
  height: 16px;
}

.forge-header__error {
  font-size: 11px;
  color: #ff4444;
  animation: pulse-text 1s ease-in-out infinite;
}

@keyframes pulse-text {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

/* Format button — mechanical stamp */
.forge-header__format-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid #1e1e1e;
  background: transparent;
  color: #4b5563;
  cursor: pointer;
  transition: transform 0.15s ease, box-shadow 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  box-shadow: 1px 1px 0 0 rgba(0,0,0,0.4);
}

.forge-header__format-btn:hover {
  color: #00e5ff;
  border-color: rgba(0,229,255,0.3);
  transform: translate(-1px, -1px);
  box-shadow: 2px 2px 0 0 rgba(0,0,0,0.4);
}

.forge-header__format-btn:active {
  transform: translate(1px, 1px);
  box-shadow: 0px 0px 0 0 rgba(0,0,0,0.4);
}

.forge-header__save-wrapper {
  position: relative;
}

.forge-header__save-btn {
  position: relative;
  padding: 8px 18px;
  font-size: 13px;
  border-radius: 2px;
  transition: all 0.15s;
  z-index: 1;
  cursor: pointer;
}

.forge-header__save-btn--success {
  background: rgba(0, 230, 118, 0.2);
  color: #00e676;
  border: 1px solid rgba(0, 230, 118, 0.3);
  box-shadow: inset 0 1px 0 0 rgba(255,255,255,0.06), 2px 2px 0 0 rgba(0,0,0,0.5);
}

.forge-header__save-btn:not(.forge-header__save-btn--disabled):not(.forge-header__save-btn--success) {
  background: #00b8cc;
  color: #0d0d0d;
  border: 1px solid #00b8cc;
  box-shadow: inset 0 1px 0 0 rgba(255,255,255,0.1), 2px 2px 0 0 rgba(0,0,0,0.5);
}

.forge-header__save-btn:not(.forge-header__save-btn--disabled):hover {
  background: #00c9db;
  transform: translate(-1px, -1px);
  box-shadow: inset 0 1px 0 0 rgba(255,255,255,0.1), 3px 3px 0 0 rgba(0,0,0,0.5);
}

.forge-header__save-btn:not(.forge-header__save-btn--disabled):active {
  transform: translate(1px, 1px);
  box-shadow: inset 0 1px 0 0 rgba(255,255,255,0.05), 0px 0px 0 0 rgba(0,0,0,0.5);
}

.forge-header__save-btn--disabled {
  background: #222222;
  color: #4b5563;
  cursor: not-allowed;
}

.forge-header__pulse-ring {
  position: absolute;
  inset: -4px;
  border-radius: 4px;
  z-index: 0;
  pointer-events: none;
  animation: neon-burst 1.2s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

@keyframes neon-burst {
  0% {
    box-shadow:
      0 0 0 0 rgba(0, 229, 255, 0.6),
      0 0 0 0 rgba(0, 229, 255, 0.3);
    opacity: 1;
  }
  50% {
    box-shadow:
      0 0 12px 4px rgba(0, 229, 255, 0.4),
      0 0 24px 8px rgba(0, 229, 255, 0.15);
    opacity: 0.8;
  }
  100% {
    box-shadow:
      0 0 20px 10px rgba(0, 229, 255, 0),
      0 0 40px 20px rgba(0, 229, 255, 0);
    opacity: 0;
  }
}
</style>
