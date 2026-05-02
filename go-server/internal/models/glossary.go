package models

import (
	"time"

	"gorm.io/gorm"
)

// GlossaryItem 术语表条目（概念定义）
type GlossaryItem struct {
	ID         uint           `gorm:"primaryKey" json:"-"`                                // 自增主键
	Term       string         `gorm:"type:varchar(255);uniqueIndex;not null" json:"term"` // 术语名
	Definition string         `gorm:"type:text;not null" json:"definition"`                // 定义
	Version    int64          `gorm:"not null;default:1" json:"version"`                   // 版本号
	Hash       string         `gorm:"type:varchar(64);not null" json:"hash"`               // 内容哈希
	DeletedAt  gorm.DeletedAt `gorm:"index" json:"-"`                                      // 软删除时间
	CreatedAt  time.Time      `json:"created_at"`                                           // 创建时间
	UpdatedAt  time.Time      `json:"updated_at"`                                           // 更新时间
}
