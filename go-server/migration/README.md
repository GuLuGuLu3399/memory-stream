# Database Migrations

Migration 文件管理数据库 schema 版本。

## 执行顺序

```bash
# 1. 重置数据库（仅开发环境）
psql -h localhost -U postgres -d memory_stream -f migration/sql/000_reset.sql

# 2. 执行完整基线 schema（6 张表 + 核心索引）
psql -h localhost -U postgres -d memory_stream -f migration/sql/001_schema.sql

# 3. 加载测试数据
psql -h localhost -U postgres -d memory_stream -f migration/sql/002_seed.sql
```

## 文件说明

| 文件           | 说明                                          |
| -------------- | --------------------------------------------- |
| 000_reset.sql  | 重置数据库（兼容新旧表结构，一键归零）        |
| 001_schema.sql | **完整基线 schema**（归零后唯一 schema 来源） |
| 002_seed.sql   | 测试数据种子                                  |

## 注意事项

- **不要使用 GORM AutoMigrate** — 本项目完全使用 migration 文件管理 schema
- **索引由 migration 管理** — Go model 中的注释说明了所有索引定义
- **执行前备份** — 生产环境执行前务必备份数据库
- **统一执行 000 → 001 → 002** — migration 目录已归零，不再维护增量脚本
