import { defineStore } from "pinia";
import { ref, computed } from "vue";

export type Chamber = "category" | "merge" | "settings" | "debug" | null;

export const useLayoutStore = defineStore("layout", () => {
  // ===== 舱室 (Chamber) 状态 =====
  // 三大功能面板共享同一舱室位 — 互斥切换
  const activeChamber = ref<Chamber>(null);

  const isCategoryPanelOpen = computed(
    () => activeChamber.value === "category",
  );
  const isSettingsOpen = computed(() => activeChamber.value === "settings");
  const isMergeConsoleOpen = computed(() => activeChamber.value === "merge");
  const isDebugPanelOpen = computed(() => activeChamber.value === "debug");

  function openChamber(chamber: NonNullable<Chamber>) {
    activeChamber.value = chamber;
  }

  function closeChamber() {
    activeChamber.value = null;
  }

  // Named convenience functions (backward-compatible)
  function openCategoryPanel() {
    openChamber("category");
  }

  function closeCategoryPanel() {
    if (activeChamber.value === "category") closeChamber();
  }

  function openSettings() {
    openChamber("settings");
  }

  function openMergeConsole() {
    openChamber("merge");
  }

  function openDebugPanel() {
    openChamber("debug");
  }

  function closeMergeConsole() {
    if (activeChamber.value === "merge") closeChamber();
  }

  // ===== 抽屉/面板状态 =====
  const isLeftDrawerOpen = ref(false);
  const isRightPanelOpen = ref(false);
  const isLeftSidebarPinned = ref(false); // 常驻侧栏模式
  const isImportPanelOpen = ref(false);

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
    activeChamber.value = null;
  }

  function openImportPanel() {
    isImportPanelOpen.value = true;
  }

  function closeImportPanel() {
    isImportPanelOpen.value = false;
  }

  return {
    // Chamber
    activeChamber,
    isCategoryPanelOpen,
    isSettingsOpen,
    isMergeConsoleOpen,
    isDebugPanelOpen,
    openChamber,
    closeChamber,
    openCategoryPanel,
    closeCategoryPanel,
    openSettings,
    openMergeConsole,
    openDebugPanel,
    closeMergeConsole,
    // Drawers
    isLeftDrawerOpen,
    isRightPanelOpen,
    isLeftSidebarPinned,
    isImportPanelOpen,
    toggleLeftDrawer,
    toggleSidebarPin,
    toggleRightPanel,
    closeLeftDrawer,
    closeRightPanel,
    closeAll,
    openImportPanel,
    closeImportPanel,
  };
});
