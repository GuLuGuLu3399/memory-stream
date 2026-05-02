// ────────────────────────────────────────────────────────────────
// vault.ts — Vault scanning + category management
// vault.ts — 目录扫描 + 分类管理
// ────────────────────────────────────────────────────────────────

import type { TreeNode } from "@memory-stream/core";
import * as bridge from "@/bridge/invoke";

export async function scanVault(): Promise<TreeNode[]> {
  return bridge.scanVault();
}

export async function moveCard(
  uuid: string,
  targetCategory: string,
): Promise<void> {
  return bridge.moveCard(uuid, targetCategory);
}

export async function createCategory(category: string): Promise<void> {
  return bridge.createCategory(category);
}

export async function renameCategory(oldName: string, newName: string): Promise<void> {
  return bridge.renameCategory(oldName, newName);
}

export async function deleteCategory(category: string): Promise<void> {
  return bridge.deleteCategory(category);
}
