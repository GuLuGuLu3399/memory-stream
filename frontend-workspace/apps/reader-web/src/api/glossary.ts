// ────────────────────────────────────────────────────────────────
// glossary.ts — Glossary API
// ────────────────────────────────────────────────────────────────

import { getClient } from './client'

export async function fetchGlossarySlim(): Promise<Record<string, string>> {
  return getClient().get<Record<string, string>>('/glossary/slim')
}
