# Database Migrations

Migration 文件管理数据库 schema 版本。

## 执行顺序

```bash
# 1. 重置数据库（仅开发环境）
psql -h localhost -U root -d devdb -f migration/sql/000_reset.sql

# 2. 执行完整基线 schema（6 张表 + 全部索引 + 全文搜索）
psql -h localhost -U root -d devdb -f migration/sql/001_schema.sql

# 3. 加载测试数据
psql -h localhost -U root -d devdb -f migration/sql/002_seed.sql
```

## 文件说明

| 文件             | 说明                                   |
|----------------|--------------------------------------|
| 000_reset.sql  | 重置数据库（DROP 全部 8 张表）                  |
| 001_schema.sql | **完整基线 schema**（6 张表 + 全部索引，含所有增量迁移） |
| 002_seed.sql   | 测试数据种子                               |

## 注意事项

- **不要使用 GORM AutoMigrate** — 本项目完全使用 migration 文件管理 schema
- **索引由 migration 管理** — Go model 中的注释说明了所有索引定义
- **执行前备份** — 生产环境执行前务必备份数据库
- **全新部署只需 000 → 001 → 002** — 增量迁移文件无需执行
