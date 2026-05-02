// 用途：底部状态栏，显示编辑器状态和同步进度
<script setup lang="ts">
import { computed, inject, ref, onMounted, onUnmounted, type Ref } from 'vue'
import { useEditorStore } from '@/stores/editor'
import { useSyncStore } from '@/stores/sync'

interface EditorState {
  cursorLine: Ref<number>
  cursorCol: Ref<number>
}

const editorState = inject<EditorState | null>('editorState', null)

const editorStore = useEditorStore()
const syncStore = useSyncStore()

const isOnline = ref(typeof navigator !== 'undefined' ? navigator.onLine : true)

const analysis = computed(() => editorStore.currentAnalysis)

const syncLabel = computed(() => {
  if (!isOnline.value) return 'OFFLINE'
  if (syncStore.status === 'syncing') return 'SYNCING'
  if (syncStore.status === 'error') return 'ERROR'
  if (syncStore.status === 'conflict') return 'CONFLICT'
  return 'READY'
})

const syncDotClass = computed(() => {
  if (!isOnline.value) return 'offline'
  if (syncStore.status === 'syncing') return 'syncing'
  if (syncStore.status === 'error' || syncStore.status === 'conflict') return 'error'
  return 'ready'
})

const saveLabel = computed(() => {
  if (editorStore.isSaving) return '保存中...'
  if (editorStore.lastSaveError) return '未保存'
  return '已保存'
})

function handleOnline() { isOnline.value = true }
function handleOffline() { isOnline.value = false }

onMounted(() => {
  window.addEventListener('online', handleOnline)
  window.addEventListener('offline', handleOffline)
})

onUnmounted(() => {
  window.removeEventListener('online', handleOnline)
  window.removeEventListener('offline', handleOffline)
})
</script>

<template>
  <footer class="status-bar">
    <div class="status-left">
      <span v-if="editorState" class="telemetry">
        <span class="status-key">LN</span>
        <span class="status-val">{{ editorState.cursorLine.value }}</span>
      </span>
      <span v-if="editorState" class="telemetry">
        <span class="status-key">COL</span>
        <span class="status-val">{{ editorState.cursorCol.value }}</span>
      </span>
      <span class="telemetry">
        <span class="status-val">{{ editorStore.realTimeStats.words }}</span>
        <span class="status-key">WORDS</span>
      </span>
      <span class="telemetry">
        <span class="status-val">{{ editorStore.realTimeStats.chars }}</span>
        <span class="status-key">CHARS</span>
      </span>
      <span class="telemetry">
        <span class="status-val">{{ editorStore.realTimeStats.lines }}</span>
        <span class="status-key">LINES</span>
      </span>
      <span v-if="analysis && analysis.toc.length > 0" class="telemetry">
        <span class="status-val">{{ analysis.toc.length }}</span>
        <span class="status-key">H</span>
      </span>
      <span v-if="analysis && analysis.outbound_links > 0" class="telemetry">
        <span class="status-val">{{ analysis.outbound_links }}</span>
        <span class="status-key">LINKS</span>
      </span>
    </div>

    <div class="status-right">
      <span class="save-label" :class="{ saving: editorStore.isSaving, error: editorStore.lastSaveError }">
        {{ saveLabel }}
      </span>
      <span class="sync-group">
        <span class="sync-label">{{ syncLabel }}</span>
        <span class="sync-dot" :class="syncDotClass" />
      </span>
    </div>
  </footer>
</template>

<style scoped>
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 22px;
  padding: 0 12px;
  background: var(--ms-void);
  border-top: 1px solid var(--ms-border);
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  flex-shrink: 0;
  user-select: none;
  font-variant-numeric: tabular-nums;
}

.status-left,
.status-right {
  display: flex;
  align-items: center;
  gap: 14px;
}

.telemetry {
  display: flex;
  align-items: center;
  gap: 3px;
  white-space: nowrap;
}

.status-key {
  font-size: 9px;
  letter-spacing: 0.1em;
  opacity: 0.45;
}

.status-val {
  color: var(--text-secondary);
  min-width: 2ch;
}

.save-label {
  white-space: nowrap;
  transition: color var(--duration-fast) var(--ease-out);
}

.save-label.saving {
  color: var(--brass);
}

.save-label.error {
  color: var(--destructive);
}

.sync-group {
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
}

.sync-label {
  font-size: 9px;
  letter-spacing: 0.1em;
  opacity: 0.45;
}

.sync-dot {
  display: inline-block;
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--ms-smoke);
  transition: background var(--duration-normal) var(--ease-out);
}

.sync-dot.syncing {
  background: var(--neon);
  animation: sync-pulse 2s ease-in-out infinite;
}

.sync-dot.error {
  background: var(--destructive);
}

.sync-dot.offline {
  background: var(--ms-border-light);
}

@keyframes sync-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
</style>
