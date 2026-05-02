
// ────────────────────────────────────────────────────────────────
// card.go — Card model
// card.go — 卡片模型
// ────────────────────────────────────────────────────────────────


package models

import (
	"encoding/json"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

// Card 卡片主表
type Card struct {
	UUID      uuid.UUID       `gorm:"type:uuid;primaryKey" json:"uuid"`                // 卡片唯一标识
	Title     string          `gorm:"type:varchar(255);not null" json:"title"`          // 标题
	Category  string          `gorm:"type:varchar(255);default:''" json:"category"`     // 分类名
	Content   string          `gorm:"type:text;not null" json:"content"`                // Markdown 原文
	AstData   json.RawMessage `gorm:"type:jsonb;not null" json:"ast_data"`              // AST JSON
	TocData   json.RawMessage `gorm:"type:jsonb;default:'[]'" json:"toc_data"`          // 目录 JSON
	Excerpt   string          `gorm:"type:text;default:''" json:"excerpt"`              // 摘要
	Version   int64           `gorm:"not null;default:1" json:"version"`                // 版本号（乐观锁）
	Hash      string          `gorm:"type:varchar(64);not null" json:"hash"`            // 内容哈希
	DeletedAt gorm.DeletedAt  `gorm:"index" json:"deleted_at,omitempty"`                // 软删除时间
	CreatedAt time.Time       `json:"created_at"`                                       // 创建时间
	UpdatedAt time.Time       `json:"updated_at"`                                       // 更新时间
}

func (Card) TableName() string { return "cards" }

// CardListItem — 轻量列表视图，不含 content/ast_data
type CardListItem struct {
	UUID      uuid.UUID `json:"uuid"`
	Title     string    `json:"title"`
	Category  string    `json:"category"`
	Excerpt   string    `json:"excerpt"`
	Version   int64     `json:"version"`
	UpdatedAt time.Time `json:"updated_at"`
}

// BacklinkItem — incoming link to a card
type BacklinkItem struct {
	UUID         uuid.UUID `json:"uuid"`
	Title        string    `json:"title"`
	RelationType string    `json:"relation_type"`
}
