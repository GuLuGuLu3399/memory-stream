// ────────────────────────────────────────────────────────────────
// SyncStatus state machine — mirrors ms-meta SyncStatus transitions
// SyncStatus 状态机 — 镜像 ms-meta SyncStatus 转换
// ────────────────────────────────────────────────────────────────
import type { SyncStatus } from "@memory-stream/types";
const TRANSITIONS: Record<SyncStatus, SyncStatus[]> = {
  synced: ["pending_push", "pending_delete"],
  pending_push: ["synced", "conflict"],
  pending_delete: ["synced"],
  conflict: ["pending_push", "synced"],
};

export function canTransition(from: SyncStatus, to: SyncStatus): boolean {
  return TRANSITIONS[from].includes(to);
}

export function nextAfterLocalEdit(status: SyncStatus): SyncStatus {
  if (status === "pending_delete") return status;
  return "pending_push";
}

export function nextAfterPush(): SyncStatus {
  return "synced";
}

export function nextAfterConflictResolve(): SyncStatus {
  return "pending_push";
}

export function isPending(status: SyncStatus): boolean {
  return status !== "synced";
}
