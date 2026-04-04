/**
 * 📦 useOfflineCache — IndexedDB 离线缓存
 *
 * 将最近浏览的卡片摘要持久化到 IndexedDB，
 * 离线时可作为降级数据源。
 *
 * 设计：
 * - 存储 100 条最近卡片摘要（id / title / excerpt / updated_at）
 * - Promise 化 IndexedDB 操作
 * - 自动清理超出上限的旧数据
 */

const DB_NAME = "memory-stream-offline";
const DB_VERSION = 1;
const STORE_NAME = "card-summaries";
const MAX_RECORDS = 100;

export interface CachedCardSummary {
  id: string;
  title: string;
  excerpt: string;
  updated_at: string;
  cached_at: number; // Date.now()
}

function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        const store = db.createObjectStore(STORE_NAME, { keyPath: "id" });
        store.createIndex("cached_at", "cached_at", { unique: false });
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

/**
 * 写入或更新一条缓存
 */
export async function cacheCardSummary(card: CachedCardSummary): Promise<void> {
  const db = await openDB();
  const tx = db.transaction(STORE_NAME, "readwrite");
  const store = tx.objectStore(STORE_NAME);

  store.put({ ...card, cached_at: Date.now() });

  tx.oncomplete = () => {
    // 写入后检查总数，异步清理
    trimIfNeeded(db);
  };
}

/**
 * 批量写入缓存
 */
export async function cacheCardSummaries(
  cards: CachedCardSummary[],
): Promise<void> {
  const db = await openDB();
  const tx = db.transaction(STORE_NAME, "readwrite");
  const store = tx.objectStore(STORE_NAME);
  const now = Date.now();

  for (const card of cards) {
    store.put({ ...card, cached_at: now });
  }

  tx.oncomplete = () => {
    trimIfNeeded(db);
  };
}

/**
 * 获取单条缓存
 */
export async function getCachedSummary(
  id: string,
): Promise<CachedCardSummary | null> {
  const db = await openDB();
  const tx = db.transaction(STORE_NAME, "readonly");
  const store = tx.objectStore(STORE_NAME);

  return new Promise((resolve, reject) => {
    const request = store.get(id);
    request.onsuccess = () => resolve(request.result ?? null);
    request.onerror = () => reject(request.error);
  });
}

/**
 * 获取所有缓存（按 cached_at 降序，最新的在前）
 */
export async function getAllCached(): Promise<CachedCardSummary[]> {
  const db = await openDB();
  const tx = db.transaction(STORE_NAME, "readonly");
  const store = tx.objectStore(STORE_NAME);

  return new Promise((resolve, reject) => {
    const request = store.getAll();
    request.onsuccess = () => {
      const results = (request.result as CachedCardSummary[]).sort(
        (a, b) => b.cached_at - a.cached_at,
      );
      resolve(results);
    };
    request.onerror = () => reject(request.error);
  });
}

/**
 * 获取缓存条数
 */
export async function getCachedCount(): Promise<number> {
  const db = await openDB();
  const tx = db.transaction(STORE_NAME, "readonly");
  const store = tx.objectStore(STORE_NAME);

  return new Promise((resolve, reject) => {
    const request = store.count();
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

/**
 * 清除所有离线缓存
 */
export async function clearOfflineCache(): Promise<void> {
  const db = await openDB();
  const tx = db.transaction(STORE_NAME, "readwrite");
  tx.objectStore(STORE_NAME).clear();
}

/**
 * 超出上限时，删除最旧的记录
 */
function trimIfNeeded(db: IDBDatabase): void {
  const tx = db.transaction(STORE_NAME, "readwrite");
  const store = tx.objectStore(STORE_NAME);
  const index = store.index("cached_at");

  const countRequest = store.count();
  countRequest.onsuccess = () => {
    const excess = countRequest.result - MAX_RECORDS;
    if (excess <= 0) return;

    // 打开游标，删除最旧的 excess 条
    const cursorRequest = index.openCursor();
    let deleted = 0;
    cursorRequest.onsuccess = () => {
      const cursor = cursorRequest.result;
      if (cursor && deleted < excess) {
        cursor.delete();
        deleted++;
        cursor.continue();
      }
    };
  };
}
