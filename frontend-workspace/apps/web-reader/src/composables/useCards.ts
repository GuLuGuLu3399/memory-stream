/**
 * 🌟 useCards — 脉冲索引流 + 抽屉详情分离
 *
 * 列表阶段：只拉轻量索引（title, excerpt, hot_score），不做 WASM 渲染
 * 详情阶段：抽屉打开时按需加载单个卡片并 WASM 渲染
 */

import { ref, shallowRef } from "vue";
import { api, type CardWithRelations, type GraphResult } from "../api";
import {
  CardListResponseSchema,
  GraphResultSchema,
  CardDetailResponseSchema,
} from "../api/schemas";
import { getCached, setCache } from "./useCardCache";

// ── RawMd 预处理：数据库中存储的是 \n 字面量，需转为真正的换行符 ──
function decodeRawMd(raw: string): string {
  return raw.replace(/\\n/g, "\n").replace(/\\t/g, "\t").replace(/\\r/g, "\r");
}

// WASM 引擎懒加载
let wasmReady = false;
let renderFromAst: ((astJson: string) => string) | null = null;
let processMarkdown:
  | ((rawMd: string) => { html: string; ast_json: string })
  | null = null;

async function ensureWasm() {
  if (wasmReady) return;
  const initWasm = (await import("wasm-engine")).default;
  const mod = await import("wasm-engine");
  await initWasm();
  renderFromAst = mod.render_from_ast;
  processMarkdown = mod.process_markdown;
  wasmReady = true;
}

/** 轻量卡片索引（列表展示用，不含 raw_md 和 ast_data） */
export interface CardIndex {
  id: string;
  title: string;
  excerpt: string;
  hot_score: number;
  updated_at: string;
  relation: "sequence" | "reference";
}

/** TOC 目录树节点 */
export interface TocItem {
  level: number;
  text: string;
  slug: string;
  children: TocItem[];
}

/** 卡片详情（抽屉展示用，含 WASM 渲染后的 HTML） */
export interface CardDetail {
  id: string;
  title: string;
  html: string;
  rawMd: string;
  updatedAt: string;
  tocData: TocItem[] | null;
}

/**
 * 从 RawMd 截取前 N 个字符作为纯文本 excerpt。
 *
 * 处理流程：去除 YAML frontmatter → 跳过标题行 → 去除 Markdown 语法 → 截断
 *
 * @param raw - 原始 Markdown 文本
 * @param maxLen - 最大字符长度，默认 80
 * @returns 纯文本摘要字符串
 */
function extractExcerpt(raw: string, maxLen = 80): string {
  if (!raw) return "";

  // 处理可能的字面量 \n（数据库中存储的转义换行）
  let text = decodeRawMd(raw);

  // 去掉 YAML frontmatter
  text = text.replace(/^---[\s\S]*?---\n?/, "");

  // 提取第一个有实质内容的段落（跳过标题）
  const lines = text.split("\n");
  const contentLines: string[] = [];
  for (const line of lines) {
    const trimmed = line.trim();
    // 跳过空行和标题行
    if (!trimmed || /^#{1,6}\s/.test(trimmed)) continue;
    // 跳过列表标记，取内容
    contentLines.push(
      trimmed.replace(/^[-*+]\s*/, "").replace(/^\d+\.\s*/, ""),
    );
    if (contentLines.join(" ").length >= maxLen) break;
  }

  // 去掉残留的 Markdown 语法
  let plain = contentLines
    .join(" ")
    .replace(/\*\*(.*?)\*\*/g, "$1")
    .replace(/\*(.*?)\*/g, "$1")
    .replace(/`{1,3}([^`]*)`{1,3}/g, "$1")
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/\[([^\]]*)\]\[[^\]]*\]/g, "$1")
    .trim();

  return plain.length > maxLen ? plain.slice(0, maxLen) + "..." : plain;
}

/**
 * useCards — 卡片数据管理 Composable。
 *
 * 提供「轻量索引 + 抽屉详情」分离加载策略：
 * - `loadIndex()`: 批量拉取卡片列表（不含 raw_md），用于脉冲流展示
 * - `loadDetail(id)`: 按需加载单卡详情，优先使用 AST 渲染，降级到完整管线
 *
 * @returns 卡片索引列表、加载状态、错误信息、加载方法
 */
export function useCards() {
  const cardIndex = shallowRef<CardIndex[]>([]);
  const loading = ref(false);
  const error = ref("");

  // ─── 列表：轻量索引（含真实边关系） ───
  async function loadIndex() {
    loading.value = true;
    error.value = "";

    try {
      const rawData = await api.listCards();

      // ── Zod 运行时校验：拦截脏数据 ──
      const data = CardListResponseSchema.parse(rawData);

      if (data.data) {
        // Fetch root graph to get real edge relationships
        let edgeMap = new Map<string, string>(); // cardId → relation
        try {
          const rawGraph: GraphResult = await api.getGraph("root", 3);
          const graph = GraphResultSchema.parse(rawGraph);
          for (const edge of graph.edges) {
            // Use the first relation found for each card
            if (!edgeMap.has(edge.target)) {
              edgeMap.set(edge.target, edge.relation);
            }
            if (!edgeMap.has(edge.source)) {
              edgeMap.set(edge.source, edge.relation);
            }
          }
        } catch {
          // Graph fetch failed — default all to reference
        }

        cardIndex.value = data.data.map((card: CardWithRelations) => ({
          id: card.id,
          title: card.title || "无标题",
          excerpt: card.excerpt || extractExcerpt(card.raw_md),
          hot_score: card.metrics?.hot_score ?? 0,
          updated_at: card.updated_at,
          relation: (edgeMap.get(card.id) || "reference") as
            | "sequence"
            | "reference",
        }));
      }
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      console.error("加载卡片索引失败:", message);
      error.value = message;
    } finally {
      loading.value = false;
    }
  }

  // ─── AbortController 去重 ───
  let detailAbort: AbortController | null = null;

  // ─── 详情：LRU 缓存 → AstData 优先 → RawMd 降级 ───
  async function loadDetail(id: string): Promise<CardDetail | null> {
    // ① LRU 缓存命中
    const cached = getCached(id);
    if (cached) return cached;

    // ② 取消前一个未完成的请求
    if (detailAbort) detailAbort.abort();
    detailAbort = new AbortController();
    const { signal } = detailAbort;

    try {
      await ensureWasm();
      const rawCard = await api.getCard(id, { signal });

      // ── Zod 运行时校验：拦截脏数据 ──
      const card = CardDetailResponseSchema.parse(rawCard);

      let html: string;
      const decodedMd = decodeRawMd(card.raw_md);

      // 🌟 优先路径：AstData → render_from_ast（跳过解析，零开销）
      const astData = card.ast_data;
      if (
        astData &&
        renderFromAst &&
        Array.isArray((astData as Record<string, unknown>).children) &&
        ((astData as Record<string, unknown>).children as unknown[]).length > 0
      ) {
        html = renderFromAst(JSON.stringify(astData));
      }
      // 降级路径：RawMd → process_markdown（完整管线）
      else if (processMarkdown) {
        html = processMarkdown(decodedMd).html;
      } else {
        return null;
      }

      // 从后端持久化的 toc_data 读取（无需运行时重算）
      const tocData: TocItem[] | null =
        (card.toc_data as TocItem[] | null) ?? null;

      const result: CardDetail = {
        id: card.id,
        title: card.title || "无标题",
        html,
        rawMd: decodedMd,
        updatedAt: card.updated_at,
        tocData,
      };

      // ③ 写入 LRU 缓存
      setCache(id, result);
      return result;
    } catch (err) {
      console.error("加载卡片详情失败:", err);
      return null;
    }
  }

  return { cardIndex, loading, error, loadIndex, loadDetail };
}
