// ────────────────────────────────────────────────────────────────
// config.ts — App configuration management via invoke
// config.ts — 通过 invoke 管理应用配置
// ────────────────────────────────────────────────────────────────

import type { AppConfig } from "@memory-stream/types";
import * as bridge from "@/bridge/invoke";

let cached: AppConfig | null = null;

export async function loadConfig(): Promise<AppConfig> {
  cached = await bridge.getConfig();
  return cached;
}

export async function updateConfig(patch: Partial<AppConfig>): Promise<void> {
  await bridge.setConfig(patch);
  if (cached) {
    cached = { ...cached, ...patch };
  }
}

export function getCached(): AppConfig | null {
  return cached;
}
