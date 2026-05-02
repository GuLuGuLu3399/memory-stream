// ────────────────────────────────────────────────────────────────
// Sync types — sync protocol payloads, aligned with Go API
// ────────────────────────────────────────────────────────────────

export type SyncManifest = Record<string, number>;

export interface SyncChange {
  id: string;
  title: string;
  content: string;
  excerpt: string;
  ast_data: string;
  toc_data: string | null;
  category: string | null;
  version: number;
  updated_at: string;
}

export interface SyncPushPayload {
  uuid: string;
  title: string;
  category: string;
  content: string;
  ast_data: string;
  toc_data?: string;
  excerpt?: string;
  version: number;
  hash: string;
  edges?: string[];
}

export interface SyncManifestItem {
  uuid: string;
  version: number;
  hash: string;
  is_deleted: boolean;
}

export interface SyncUpsertResponse {
  uuid: string;
  version: number;
}

export interface SyncDeleteResponse {
  id: string;
  op: "delete";
}

export interface SyncResult {
  pulled: number;
  pushed: number;
  deleted: number;
  conflicts: number;
}
