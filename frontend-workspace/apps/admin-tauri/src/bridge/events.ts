// ────────────────────────────────────────────────────────────────
// events.ts — Tauri event subscriptions (Layer 3: background push)
// Pub/Sub: Rust pushes, frontend listens
// Tauri 事件订阅（第 3 层：后台推送）
// 发布/订阅：Rust 推送，前端监听
// ────────────────────────────────────────────────────────────────

import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ── Event payloads ──────────────────────────────────────────────

export interface SyncProgressEvent {
  status: string;
  progress: number;
}

export interface FileChangeEvent {
  path: string;
  kind: "create" | "modify" | "delete";
}

// ── Subscriptions ───────────────────────────────────────────────

export function onSyncProgress(
  handler: (e: SyncProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<SyncProgressEvent>("sync:progress", (event) =>
    handler(event.payload),
  );
}

export function onFileChange(
  handler: (e: FileChangeEvent) => void,
): Promise<UnlistenFn> {
  return listen<FileChangeEvent>("fs:change", (event) =>
    handler(event.payload),
  );
}
