// 用途：Frontmatter 编辑弹窗，显示和编辑卡片元数据
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useEditorStore } from '@/stores/editor'
import { useTreeStore } from '@/stores/tree'
import { useLayoutStore } from '@/stores/layout'
import { useToast } from '@/composables/core/useToast'
import { useFrontmatter } from '@/composables/editor/useFrontmatter'
import { renameCard, moveCard } from '@/bridge/invoke'

const emit = defineEmits<{ close: [] }>()

const store = useEditorStore()
const treeStore = useTreeStore()
const layout = useLayoutStore()
const toast = useToast()

const { yamlDraft, parseYamlDraft: parseYaml, syncDraft } = useFrontmatter(() => store.currentMeta)

const rawMode = ref(false)
const savedField = ref<string | null>(null)
const isRenaming = ref(false)

syncDraft()

const yamlTitle = computed({
  get: () => yamlDraft.value.match(/^title:\s*(.+)$/m)?.[1] ?? '',
  set: (v) => { yamlDraft.value = yamlDraft.value.replace(/^(title:\s*).+$/m, `$1${v}`) },
})

const yamlCategory = computed({
  get: () => yamlDraft.value.match(/^category:\s*(.+)$/m)?.[1] ?? '',
  set: (v) => { yamlDraft.value = yamlDraft.value.replace(/^(category:\s*).+$/m, `$1${v}`) },
})

async function handleFieldBlur(field: string) {
  const uuid = store.currentUuid
  const current = store.currentMeta
  if (!uuid || !current || isRenaming.value) return

  if (field === 'title') {
    const nextTitle = yamlTitle.value.trim()
    if (!nextTitle || nextTitle === current.title.trim()) return
    isRenaming.value = true
    try {
      const newMeta = await renameCard(uuid, nextTitle)
      store.currentMeta = newMeta
      await treeStore.loadTree()
      savedField.value = 'title'
      setTimeout(() => { savedField.value = null }, 300)
    } catch (e) {
      console.error('[FrontmatterModal] title blur save failed:', e)
      toast.error('标题修改失败')
    } finally {
      isRenaming.value = false
      syncDraft()
    }
  } else if (field === 'category') {
    const nextCategory = yamlCategory.value.trim()
    if (nextCategory === (current.category?.trim() ?? '')) return
    isRenaming.value = true
    try {
      await moveCard(uuid, nextCategory || '未分类')
      await treeStore.loadTree()
      savedField.value = 'category'
      setTimeout(() => { savedField.value = null }, 300)
    } catch (e) {
      console.error('[FrontmatterModal] category blur save failed:', e)
      toast.error('分类修改失败')
    } finally {
      isRenaming.value = false
      syncDraft()
    }
  }
}

async function applyYamlDraft() {
  const uuid = store.currentUuid
  const current = store.currentMeta
  if (!uuid || !current) return

  const parsed = parseYaml(yamlDraft.value)
  const nextTitle = parsed.title?.trim() || current.title
  const nextCategory = parsed.category?.trim() || current.category

  isRenaming.value = true
  try {
    if (nextTitle !== current.title) {
      const newMeta = await renameCard(uuid, nextTitle)
      store.currentMeta = newMeta
    }
    if (nextCategory !== current.category) {
      await moveCard(uuid, nextCategory)
    }
    await treeStore.loadTree()
    toast.success('YAML 已应用')
  } catch (e) {
    console.error('[FrontmatterModal] yaml apply failed:', e)
    toast.error('YAML 应用失败')
  } finally {
    isRenaming.value = false
    syncDraft()
  }
}

function handleClose() {
  emit('close')
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.stopPropagation()
    handleClose()
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="layout.activeOverlay === 'frontmatter'" class="fm-backdrop" @click.self="handleClose" @keydown="handleKeydown">
      <div class="fm-panel">
        <div class="fm-head">
          <span class="fm-title">METADATA INSPECTOR</span>
          <div class="fm-head-right">
            <button type="button" class="fm-raw-toggle" :class="{ active: rawMode }" @click="rawMode = !rawMode">RAW</button>
            <kbd class="fm-esc-hint">ESC</kbd>
          </div>
        </div>
        <div v-if="rawMode" class="fm-body">
          <textarea
            v-model="yamlDraft"
            class="fm-raw-editor"
            @keydown.ctrl.s.prevent="applyYamlDraft"
            @keydown.escape.stop="handleClose"
          />
        </div>
        <div v-else class="fm-body">
          <div class="fm-grid">
            <div class="fm-field">
              <label class="fm-label">TITLE</label>
              <input v-model="yamlTitle" type="text" class="fm-input" :class="{ 'fm-saved': savedField === 'title' }" :disabled="isRenaming" @blur="handleFieldBlur('title')" />
            </div>
            <div class="fm-field">
              <label class="fm-label">CATEGORY</label>
              <input v-model="yamlCategory" type="text" class="fm-input" :class="{ 'fm-saved': savedField === 'category' }" :disabled="isRenaming" @blur="handleFieldBlur('category')" />
            </div>
            <div class="fm-field">
              <label class="fm-label">UUID</label>
              <span class="fm-static">{{ store.currentMeta?.uuid }}</span>
            </div>
            <div class="fm-field">
              <label class="fm-label">CREATED</label>
              <span class="fm-static">{{ store.currentMeta?.created_at }}</span>
            </div>
            <div class="fm-field">
              <label class="fm-label">UPDATED</label>
              <span class="fm-static">{{ store.currentMeta?.updated_at }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.fm-backdrop {
  position: fixed;
  inset: 0;
  background: oklch(0 0 0 / 0.6);
  backdrop-filter: blur(4px);
  z-index: var(--z-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fm-fade-in 200ms cubic-bezier(0.25, 1, 0.5, 1);
}

.fm-panel {
  width: 500px;
  max-height: 80vh;
  background: var(--ms-panel);
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  box-shadow: 0 20px 40px oklch(0 0 0 / 0.8), 0 0 0 1px var(--ms-border-light);
  display: flex;
  flex-direction: column;
  animation: fm-scale-in 200ms cubic-bezier(0.25, 1, 0.5, 1);
}

.fm-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid var(--ms-border);
  flex-shrink: 0;
}

.fm-title {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.12em;
  color: var(--text-muted);
}

.fm-head-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.fm-raw-toggle {
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.1em;
  padding: 2px 8px;
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: border-color 150ms cubic-bezier(0.25, 1, 0.5, 1),
    color 150ms cubic-bezier(0.25, 1, 0.5, 1),
    background 150ms cubic-bezier(0.25, 1, 0.5, 1);
}

.fm-raw-toggle:hover {
  border-color: var(--neon);
  color: var(--neon);
}

.fm-raw-toggle.active {
  background: color-mix(in oklch, var(--neon) 12%, transparent);
  border-color: var(--neon);
  color: var(--neon);
}

.fm-esc-hint {
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.08em;
  padding: 2px 6px;
  border: 1px solid var(--ms-border);
  border-radius: 2px;
  color: var(--ms-smoke);
  opacity: 0.5;
  pointer-events: none;
}

.fm-body {
  flex: 1;
  padding: 18px;
  overflow-y: auto;
}

.fm-grid {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.fm-field {
  display: grid;
  grid-template-columns: 90px 1fr;
  align-items: baseline;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid var(--ms-border);
}

.fm-label {
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.1em;
  color: var(--text-muted);
  opacity: 0.6;
}

.fm-input {
  width: 100%;
  padding: 2px 0;
  background: transparent;
  border: none;
  border-bottom: 1px solid transparent;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 12px;
  outline: none;
  box-shadow: none;
  transition: border-color 150ms cubic-bezier(0.25, 1, 0.5, 1);
}

.fm-input:hover {
  border-bottom-color: var(--ms-border);
}

.fm-input:focus {
  border-bottom-color: var(--neon);
}

.fm-input:disabled {
  color: var(--text-muted);
  opacity: 0.6;
  cursor: default;
}

.fm-saved {
  border-color: oklch(0.6 0.15 145);
  transition: border-color 300ms;
}

.fm-static {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  opacity: 0.5;
  word-break: break-all;
}

.fm-raw-editor {
  width: 100%;
  min-height: 200px;
  padding: 0;
  background: transparent;
  border: none;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.6;
  resize: none;
  outline: none;
}

@keyframes fm-fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes fm-scale-in {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}
</style>
