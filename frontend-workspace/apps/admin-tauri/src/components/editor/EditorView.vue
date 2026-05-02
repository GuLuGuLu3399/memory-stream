// 用途：编辑器主视图，包含编辑/预览模式切换、同步冲突处理和图谱面板
<script setup lang="ts">
import { ref, watch, provide, shallowRef, computed, onMounted, onUnmounted } from 'vue'
import { Network } from 'lucide-vue-next'
import { useEditorStore } from '@/stores/editor'
import { useTreeStore } from '@/stores/tree'
import { useLayoutStore } from '@/stores/layout'
import { useSyncStore } from '@/stores/sync'
import { useGlossaryStore } from '@/stores/glossary'
import { useToast } from '@/composables/core/useToast'
import { AstRenderer, TermPopup } from '@memory-stream/ui-shared'
import { materializeGhost, moveCard, parseMarkdown, readCardFile, renameCard, getBacklinks, getTrunkNavigation, resolveConflictKeepLocal, resolveConflictKeepRemote, syncPush } from '@/bridge/invoke'
import MsEditor from './MsEditor.vue'
import FloatingToolbar from './FloatingToolbar.vue'
import EditorEmpty from './EditorEmpty.vue'
import ConflictBanner from './ConflictBanner.vue'
import DisambigDialog from '@/components/base/DisambigDialog.vue'
import type { TitleMatch } from '@/stores/tree'
import type { BacklinkItem } from '@memory-stream/types'
import type { NavNode } from '@/bridge/invoke'
import { EditorViewKey } from './cm-injection'
import { formatWithPrettier } from './cm-format'
import type { LintItem } from './cm-linter'

type ViewMode = 'edit' | 'split' | 'preview'

const store = useEditorStore()
const treeStore = useTreeStore()
const syncStore = useSyncStore()
const glossaryStore = useGlossaryStore()
glossaryStore.load()

const termPopup = ref<{ visible: boolean; term: string; definition: string; x: number; y: number }>({
  visible: false, term: '', definition: '', x: 0, y: 0,
})
const toast = useToast()

const zenClass = computed(() => layout.zenMode ? 'zen-mode' : '')

const isConflicted = computed(() =>
    syncStore.conflictedUuids.includes(store.currentUuid ?? '')
)
const isResolving = ref(false)

async function keepLocal() {
    if (!store.currentUuid || isResolving.value) return
    isResolving.value = true
    try {
        await resolveConflictKeepLocal(store.currentUuid)
        syncStore.conflictedUuids = syncStore.conflictedUuids.filter(id => id !== store.currentUuid)
        toast.success('已保留本地版本')
        syncPush().catch(e => console.error('[EditorView] background push failed:', e))
    } catch (e) {
        toast.error('解决冲突失败')
        console.error('[EditorView] keep local failed:', e)
    } finally {
        isResolving.value = false
    }
}

async function keepRemote() {
    if (!store.currentUuid || isResolving.value) return
    isResolving.value = true
    try {
        await resolveConflictKeepRemote(store.currentUuid)
        syncStore.conflictedUuids = syncStore.conflictedUuids.filter(id => id !== store.currentUuid)
        await loadSelectedCard(store.currentUuid)
        toast.success('已载入云端版本')
        syncPush().catch(e => console.error('[EditorView] background push failed:', e))
    } catch (e) {
        toast.error('解决冲突失败')
        console.error('[EditorView] keep remote failed:', e)
    } finally {
        isResolving.value = false
    }
}

async function handleManualSave(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault()
        const ok = await store.manualSave()
        if (ok) toast.show('已保存到本地', 'success', 2000)
    }
}

onMounted(() => {
  window.addEventListener('keydown', handleManualSave)
  window.addEventListener('click', closeBacklinkPopover)
})
onUnmounted(() => {
  window.removeEventListener('keydown', handleManualSave)
  window.removeEventListener('click', closeBacklinkPopover)
})

function closeBacklinkPopover() {
  if (backlinkPopoverOpen.value) backlinkPopoverOpen.value = false
  if (trunkPopoverOpen.value) trunkPopoverOpen.value = false
}
const layout = useLayoutStore()

const categoryDropdownOpen = ref(false)
const categoryList = computed(() => {
  const cats = treeStore.categories
    .filter((c) => c.children.length > 0)
    .map((c) => c.name)
  return ['未分类', ...cats]
})

let loadRequestId = 0
const titleDraft = ref('')
const isRenaming = ref(false)
const viewMode = ref<ViewMode>('split')
const backlinks = ref<BacklinkItem[]>([])
const trunkParents = ref<NavNode[]>([])
const trunkChildren = ref<NavNode[]>([])
const trunkPopoverOpen = ref(false)
const trunkTotal = computed(() => trunkParents.value.length + trunkChildren.value.length)

defineEmits<{ openPalette: [] }>()

const cursorLine = ref(1)
const cursorCol = ref(1)

provide('editorState', { cursorLine, cursorCol })

// Format state
const isFormatting = ref(false)

async function handleFormat() {
  const v = cmView.value
  if (!v || isFormatting.value) return
  isFormatting.value = true
  const changed = await formatWithPrettier(v)
  isFormatting.value = false
  if (changed) toast.success('格式化完成')
  else toast.success('内容已是最佳格式')
}

// Lint health state
const lintItems = ref<LintItem[]>([])
const lintWarnings = computed(() => lintItems.value.filter((i) => i.severity === 'warning').length)
const lintErrors = computed(() => lintItems.value.filter((i) => i.severity === 'error').length)
const healthClass = computed(() => {
  if (lintErrors.value > 0) return 'health-error'
  if (lintWarnings.value > 0) return 'health-warning'
  return 'health-ok'
})
const healthPopoverOpen = ref(false)
const backlinkPopoverOpen = ref(false)

function handleLintChange(items: LintItem[]) {
  lintItems.value = items
}

const msEditorRef = ref<InstanceType<typeof MsEditor> | null>(null)
const cmView = shallowRef<import('@codemirror/view').EditorView | null>(null)
provide(EditorViewKey, cmView)

watch(() => msEditorRef.value?.view, (v) => {
  if (v) cmView.value = v
})

watch(
    () => store.currentMeta?.title,
    (title) => {
        if (!isRenaming.value) {
            titleDraft.value = title ?? ''
        }
    },
    { immediate: true },
)

async function loadSelectedCard(uuid: string) {
    const requestId = ++loadRequestId

    try {
        const rawText = await readCardFile(uuid)
        if (requestId !== loadRequestId) return

        const parsed = await parseMarkdown(rawText)
        if (requestId !== loadRequestId) return

        store.loadArticle(uuid, parsed)

        getBacklinks(uuid).then(bl => {
            if (requestId === loadRequestId) backlinks.value = bl
        }).catch(() => {})

        getTrunkNavigation(uuid).then(nav => {
            if (requestId === loadRequestId) {
                trunkParents.value = nav.parents
                trunkChildren.value = nav.children
            }
        }).catch(() => {})
    } catch (e) {
        if (requestId !== loadRequestId) return
        console.error('[EditorView] failed to load card:', e)
        store.clear()
    }
}

async function commitTitleChange() {
    const uuid = store.currentUuid
    if (!uuid) return

    const nextTitle = titleDraft.value.trim()
    const currentTitle = store.currentMeta?.title?.trim() ?? ''
    if (!nextTitle || nextTitle === currentTitle) {
        titleDraft.value = currentTitle
        return
    }

    isRenaming.value = true
    try {
        const newMeta = await renameCard(uuid, nextTitle)
        store.currentMeta = newMeta
        await treeStore.loadTree()
        toast.success('标题已更新')
    } catch (e) {
        console.error('[EditorView] rename failed:', e)
        titleDraft.value = currentTitle
        toast.error('标题修改失败')
    } finally {
        isRenaming.value = false
    }
}

function handleTitleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
        e.preventDefault()
            ; (e.target as HTMLInputElement).blur()
    }

    if (e.key === 'Escape') {
        titleDraft.value = store.currentMeta?.title ?? ''
            ; (e.target as HTMLInputElement).blur()
    }
}

watch(
    () => treeStore.activeCardUuid,
    (uuid) => {
        if (!uuid) {
            loadRequestId += 1
            store.clear()
            backlinks.value = []
            trunkParents.value = []
            trunkChildren.value = []
            return
        }

        void loadSelectedCard(uuid)
    },
    { immediate: true },
)

const disambigOpen = ref(false)
const disambigTarget = ref('')
const disambigOptions = ref<TitleMatch[]>([])

async function loadCard(uuid: string) {
    const rawText = await readCardFile(uuid)
    const parsed = await parseMarkdown(rawText)
    treeStore.setActive(uuid)
    store.loadArticle(uuid, parsed)
}

async function handleWikiNavigation(target: string) {
    try {
        const matches = treeStore.lookupByTitle(target)

        if (matches.length === 0) {
            const card = await materializeGhost(target)
            await loadCard(card.uuid)
        } else if (matches.length === 1) {
            await loadCard(matches[0].uuid)
        } else {
            disambigTarget.value = target
            disambigOptions.value = matches
            disambigOpen.value = true
        }
    } catch (e) {
        console.error('[WikiNav] failed:', e)
    }
}

async function confirmDisambig(uuid: string) {
    disambigOpen.value = false
    await loadCard(uuid)
}

function handleHeadingStatusToggle(payload: { level: number; headingText: string; currentStatus: string }) {
    const next = toggleHeadingStatus(store.rawContent, payload.level, payload.headingText, payload.currentStatus)
    if (next !== store.rawContent) store.rawContent = next
}

function toggleHeadingStatus(raw: string, level: number, text: string, status: string): string {
    const cycle: Record<string, string> = { none: '[x]', Done: '[ ]', Undone: '[~]', Unclear: '' }
    const marker = cycle[status] ?? ''
    const hashes = '#'.repeat(level)
    const escaped = text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
    const patterns: Record<string, string> = {
        none: `^(${hashes})\\s+${escaped}`,
        Done: `^(${hashes})\\s+\\[x\\]\\s*${escaped}`,
        Undone: `^(${hashes})\\s+\\[ \\]\\s*${escaped}`,
        Unclear: `^(${hashes})\\s+\\[~\\]\\s*${escaped}`,
    }
    const regex = new RegExp(patterns[status] ?? '', 'm')
    if (!regex.test(raw)) return raw
    const replacement = marker ? `${hashes} ${marker} ${text}` : `${hashes} ${text}`
    return raw.replace(regex, replacement)
}

function handleConceptRefHover(payload: { term: string; x: number; y: number }) {
    const def = glossaryStore.lookup(payload.term)
    if (def) {
        termPopup.value = { visible: true, term: payload.term, definition: def, x: payload.x, y: payload.y }
    }
}

function handleConceptRefLeave() {
    termPopup.value.visible = false
}

function handleCursorChange(line: number, col: number) {
    cursorLine.value = line
    cursorCol.value = col
}

async function selectCategory(category: string) {
    categoryDropdownOpen.value = false
    const uuid = store.currentUuid
    if (!uuid) return
    const currentCategory = store.currentMeta?.category ?? '未分类'
    if (category === currentCategory) return
    try {
        await moveCard(uuid, category === '未分类' ? '' : category)
        await treeStore.loadTree()
        toast.success('分类已更新')
    } catch {
        toast.error('分类转移失败')
    }
}
</script>

<template>
    <div v-if="store.currentUuid" class="editor-arena" :class="zenClass">
        <div class="editor-meta-bar">
            <div class="editor-meta-left">
                <div class="meta-category-wrapper">
                    <button class="meta-category-tag" @click="categoryDropdownOpen = !categoryDropdownOpen">
                        {{ store.currentMeta?.category || '未分类' }}
                        <svg width="8" height="5" viewBox="0 0 8 5"><path d="M1 1L4 4L7 1" stroke="currentColor" stroke-width="1.2" fill="none"/></svg>
                    </button>
                    <div v-if="categoryDropdownOpen" class="meta-category-dropdown">
                        <button v-for="cat in categoryList" :key="cat" class="category-option" :class="{ active: cat === (store.currentMeta?.category || '未分类') }" @click="selectCategory(cat)">
                            {{ cat }}
                        </button>
                    </div>
                </div>
                <input v-model="titleDraft" class="meta-title-input" :disabled="isRenaming"
                    :placeholder="store.currentMeta?.title || '未命名卡片'" @blur="commitTitleChange"
                    @keydown="handleTitleKeydown" />
                <div v-if="trunkTotal > 0" class="trunk-dropdown-wrapper">
                    <button class="trunk-dropdown-trigger" :class="{ active: trunkPopoverOpen }"
                        @click.stop="trunkPopoverOpen = !trunkPopoverOpen" :title="`Trunk 导航 (${trunkTotal})`">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M8 3l-5 9 5 9" /><path d="M16 3l5 9-5 9" />
                        </svg>
                        <span class="trunk-dropdown-count">{{ trunkTotal }}</span>
                    </button>
                    <div v-if="trunkPopoverOpen" class="trunk-dropdown-popover" @click.stop>
                        <template v-if="trunkParents.length > 0">
                            <div class="trunk-dropdown-section">上一级</div>
                            <button v-for="p in trunkParents" :key="p.uuid" class="trunk-dropdown-item"
                                @click="trunkPopoverOpen = false; treeStore.setActive(p.uuid)">
                                <span class="trunk-dropdown-arrow">&larr;</span>
                                <span class="trunk-dropdown-label">{{ p.title }}</span>
                            </button>
                        </template>
                        <template v-if="trunkChildren.length > 0">
                            <div class="trunk-dropdown-section" :class="{ 'trunk-dropdown-divider': trunkParents.length > 0 }">下一级</div>
                            <button v-for="c in trunkChildren" :key="c.uuid" class="trunk-dropdown-item"
                                @click="trunkPopoverOpen = false; treeStore.setActive(c.uuid)">
                                <span class="trunk-dropdown-label">{{ c.title }}</span>
                                <span class="trunk-dropdown-arrow">&rarr;</span>
                            </button>
                        </template>
                    </div>
                </div>
            </div>
            <div class="editor-meta-right">
                <div class="editor-view-modes">
                    <button class="mode-btn" :class="{ active: viewMode === 'edit' }" title="Edit" @click="viewMode = 'edit'">
                        <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M5.5 3L1.5 8L5.5 13" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/><path d="M10.5 3L14.5 8L10.5 13" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/></svg>
                    </button>
                    <button class="mode-btn" :class="{ active: viewMode === 'split' }" title="Split" @click="viewMode = 'split'">
                        <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><rect x="1.5" y="2.5" width="5" height="11" rx="0.5" stroke="currentColor" stroke-width="1.2"/><rect x="9.5" y="2.5" width="5" height="11" rx="0.5" stroke="currentColor" stroke-width="1.2"/></svg>
                    </button>
                    <button class="mode-btn" :class="{ active: viewMode === 'preview' }" title="Preview" @click="viewMode = 'preview'">
                        <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M1.5 8C1.5 8 4 3.5 8 3.5C12 3.5 14.5 8 14.5 8C14.5 8 12 12.5 8 12.5C4 12.5 1.5 8 1.5 8Z" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/><circle cx="8" cy="8" r="2" stroke="currentColor" stroke-width="1.2"/></svg>
                    </button>
                </div>
                <button type="button" class="meta-action-btn" title="一键优化" :disabled="isFormatting" @click="handleFormat">
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M8 1L10 6L15 6.5L11.5 10L12.5 15L8 12.5L3.5 15L4.5 10L1 6.5L6 6L8 1Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/></svg>
                </button>
                <button type="button" class="meta-action-btn" :class="{ active: layout.activeOverlay === 'frontmatter' }" title="Frontmatter (Ctrl+I)" @click="layout.openOverlay('frontmatter')">
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                        <rect x="2" y="2" width="12" height="2" rx="0.5" stroke="currentColor" stroke-width="1.2"/>
                        <rect x="2" y="7" width="8" height="2" rx="0.5" stroke="currentColor" stroke-width="1.2"/>
                        <rect x="2" y="12" width="10" height="2" rx="0.5" stroke="currentColor" stroke-width="1.2"/>
                    </svg>
                </button>
                <button type="button" class="mode-btn" :class="{ active: layout.rightPanel === 'graph' }" title="局部雷达" @click="layout.toggleRightPanel('graph')">
                    <Network :size="14" :stroke-width="1.5" />
                </button>
                <div v-if="backlinks.length > 0" class="backlink-badge"
                    @click.stop="backlinkPopoverOpen = !backlinkPopoverOpen">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
                        <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
                    </svg>
                    <span>{{ backlinks.length }}</span>
                </div>
                <div v-if="backlinkPopoverOpen && backlinks.length > 0" class="backlink-popover">
                    <div class="backlink-popover-title">BACKLINKS</div>
                    <button v-for="bl in backlinks" :key="bl.uuid" class="backlink-popover-item"
                        @click="backlinkPopoverOpen = false; treeStore.setActive(bl.uuid)">
                        <span class="backlink-relation">{{ bl.relation_type === 'trunk' ? 'T' : 'L' }}</span>
                        <span class="backlink-title">{{ bl.title }}</span>
                    </button>
                </div>
                <div class="health-indicator" :class="healthClass" @mouseenter="healthPopoverOpen = true" @mouseleave="healthPopoverOpen = false">
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M8 1L14 4V8C14 11.3 11.3 14.5 8 15C4.7 14.5 2 11.3 2 8V4L8 1Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/></svg>
                    <span v-if="lintWarnings + lintErrors > 0" class="health-badge">{{ lintWarnings + lintErrors }}</span>
                </div>
                <div v-if="healthPopoverOpen && lintItems.length > 0" class="health-popover">
                    <div v-for="(item, idx) in lintItems" :key="idx" class="health-item" :class="item.severity">
                        <span class="health-line">L{{ item.line }}</span>
                        <span class="health-msg">{{ item.message }}</span>
                    </div>
                </div>
            </div>
        </div>
        <ConflictBanner v-if="isConflicted" :disabled="isResolving" @keep-local="keepLocal" @keep-remote="keepRemote" />
        <div class="editor-workspace">
            <div v-if="viewMode !== 'preview'" class="editor-pane input-pane">
                <MsEditor
                    ref="msEditorRef"
                    :model-value="store.rawContent"
                    @update:model-value="store.rawContent = $event"
                    @cursor-change="handleCursorChange"
                    @lint-change="handleLintChange"
                />
            </div>

            <div v-if="viewMode === 'split'" class="editor-divider" />

            <div v-if="viewMode !== 'edit'" class="editor-pane preview-pane">
                <div class="preview-content">
                    <AstRenderer v-if="store.currentAst" :node="store.currentAst"
                        @navigate="handleWikiNavigation"
                        @toggle-heading-status="handleHeadingStatusToggle"
                        @concept-ref-hover="handleConceptRefHover"
                        @concept-ref-leave="handleConceptRefLeave" />
                    <div v-else class="preview-empty">等待引擎解析...</div>
                    <TermPopup v-if="termPopup.visible" :term="termPopup.term" :definition="termPopup.definition"
                               :x="termPopup.x" :y="termPopup.y" @close="termPopup.visible = false" />
                </div>
            </div>

            <FloatingToolbar />
        </div>

    </div>

    <EditorEmpty v-else @open-palette="$emit('openPalette')" />

    <DisambigDialog
        v-if="disambigOpen"
        :title="disambigTarget"
        :options="disambigOptions"
        @select="confirmDisambig"
        @cancel="disambigOpen = false"
    />
</template>

<style scoped>
.editor-arena {
    display: flex;
    flex: 1;
    flex-direction: column;
    overflow: hidden;
}

.editor-arena.zen-mode > .editor-meta-bar {
    max-width: 70ch;
    margin: 0 auto;
}

.editor-arena.zen-mode > .editor-workspace > .editor-pane {
    max-width: 70ch;
    margin: 0 auto;
}

.editor-meta-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    border-bottom: 1px solid var(--ms-border);
    background: var(--ms-void);
    flex-shrink: 0;
}

.trunk-dropdown-wrapper {
    position: relative;
    flex-shrink: 0;
}

.trunk-dropdown-trigger {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 2px 6px;
    background: transparent;
    border: 1px solid var(--ms-border);
    border-radius: 2px;
    color: var(--text-muted);
    cursor: pointer;
    font-family: var(--font-mono);
    font-size: 10px;
    transition: color 150ms, border-color 150ms;
}

.trunk-dropdown-trigger:hover,
.trunk-dropdown-trigger.active {
    color: var(--neon);
    border-color: var(--neon);
}

.trunk-dropdown-count {
    line-height: 1;
}

.trunk-dropdown-popover {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    min-width: 200px;
    max-width: 320px;
    max-height: 280px;
    overflow-y: auto;
    background: var(--ms-void);
    border: 1px solid var(--ms-border-light);
    border-radius: 2px;
    box-shadow: 0 8px 24px oklch(0 0 0 / 0.6);
    z-index: var(--z-popover, 50);
    padding: 4px 0;
}

.trunk-dropdown-section {
    padding: 4px 10px 2px;
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-muted);
}

.trunk-dropdown-divider {
    margin-top: 4px;
    border-top: 1px solid var(--ms-border);
    padding-top: 6px;
}

.trunk-dropdown-item {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 4px 10px;
    background: transparent;
    border: none;
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
    text-align: left;
    transition: color 100ms, background 100ms;
}

.trunk-dropdown-item:hover {
    color: var(--neon);
    background: var(--ms-surface);
}

.trunk-dropdown-arrow {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--neon);
    opacity: 0.5;
    flex-shrink: 0;
}

.trunk-dropdown-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.editor-meta-left {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    min-width: 0;
}

.meta-category-tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    background: var(--ms-surface);
    border: none;
    border-radius: 2px;
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    white-space: nowrap;
    flex-shrink: 0;
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-hydraulic),
        color var(--duration-fast) var(--ease-hydraulic);
}

.meta-category-tag:hover {
    background: var(--ms-carbon);
    color: var(--text-secondary);
}

.meta-category-tag svg {
    opacity: 0.5;
}

.meta-category-wrapper {
    position: relative;
}

.meta-category-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 4px;
    min-width: 120px;
    max-height: 200px;
    overflow-y: auto;
    background: var(--ms-void);
    border: 1px solid var(--ms-border);
    border-radius: 2px;
    box-shadow: 0 4px 16px oklch(0 0 0 / 0.4);
    z-index: var(--z-float);
}

.category-option {
    display: block;
    width: 100%;
    padding: 6px 10px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 11px;
    text-align: left;
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-hydraulic),
        color var(--duration-fast) var(--ease-hydraulic);
}

.category-option:hover {
    background: var(--ms-surface);
    color: var(--neon);
}

.category-option.active {
    color: var(--neon);
}

.meta-title-input {
    flex: 1;
    min-width: 0;
    max-width: 60%;
    background: transparent;
    border: none;
    border-bottom: 1px solid transparent;
    outline: none;
    padding: 0 0 1px;
    font-family: var(--font-sans);
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    caret-color: var(--neon);
    transition: border-color var(--duration-fast) var(--ease-hydraulic);
}

.meta-title-input::placeholder {
    color: var(--text-muted);
}

.meta-title-input:focus {
    border-bottom-color: var(--neon);
}

.editor-meta-right {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
    position: relative;
}

.meta-action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 24px;
    border: none;
    border-radius: 2px;
    background: transparent;
    color: var(--ms-smoke);
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-hydraulic),
        color var(--duration-fast) var(--ease-hydraulic);
}

.meta-action-btn:hover {
    background: var(--ms-surface);
    color: var(--neon);
}

.editor-view-modes {
    display: flex;
    gap: 1px;
}

.mode-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 24px;
    border: none;
    border-radius: 2px;
    background: transparent;
    color: var(--ms-smoke);
    cursor: pointer;
    position: relative;
    transition: background var(--duration-fast) var(--ease-hydraulic),
        color var(--duration-fast) var(--ease-snap),
        transform var(--duration-fast) var(--ease-snap);
}

.mode-btn:hover {
    background: var(--ms-surface);
    color: var(--text-secondary);
}

.mode-btn:active {
    transform: scale(0.92);
}

.mode-btn.active {
    color: var(--neon);
    box-shadow: 0 1px 0 var(--neon);
}

.mode-btn.active:hover {
    background: color-mix(in oklch, var(--neon) 8%, transparent);
}

/* Health indicator */
.health-indicator {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 24px;
    cursor: default;
    color: oklch(0.7 0.15 150);
    transition: color var(--duration-fast) var(--ease-hydraulic);
}

.health-indicator.health-warning {
    color: oklch(0.75 0.15 85);
}

.health-indicator.health-error {
    color: oklch(0.65 0.2 25);
}

.health-badge {
    position: absolute;
    top: 0;
    right: 0;
    min-width: 12px;
    height: 12px;
    padding: 0 3px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    background: currentColor;
    color: var(--ms-void);
    font-family: var(--font-mono);
    font-size: 8px;
    font-weight: 700;
    line-height: 1;
}

.health-popover {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    min-width: 240px;
    max-width: 320px;
    max-height: 200px;
    overflow-y: auto;
    background: var(--ms-void);
    border: 1px solid var(--ms-border);
    border-radius: 2px;
    box-shadow: 0 4px 16px oklch(0 0 0 / 0.4);
    z-index: var(--z-float);
    padding: 4px 0;
}

.health-item {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 6px 10px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
}

.health-item.warning .health-line {
    color: oklch(0.75 0.15 85);
}

.health-item.error .health-line {
    color: oklch(0.65 0.2 25);
}

.health-line {
    flex-shrink: 0;
    font-weight: 600;
    min-width: 24px;
}

.health-msg {
    flex: 1;
    min-width: 0;
    word-break: break-word;
}

.editor-workspace {
    display: flex;
    flex: 1;
    min-width: 0;
    overflow: hidden;
    position: relative;
}

.editor-pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.input-pane {
    background: var(--ms-carbon);
    color: var(--text-primary);
}

.input-pane :deep(.ms-editor) {
    height: 100%;
}

.preview-pane {
    background: var(--ms-deep);
    color: var(--text-primary);
}

.editor-divider {
    width: 1px;
    flex-shrink: 0;
    background: var(--ms-border);
}

.preview-content {
    flex: 1;
    padding: 16px 20px;
    overflow-y: auto;
    overflow-x: hidden;
    font-family: var(--font-sans);
    font-size: 15px;
    line-height: 1.75;
    -webkit-mask-image: linear-gradient(to bottom, transparent 0, black 14px, black calc(100% - 14px), transparent 100%);
    mask-image: linear-gradient(to bottom, transparent 0, black 14px, black calc(100% - 14px), transparent 100%);
}

.preview-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
    font-size: 13px;
}

/* ── Backlinks HUD ── */
.backlink-badge {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 6px;
    height: 24px;
    border: 1px solid var(--ms-border);
    border-radius: 2px;
    background: transparent;
    color: var(--brass);
    font-family: var(--font-mono);
    font-size: 11px;
    cursor: pointer;
    transition: border-color var(--duration-fast) var(--ease-hydraulic),
        color var(--duration-fast) var(--ease-hydraulic);
}

.backlink-badge:hover {
    border-color: var(--brass);
    color: var(--text-primary);
}

.backlink-popover {
    position: absolute;
    top: 100%;
    right: 0;
    min-width: 200px;
    max-width: 320px;
    padding: 8px;
    background: var(--ms-void);
    border: 1px solid var(--ms-border);
    border-radius: 2px;
    box-shadow: 0 4px 16px oklch(0 0 0 / 0.5);
    z-index: 100;
    animation: popover-in 150ms cubic-bezier(0.33, 0, 0.2, 1) both;
}

@keyframes popover-in {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
}

.backlink-popover-title {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.08em;
    color: var(--text-muted);
    margin-bottom: 6px;
}

.backlink-popover-item {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 4px 8px;
    border: none;
    border-radius: 2px;
    background: transparent;
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
    text-align: left;
    transition: background var(--duration-fast) var(--ease-hydraulic);
}

.backlink-popover-item:hover {
    background: var(--ms-carbon);
    color: var(--text-primary);
}

.backlink-relation {
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 700;
    color: var(--brass);
}

.backlink-title {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
}

</style>
