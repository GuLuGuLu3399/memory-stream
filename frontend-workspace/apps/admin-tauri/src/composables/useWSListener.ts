import { onMounted, onUnmounted } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useKnowledgeStore } from "../stores/knowledge";

export function useWSListener() {
  const store = useKnowledgeStore();
  const unlisteners: UnlistenFn[] = [];

  onMounted(async () => {
    unlisteners.push(
      await listen("layout_synced", (event) => {
        store.updateLayouts(
          event.payload as { id: string; x: number; y: number }[],
        );
      }),
    );

    unlisteners.push(
      await listen("graph_changed", () => {
        store.silentRefresh();
      }),
    );

    unlisteners.push(
      await listen("ws_card_created", () => {
        store.loadRecent();
        store.loadOrphans();
      }),
    );

    unlisteners.push(
      await listen("ws_card_updated", () => {
        store.loadRecent();
        store.loadOrphans();
        // Self-triggered event: skip activeCard reload to avoid dirty state desync
      }),
    );

    unlisteners.push(
      await listen("ws_card_deleted", () => {
        store.loadRecent();
        store.loadOrphans();
      }),
    );

    unlisteners.push(
      await listen("ws_cards_merged", () => {
        store.loadRecent();
        store.loadOrphans();
        if (store.activeCard?.id) {
          store.loadAndActivateCard(store.activeCard.id);
        }
      }),
    );
  });

  onUnmounted(() => {
    unlisteners.forEach((fn) => fn());
  });
}
