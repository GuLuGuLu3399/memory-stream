import { onMounted, onUnmounted } from "vue";
import { useLayoutStore } from "../stores/layout";

export function useGlobalShortcuts() {
  const layoutStore = useLayoutStore();

  function handleKeydown(e: KeyboardEvent) {
    // Don't trigger if user is typing in an input/textarea
    const tag = (e.target as HTMLElement)?.tagName?.toLowerCase();
    if (tag === "input" || tag === "textarea") return;

    // Ctrl+B → 切换左侧抽屉
    if ((e.metaKey || e.ctrlKey) && e.key === "b") {
      e.preventDefault();
      layoutStore.toggleLeftDrawer();
    }

  // Ctrl+\\ (new) or Ctrl+G (backward compat) → 切换右侧图谱面板
  if ((e.metaKey || e.ctrlKey) && (e.key === '\\' || e.key === 'g')) {
      e.preventDefault();
      layoutStore.toggleRightPanel();
    }
  }

  onMounted(() => window.addEventListener("keydown", handleKeydown));
  onUnmounted(() => window.removeEventListener("keydown", handleKeydown));
}
