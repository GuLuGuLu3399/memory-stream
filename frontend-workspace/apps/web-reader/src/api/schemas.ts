/**
 * @module schemas
 *
 * Zod 运行时校验 Schema — API 响应类型安全闭环
 *
 * 在 API 数据入口处进行 parse 校验，
 * 将脏数据拦截在渲染层之外，彻底消灭 undefined 导致的白屏。
 */

import { z } from "zod";

// ============================================================================
// 卡片相关
// ============================================================================

/** 卡片指标（热度、浏览量） */
export const CardMetricsSchema = z.object({
  card_id: z.string(),
  view_count: z.number(),
  hot_score: z.number(),
  updated_at: z.string(),
});

/** 包含关联数据的卡片详情 */
export const CardWithRelationsSchema = z.object({
  id: z.string(),
  title: z.string(),
  raw_md: z.string(),
  excerpt: z.string(),
  ast_data: z.preprocess((val) => {
    if (typeof val === "string") {
      try {
        return JSON.parse(val);
      } catch {
        return null;
      }
    }
    return val;
  }, z.record(z.string(), z.unknown()).nullable()),
  toc_data: z.array(z.unknown()).nullable().optional(),
  category_id: z.number().nullable().optional(),
  created_at: z.string(),
  updated_at: z.string(),
  category: z
    .object({
      id: z.number(),
      name: z.string(),
      description: z.string(),
    })
    .nullable()
    .optional(),
  metrics: CardMetricsSchema.nullable(),
});

/** 卡片列表响应（匹配 Go 后端 PaginatedResult 结构） */
export const CardListResponseSchema = z.object({
  data: z.array(CardWithRelationsSchema),
  has_more: z.boolean(),
  next_cursor: z.string().optional(),
  total_count: z.number(),
});

/** 单个卡片响应（复用 CardWithRelationsSchema） */
export const CardDetailResponseSchema = CardWithRelationsSchema;

// ============================================================================
// 图谱相关
// ============================================================================

/** 图谱节点 */
export const GraphNodeSchema = z.object({
  id: z.string(),
  title: z.string(),
});

/** 图谱边 */
export const GraphEdgeSchema = z.object({
  source: z.string(),
  target: z.string(),
  relation: z.string(),
});

/** 图谱查询结果 */
export const GraphResultSchema = z.object({
  nodes: z.array(GraphNodeSchema),
  edges: z.array(GraphEdgeSchema),
});

// ============================================================================
// 大纲相关
// ============================================================================

/** 大纲主题 */
export const OutlineTopicSchema = z.object({
  id: z.string(),
  label: z.string(),
  card_count: z.number(),
});

/** 大纲聚类 */
export const OutlineClusterSchema = z.object({
  id: z.string(),
  title: z.string(),
  topic_id: z.string(),
  created_at: z.string(),
});

/** 大纲查询结果 */
export const OutlineResultSchema = z.object({
  topics: z.array(OutlineTopicSchema),
  clusters: z.array(OutlineClusterSchema),
});

// ============================================================================
// 反向引用（Backlinks）
// ============================================================================

/** 单条反向引用 */
export const BacklinkItemSchema = z.object({
  source_id: z.string(),
  source_title: z.string(),
  relation_type: z.string(),
});

/** 反向引用响应 */
export const BacklinksResponseSchema = z.object({
  card_id: z.string(),
  backlinks: z.array(BacklinkItemSchema),
});

// ============================================================================
// Discover 热门卡片
// ============================================================================

/** Discover 响应 */
export const DiscoverResponseSchema = z.object({
  cards: z.array(CardWithRelationsSchema),
});

export const SearchResultItemSchema = z.object({
  id: z.string(),
  title: z.string(),
  excerpt: z.string(),
  rank: z.number(),
});

export const SearchResponseSchema = z.object({
  results: z.array(SearchResultItemSchema),
  total: z.number(),
  query: z.string(),
});

export type InferredCardMetrics = z.infer<typeof CardMetricsSchema>;
export type InferredCardWithRelations = z.infer<typeof CardWithRelationsSchema>;
export type InferredGraphNode = z.infer<typeof GraphNodeSchema>;
export type InferredGraphEdge = z.infer<typeof GraphEdgeSchema>;
export type InferredGraphResult = z.infer<typeof GraphResultSchema>;
export type InferredOutlineResult = z.infer<typeof OutlineResultSchema>;
export type InferredBacklinksResponse = z.infer<typeof BacklinksResponseSchema>;
export type InferredBacklinkItem = z.infer<typeof BacklinkItemSchema>;
export type InferredDiscoverResponse = z.infer<typeof DiscoverResponseSchema>;
export type InferredSearchResponse = z.infer<typeof SearchResponseSchema>;
