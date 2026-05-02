// Package models ────────────────────────────────────────────────────────────────
// sync.go — 同步负载和图结构的模型
// ────────────────────────────────────────────────────────────────
package models

import (
	"encoding/json"
	"time"
)

// ── Pull（拉取清单）────────────────────────────────────────────

// CardWithEdges 卡片详情 + 出边关系（GetCard 响应）
type CardWithEdges struct {
	UUID      string            `json:"uuid"`
	Title     string            `json:"title"`
	Category  string            `json:"category"`
	Content   string            `json:"content"`
	AstData   json.RawMessage   `json:"ast_data"`
	TocData   json.RawMessage   `json:"toc_data"`
	Excerpt   string            `json:"excerpt"`
	Version   int64             `json:"version"`
	Hash      string            `json:"hash"`
	CreatedAt time.Time         `json:"created_at"`
	UpdatedAt time.Time         `json:"updated_at"`
	Edges     []SyncEdgePayload `json:"edges"`
}

// SyncManifestResponse 拉取清单响应
type SyncManifestResponse struct {
	Cursor  time.Time          `json:"cursor"`  // 增量游标（上次同步时间戳）
	Changes []SyncManifestItem `json:"changes"` // 变更条目列表
}

// SyncManifestItem 清单中的单个变更条目
type SyncManifestItem struct {
	UUID      string    `json:"uuid"`       // 卡片 UUID
	Version   int64     `json:"version"`    // 当前版本号
	Hash      string    `json:"hash"`       // 内容哈希
	UpdatedAt time.Time `json:"updated_at"` // 最后更新时间
	IsDeleted bool      `json:"is_deleted"` // 是否已软删除
}

// ── Push（批量推送）────────────────────────────────────────────

// SyncBatchRequest 批量推送请求
type SyncBatchRequest struct {
	Cards []SyncCardPayload `json:"cards" binding:"required"` // 待推送的卡片列表
}

// SyncCardPayload 单张卡片的推送负载
type SyncCardPayload struct {
	UUID     string            `json:"uuid" binding:"required"` // 卡片 UUID
	Title    string            `json:"title" binding:"required"` // 标题
	Category string            `json:"category"`                // 分类名
	Content  string            `json:"content" binding:"required"` // Markdown 原文
	AstData  json.RawMessage   `json:"ast_data"`                // AST JSON
	TocData  json.RawMessage   `json:"toc_data"`                // 目录 JSON
	Excerpt  string            `json:"excerpt"`                 // 摘要
	Version  int64             `json:"version"`                 // 客户端版本号
	Hash     string            `json:"hash" binding:"required"` // 内容哈希
	Edges    []SyncEdgePayload `json:"edges"`                   // 关联边列表
}

// SyncEdgePayload 卡片间的边关系负载
type SyncEdgePayload struct {
	TargetUUID   string `json:"target_uuid"`                   // 目标卡片 UUID
	RelationType string `json:"relation_type"` // 关系类型：trunk | link
}

// SyncBatchResponse 批量推送响应
type SyncBatchResponse struct {
	Accepted  []SyncAcceptedItem `json:"accepted"`  // 已接受的卡片
	Conflicts []SyncConflict     `json:"conflicts"` // 版本冲突的卡片
	Rejected  []SyncReject       `json:"rejected"`  // 被拒绝的卡片
}

// SyncAcceptedItem 已接受的卡片结果
type SyncAcceptedItem struct {
	UUID      string    `json:"uuid"`       // 卡片 UUID
	Version   int64     `json:"version"`    // 服务端新版本号
	UpdatedAt time.Time `json:"updated_at"` // 服务端更新时间
}

// SyncConflict 版本冲突详情
type SyncConflict struct {
	UUID          string `json:"uuid"`           // 冲突卡片 UUID
	ServerVersion int64  `json:"server_version"` // 服务端版本号
	ServerHash    string `json:"server_hash"`    // 服务端内容哈希
}

// SyncReject 拒绝原因
type SyncReject struct {
	UUID   string `json:"uuid"`   // 被拒绝的卡片 UUID
	Reason string `json:"reason"` // 拒绝原因
}

// ── Trunk Sync（Trunk 独立同步）────────────────────────────────────

// RelationsSyncRequest 全量 trunk 关系同步请求
type RelationsSyncRequest struct {
	Relations []RelationPayload `json:"relations" binding:"required"`
}

// RelationPayload 单条关系负载
type RelationPayload struct {
	SourceUUID   string `json:"source_uuid" binding:"required"`
	TargetUUID   string `json:"target_uuid" binding:"required"`
	RelationType string `json:"relation_type" binding:"required"`
}

// RelationsSyncResponse 关系同步响应
type RelationsSyncResponse struct {
	Accepted int `json:"accepted"`
}

// ── Graph（图结构）─────────────────────────────────────────────

// GraphNode 图节点（轻量卡片表示）
type GraphNode struct {
	ID    string `json:"id"`    // 节点 ID（即卡片 UUID）
	Title string `json:"title"` // 节点标题
}

// GraphEdge 图中的有向边
type GraphEdge struct {
	Source       string `json:"source"`                   // 源节点 ID
	Target       string `json:"target"`                   // 目标节点 ID
	RelationType string `json:"relation"` // 关系类型：trunk | link
}

// FullGraph 完整图结构
type FullGraph struct {
	Nodes []GraphNode `json:"nodes"` // 全部节点
	Edges []GraphEdge `json:"edges"` // 全部边
}

// ── Glossary Sync（术语表同步）──────────────────────────────────

// GlossarySyncManifest 术语表拉取清单响应
type GlossarySyncManifest struct {
	Cursor  time.Time              `json:"cursor"`  // 增量游标
	Changes []GlossaryManifestItem `json:"changes"` // 变更条目列表
}

// GlossaryManifestItem 术语表清单中的单个变更条目
type GlossaryManifestItem struct {
	Term      string    `json:"term"`       // 术语名
	Version   int64     `json:"version"`    // 当前版本号
	Hash      string    `json:"hash"`       // 内容哈希
	UpdatedAt time.Time `json:"updated_at"` // 最后更新时间
	IsDeleted bool      `json:"is_deleted"` // 是否已软删除
}

// GlossaryBatchRequest 术语表批量推送请求
type GlossaryBatchRequest struct {
	Items []GlossaryItemPayload `json:"items" binding:"required"` // 待推送的术语列表
}

// GlossaryItemPayload 单条术语的推送负载
type GlossaryItemPayload struct {
	Term       string `json:"term" binding:"required"`       // 术语名
	Definition string `json:"definition" binding:"required"` // 定义
	Version    int64  `json:"version"`                       // 客户端版本号
	Hash       string `json:"hash" binding:"required"`       // 内容哈希
}

// GlossaryBatchResponse 术语表批量推送响应
type GlossaryBatchResponse struct {
	Accepted  []string           `json:"accepted"`  // 已接受的术语名列表
	Conflicts []GlossaryConflict `json:"conflicts"` // 版本冲突的术语
	Rejected  []GlossaryReject   `json:"rejected"`  // 被拒绝的术语
}

// GlossaryConflict 术语表版本冲突详情
type GlossaryConflict struct {
	Term          string `json:"term"`           // 冲突术语名
	ServerVersion int64  `json:"server_version"` // 服务端版本号
	ServerHash    string `json:"server_hash"`    // 服务端内容哈希
}

// GlossaryReject 术语表拒绝原因
type GlossaryReject struct {
	Term   string `json:"term"`   // 被拒绝的术语名
	Reason string `json:"reason"` // 拒绝原因
}
