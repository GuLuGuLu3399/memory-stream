package logger

import (
	"time"

	"gorm.io/gorm"
)

const slowQueryThreshold = 200 * time.Millisecond

// RegisterSlowQueryPlugin 注册 GORM 慢查询监控回调。
// 记录耗时超过 200ms 的 SQL 语句到 zap 日志。
// 覆盖所有操作类型：Query、Create、Update、Delete、Raw。
func RegisterSlowQueryPlugin(db *gorm.DB) {
	// Query
	_ = db.Callback().Query().Before("gorm:query").Register("slow_query:start", func(d *gorm.DB) {
		d.InstanceSet("slow_query:start_time", time.Now())
	})
	_ = db.Callback().Query().After("gorm:query").Register("slow_query:log", func(d *gorm.DB) {
		logSlowQuery(d, "query")
	})

	// Create
	_ = db.Callback().Create().Before("gorm:create").Register("slow_query:create:start", func(d *gorm.DB) {
		d.InstanceSet("slow_query:start_time", time.Now())
	})
	_ = db.Callback().Create().After("gorm:create").Register("slow_query:create:log", func(d *gorm.DB) {
		logSlowQuery(d, "create")
	})

	// Update
	_ = db.Callback().Update().Before("gorm:update").Register("slow_query:update:start", func(d *gorm.DB) {
		d.InstanceSet("slow_query:start_time", time.Now())
	})
	_ = db.Callback().Update().After("gorm:update").Register("slow_query:update:log", func(d *gorm.DB) {
		logSlowQuery(d, "update")
	})

	// Delete
	_ = db.Callback().Delete().Before("gorm:delete").Register("slow_query:delete:start", func(d *gorm.DB) {
		d.InstanceSet("slow_query:start_time", time.Now())
	})
	_ = db.Callback().Delete().After("gorm:delete").Register("slow_query:delete:log", func(d *gorm.DB) {
		logSlowQuery(d, "delete")
	})

	// Raw
	_ = db.Callback().Raw().Before("gorm:raw").Register("slow_query:raw:start", func(d *gorm.DB) {
		d.InstanceSet("slow_query:start_time", time.Now())
	})
	_ = db.Callback().Raw().After("gorm:raw").Register("slow_query:raw:log", func(d *gorm.DB) {
		logSlowQuery(d, "raw")
	})
}

func logSlowQuery(d *gorm.DB, operation string) {
	startTime, ok := d.InstanceGet("slow_query:start_time")
	if !ok {
		return
	}

	elapsed := time.Since(startTime.(time.Time))
	if elapsed > slowQueryThreshold {
		sql := d.Statement.SQL.String()
		if sql == "" {
			return
		}
		Log.Warnf("[SLOW SQL][%s] %s | elapsed: %v | rows: %d", operation, sql, elapsed, d.RowsAffected)
	}
}
