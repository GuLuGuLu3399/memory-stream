package storage

import (
	"os"
	"time"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

func InitDB() *gorm.DB {
	dsn := os.Getenv("DATABASE_URL")
	if dsn == "" {
		logger.Log.Fatal("DATABASE_URL is not set in environment")
	}

	db, err := gorm.Open(postgres.Open(dsn), &gorm.Config{})
	if err != nil {
		logger.Log.Fatalf("unable to connect database: %v", err)
	}

	sqlDB, err := db.DB()
	if err != nil {
		logger.Log.Fatalf("unable to get database instance: %v", err)
	}
	sqlDB.SetMaxIdleConns(10)
	sqlDB.SetMaxOpenConns(25)
	sqlDB.SetConnMaxLifetime(5 * time.Minute)

	logger.Log.Info("database connected")
	return db
}
