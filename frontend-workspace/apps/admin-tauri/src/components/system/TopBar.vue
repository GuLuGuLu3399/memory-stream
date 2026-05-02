// 用途：顶部工具栏，提供菜单、搜索、同步和设置入口
<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { Menu, Search, X, Globe, RefreshCw, Settings } from 'lucide-vue-next'
import { useSyncStore } from '@/stores/sync'
import { useTreeStore } from '@/stores/tree'
import WindowControls from './WindowControls.vue'
import SyncConflictModal from './SyncConflictModal.vue'

defineProps<{
  starmapActive?: boolean
}>()

const emit = defineEmits<{
  toggleSidebar: []
  openPalette: []
  openStarmap: []
  openSettings: []
}>()

const syncStore = useSyncStore()
const treeStore = useTreeStore()
const hasActiveCard = computed(() => !!treeStore.activeCardUuid)

const isOnline = ref(typeof navigator !== 'undefined' ? navigator.onLine : true)
const conflictModalOpen = ref(false)

function handleOnline() { isOnline.value = true }
function handleOffline() { isOnline.value = false }

onMounted(async () => {
  await syncStore.init()
  window.addEventListener('online', handleOnline)
  window.addEventListener('offline', handleOffline)
})

onUnmounted(() => {
  syncStore.destroy()
  window.removeEventListener('online', handleOnline)
  window.removeEventListener('offline', handleOffline)
})
</script>

<template>
  <header class="top-bar">
    <div class="top-bar-left">
      <button class="top-bar-btn" aria-label="Toggle sidebar" @click="emit('toggleSidebar')">
        <Menu :size="15" :stroke-width="1.5" />
      </button>
      <button
        v-if="hasActiveCard"
        class="top-bar-btn close-card-btn"
        aria-label="Close card"
        title="关闭当前卡片"
        @click="treeStore.setActive(null)"
      >
        <X :size="14" :stroke-width="1.5" />
      </button>
      <button class="top-bar-btn" aria-label="Command palette" @click="emit('openPalette')">
        <Search :size="14" :stroke-width="1.5" />
      </button>
    </div>

    <span class="top-bar-title">Memory Stream</span>

    <div class="top-bar-right">
      <button
        class="top-bar-btn"
        aria-label="Settings"
        title="设置 (Ctrl+,)"
        @click="emit('openSettings')"
      >
        <Settings :size="14" :stroke-width="1.5" />
      </button>
      <button
        class="top-bar-btn starmap-btn"
        :class="{ active: starmapActive }"
        aria-label="全局星图"
        @click="emit('openStarmap')"
      >
        <Globe :size="14" :stroke-width="1.5" />
      </button>
      <button
        class="top-bar-btn sync-btn"
        :class="syncStore.status"
        :disabled="syncStore.isBusy"
        title="同步"
        @click="syncStore.syncNow()"
      >
        <RefreshCw :size="14" :stroke-width="1.5" :class="{ spinning: syncStore.isBusy }" />
        <span
          v-if="syncStore.conflictedUuids.length > 0"
          class="conflict-badge"
          @click.stop="conflictModalOpen = true"
        >
          {{ syncStore.conflictedUuids.length }}
        </span>
      </button>
      <WindowControls />
    </div>

    <SyncConflictModal @close="conflictModalOpen = false" />
  </header>
</template>

<style scoped>
.top-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 38px;
  padding: 0;
  background: var(--ms-void);
  border-bottom: 1px solid var(--ms-border);
  z-index: var(--z-chrome);
  flex-shrink: 0;
  user-select: none;
  -webkit-app-region: drag;
}

.top-bar-left,
.top-bar-right {
  display: flex;
  align-items: center;
  gap: 2px;
  -webkit-app-region: no-drag;
}

.top-bar-left {
  padding-left: 10px;
}

.top-bar-title {
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 500;
  letter-spacing: 0.06em;
  color: var(--text-muted);
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  pointer-events: none;
}

.top-bar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background var(--duration-fast) var(--ease-out),
    color var(--duration-fast) var(--ease-out);
}

.top-bar-btn:hover {
  background: var(--ms-surface);
  color: var(--text-primary);
}

.close-card-btn:hover {
  color: var(--destructive);
}

.starmap-btn.active {
  color: var(--neon);
  background: var(--neon-dim);
}

.sync-btn {
  position: relative;
  color: var(--text-muted);
}

.sync-btn:hover {
  color: var(--text-primary);
}

.sync-btn.syncing {
  color: var(--brass);
}

.sync-btn.error {
  color: var(--destructive);
}

.sync-btn.conflict {
  color: var(--brass);
}

.spinning {
  animation: spin 1s linear infinite;
}

.conflict-badge {
  position: absolute;
  top: 2px;
  right: 2px;
  min-width: 14px;
  height: 14px;
  padding: 0 3px;
  border-radius: 7px;
  background: var(--destructive);
  color: white;
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 600;
  line-height: 14px;
  text-align: center;
  cursor: pointer;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
