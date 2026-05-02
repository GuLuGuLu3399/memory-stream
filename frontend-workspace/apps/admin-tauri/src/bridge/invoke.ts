// ────────────────────────────────────────────────────────────────
// invoke.ts — Tauri command IPC (Layer 1: mutations, config, validation)
// Every function maps 1:1 to a Rust #[tauri::command]
// Tauri 命令 IPC（第 1 层：变更、配置、验证）
// 每个函数与 Rust #[tauri::command] 1:1 映射
// ────────────────────────────────────────────────────────────────

import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  CardIndex,
  FtsHit,
  CardMeta,
  ParsedDocument,
  FullGraph,
  SubgraphResult,
  BacklinkItem,
} from "@memory-stream/types";
import type { TreeNode } from "@memory-stream/core";
import type { AstNode } from "@memory-stream/types";

export interface SyncConflictEntry {
  uuid: string;
  title: string;
  local_path: string;
  conflict_copy_path: string;
  local_bytes: number;
  remote_bytes: number;
  local_updated_at: string;
  remote_updated_at: string;
}

export interface SyncRunReport {
  downloaded: number;
  uploaded: number;
  deleted: number;
  conflicts: SyncConflictEntry[];
}

// ── Config ──────────────────────────────────────────────────────

export function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

export function setConfig(patch: {
  api_base_url?: string;
  vault_path?: string;
  theme?: string;
}): Promise<void> {
  return invoke("set_config", { patch });
}

export function setStorageConfig(
  storage: import("@memory-stream/types").StorageConfig,
): Promise<void> {
  return invoke("set_storage_config", { storage });
}

export function clearStorageConfig(): Promise<void> {
  return invoke("clear_storage_config");
}

// ── Card CRUD ───────────────────────────────────────────────────

export function readCardFile(uuid: string): Promise<string> {
  return invoke<string>("read_card_file", { uuid });
}

export function createCard(
  title: string,
  category?: string,
): Promise<CardIndex> {
  return invoke<CardIndex>("create_card", { title, category });
}

export function deleteCard(uuid: string, soft = true): Promise<void> {
  return invoke("delete_card", { uuid, soft });
}

export function renameCard(uuid: string, newTitle: string): Promise<CardMeta> {
  return invoke<CardMeta>("rename_card", { uuid, newTitle });
}

// ── Ghost node ──────────────────────────────────────────────────

export function checkTitleExists(title: string): Promise<boolean> {
  return invoke<boolean>("check_title_exists", { title });
}

export function materializeGhost(
  title: string,
  category?: string,
): Promise<CardIndex> {
  return invoke<CardIndex>("materialize_ghost", { title, category });
}

// ── Vault ───────────────────────────────────────────────────────

export function scanVault(): Promise<TreeNode[]> {
  return invoke<TreeNode[]>("scan_vault_tree");
}

export function moveCard(uuid: string, targetCategory: string): Promise<void> {
  return invoke("move_card", { uuid, targetCategory });
}

export function createCategory(category: string): Promise<void> {
  return invoke("create_category", { category });
}

export function renameCategory(oldName: string, newName: string): Promise<void> {
  return invoke("rename_category", { oldName, newName });
}

export function deleteCategory(category: string): Promise<void> {
  return invoke("delete_category", { category });
}

export function scanAndHeal(): Promise<number> {
  return invoke<number>("scan_and_heal");
}

// ── FTS5 Search ─────────────────────────────────────────────────

export function searchFts(query: string, limit = 20): Promise<FtsHit[]> {
  return invoke<FtsHit[]>("search_fts", { query, limit });
}

// ── AST ─────────────────────────────────────────────────────────

export function parseMarkdown(rawText: string): Promise<ParsedDocument> {
  return invoke<ParsedDocument>("parse_markdown", { rawText });
}

export function parseLiveMarkdown(content: string): Promise<AstNode> {
  return invoke<AstNode>("parse_live_markdown", { content });
}

export function saveDocumentIO(
  meta: CardMeta,
  content: string,
): Promise<CardMeta> {
  return invoke<CardMeta>("save_document_io", { meta, content });
}

// ── Auth ────────────────────────────────────────────────────────

export function loginToServer(username: string, password: string): Promise<string> {
  return invoke<string>("login_to_server", { username, password });
}

// ── Sync ────────────────────────────────────────────────────────

export function syncPull(): Promise<void> {
  return invoke("sync_pull");
}

export function syncPush(): Promise<void> {
  return invoke("sync_push");
}

export function syncDeleteTombstones(): Promise<void> {
  return invoke("sync_delete_tombstones");
}

export function syncNow(): Promise<SyncRunReport> {
  return invoke<SyncRunReport>("sync_now");
}

export function resolveConflict(uuid: string): Promise<void> {
  return invoke("resolve_conflict", { uuid });
}

export function resolveConflictKeepLocal(uuid: string): Promise<void> {
  return invoke("resolve_conflict_keep_local", { uuid });
}

export function resolveConflictKeepRemote(uuid: string): Promise<void> {
  return invoke("resolve_conflict_keep_remote", { uuid });
}

// ── Assets / OSS ────────────────────────────────────────────────

export function uploadImage(
  imageData: Uint8Array,
  filename: string,
): Promise<string> {
  return invoke<string>("upload_image", {
    imageData: Array.from(imageData),
    filename,
  });
}

// ── Graph ───────────────────────────────────────────────────────

export function getFullGraph(): Promise<FullGraph> {
  return invoke<FullGraph>("get_full_graph");
}

export function getGraphNeighborhood(
  uuid: string,
  depth = 2,
): Promise<SubgraphResult> {
  return invoke<SubgraphResult>("get_graph_neighborhood", { uuid, depth });
}

export function createTrunk(
  source: string,
  target: string,
): Promise<void> {
  return invoke("create_trunk", { source, target });
}

export function deleteTrunk(
  source: string,
  target: string,
): Promise<void> {
  return invoke("delete_trunk_with_fallback", { source, target });
}

export function createLink(
  source: string,
  target: string,
): Promise<void> {
  return invoke("create_link", { source, target });
}

export function reverseTrunk(
  source: string,
  target: string,
): Promise<void> {
  return invoke("reverse_trunk", { source, target });
}

export function getBacklinks(uuid: string): Promise<BacklinkItem[]> {
  return invoke<BacklinkItem[]>("get_backlinks", { uuid });
}

export interface NavNode {
  uuid: string;
  title: string;
}

export interface TrunkNav {
  parents: NavNode[];
  children: NavNode[];
}

export function getTrunkNavigation(uuid: string): Promise<TrunkNav> {
  return invoke<TrunkNav>("get_trunk_navigation", { uuid });
}

// ── Glossary ──────────────────────────────────────────────────────

export function readGlossary(): Promise<Record<string, string>> {
  return invoke<Record<string, string>>("read_glossary");
}

export function saveGlossary(
  glossary: Record<string, string>,
): Promise<void> {
  return invoke("save_glossary", { glossary });
}
