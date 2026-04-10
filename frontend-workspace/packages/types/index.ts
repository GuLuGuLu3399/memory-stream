/**
 * @memory-stream/types — 全栈共享 TypeScript 类型字典
 *
 * 本文件是 memory-stream 项目的唯一类型真相源 (Single Source of Truth)。
 * Go 后端的 JSON Tag 对应此处的字段名，任何模型变更必须同步修改。
 *
 * Tauri IPC 类型由 ts-rs 从 Rust 结构体自动生成，见 ./ipc.ts
 *
 * @module @memory-stream/types
 */

// ============================================================================
// Tauri IPC 类型（ts-rs 自动生成）
// ============================================================================

export * as IpcTypes from './ipc.js';

// ============================================================================
// 基础类型
// ============================================================================

/** ISO 8601 日期字符串 (e.g. "2024-01-15T08:30:00Z") */
export type ISOTimestamp = string;

/** UUID v4 字符串 */
export type UUID = string;

// ============================================================================
// 卡片 (Card)
// ============================================================================

/**
 * 知识卡片 — 项目的核心实体
 *
 * 一张 Card 对应一段 Markdown 知识片段，包含原文、AST 结构化数据，
 * 以及分类归属。通过 Edge 与其他 Card 建立图谱关系。
 */
export interface Card {
  /** 卡片唯一标识 (UUID) */
  id: UUID;
  /** 卡片标题 */
  title: string;
  /** 摘要文本，用于列表展示 */
  excerpt: string;
  /** 原始 Markdown 内容 */
  raw_md?: string;
  /** AST 结构化 JSON 字符串 */
  ast_data?: string;
  /** 所属分类 ID，未分类时为 null */
  category_id: number | null;
  /** 创建时间 */
  created_at: ISOTimestamp;
  /** 最后更新时间 */
  updated_at: ISOTimestamp;
}

/** 创建卡片请求载荷 */
export interface CreateCardPayload {
  title: string;
  raw_md: string;
  category_id?: number | null;
}

/** 更新卡片请求载荷 */
export interface UpdateCardPayload {
  id: UUID;
  title?: string;
  raw_md?: string;
  ast_data?: string;
  category_id?: number | null;
}

// ============================================================================
// 边 / 关系 (Edge)
// ============================================================================

/** 关系类型枚举 */
export type RelationType = "sequence" | "reference";

/**
 * 知识图谱边 — 描述两张卡片之间的关系
 *
 * - sequence: 时序关系（A 在 B 之前）
 * - reference: 引用关系（A 引用了 B）
 */
export interface Edge {
  /** 边唯一标识 */
  id: number;
  /** 起始卡片 UUID */
  source_id: UUID;
  /** 目标卡片 UUID */
  target_id: UUID;
  /** 关系类型 */
  relation_type: RelationType;
  /** 创建时间 */
  created_at: ISOTimestamp;
}

/** 创建边请求载荷 */
export interface CreateEdgePayload {
  source_id: UUID;
  target_id: UUID;
  relation_type: RelationType;
}

// ============================================================================
// 分类 (Category)
// ============================================================================

/**
 * 知识分类 — 用于组织卡片
 *
 * 支持树形结构，通过 parent_id 构建层级关系。
 */
export interface Category {
  /** 分类唯一标识 */
  id: number;
  /** 分类名称 */
  name: string;
  /** 分类描述 */
  description?: string;
  /** 父分类 ID，顶级分类为 null */
  parent_id: number | null;
  /** 创建时间 */
  created_at: ISOTimestamp;
  /** 最后更新时间 */
  updated_at: ISOTimestamp;
  /** 主题色 key (e.g. "cyan", "orange") — mapped via THEME_DICT on frontend */
  theme_color?: string | null;
}

/** 创建分类请求载荷 */
export interface CreateCategoryPayload {
  name: string;
  parent_id?: number | null;
}

// ============================================================================
// 图谱可视化 (Graph)
// ============================================================================

/**
 * 图谱节点 — 用于前端图可视化渲染
 *
 * 是 Card 在图谱视图中的精简表示，只保留渲染所需的字段。
 */
export interface GraphNode {
  /** 节点唯一标识 (对应 Card.id) */
  id: UUID;
  /** 节点标签 (对应 Card.title) */
  label: string;
  /** 所属分类 ID，用于节点着色 */
  category_id: number | null;
}

/**
 * 图谱边 — 用于前端图可视化渲染
 *
 * 是 Edge 在图谱视图中的精简表示。
 */
export interface GraphEdge {
  /** 起始节点 UUID */
  source_id: UUID;
  /** 目标节点 UUID */
  target_id: UUID;
  /** 关系类型 */
  relation_type: RelationType;
}

/**
 * 完整图谱数据 — 一次查询返回的节点 + 边集合
 */
export interface GraphData {
  /** 所有节点 */
  nodes: GraphNode[];
  /** 所有边 */
  edges: GraphEdge[];
}

// ============================================================================
// API 通用响应
// ============================================================================

/**
 * Go 后端统一响应格式
 *
 * 所有 API 端点均返回此结构，前端通过 code 判断业务状态。
 */
export interface ApiResponse<T> {
  /** 业务状态码，0 表示成功 */
  code: number;
  /** 响应消息 */
  message: string;
  /** 响应数据 */
  data: T;
}

// ============================================================================
// 分页响应
// ============================================================================

/** 分页响应 — Go PaginatedResult 映射 */
export interface PaginatedResponse<T> {
  data: T[];
  has_more: boolean;
  total_count: number;
  next_cursor?: string;
}

/** 卡片列表项（API 响应）— Card 精简表示，含图谱坐标 */
export interface CardListItem {
  id: UUID;
  title: string;
  excerpt?: string;
  raw_md?: string;
  x: number;
  y: number;
  updated_at: string;
  category_id: number | null;
}

// ============================================================================
// 渲染引擎 (与 ui-shared 对齐)
// ============================================================================

/** Markdown 渲染结果 */
export interface RenderResult {
  html: string;
  ast_json: string;
}

/** 解析引擎函数签名 */
export type ParseEngine = (markdown: string) => Promise<RenderResult>;

/** 保存事件回调参数 */
export interface SavePayload {
  rawMd: string;
  astJson: string;
}

// ============================================================================
// Tauri IPC 请求载荷
// ============================================================================

/**
 * Rust api_request 命令参数
 *
 * 通过 Tauri invoke 调用时传递的参数结构，
 * Rust 侧根据 method 构建对应的 HTTP 请求。
 */
export interface ApiRequestParams {
  /** HTTP 方法: GET | POST | PUT | DELETE */
  method: string;
  /** API 端点路径 (e.g. "/api/v1/cards") */
  endpoint: string;
  /** 请求体 JSON (POST/PUT 时使用) */
  body?: unknown;
}
