// 用途：应用主布局，包含顶栏、侧边栏、编辑器和状态栏
<script setup lang="ts">
import { ref, onMounted, onUnmounted, provide } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { UnlistenFn } from '@tauri-apps/api/event'
import TopBar from './TopBar.vue'
import StatusBar from './StatusBar.vue'
import ResizeHandle from '../base/ResizeHandle.vue'
import Sidebar from '../vault/Sidebar.vue'
import CommandPalette from '../vault/CommandPalette.vue'
import ToastContainer from '../base/ToastContainer.vue'
import GraphPanel from '../graph/GraphPanel.vue'
import StarmapView from '../graph/StarmapView.vue'
import SettingsPanel from './SettingsPanel.vue'
import FrontmatterModal from '../editor/FrontmatterModal.vue'
import * as configService from '@/services/config'
import * as cardService from '@/services/card'
import { useTreeStore } from '@/stores/tree'
import { useEditorStore } from '@/stores/editor'
import { useLayoutStore } from '@/stores/layout'
import { useToast } from '@/composables/core/useToast'
const layout = useLayoutStore()
const editorStore = useEditorStore()
const sidebarWidth = ref(220)
const graphPanelWidth = ref(300)
const graphPanelRef = ref<InstanceType<typeof GraphPanel> | null>(null)
const treeStore = useTreeStore()
const toast = useToast()
const appWindow = getCurrentWindow()
let unlistenResize: UnlistenFn | null = null

async function bootApplication() {
  try {
    await configService.loadConfig()
  } catch (error) {
    console.error('[boot] failed to load config:', error)
  }
  await treeStore.loadTree()
}

async function handleGlobalKeydown(e: KeyboardEvent) {
  // F11: toggle fullscreen zen mode
  if (e.key === 'F11') {
    e.preventDefault()
    const isFs = await appWindow.isFullscreen()
    if (isFs) {
      await appWindow.setFullscreen(false)
      layout.exitZen()
    } else {
      layout.enterZen()
      await appWindow.setFullscreen(true)
    }
    return
  }

  // ESC: layered exit (overlay → zen → right panel → left panel)
  if (e.key === 'Escape') {
    if (layout.activeOverlay !== 'none') {
      layout.closeOverlay()
      e.preventDefault()
      return
    }
    if (layout.zenMode) {
      await appWindow.setFullscreen(false)
      layout.exitZen()
      return
    }
    if (layout.rightPanel !== 'none') {
      layout.rightPanel = 'none'
      return
    }
    if (layout.leftPanel !== 'none') {
      layout.leftPanel = 'none'
      return
    }
    return
  }

  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault()
    layout.openOverlay('commandPalette')
    return
  }
  if ((e.ctrlKey || e.metaKey) && e.key === 'g') {
    e.preventDefault()
    layout.openOverlay('globalGraph')
    return
  }
  if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
    e.preventDefault()
    if (layout.activeOverlay === 'globalGraph') {
      layout.closeOverlay()
    }
    layout.toggleLeftPanel('explorer')
    return
  }
  if ((e.ctrlKey || e.metaKey) && e.key === ',') {
    e.preventDefault()
    layout.openOverlay('settings')
    return
  }
  if ((e.ctrlKey || e.metaKey) && e.key === 'p') {
    e.preventDefault()
    layout.openOverlay('commandPalette')
    return
  }
  if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
    e.preventDefault()
    void handleNewCard()
    return
  }
  if ((e.ctrlKey || e.metaKey) && e.key === 'i') {
    e.preventDefault()
    if (editorStore.currentUuid) {
      layout.openOverlay('frontmatter')
    }
    return
  }
}

async function handleNewCard() {
  try {
    const card = await cardService.createCard('未命名卡片')
    treeStore.setActive(card.uuid)
    await treeStore.loadTree()
  } catch {
    toast.error('新建失败')
  }
}

function handleGraphNavigate(uuid: string) {
  treeStore.setActive(uuid)
}

function handleStarmapNavigate(uuid: string) {
  layout.closeOverlay()
  treeStore.setActive(uuid)
}

provide('layoutStore', layout)
provide('refreshGraph', () => graphPanelRef.value?.refreshGraph())

onMounted(async () => {
  window.addEventListener('keydown', handleGlobalKeydown)
  unlistenResize = await appWindow.onResized(async () => {
    const isFs = await appWindow.isFullscreen()
    if (!isFs && layout.zenMode) {
      layout.exitZen()
    }
  })
  void bootApplication()
})
onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeydown)
  unlistenResize?.()
})
</script>

<template>
  <div class="app-layout">
    <TopBar
      :starmap-active="layout.activeOverlay === 'globalGraph'"
      @toggle-sidebar="layout.toggleLeftPanel('explorer')"
      @open-palette="layout.openOverlay('commandPalette')"
      @open-starmap="layout.openOverlay('globalGraph')"
      @open-settings="layout.openOverlay('settings')"
    />

    <div class="app-body">
      <!-- Left sidebar -->
      <aside
        class="sidebar sidebar-left"
        :class="{ collapsed: layout.leftPanel === 'none' }"
        :style="{ width: layout.leftPanel !== 'none' ? `${sidebarWidth}px` : '0px' }"
      >
        <Sidebar />
      </aside>
      <ResizeHandle
        v-if="layout.leftPanel !== 'none'"
        v-model="sidebarWidth"
        side="left"
        :min-width="160"
        :max-width="400"
      />

      <!-- Main content -->
      <main class="main-content">
        <slot></slot>
      </main>

      <!-- Right panel -->
      <ResizeHandle
        v-if="layout.rightPanel !== 'none'"
        v-model="graphPanelWidth"
        side="right"
        :min-width="200"
        :max-width="500"
      />
      <aside
        class="sidebar sidebar-right"
        :class="{ collapsed: layout.rightPanel === 'none' }"
        :style="{ width: layout.rightPanel !== 'none' ? `${graphPanelWidth}px` : '0px' }"
      >
        <GraphPanel
          v-if="layout.rightPanel === 'graph'"
          ref="graphPanelRef"
          :active-card-uuid="treeStore.activeCardUuid"
          @navigate="handleGraphNavigate"
        />
      </aside>
    </div>

    <StatusBar />

    <!-- Unified overlay layer -->
    <Teleport to="body">
      <!-- Global graph overlay -->
      <div v-if="layout.activeOverlay === 'globalGraph'" class="starmap-overlay">
        <StarmapView @navigate="handleStarmapNavigate" />
      </div>

      <!-- Command palette overlay -->
      <Transition name="overlay">
        <CommandPalette
          v-if="layout.activeOverlay === 'commandPalette'"
          @close="layout.closeOverlay()"
          @open-graph="layout.closeOverlay(); layout.toggleRightPanel('graph')"
          @open-settings="layout.openOverlay('settings')"
        />
      </Transition>

      <!-- Settings overlay -->
      <Transition name="overlay">
        <SettingsPanel v-if="layout.activeOverlay === 'settings'" @close="layout.closeOverlay()" />
      </Transition>

      <!-- Frontmatter inspector overlay -->
      <Transition name="overlay">
        <FrontmatterModal v-if="layout.activeOverlay === 'frontmatter'" @close="layout.closeOverlay()" />
      </Transition>
    </Teleport>

    <!-- Global toast notifications -->
    <ToastContainer />
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.sidebar {
  flex-shrink: 0;
  height: 100%;
  overflow: hidden;
  transition: width var(--duration-slow) var(--ease-out);
}

.sidebar-left {
  background: var(--ms-void);
  border-right: 1px solid var(--ms-border);
}

.sidebar-left.collapsed {
  border-right: none;
}

.sidebar-right {
  background: var(--ms-deep);
  border-left: 1px solid var(--ms-border);
  min-height: 0;
  min-width: 0;
}

.sidebar-right.collapsed {
  border-left: none;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--ms-deep);
}

/* ── Overlay transitions ── */
.overlay-enter-active,
.overlay-leave-active {
  transition: opacity 160ms var(--ease-out);
}

.overlay-enter-from,
.overlay-leave-to {
  opacity: 0;
}

.overlay-mask {
  position: fixed;
  inset: 0;
  background: oklch(0 0 0 / 0.5);
  z-index: var(--z-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
}

.overlay-fullscreen {
  width: 100%;
  height: 100%;
}

.starmap-overlay {
  position: fixed;
  inset: 0;
  background: var(--ms-deep);
  z-index: var(--z-overlay);
}
</style>
