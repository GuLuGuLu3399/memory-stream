import { defineStore } from "pinia";
import { ref } from "vue";

export const useLayoutStore = defineStore("layout", () => {
  // ===== 抽屉/面板状态 =====
  const isLeftDrawerOpen = ref(false);
  const isRightPanelOpen = ref(false);
  const isLeftSidebarPinned = ref(false); // 常驻侧栏模式
  const isCategoryPanelOpen = ref(false)
  const isSettingsOpen = ref(false)
  const isMergeConsoleOpen = ref(false)
  const isImportPanelOpen = ref(false)

  function toggleLeftDrawer() {
    isLeftDrawerOpen.value = !isLeftDrawerOpen.value;
    // 关闭另一侧（互斥模式，可选）
    if (isLeftDrawerOpen.value) {
      isRightPanelOpen.value = false;
    }
  }

  function toggleSidebarPin() {
    isLeftSidebarPinned.value = !isLeftSidebarPinned.value;
    if (isLeftSidebarPinned.value) {
      isLeftDrawerOpen.value = true; // pin 时确保展开
    }
  }

  function toggleRightPanel() {
    isRightPanelOpen.value = !isRightPanelOpen.value;
    if (isRightPanelOpen.value) {
      isLeftDrawerOpen.value = false;
    }
  }

  function closeLeftDrawer() {
    if (isLeftSidebarPinned.value) return; // 常驻模式不允许关闭
    isLeftDrawerOpen.value = false;
  }

  function closeRightPanel() {
    isRightPanelOpen.value = false;
  }

  function closeAll() {
    isLeftDrawerOpen.value = false;
    isRightPanelOpen.value = false;
  }

  function openCategoryPanel() {
    isSettingsOpen.value = false;
    isMergeConsoleOpen.value = false;
    isCategoryPanelOpen.value = true;
  }

  function closeCategoryPanel() {
    isCategoryPanelOpen.value = false;
  }

  function openSettings() {
    isCategoryPanelOpen.value = false;
    isMergeConsoleOpen.value = false;
    isSettingsOpen.value = true;
  }

  function openMergeConsole() {
    isSettingsOpen.value = false;
    isCategoryPanelOpen.value = false;
    isMergeConsoleOpen.value = true;
  }

  function closeMergeConsole() {
    isMergeConsoleOpen.value = false
  }

  function openImportPanel() {
    isImportPanelOpen.value = true
  }

  function closeImportPanel() {
    isImportPanelOpen.value = false
  }

  return {
    isLeftDrawerOpen,
    isRightPanelOpen,
    isLeftSidebarPinned,
    isCategoryPanelOpen,
    isSettingsOpen,
    isMergeConsoleOpen,
    isImportPanelOpen,
    toggleLeftDrawer,
    toggleSidebarPin,
    toggleRightPanel,
    closeLeftDrawer,
    closeRightPanel,
    closeAll,
    openCategoryPanel,
    closeCategoryPanel,
    openSettings,
    openMergeConsole,
    closeMergeConsole,
    openImportPanel,
    closeImportPanel,
  };
});
