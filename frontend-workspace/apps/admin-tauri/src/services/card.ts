// ────────────────────────────────────────────────────────────────
// card.ts — Card CRUD + ghost node materialization
// card.ts — 卡片 CRUD + 虚节点实体化
// ────────────────────────────────────────────────────────────────

import type { CardIndex, CardMeta } from "@memory-stream/types";
import * as bridge from "@/bridge/invoke";
import { extractWikilinkTargets } from "@memory-stream/core";

export async function createCard(
  title: string,
  category?: string,
): Promise<CardIndex> {
  return bridge.createCard(title, category);
}

export async function deleteCard(uuid: string, soft = true): Promise<void> {
  return bridge.deleteCard(uuid, soft);
}

export async function renameCard(
  uuid: string,
  newTitle: string,
): Promise<CardMeta> {
  return bridge.renameCard(uuid, newTitle);
}

/**
 * Ghost node materialization:
 * Extract [[title]] links from text, check existence, create missing cards.
 */
export async function materializeGhosts(
  text: string,
  category?: string,
): Promise<CardIndex[]> {
  const targets = extractWikilinkTargets(text);
  const created: CardIndex[] = [];

  for (const title of targets) {
    const exists = await bridge.checkTitleExists(title);
    if (!exists) {
      created.push(await bridge.materializeGhost(title, category));
    }
  }

  return created;
}
