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

const REFERENCE_LINK_SELECTOR =
  "a.reference-link, a.wikilink, a[data-card-id], a[data-card-name]";

function extractCardIdFromHref(href: string | null): string | null {
  if (!href) return null;

  if (href.startsWith("memory-stream://card/")) {
    return (
      decodeURIComponent(
        href.replace("memory-stream://card/", "").split("?")[0] ?? "",
      ) || null
    );
  }

  if (href.startsWith("tauri://card/")) {
    return (
      decodeURIComponent(
        href.replace("tauri://card/", "").split("?")[0] ?? "",
      ) || null
    );
  }

  if (href.startsWith("/cards/")) {
    return (
      decodeURIComponent(href.replace("/cards/", "").split("?")[0] ?? "") ||
      null
    );
  }
  if (href.startsWith("/card/")) {
    return (
      decodeURIComponent(href.replace("/card/", "").split("?")[0] ?? "") || null
    );
  }

  try {
    const url = new URL(href, window.location.origin);
    if (url.pathname.startsWith("/cards/")) {
      return decodeURIComponent(url.pathname.replace("/cards/", "")) || null;
    }
    if (url.pathname.startsWith("/card/")) {
      return decodeURIComponent(url.pathname.replace("/card/", "")) || null;
    }
  } catch {
    // ignore invalid URLs
  }

  return null;
}

export function useDetailDrawer(selectedId: () => string | null) {
  const { loadDetail } = useCards();
  const titleToIdCache = new Map<string, string>();

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
  function onProseMouseOver(
    e: MouseEvent,
    highlightNode: (cardId: string | null) => void,
  ) {
    const target = (e.target as HTMLElement).closest(REFERENCE_LINK_SELECTOR);
    if (target) {
      const cardId =
        (target as HTMLAnchorElement).dataset.cardId ||
        extractCardIdFromHref(
          (target as HTMLAnchorElement).getAttribute("href"),
        );
      if (cardId) highlightNode(cardId);
    }
  }

  function onProseMouseOut(
    e: MouseEvent,
    highlightNode: (cardId: string | null) => void,
  ) {
    const target = (e.target as HTMLElement).closest(REFERENCE_LINK_SELECTOR);
    if (target) {
      highlightNode(null);
    }
  }

  async function onProseClick(
    e: MouseEvent,
    selectNode: (cardId: string) => void,
  ) {
    const target = (e.target as HTMLElement).closest(
      REFERENCE_LINK_SELECTOR,
    ) as HTMLAnchorElement | null;
    if (!target) return;

    e.preventDefault();

    const fromData = target.dataset.cardId;
    if (fromData) {
      selectNode(fromData);
      return;
    }

    const fromHref = extractCardIdFromHref(target.getAttribute("href"));
    if (fromHref) {
      selectNode(fromHref);
      return;
    }

    const cardName = target.dataset.cardName?.trim();
    if (!cardName) return;

    const cached = titleToIdCache.get(cardName);
    if (cached) {
      selectNode(cached);
      return;
    }

    try {
      const search = await api.searchCards(cardName, 10, 0);
      const exact =
        search.results.find((item) => item.title.trim() === cardName) ??
        search.results[0];
      if (exact?.id) {
        titleToIdCache.set(cardName, exact.id);
        selectNode(exact.id);
      }
    } catch {
      // 点击降级：查不到就保持静默，不阻塞阅读流
    }
  }

  // Backlink navigation
  function navigateToBacklink(
    cardId: string,
    selectNode: (cardId: string) => void,
  ) {
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
    onProseClick,
    navigateToBacklink,
    getBadgeClass,
  };
}
