<script setup lang="ts">
import { computed, ref } from 'vue'
import { AlertTriangle, Loader2 } from 'lucide-vue-next'
import { useSyncStore } from '@/stores/sync'
import { useTreeStore } from '@/stores/tree'
import { useToast } from '@/composables/core/useToast'

const syncStore = useSyncStore()
const treeStore = useTreeStore()
const toast = useToast()

const emit = defineEmits<{ close: [] }>()

// Build uuid → { title, category } lookup from tree
const cardLookup = computed(() => {
  const map = new Map<string, { title: string; category: string }>()
  function walk(nodes: any[], parentName: string) {
    for (const node of nodes) {
      if (!node.is_dir) {
        map.set(node.id, { title: node.name, category: parentName })
      }
      if (node.children?.length) {
        walk(node.children, node.is_dir ? node.name : parentName)
      }
    }
  }
  walk(treeStore.categories, '')
  return map
})

const items = computed(() =>
  syncStore.conflictedUuids.map(uuid => ({
    uuid,
    title: cardLookup.value.get(uuid)?.title ?? uuid,
    category: cardLookup.value.get(uuid)?.category ?? '',
    short: uuid.slice(0, 8),
  })),
)

const resolving = ref(new Set<string>())

async function resolveOne(uuid: string, strategy: 'local' | 'remote') {
  resolving.value.add(uuid)
  try {
    await syncStore.resolveConflict(uuid, strategy)
    toast.success(strategy === 'local' ? '已保留本地版本' : '已载入云端版本')
    await treeStore.loadTree()
  } catch (e) {
    toast.error(`解决失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    resolving.value.delete(uuid)
  }
}

const batchBusy = ref(false)

async function resolveAll(strategy: 'local' | 'remote') {
  batchBusy.value = true
  const uuids = [...syncStore.conflictedUuids]
  const results = await Promise.allSettled(
    uuids.map(async (uuid) => {
      await syncStore.resolveConflict(uuid, strategy)
      resolving.value.add(uuid)
    }),
  )
  const failed = uuids.filter((_, i) => results[i].status === 'rejected')
  resolving.value.clear()
  if (failed.length === 0) {
    toast.success(strategy === 'local' ? '全部保留本地版本' : '全部载入云端版本')
  } else {
    toast.error(`${failed.length} 项解决失败`)
  }
  await treeStore.loadTree()
  batchBusy.value = false
}
</script>

<template>
  <Transition name="fade">
    <div v-if="syncStore.conflictedUuids.length" class="conflict-overlay" @click.self="emit('close')">
      <div class="conflict-modal" role="dialog" aria-modal="true" aria-label="冲突指挥中心">
        <header class="conflict-header">
          <div class="header-left">
            <AlertTriangle :size="16" class="header-icon" />
            <h3>冲突指挥中心</h3>
            <span class="conflict-count">{{ syncStore.conflictedUuids.length }}</span>
          </div>
          <button class="close-btn" @click="emit('close')">关闭</button>
        </header>

        <div class="batch-bar">
          <span class="batch-label">批量操作</span>
          <div class="batch-actions">
            <button class="batch-btn batch-local" :disabled="batchBusy" @click="resolveAll('local')">
              <template v-if="batchBusy"><Loader2 :size="12" class="spin" /></template>
              全部保留本地
            </button>
            <button class="batch-btn batch-remote" :disabled="batchBusy" @click="resolveAll('remote')">
              <template v-if="batchBusy"><Loader2 :size="12" class="spin" /></template>
              全部载入云端
            </button>
          </div>
        </div>

        <TransitionGroup name="list" tag="div" class="conflict-list">
          <div v-for="item in items" :key="item.uuid" class="conflict-row">
            <div class="row-info">
              <span class="row-title">{{ item.title }}</span>
              <span v-if="item.category" class="row-cat">{{ item.category }}</span>
              <span class="row-uuid">{{ item.short }}</span>
            </div>
            <div class="row-actions">
              <button
                class="resolve-btn btn-local"
                :disabled="resolving.has(item.uuid) || batchBusy"
                @click="resolveOne(item.uuid, 'local')"
              >
                <Loader2 v-if="resolving.has(item.uuid)" :size="12" class="spin" />
                保留本地
              </button>
              <button
                class="resolve-btn btn-remote"
                :disabled="resolving.has(item.uuid) || batchBusy"
                @click="resolveOne(item.uuid, 'remote')"
              >
                <Loader2 v-if="resolving.has(item.uuid)" :size="12" class="spin" />
                载入云端
              </button>
            </div>
          </div>
        </TransitionGroup>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 200ms ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.conflict-overlay {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal);
  background: color-mix(in oklch, var(--ms-void) 80%, transparent);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.conflict-modal {
  width: min(640px, 100%);
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--ms-border);
  border-radius: 8px;
  background: var(--ms-deep);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
}

.conflict-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--ms-border);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-icon {
  color: var(--destructive);
}

.conflict-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.03em;
}

.conflict-count {
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--destructive);
  color: white;
}

.close-btn {
  border: none;
  border-radius: 4px;
  background: var(--ms-surface);
  color: var(--text-secondary);
  height: 28px;
  padding: 0 12px;
  font-size: 12px;
  cursor: pointer;
  transition: background 120ms ease;
}

.close-btn:hover {
  background: var(--ms-surface-hover);
}

.batch-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-bottom: 1px solid var(--ms-border);
  background: var(--ms-surface);
}

.batch-label {
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.batch-actions {
  display: flex;
  gap: 6px;
}

.batch-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  border: none;
  border-radius: 4px;
  height: 28px;
  padding: 0 10px;
  font-size: 12px;
  cursor: pointer;
  transition: background 120ms ease, opacity 120ms ease;
}

.batch-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.batch-local {
  background: var(--neon-dim);
  color: var(--neon);
}

.batch-local:hover:not(:disabled) {
  background: color-mix(in oklch, var(--neon) 20%, transparent);
}

.batch-remote {
  background: var(--brass-dim);
  color: var(--brass);
}

.batch-remote:hover:not(:disabled) {
  background: color-mix(in oklch, var(--brass) 20%, transparent);
}

.conflict-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

/* TransitionGroup animations */
.list-enter-active {
  transition: all 200ms ease-out;
}
.list-leave-active {
  transition: all 200ms ease-in;
}
.list-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
.list-enter-from {
  opacity: 0;
  transform: translateX(-10px);
}
.list-move {
  transition: transform 200ms ease;
}

.conflict-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 12px;
  border-radius: 4px;
  background: var(--ms-surface);
  transition: background 120ms ease;
}

.conflict-row:hover {
  background: var(--ms-surface-hover);
}

.row-info {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex: 1;
}

.row-title {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row-cat {
  font-size: 10px;
  color: var(--text-muted);
  padding: 1px 5px;
  border: 1px solid var(--ms-border);
  border-radius: 3px;
  flex-shrink: 0;
}

.row-uuid {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-muted);
  flex-shrink: 0;
}

.row-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.resolve-btn {
  display: flex;
  align-items: center;
  gap: 3px;
  border: none;
  border-radius: 3px;
  height: 26px;
  padding: 0 8px;
  font-size: 11px;
  cursor: pointer;
  transition: background 120ms ease, opacity 120ms ease;
}

.resolve-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-local {
  background: var(--neon-dim);
  color: var(--neon);
}

.btn-local:hover:not(:disabled) {
  background: color-mix(in oklch, var(--neon) 18%, transparent);
}

.btn-remote {
  background: var(--brass-dim);
  color: var(--brass);
}

.btn-remote:hover:not(:disabled) {
  background: color-mix(in oklch, var(--brass) 18%, transparent);
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
