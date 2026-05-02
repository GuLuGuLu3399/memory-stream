-- ============================================================================
-- Memory Stream — 数据库重置脚本（归零基线）
-- ============================================================================
-- 用途：开发环境一键清空，兼容新旧表结构
-- 执行：psql -h localhost -U root -d devdb -f 000_reset.sql
-- ============================================================================

DROP FUNCTION IF EXISTS set_updated_at() CASCADE;
DROP TYPE IF EXISTS relation_type_enum CASCADE;

-- 当前基线表
DROP TABLE IF EXISTS sync_change_log CASCADE;
DROP TABLE IF EXISTS relations CASCADE;
DROP TABLE IF EXISTS cards CASCADE;
DROP TABLE IF EXISTS users CASCADE;

-- 历史遗留表（兼容清理）
DROP TABLE IF EXISTS card_edges CASCADE;
DROP TABLE IF EXISTS card_layouts CASCADE;
DROP TABLE IF EXISTS categories CASCADE;
DROP TABLE IF EXISTS edges CASCADE;
DROP TABLE IF EXISTS card_metrics CASCADE;
DROP TABLE IF EXISTS tree_cards CASCADE;
DROP TABLE IF EXISTS trees CASCADE;
DROP VIEW IF EXISTS v_categories CASCADE;

SELECT 'Database reset complete' AS status;
