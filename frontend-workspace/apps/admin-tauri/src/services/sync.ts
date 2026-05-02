// ────────────────────────────────────────────────────────────────
// sync.ts — Sync engine orchestration (Pull / Push / Tombstone cleanup)
// sync.ts — 同步引擎编排（拉取 / 推送 / 墓碑清理）
// ────────────────────────────────────────────────────────────────

import * as bridge from "@/bridge/invoke";
import type { SyncRunReport } from "@/bridge/invoke";

export type { SyncConflictEntry, SyncRunReport } from "@/bridge/invoke";

export async function pull(): Promise<void> {
  return bridge.syncPull();
}

export async function push(): Promise<void> {
  return bridge.syncPush();
}

export async function deleteTombstones(): Promise<void> {
  return bridge.syncDeleteTombstones();
}

export async function fullSync(): Promise<void> {
  await pull();
  await push();
  await deleteTombstones();
}

export async function syncNow(): Promise<SyncRunReport> {
  return bridge.syncNow();
}
