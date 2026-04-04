package models

import "time"

// User 用户表 - 支持双 Token 认证
//
// 约束说明（由 migration/003_users.sql 管理）：
//   - username: UNIQUE 约束（已通过 gorm:"uniqueIndex" 定义）
//   - role: CHECK 约束，仅允许 'admin', 'user', 'guest'
//
// 索引说明：
//   - idx_users_username (username) - 冗余索引，UNIQUE 已自带索引
type User struct {
	ID           string    `json:"id" gorm:"type:uuid;primaryKey;default:gen_random_uuid()"`
	Username     string    `json:"username" gorm:"type:varchar(50);uniqueIndex;not null"`
	PasswordHash string    `json:"-" gorm:"column:password_hash;type:varchar(255);not null"`
	Role         string    `json:"role" gorm:"type:varchar(20);not null;default:'guest'"`
	CreatedAt    time.Time `json:"created_at"`
	UpdatedAt    time.Time `json:"updated_at"`
}

func (User) TableName() string { return "users" }
