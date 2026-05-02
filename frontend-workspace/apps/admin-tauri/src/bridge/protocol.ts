// ────────────────────────────────────────────────────────────────
// protocol.ts — Data layer facade
// ms:// protocol for asset URLs; invoke for API calls
// 数据层门面 — 资源 URL 走 ms:// 协议，API 调用走 invoke
// ────────────────────────────────────────────────────────────────

import type {
  SubgraphResult,
  FullGraph,
} from "@memory-stream/types";
import * as invokeBridge from "@/bridge/invoke";

const MS_BASE = "ms://localhost";

// ── Graph (via invoke) ─────────────────────────────────────────

export function fetchFullGraph(): Promise<FullGraph> {
  return invokeBridge.getFullGraph();
}

export function fetchNeighborhood(
  uuid: string,
  depth = 2,
): Promise<SubgraphResult> {
  return invokeBridge.getGraphNeighborhood(uuid, depth);
}

export function createTrunkEdge(
  source: string,
  target: string,
): Promise<void> {
  return invokeBridge.createTrunk(source, target);
}

export function deleteTrunkEdge(
  source: string,
  target: string,
): Promise<void> {
  return invokeBridge.deleteTrunk(source, target);
}

export function createLinkEdge(
  source: string,
  target: string,
): Promise<void> {
  return invokeBridge.createLink(source, target);
}

export function reverseTrunkEdge(
  source: string,
  target: string,
): Promise<void> {
  return invokeBridge.reverseTrunk(source, target);
}

// ── Assets (ms:// protocol for img src) ────────────────────────

export function assetUrl(path: string): string {
  return `${MS_BASE}/assets/${encodeURIComponent(path)}`;
}
