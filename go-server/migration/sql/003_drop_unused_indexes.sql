-- ============================================================================
-- Memory Stream v3.4 — 删除无查询命中的冗余索引
-- ============================================================================
-- 执行：psql -h localhost -U root -d devdb -f 003_drop_unused_indexes.sql
--
-- 背景：经全栈审计确认以下索引无任何查询路径命中，
--       白白消耗 INSERT/UPDATE 时的 B+ 树维护开销。
--
-- 决策记录：
--   idx_cards_ast_data_gin     → GIN 索引，无 JSONB 查询使用 ast_data 列
--   idx_cards_category_created → 复合索引，无 (category_id, created_at) 联合查询
--   idx_cards_toc_data         → GIN 条件索引，无 JSONB 查询使用 toc_data 列
-- ============================================================================

DROP INDEX IF EXISTS idx_cards_ast_data_gin;
DROP INDEX IF EXISTS idx_cards_category_created;
DROP INDEX IF EXISTS idx_cards_toc_data;
