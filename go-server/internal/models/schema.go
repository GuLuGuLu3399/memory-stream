package models

import (
	"database/sql/driver"
	"encoding/json"
	"time"
)

type JSONB json.RawMessage

func (j *JSONB) Value() (driver.Value, error) {
	if j == nil || len(*j) == 0 {
		return nil, nil
	}
	return []byte(*j), nil
}

func (j *JSONB) Scan(value interface{}) error {
	if value == nil {
		*j = nil
		return nil
	}
	bytes, ok := value.([]byte)
	if !ok {
		return json.Unmarshal(value.([]uint8), j)
	}
	*j = append((*j)[0:0], bytes...)
	return nil
}

func (j *JSONB) MarshalJSON() ([]byte, error) {
	if j == nil || len(*j) == 0 {
		return []byte("null"), nil
	}
	return *j, nil
}

func (j *JSONB) UnmarshalJSON(data []byte) error {
	*j = append((*j)[0:0], data...)
	return nil
}

type Category struct {
	ID          uint      `json:"id" gorm:"primaryKey"`
	Name        string    `json:"name" gorm:"type:varchar(100);uniqueIndex;not null"`
	Description string    `json:"description" gorm:"type:text"`
	CreatedAt   time.Time `json:"created_at"`
	// ParentID defines the hierarchical relationship to another category
	// Nullable: root categories have nil parent
	ParentID *uint `json:"parent_id" gorm:"index"`
	// SortOrder defines the display order among siblings
	SortOrder int `json:"sort_order" gorm:"default:0"`
	// ThemeColor is a preset key like "cyan", "orange" — mapped to styles on frontend
	ThemeColor *string `json:"theme_color" gorm:"type:varchar(20)"`
}

// CategoryTreeNode represents a category in a hierarchical tree structure
type CategoryTreeNode struct {
	ID          uint               `json:"id"`
	Name        string             `json:"name"`
	Description string             `json:"description"`
	ParentID    *uint              `json:"parent_id"`
	SortOrder   int                `json:"sort_order"`
	CreatedAt   time.Time          `json:"created_at"`
	ThemeColor  *string            `json:"theme_color"`
	Children    []CategoryTreeNode `json:"children"`
}

// CardEdge 卡片关系表 - 支撑双线脊椎图谱 (DAG)
//
// 索引说明（由 migration/001_schema.sql 管理）：
//   - idx_card_edges_source_type (source_id, relation_type)
//   - idx_card_edges_target_type (target_id, relation_type)
//   - idx_card_edges_target_id   (target_id)
type CardEdge struct {
	SourceID     string    `json:"source_id" gorm:"type:uuid;primaryKey;index:idx_target"`
	TargetID     string    `json:"target_id" gorm:"type:uuid;primaryKey;index:idx_target"`
	RelationType string    `json:"relation_type" gorm:"column:relation_type;type:varchar(20);not null;index;check:relation_type IN ('sequence','reference')"`
	CreatedAt    time.Time `json:"created_at"`
}

func (CardEdge) TableName() string { return "card_edges" }

// CardMetrics 卡片热度量量表 - 支持 UPSERT 高频更新
//
// 索引说明（由 migration/001_schema.sql 管理）：
//   - idx_card_metrics_hot_score (hot_score DESC NULLS LAST)
type CardMetrics struct {
	CardID    string    `json:"card_id" gorm:"type:uuid;primaryKey"`
	ViewCount int64     `json:"view_count" gorm:"not null;default:0"`
	HotScore  float64   `json:"hot_score" gorm:"not null;default:0"`
	UpdatedAt time.Time `json:"updated_at"`
}

func (CardMetrics) TableName() string { return "card_metrics" }

type CardLayout struct {
	CardID    string    `json:"card_id" gorm:"type:uuid;primaryKey"`
	X         float64   `json:"x" gorm:"not null;default:0"`
	Y         float64   `json:"y" gorm:"not null;default:0"`
	UpdatedAt time.Time `json:"updated_at"`
}

func (CardLayout) TableName() string { return "card_layouts" }

// Card 知识卡片 - 图谱的最小节点单元
//
// 索引说明（由 migration/001_schema.sql 管理，勿用 AutoMigrate）：
//   - idx_cards_created_at_desc  (created_at DESC)
//   - idx_cards_updated_at_desc  (updated_at DESC)
//   - idx_cards_category_id      (category_id)
//   - idx_cards_title            (title)
//   - idx_cards_ast_data_gin     (ast_data GIN)
//   - idx_cards_category_created (category_id, created_at DESC)
//   - idx_cards_toc_data         (toc_data GIN, partial)
type Card struct {
	ID         string       `json:"id" gorm:"type:uuid;primaryKey;default:gen_random_uuid()"`
	Title      string       `json:"title" gorm:"type:varchar(255)"`
	RawMd      string       `json:"raw_md" gorm:"type:text;not null"`
	Excerpt    string       `json:"excerpt" gorm:"type:text;not null;default:''"`
	AstData    JSONB        `json:"ast_data" gorm:"type:jsonb;not null"`
	TocData    JSONB        `json:"toc_data" gorm:"type:jsonb"`
	CategoryID *uint        `json:"category_id,omitempty" gorm:"index"`
	UpdatedAt  time.Time    `json:"updated_at" gorm:"index"`
	CreatedAt  time.Time    `json:"created_at"`
	Category   *Category    `json:"category,omitempty" gorm:"foreignKey:CategoryID"`
	Metrics    *CardMetrics `json:"metrics,omitempty" gorm:"foreignKey:CardID"`
}
