/**
 * useDetailDrawer — Detail drawer state and logic
 *
 * Manages card detail loading, backlinks fetching, and prose hover interactions.
 */

import { ref, watch } from "vue";
import { useCards } from "./useCards";
import type { CardDetail } from "./useCards";
import { api } from "../api";
import type { InferredBacklinkItem } from "../api/schemas";

export interface BacklinkItem extends InferredBacklinkItem {}

export function useDetailDrawer(selectedId: () => string | null) {
  const { loadDetail } = useCards();

  // State
  const detail = ref<CardDetail | null>(null);
  const loading = ref(false);
  const createdAt = ref("");
  const backlinks = ref<BacklinkItem[]>([]);
  const backlinksLoading = ref(false);
  const backlinksOpen = ref(false);

  // Watch selectedId changes
  watch(selectedId, async (newId) => {
    if (!newId) {
      detail.value = null;
      backlinks.value = [];
      createdAt.value = "";
      return;
    }

    loading.value = true;
    detail.value = null;
    backlinks.value = [];
    createdAt.value = "";

    try {
      const result = await loadDetail(newId);
      detail.value = result;

      // Fetch backlinks
      backlinksLoading.value = true;
      try {
        const res = await api.getBacklinks(newId);
        backlinks.value = res.backlinks || [];
      } catch {
        backlinks.value = [];
      } finally {
        backlinksLoading.value = false;
      }

      // Fetch card details to get created_at
      try {
        const cardData = await api.getCard(newId);
        createdAt.value = cardData.created_at || "";
      } catch {
        createdAt.value = "";
      }
    } finally {
      loading.value = false;
    }
  });

  // Prose hover handlers
  function onProseMouseOver(e: MouseEvent, highlightNode: (cardId: string | null) => void) {
    const target = (e.target as HTMLElement).closest("a.wikilink, a[data-card-id]");
    if (target) {
      const cardId = (target as HTMLAnchorElement).dataset.cardId ||
                    (target as HTMLAnchorElement).getAttribute("href")?.replace("/card/", "");
      if (cardId) highlightNode(cardId);
    }
  }

  function onProseMouseOut(e: MouseEvent, highlightNode: (cardId: string | null) => void) {
    const target = (e.target as HTMLElement).closest("a.wikilink, a[data-card-id]");
    if (target) {
      highlightNode(null);
    }
  }

  // Backlink navigation
  function navigateToBacklink(cardId: string, selectNode: (cardId: string) => void) {
    selectNode(cardId);
  }

  // Badge styling
  function getBadgeClass(relationType: string): string {
    if (relationType === "sequence") {
      return "text-xuepo bg-xuepo/10 border-xuepo/30";
    }
    return "text-ms-smoke bg-ms-smoke/10 border-ms-smoke/30";
  }

  return {
    // State
    detail,
    loading,
    createdAt,
    backlinks,
    backlinksLoading,
    backlinksOpen,

    // Methods
    onProseMouseOver,
    onProseMouseOut,
    navigateToBacklink,
    getBadgeClass,
  };
}
