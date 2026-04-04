/**
 * 🌟 useKeyboardNav — 全局键盘导航
 *
 * 快捷键映射：
 * - Escape：关闭抽屉 / 取消选中
 * - ← / →：DetailDrawer 中切换上/下一张卡片
 * - ↑ / ↓：ListView 中上下移动选中卡片
 * - Ctrl+K / Cmd+K：聚焦搜索（预留）
 */

import { onMounted, onUnmounted } from "vue";
import { useGraphStore } from "../store/useGraphStore";
import { useCards } from "./useCards";

export function useKeyboardNav() {
  const store = useGraphStore();
  const { cardIndex } = useCards();

  function handleKeydown(e: KeyboardEvent) {
    // ── Cmd+K / Ctrl+K：全局搜索面板 ──
    if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      store.toggleCommandPalette();
      return;
    }

    // ── Escape：关闭抽屉 / 关闭命令面板 ──
    if (e.key === "Escape") {
      if (store.commandPaletteOpen) {
        store.commandPaletteOpen = false;
        e.preventDefault();
        return;
      }
      if (store.selectedId) {
        store.selectNode(null);
        e.preventDefault();
      }
      return;
    }

    // ── 方向键：需要当前有选中卡片 ──
    if (!store.selectedId || cardIndex.value.length === 0) return;

    const currentIndex = cardIndex.value.findIndex(
      (c) => c.id === store.selectedId,
    );
    if (currentIndex === -1) return;

    // ── ← / ↑：上一张 ──
    if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
      e.preventDefault();
      const prevIndex = currentIndex > 0 ? currentIndex - 1 : currentIndex;
      store.selectNode(cardIndex.value[prevIndex].id);
      return;
    }

    // ── → / ↓：下一张 ──
    if (e.key === "ArrowRight" || e.key === "ArrowDown") {
      e.preventDefault();
      const nextIndex =
        currentIndex < cardIndex.value.length - 1
          ? currentIndex + 1
          : currentIndex;
      store.selectNode(cardIndex.value[nextIndex].id);
      return;
    }
  }

  onMounted(() => {
    window.addEventListener("keydown", handleKeydown);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", handleKeydown);
  });
}
