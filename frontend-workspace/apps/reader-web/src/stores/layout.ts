// ────────────────────────────────────────────────────────────────
// layout.ts — Layout state (sheets, overlays, auto-hide)
// ────────────────────────────────────────────────────────────────

import { defineStore } from 'pinia'
import { ref } from 'vue'

export type SheetName = 'toc' | 'meta' | 'graph'

export const useLayoutStore = defineStore('layout', () => {
  const activeSheet = ref<SheetName | null>(null)

  // Global overlay panels
  const searchOpen = ref(false)
  const outlineOpen = ref(false)

  function toggleSheet(name: SheetName) {
    activeSheet.value = activeSheet.value === name ? null : name
  }

  function closeSheet() {
    activeSheet.value = null
  }

  function openSearch() {
    searchOpen.value = true
  }

  function closeSearch() {
    searchOpen.value = false
  }

  function toggleSearch() {
    searchOpen.value = !searchOpen.value
  }

  function openOutline() {
    outlineOpen.value = true
  }

  function closeOutline() {
    outlineOpen.value = false
  }

  function toggleOutline() {
    outlineOpen.value = !outlineOpen.value
  }

  function closeAll() {
    activeSheet.value = null
    searchOpen.value = false
    outlineOpen.value = false
  }

  return {
    activeSheet,
    searchOpen,
    outlineOpen,
    toggleSheet,
    closeSheet,
    openSearch,
    closeSearch,
    toggleSearch,
    openOutline,
    closeOutline,
    toggleOutline,
    closeAll,
  }
})
