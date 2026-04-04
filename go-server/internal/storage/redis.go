package storage

import (
	"context"
	"os"
	"strconv"
	"time"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"github.com/redis/go-redis/v9"
)

func InitRedis() *redis.Client {
	addr := os.Getenv("REDIS_ADDR")
	if addr == "" {
		addr = "localhost:6379"
	}

	poolSize := envInt("REDIS_POOL_SIZE", 10)
	minIdle := envInt("REDIS_MIN_IDLE", 3)
	dialTimeout := envDuration("REDIS_DIAL_TIMEOUT", 5*time.Second)
	readTimeout := envDuration("REDIS_READ_TIMEOUT", 3*time.Second)
	writeTimeout := envDuration("REDIS_WRITE_TIMEOUT", 3*time.Second)

	rdb := redis.NewClient(&redis.Options{
		Addr:         addr,
		Password:     os.Getenv("REDIS_PASSWORD"),
		DB:           0,
		PoolSize:     poolSize,
		MinIdleConns: minIdle,
		DialTimeout:  dialTimeout,
		ReadTimeout:  readTimeout,
		WriteTimeout: writeTimeout,
		MaxRetries:   3,
	})

	if err := rdb.Ping(context.Background()).Err(); err != nil {
		logger.Log.Fatalf("unable to connect Redis: %v", err)
	}

	logger.Log.Infof("Redis connected (pool=%d, idle=%d)", poolSize, minIdle)
	return rdb
}

func envInt(key string, fallback int) int {
	if v := os.Getenv(key); v != "" {
		if n, err := strconv.Atoi(v); err == nil {
			return n
		}
	}
	return fallback
}

func envDuration(key string, fallback time.Duration) time.Duration {
	if v := os.Getenv(key); v != "" {
		if d, err := time.ParseDuration(v); err == nil {
			return d
		}
	}
	return fallback
}
