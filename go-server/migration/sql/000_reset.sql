-- ============================================================================
-- Memory Stream — 数据库重置脚本
-- ============================================================================
-- 用途：开发过程中快速归零，DROP 全部表（含遗留的 trees / tree_cards）
-- 执行：psql -h localhost -U root -d devdb -f 000_reset.sql
-- ============================================================================

DROP TABLE IF EXISTS card_layouts  CASCADE;
DROP TABLE IF EXISTS card_metrics  CASCADE;
DROP TABLE IF EXISTS card_edges    CASCADE;
DROP TABLE IF EXISTS tree_cards    CASCADE;
DROP TABLE IF EXISTS trees         CASCADE;
DROP TABLE IF EXISTS cards         CASCADE;
DROP TABLE IF EXISTS categories    CASCADE;
DROP TABLE IF EXISTS users         CASCADE;

SELECT 'Database reset complete' AS status;
