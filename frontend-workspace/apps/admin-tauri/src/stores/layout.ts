// 用途：面板布局状态管理，控制侧边栏、图谱和禅模式
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useLayoutStore = defineStore('layout', () => {
  const leftPanel = ref<'explorer' | 'search' | 'none'>('explorer')
  const rightPanel = ref<'graph' | 'none'>('none')
  const activeOverlay = ref<'none' | 'globalGraph' | 'commandPalette' | 'settings' | 'frontmatter'>('none')
  const zenMode = ref(false)

  const preZenLeftPanel = ref<typeof leftPanel.value>('explorer')
  const preZenRightPanel = ref<typeof rightPanel.value>('none')

  const hasOverlay = computed(() => activeOverlay.value !== 'none')

  function openOverlay(name: typeof activeOverlay.value) {
    if (activeOverlay.value === name) {
      activeOverlay.value = 'none'
      return
    }
    activeOverlay.value = name
  }

  function closeOverlay() {
    activeOverlay.value = 'none'
  }

  function toggleLeftPanel(name: 'explorer' | 'search') {
    leftPanel.value = leftPanel.value === name ? 'none' : name
  }

  function toggleRightPanel(name: 'graph') {
    rightPanel.value = rightPanel.value === name ? 'none' : name
  }

  function enterZen() {
    preZenLeftPanel.value = leftPanel.value
    preZenRightPanel.value = rightPanel.value
    leftPanel.value = 'none'
    rightPanel.value = 'none'
    activeOverlay.value = 'none'
    zenMode.value = true
  }

  function exitZen() {
    leftPanel.value = preZenLeftPanel.value
    rightPanel.value = preZenRightPanel.value
    zenMode.value = false
  }

  return {
    leftPanel,
    rightPanel,
    activeOverlay,
    zenMode,
    preZenLeftPanel,
    preZenRightPanel,
    hasOverlay,
    openOverlay,
    closeOverlay,
    toggleLeftPanel,
    toggleRightPanel,
    enterZen,
    exitZen,
  }
})
