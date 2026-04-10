/**
 * @module api
 *
 * Web Reader 统一 API 客户端 — 请求拦截器封装
 *
 * 基于 fetch 封装的 HTTP 客户端，提供：
 * - 统一 BaseURL 管理（通过环境变量注入）
 * - 请求头自动注入（Content-Type、Authorization Token）
 * - JWT 401 拦截 + Token 无感刷新（请求队列 + 静默重放）
 * - Zod 运行时校验集成
 * - 完整的 TypeScript 类型推导
 *
 * 所有 Web Reader 的后端请求必须通过此模块发出，
 * 禁止在组件或 composable 中直接写 fetch()。
 */

import type { ZodType } from "zod";

// ============================================================================
// 配置
// ============================================================================

/** API 基础 URL，优先从环境变量读取，默认指向本地开发服务器 */
const BASE_URL =
  import.meta.env.VITE_API_BASE_URL || "http://localhost:8080/api/v1";

/** Token 存储 key */
const TOKEN_KEY = "memory_stream_token";
const REFRESH_TOKEN_KEY = "memory_stream_refresh_token";

/** Refresh Token 端点（可通过环境变量覆盖） */
const REFRESH_ENDPOINT =
  import.meta.env.VITE_REFRESH_ENDPOINT || "/auth/refresh";

// ============================================================================
// 请求选项
// ============================================================================

/** 通用请求选项 */
interface RequestOptions {
  /** HTTP 方法，默认 GET */
  method?: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  /** 请求体（自动序列化为 JSON） */
  body?: unknown;
  /** URL 查询参数 */
  params?: Record<string, string>;
  /** 是否需要认证（默认 false，预留） */
  auth?: boolean;
  /** AbortSignal，用于取消请求 */
  signal?: AbortSignal;
  /** Zod Schema，用于运行时校验响应数据 */
  schema?: ZodType;
}

// ============================================================================
// Token 管理
// ============================================================================

/**
 * 获取存储的认证 Token
 *
 * @returns Token 字符串或 null
 */
export function getAuthToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

/**
 * 获取存储的 Refresh Token
 *
 * @returns Refresh Token 字符串或 null
 */
function getRefreshToken(): string | null {
  return localStorage.getItem(REFRESH_TOKEN_KEY);
}

/**
 * 存储认证 Token
 *
 * @param token - JWT Token 字符串
 */
export function setAuthToken(token: string): void {
  localStorage.setItem(TOKEN_KEY, token);
}

/**
 * 存储 Refresh Token
 *
 * @param token - Refresh Token 字符串
 */
export function setRefreshToken(token: string): void {
  localStorage.setItem(REFRESH_TOKEN_KEY, token);
}

/** 清除存储的认证 Token */
export function clearAuthToken(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(REFRESH_TOKEN_KEY);
}

// ============================================================================
// 静默访客登录 — Web Reader 无感初始化
// ============================================================================

/** Guest 账号凭据（可通过环境变量覆盖） */
const GUEST_USERNAME = import.meta.env.VITE_GUEST_USERNAME || "guest";
const GUEST_PASSWORD = import.meta.env.VITE_GUEST_PASSWORD || "guest123";

/** 静默登录是否正在进行中 */
let guestLoginPromise: Promise<void> | null = null;

/**
 * 静默访客登录
 *
 * Web Reader 不需要登录界面。应用启动时自动使用 guest 账号登录，
 * 拿到合法 JWT 后所有后续请求自动携带。
 * - 如果已有有效 Token → 跳过
 * - 如果没有 → 静默 POST /auth/login
 * - 如果 guest 账号不存在（服务端未初始化）→ 静默失败，不阻断 UI
 */
export async function silentGuestLogin(): Promise<void> {
  // 已有 Token，跳过
  if (getAuthToken()) return;

  // 防止并发重复登录
  if (guestLoginPromise) return guestLoginPromise;

  guestLoginPromise = (async () => {
    try {
      const res = await fetch(`${BASE_URL}/auth/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          username: GUEST_USERNAME,
          password: GUEST_PASSWORD,
        }),
      });

      if (!res.ok) {
        console.warn("[Guest Login] 静默登录失败，服务端可能未初始化");
        return;
      }

      const data = await res.json();
      if (data.access_token) setAuthToken(data.access_token);
      if (data.refresh_token) setRefreshToken(data.refresh_token);
    } catch {
      console.warn("[Guest Login] 无法连接到服务器");
    } finally {
      guestLoginPromise = null;
    }
  })();

  return guestLoginPromise;
}

// ============================================================================
// JWT 无感刷新 — 请求队列机制
// ============================================================================

/** 是否正在刷新 Token */
let isRefreshing = false;

/** 暂存队列最大容量，超出时直接拒绝新请求 */
const MAX_PENDING_QUEUE_SIZE = 20;

/** 因 Token 过期而暂存的请求队列 */
let pendingQueue: Array<{
  resolve: (value: unknown) => void;
  reject: (reason: unknown) => void;
  execute: () => Promise<unknown>;
}> = [];

/**
 * 处理队列中的暂存请求
 *
 * 刷新成功后逐一重放；刷新失败则全部 reject。
 */
function processQueue(error: unknown): void {
  pendingQueue.forEach(({ resolve, reject, execute }) => {
    if (error) {
      reject(error);
    } else {
      execute().then(resolve).catch(reject);
    }
  });
  pendingQueue = [];
}

/**
 * 静默刷新 Token
 *
 * 使用 Refresh Token 调用刷新接口，成功后更新存储并重放队列。
 * 绝对不会打断用户心流或强制刷新页面。
 */
async function refreshAuthToken(): Promise<string> {
  const refreshToken = getRefreshToken();

  if (!refreshToken) {
    clearAuthToken();
    throw new Error("No refresh token available");
  }

  const res = await fetch(`${BASE_URL}${REFRESH_ENDPOINT}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ refresh_token: refreshToken }),
  });

  if (!res.ok) {
    clearAuthToken();
    throw new Error(`Refresh failed: ${res.status}`);
  }

  const data = await res.json();
  const newToken = data.access_token || data.token;
  const newRefreshToken = data.refresh_token;

  if (newToken) {
    setAuthToken(newToken);
  }
  if (newRefreshToken) {
    setRefreshToken(newRefreshToken);
  }

  return newToken;
}

// ============================================================================
// 核心请求函数
// ============================================================================

/**
 * 统一 HTTP 请求函数
 *
 * 封装 fetch 调用，自动处理：
 * 1. URL 拼接（BASE_URL + endpoint + query params）
 * 2. 请求头注入（Content-Type、Authorization）
 * 3. 响应状态码拦截（401 → 静默刷新 + 重放）
 * 4. Zod 运行时校验（如果提供了 schema）
 * 5. JSON 自动解析
 *
 * @typeParam T - 响应数据的 TypeScript 类型
 * @param endpoint - API 端点路径（如 "/cards"、"/categories/1"）
 * @param options - 请求选项
 * @returns 解析后的 JSON 响应体
 */
async function request<T>(
  endpoint: string,
  options: RequestOptions = {},
): Promise<T> {
  const { method = "GET", body, params, auth: _auth = false, schema } = options;
  void _auth;

  // 构建 URL（拼接查询参数）
  let url = `${BASE_URL}${endpoint}`;
  if (params) {
    const searchParams = new URLSearchParams(params);
    url += `?${searchParams.toString()}`;
  }

  // 构建请求头
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };

  // Token 注入（当 Token 存在时自动注入）
  const token = getAuthToken();
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  // 构建 fetch 选项
  const fetchOptions: RequestInit = {
    method,
    headers,
    signal: options.signal,
  };
  if (body && method !== "GET") {
    fetchOptions.body = JSON.stringify(body);
  }

  // 发起请求
  const res = await fetch(url, fetchOptions);

  // ── 401 拦截：静默刷新 Token 或 重新访客登录 ──
  if (res.status === 401) {
    // 如果没有 refresh token，尝试静默访客登录
    if (!getRefreshToken()) {
      await silentGuestLogin();
      if (getAuthToken()) {
        return request<T>(endpoint, { ...options, signal: undefined });
      }
    }

    if (!getRefreshToken()) {
      throw new Error("Authentication failed");
    }
    // 如果已经在刷新中，将当前请求加入等待队列
    if (isRefreshing) {
      return new Promise<T>((resolve, reject) => {
        if (pendingQueue.length >= MAX_PENDING_QUEUE_SIZE) {
          reject(new Error("Too many pending requests, please try again"));
          return;
        }
        pendingQueue.push({
          resolve: resolve as (value: unknown) => void,
          reject,
          execute: () =>
            request<T>(endpoint, { ...options, signal: undefined }),
        });
      });
    }

    // 开始刷新流程
    isRefreshing = true;

    try {
      await refreshAuthToken();
      // 刷新成功 → 重放队列
      processQueue(null);
      // 重放当前请求
      return request<T>(endpoint, { ...options, signal: undefined });
    } catch (refreshError) {
      // 刷新失败 → 拒绝队列中的所有请求
      processQueue(refreshError);
      throw refreshError;
    } finally {
      isRefreshing = false;
    }
  }

  // HTTP 状态码拦截（非 2xx）
  if (!res.ok) {
    let errorMsg = `API Error: ${res.status} ${res.statusText}`;
    try {
      const errorBody = await res.json();
      if (errorBody.message) {
        errorMsg = `API Error ${res.status}: ${errorBody.message}`;
      }
    } catch {
      // JSON 解析失败，使用默认错误消息
    }
    throw new Error(errorMsg);
  }

  // 解析 JSON
  const data: T = await res.json();

  // ── Zod 运行时校验 ──
  if (schema) {
    return schema.parse(data) as T;
  }

  return data;
}

// ============================================================================
// Web Reader 扩展类型（包含后端关联数据）
// ============================================================================

/** 卡片指标（热度、浏览量） */
export interface CardMetrics {
  card_id: string;
  view_count: number;
  hot_score: number;
  updated_at: string;
}

/** 包含关联数据的卡片详情（后端返回的完整结构） */
/** AST 数据结构 — 解析后的 JSON 对象 */
export interface AstData {
  children?: AstData[];
  [key: string]: unknown;
}

export interface CardWithRelations {
  id: string;
  title: string;
  raw_md: string;
  excerpt: string;
  ast_data: AstData | null;
  category_id?: number | null;
  created_at: string;
  updated_at: string;
  category?: { id: number; name: string; description: string } | null;
  metrics: CardMetrics | null;
}

/** 图谱节点（匹配后端 JSON 响应格式） */
export interface GraphNode {
  id: string;
  title: string;
}

/** 图谱边（匹配后端 JSON 响应格式：source/target/relation） */
export interface GraphEdge {
  source: string;
  target: string;
  relation: string;
}

/** 图谱查询结果 */
export interface GraphResult {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

/** 大纲主题 */
export interface OutlineTopic {
  id: string;
  label: string;
  card_count: number;
}

/** 大纲聚类 */
export interface OutlineCluster {
  id: string;
  title: string;
  topic_id: string;
  created_at: string;
}

/** 大纲查询结果 */
export interface OutlineResult {
  topics: OutlineTopic[];
  clusters: OutlineCluster[];
}

// ============================================================================
// API 方法集合
// ============================================================================

/**
 * Web Reader API 方法集合
 *
 * 所有方法返回 Promise，自动处理错误和类型推导。
 * 支持传入 Zod Schema 进行运行时校验。
 *
 * @example
 * ```typescript
 * import { api } from '@/api';
 * import { CardListResponseSchema } from '@/api/schemas';
 *
 * // 带 Zod 校验的请求
 * const data = await api.listCards(CardListResponseSchema);
 * ```
 */
export const api = {
  // ---- 卡片 ----

  /** 获取卡片列表（匹配 Go 后端 PaginatedResult 结构） */
  listCards: <S extends ZodType | undefined = undefined>(schema?: S) =>
    request<{
      data: CardWithRelations[];
      has_more: boolean;
      next_cursor?: string;
      total_count: number;
    }>("/cards", schema ? { schema } : undefined),

  /** 获取单个卡片详情（包含完整 raw_md 和 AST 数据） */
  getCard: (id: string, options?: RequestOptions) =>
    request<CardWithRelations>(`/cards/${id}`, options),

  /** 获取 Discover 热门卡片（支持排序和分页） */
  discoverCards: (sort = "hot", page = 1, pageSize = 20) =>
    request<{ cards: CardWithRelations[] }>("/cards/discover", {
      params: { sort, page: String(page), page_size: String(pageSize) },
    }),

  // ---- 图谱 ----

  /** 获取指定卡片的图谱（含邻居节点和边） */
  getGraph: (cardId: string, depth = 2) =>
    request<GraphResult>(`/cards/${cardId}/graph`, {
      params: { depth: String(depth) },
    }),

  /** 获取全量图谱数据（所有节点 + 所有边，含孤岛） */
  getFullGraph: () => request<GraphResult>("/graph/all"),

  /** 获取大纲视图数据（Topic + Cluster 层级结构） */
  getOutline: (categoryId?: string) =>
    request<OutlineResult>("/graph/outline", {
      params: categoryId ? { category_id: categoryId } : undefined,
    }),

  /** 获取局部图谱详情（以指定卡片为中心） */
  getDetail: (cardId: string, depth = 2) =>
    request<GraphResult>(`/graph/detail/${cardId}`, {
      params: { depth: String(depth) },
    }),

  // ---- 反向引用 ----

  /** 获取指向当前卡片的所有反向引用（Backlinks） */
  getBacklinks: (cardId: string) =>
    request<{
      card_id: string;
      backlinks: Array<{
        source_id: string;
        source_title: string;
        relation_type: string;
        context_snippet?: string;
      }>;
    }>(`/cards/${cardId}/backlinks`),

  // ---- 搜索 ----

  /** 搜索卡片（全文搜索） */
  searchCards: (query: string, limit = 20, offset = 0) =>
    request<{
      results: Array<{
        id: string;
        title: string;
        excerpt: string;
        rank: number;
      }>;
      total: number;
      query: string;
    }>("/search", {
      params: { q: query, limit: String(limit), offset: String(offset) },
    }),
};
