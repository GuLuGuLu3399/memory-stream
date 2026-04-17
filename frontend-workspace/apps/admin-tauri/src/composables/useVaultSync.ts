import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSysConfigStore } from "../stores/sysconfig";

export function useVaultSync() {
  const isWatching = ref(false);
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  async function startWatcher(vaultPath: string) {
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }

    await invoke("start_watcher", { watchDir: vaultPath });
    isWatching.value = true;
    // Poll every 5 seconds for local changes
    pollTimer = setInterval(async () => {
      try {
        const result = await invoke<{
          uploaded: number;
          created: number;
          errors: string[];
        }>("process_local_changes");
        if (result.uploaded > 0 || result.created > 0) {
          const store = useSysConfigStore();
          store.syncStats.uploaded = result.uploaded;
          store.syncStats.created = result.created;
          // Refresh workspace to show changes
          const { useKnowledgeStore } = await import("../stores/knowledge");
          useKnowledgeStore().refreshWorkspace();
        }
      } catch {
        // Silent - watcher might not be running
      }
    }, 5000);
  }

  async function stopWatcher() {
    if (pollTimer) clearInterval(pollTimer);
    pollTimer = null;
    try {
      await invoke("stop_watcher");
    } catch {
      // Silent fallback: local timer has already been cleared.
    }
    isWatching.value = false;
  }

  return { isWatching, startWatcher, stopWatcher };
}
