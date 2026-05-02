// ────────────────────────────────────────────────────────────────
// relation.go — Relation model
// relation.go — 关系模型
// ────────────────────────────────────────────────────────────────

package models

import (
	"database/sql/driver"

	"github.com/google/uuid"
)

// RelationType — PG ENUM relation_type_enum, 对齐 Rust RelationRecord
// Rust: "trunk" | "link" | "tag" — tag 是本地专属，Go 只有 trunk 和 link
type RelationType string

const (
	RelationTypeTrunk RelationType = "trunk"
	RelationTypeLink  RelationType = "link"
)

func (r *RelationType) String() string {
	if r == nil {
		return ""
	}
	return string(*r)
}

func (r *RelationType) Scan(value interface{}) error {
	if value == nil {
		*r = RelationTypeLink
		return nil
	}
	switch v := value.(type) {
	case string:
		*r = RelationType(v)
	case []byte:
		*r = RelationType(v)
	}
	return nil
}

func (r *RelationType) Value() (driver.Value, error) {
	if r == nil {
		return nil, nil
	}
	return string(*r), nil
}

// Relation 卡片间有向关系（父子主干 / 普通链接）
type Relation struct {
	SourceUUID   uuid.UUID    `gorm:"type:uuid;primaryKey" json:"source_uuid"`                    // 源卡片 UUID
	TargetUUID   uuid.UUID    `gorm:"type:uuid;primaryKey" json:"target_uuid"`                    // 目标卡片 UUID
	RelationType RelationType `gorm:"type:relation_type_enum;default:'link'" json:"relation_type"` // 关系类型（trunk/link）
}

func (Relation) TableName() string { return "relations" }
