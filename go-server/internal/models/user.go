// Package models defines database models for the Memory Stream server.
package models

import (
	"time"

	"github.com/google/uuid"
)

// User 认证用户
type User struct {
	ID           uuid.UUID `gorm:"type:uuid;primaryKey;default:gen_random_uuid()" json:"id"` // 用户 ID
	Username     string    `gorm:"type:varchar(50);uniqueIndex;not null" json:"username"`    // 用户名
	PasswordHash string    `gorm:"column:password_hash;type:varchar(255);not null" json:"-"` // 密码哈希
	Role         string    `gorm:"type:varchar(20);not null;default:'guest'" json:"role"`    // 角色
	CreatedAt    time.Time `json:"created_at"`                                                // 创建时间
	UpdatedAt    time.Time `json:"updated_at"`                                                // 更新时间
}

func (User) TableName() string { return "users" }
