<script setup lang="ts">
// 用途：窗口控制按钮，提供最小化、最大化和关闭功能
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

const appWindow = getCurrentWindow()
const maximized = ref(false)

async function minimize() {
  await appWindow.minimize()
}

async function toggleMaximize() {
  await appWindow.toggleMaximize()
}

async function close() {
  await appWindow.close()
}

let unlisten: (() => void) | null = null

onMounted(async () => {
  maximized.value = await appWindow.isMaximized()
  unlisten = await appWindow.onResized(async () => {
    maximized.value = await appWindow.isMaximized()
  })
})

onUnmounted(() => {
  unlisten?.()
})
</script>

<template>
  <div class="window-controls">
    <button class="ctrl-btn" title="最小化" @click="minimize">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
        <line x1="3" y1="6" x2="9" y2="6" stroke="currentColor" stroke-width="1" />
      </svg>
    </button>
    <button class="ctrl-btn" title="最大化" @click="toggleMaximize">
      <svg v-if="!maximized" width="12" height="12" viewBox="0 0 12 12" fill="none">
        <rect x="3" y="3" width="6" height="6" stroke="currentColor" stroke-width="1" rx="1" />
      </svg>
      <svg v-else width="12" height="12" viewBox="0 0 12 12" fill="none">
        <rect x="4" y="1.5" width="5.5" height="5.5" stroke="currentColor" stroke-width="1" rx="1" />
        <rect x="2.5" y="4" width="5.5" height="5.5" fill="var(--ms-void)" stroke="currentColor" stroke-width="1" rx="1" />
      </svg>
    </button>
    <button class="ctrl-btn ctrl-close" title="关闭" @click="close">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
        <line x1="3" y1="3" x2="9" y2="9" stroke="currentColor" stroke-width="1" />
        <line x1="9" y1="3" x2="3" y2="9" stroke="currentColor" stroke-width="1" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.window-controls {
  display: flex;
  height: 100%;
  -webkit-app-region: no-drag;
}

.ctrl-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease;
}

.ctrl-btn:hover {
  background: var(--ms-surface);
  color: var(--text-secondary);
}

.ctrl-close:hover {
  background: var(--destructive);
  color: white;
}
</style>
