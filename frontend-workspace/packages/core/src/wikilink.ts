// ────────────────────────────────────────────────────────────────
// Wikilink utilities — [[target]] and [[target|alias]] extraction
// Wikilink 工具 — [[target]] 和 [[target|alias]] 的提取
// ────────────────────────────────────────────────────────────────

export interface WikilinkMatch {
  target: string;
  alias: string | null;
  raw: string;
}

const WIKILINK_RE = /\[\[(.+?)(?:\|(.+?))?\]\]/g;

export function extractWikilinks(text: string): WikilinkMatch[] {
  const seen = new Set<string>();
  const results: WikilinkMatch[] = [];

  for (const m of text.matchAll(WIKILINK_RE)) {
    const target = m[1];
    if (!seen.has(target)) {
      seen.add(target);
      results.push({
        target,
        alias: m[2] ?? null,
        raw: m[0],
      });
    }
  }

  return results;
}

export function extractWikilinkTargets(text: string): string[] {
  return extractWikilinks(text).map((l) => l.target);
}

export function replaceWikilinks(
  text: string,
  replacer: (target: string, alias: string | null) => string,
): string {
  return text.replace(WIKILINK_RE, (_match, target, alias) =>
    replacer(target, alias ?? null),
  );
}
