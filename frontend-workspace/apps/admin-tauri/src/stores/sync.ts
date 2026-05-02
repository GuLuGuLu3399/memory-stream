// ────────────────────────────────────────────────────────────────
// sync.ts — Sync state (status, progress, pending counts, conflicts)
// ────────────────────────────────────────────────────────────────

import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import * as syncService from "@/services/sync";
import { onSyncProgress } from "@/bridge/events";
import { resolveConflictKeepLocal, resolveConflictKeepRemote } from "@/bridge/invoke";
import type { SyncRunReport } from "@/services/sync";

export const useSyncStore = defineStore("sync", () => {
  const status = ref<"idle" | "syncing" | "conflict" | "error">("idle");
  const progress = ref(0);
  const pendingCount = ref(0);
  const lastError = ref<string | null>(null);
  const lastReport = ref<SyncRunReport | null>(null);
  const conflicts = computed(() => lastReport.value?.conflicts ?? []);
  const isBusy = computed(() => status.value === "syncing");

  // Authoritative conflict set — persisted in SQLite across app restarts
  const conflictedUuids = ref<string[]>([]);

  async function loadConflictedUuids() {
    try {
      conflictedUuids.value = await invoke<string[]>("get_conflicted_uuids");
    } catch {
      // non-fatal — may not exist yet
    }
  }

  // Subscribe to background sync progress events
  let unlisten: (() => void) | null = null;
  let initPromise: Promise<void> | null = null;

  async function init() {
    if (initPromise) return initPromise;
    initPromise = (async () => {
      if (unlisten) return;
      unlisten = await onSyncProgress((e) => {
        progress.value = e.progress;
        if (e.status === "syncing") status.value = "syncing";
        if (e.status === "conflict") status.value = "conflict";
      });
      await loadConflictedUuids();
    })();
    await initPromise;
  }

  function destroy() {
    unlisten?.();
    unlisten = null;
  }

  async function syncNow() {
    if (isBusy.value) return;

    status.value = "syncing";
    progress.value = 0;
    lastError.value = null;
    try {
      const report = await syncService.syncNow();
      lastReport.value = report;
      pendingCount.value = report.conflicts.length;
      status.value = report.conflicts.length > 0 ? "conflict" : "idle";
    } catch (e) {
      status.value = "error";
      lastError.value = e instanceof Error ? e.message : String(e);
    } finally {
      if (status.value === "syncing") status.value = "idle";
      // Refresh authoritative conflict set after every sync
      await loadConflictedUuids();
    }
  }

  async function resolveConflict(uuid: string, strategy: 'local' | 'remote') {
    if (strategy === 'local') {
      await resolveConflictKeepLocal(uuid)
    } else {
      await resolveConflictKeepRemote(uuid)
    }
    conflictedUuids.value = conflictedUuids.value.filter(id => id !== uuid)
  }

  return {
    status,
    progress,
    pendingCount,
    lastError,
    lastReport,
    conflicts,
    conflictedUuids,
    isBusy,
    init,
    destroy,
    syncNow,
    loadConflictedUuids,
    resolveConflict,
  };
});
