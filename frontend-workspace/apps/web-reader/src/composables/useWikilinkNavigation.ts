import { api } from "../api";

const UUID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

const titleToIdCache = new Map<string, string>();
const pendingResolutions = new Map<string, Promise<string | null>>();

function isUuid(value: string): boolean {
  return UUID_RE.test(value);
}

function normalizeTarget(target: string): string {
  return decodeURIComponent(target).trim();
}

async function resolveByTitle(title: string): Promise<string | null> {
  const cached = titleToIdCache.get(title);
  if (cached) return cached;

  const existingPending = pendingResolutions.get(title);
  if (existingPending) return existingPending;

  const task = (async () => {
    try {
      const res = await api.searchCards(title, 20, 0);
      const exact =
        res.results.find((item) => item.title.trim() === title) ||
        res.results.find(
          (item) => item.title.trim().toLowerCase() === title.toLowerCase(),
        ) ||
        res.results[0];

      if (!exact?.id) return null;

      titleToIdCache.set(title, exact.id);
      return exact.id;
    } catch {
      return null;
    } finally {
      pendingResolutions.delete(title);
    }
  })();

  pendingResolutions.set(title, task);
  return task;
}

export async function resolveWikilinkTarget(
  rawTarget: string,
): Promise<string | null> {
  const target = normalizeTarget(rawTarget);
  if (!target) return null;

  if (isUuid(target)) return target;
  return resolveByTitle(target);
}
