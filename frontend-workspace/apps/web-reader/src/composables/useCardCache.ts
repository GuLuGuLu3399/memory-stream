/**
 * 🌟 useCardCache — WASM 渲染结果 LRU 缓存
 *
 * 避免重复 WASM 渲染同一卡片。
 * 缓存容量 30 张，LRU 策略自动淘汰最久未访问的条目。
 */

import type { CardDetail } from "./useCards";

const MAX_CACHE = 30;
const cache = new Map<string, CardDetail>();

/**
 * 查询缓存，命中时自动提升到最新位置（LRU）。
 */
export function getCached(id: string): CardDetail | null {
  if (cache.has(id)) {
    const item = cache.get(id)!;
    // LRU: 删除后重新插入，移到末尾
    cache.delete(id);
    cache.set(id, item);
    return item;
  }
  return null;
}

/**
 * 写入缓存，超过容量时淘汰最老的条目。
 */
export function setCache(id: string, detail: CardDetail): void {
  if (cache.size >= MAX_CACHE) {
    const firstKey = cache.keys().next().value;
    if (firstKey) cache.delete(firstKey);
  }
  cache.set(id, detail);
}

/**
 * 清除全部缓存。
 */
export function clearCache(): void {
  cache.clear();
}

/**
 * 获取当前缓存大小（调试用）。
 */
export function getCacheSize(): number {
  return cache.size;
}
