package storage

import (
	"fmt"
	"os"
	"time"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

func InitDB() (*gorm.DB, error) {
	dsn := os.Getenv("DATABASE_URL")
	if dsn == "" {
		return nil, fmt.Errorf("DATABASE_URL is not set in environment")
	}

	db, err := gorm.Open(postgres.Open(dsn), &gorm.Config{
		SkipDefaultTransaction: true,
	})
	if err != nil {
		return nil, fmt.Errorf("unable to connect database: %w", err)
	}

	sqlDB, err := db.DB()
	if err != nil {
		return nil, fmt.Errorf("unable to get database instance: %w", err)
	}
	sqlDB.SetMaxIdleConns(20)
	sqlDB.SetMaxOpenConns(50)
	sqlDB.SetConnMaxLifetime(5 * time.Minute)

	// Fast-fail when core tables are missing
	var cardsTable string
	if err := db.Raw("SELECT to_regclass('public.cards')").Scan(&cardsTable).Error; err != nil {
		return nil, fmt.Errorf("failed to verify schema: %w", err)
	}
	if cardsTable == "" {
		return nil, fmt.Errorf("database schema is missing (table public.cards not found). Run migrations: 000_reset.sql -> 001_schema.sql")
	}

	logger.Log.Info("database connected")
	return db, nil
}
