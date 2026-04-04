-- ============================================================================
-- Memory Stream v3.4 — Seed Data
-- ============================================================================
-- 用途：标准测试数据集，覆盖全部测试场景
-- 前置：需先执行 001_schema.sql
-- 执行：psql -h localhost -U root -d devdb -f 002_seed.sql
-- ============================================================================

BEGIN;

-- ── 分类 ──
INSERT INTO categories (id, name, description, created_at) VALUES
    (1, 'Rust',   'Rust 语言生态与最佳实践',   NOW()),
    (2, 'Vue',    'Vue.js 前端框架',            NOW()),
    (3, '架构',   '系统设计与架构模式',          NOW()),
    (4, '工具链', '开发工具与效率提升',          NOW()),
    (5, 'Go',     'Go 语言后端开发',            NOW()),
    (6, '数据库', 'PostgreSQL 与数据存储',       NOW())
ON CONFLICT (id) DO UPDATE SET
    name        = EXCLUDED.name,
    description = EXCLUDED.description;

ALTER SEQUENCE categories_id_seq RESTART WITH 100;

-- ── 卡片（5 张） ──

-- A: 根节点（未分类，测试 Inbox 兜底）
INSERT INTO cards (id, title, excerpt, raw_md, ast_data, category_id, created_at, updated_at) VALUES
    ('a0000000-0000-0000-0000-000000000001'::UUID,
     '我的知识系统规划',
     '建立可复用的知识图谱，实现语义缩放和双线视觉',
     '# 我的知识系统规划\n\n## 核心目标\n- 建立可复用的知识图谱\n- 实现语义缩放和双线视觉\n\n## 技术栈\n- 后端: Go + PostgreSQL\n- 前端: Vue 3 + Vue Flow',
     '{"type":"Root","children":[{"type":"Heading","level":1,"children":[{"type":"Text","value":"我的知识系统规划"}]}]}'::JSONB,
     NULL,
     NOW() - INTERVAL '10 days',
     NOW());

-- B: 主干节点（架构分类）
INSERT INTO cards (id, title, excerpt, raw_md, ast_data, category_id, created_at, updated_at) VALUES
    ('a0000000-0000-0000-0000-000000000002'::UUID,
     'Phase 1 - 基础架构',
     '完成数据库设计、Go 后端 API、Vue Flow 集成',
     '# Phase 1: 基础架构\n\n## 完成项\n- [x] 数据库设计\n- [x] Go 后端 API\n- [x] Vue Flow 集成\n\n## 核心收获\n- 理解了 UUID 与前端 ID 的映射\n- 掌握了 GORM 的软删除机制',
     '{"type":"Root","children":[{"type":"Heading","level":1,"children":[{"type":"Text","value":"Phase 1: 基础架构"}]}]}'::JSONB,
     3,
     NOW() - INTERVAL '8 days',
     NOW());

-- C: 主干节点（架构分类）
INSERT INTO cards (id, title, excerpt, raw_md, ast_data, category_id, created_at, updated_at) VALUES
    ('a0000000-0000-0000-0000-000000000003'::UUID,
     'Phase 2 - 语义缩放',
     '实现 0.5x Topic、1.0x Cluster、2.0x Detail 三级视图',
     '# Phase 2: 语义缩放\n\n## 目标\n- 实现 0.5x (Topic) 视图\n- 实现 1.0x (Cluster) 视图\n- 实现 2.0x (Detail) 视图\n\n## 关键技术\n- Vue Flow 的 parentNode 机制\n- 前端动态样式计算',
     '{"type":"Root","children":[{"type":"Heading","level":1,"children":[{"type":"Text","value":"Phase 2: 语义缩放"}]}]}'::JSONB,
     3,
     NOW() - INTERVAL '6 days',
     NOW());

-- D: 引用节点（工具分类）
INSERT INTO cards (id, title, excerpt, raw_md, ast_data, category_id, created_at, updated_at) VALUES
    ('a0000000-0000-0000-0000-000000000004'::UUID,
     'Vue Flow 最佳实践',
     'Node Types、Edge Types、Handle 系统的核心概念',
     '# Vue Flow 最佳实践\n\n## 官方文档\n- [Vue Flow 官网](https://vueflow.dev/)\n\n## 核心概念\n- Node Types\n- Edge Types\n- Handle 系统',
     '{"type":"Root","children":[{"type":"Heading","level":1,"children":[{"type":"Text","value":"Vue Flow 最佳实践"}]}]}'::JSONB,
     4,
     NOW() - INTERVAL '7 days',
     NOW());

-- E: 孤立节点（Rust 分类，测试 root 解析器兜底）
INSERT INTO cards (id, title, excerpt, raw_md, ast_data, category_id, created_at, updated_at) VALUES
    ('a0000000-0000-0000-0000-000000000005'::UUID,
     'Rust 学习笔记',
     '所有权规则、借用规则等 Rust 核心概念',
     '# Rust 学习笔记\n\n## 所有权规则\n1. 每个值都有一个所有者\n2. 同一时刻只能有一个所有者\n3. 当所有者离开作用域，值被丢弃\n\n## 借用规则\n- 不可变借用可以有多个\n- 可变借用只能有一个',
     '{"type":"Root","children":[{"type":"Heading","level":1,"children":[{"type":"Text","value":"Rust 学习笔记"}]}]}'::JSONB,
     1,
     NOW(),
     NOW());

-- ── 图谱关系（脊椎 + 网格） ──

-- 主干演进 (Sequence)：A → B → C
INSERT INTO card_edges (source_id, target_id, relation_type, created_at) VALUES
    ('a0000000-0000-0000-0000-000000000001'::UUID,
     'a0000000-0000-0000-0000-000000000002'::UUID,
     'sequence',
     NOW() - INTERVAL '8 days'),

    ('a0000000-0000-0000-0000-000000000002'::UUID,
     'a0000000-0000-0000-0000-000000000003'::UUID,
     'sequence',
     NOW() - INTERVAL '6 days')
ON CONFLICT (source_id, target_id) DO NOTHING;

-- 关联网格 (Reference)：C → D, A → E
INSERT INTO card_edges (source_id, target_id, relation_type, created_at) VALUES
    ('a0000000-0000-0000-0000-000000000003'::UUID,
     'a0000000-0000-0000-0000-000000000004'::UUID,
     'reference',
     NOW() - INTERVAL '5 days'),

    ('a0000000-0000-0000-0000-000000000001'::UUID,
     'a0000000-0000-0000-0000-000000000005'::UUID,
     'reference',
     NOW())
ON CONFLICT (source_id, target_id) DO NOTHING;

-- ── 热度指标（3 张卡片） ──
INSERT INTO card_metrics (card_id, view_count, hot_score, updated_at) VALUES
    ('a0000000-0000-0000-0000-000000000001'::UUID, 42, 16.8, NOW()),
    ('a0000000-0000-0000-0000-000000000002'::UUID, 28, 11.2, NOW()),
    ('a0000000-0000-0000-0000-000000000003'::UUID, 15,  6.0, NOW())
ON CONFLICT (card_id) DO NOTHING;

-- ── 布局坐标（5 张卡片，脊椎 + 分支） ──
INSERT INTO card_layouts (card_id, x, y, updated_at) VALUES
    ('a0000000-0000-0000-0000-000000000001'::UUID,    0,   0, NOW()),
    ('a0000000-0000-0000-0000-000000000002'::UUID,    0, 150, NOW()),
    ('a0000000-0000-0000-0000-000000000003'::UUID,    0, 300, NOW()),
    ('a0000000-0000-0000-0000-000000000004'::UUID,  250, 300, NOW()),
    ('a0000000-0000-0000-0000-000000000005'::UUID, -250,   0, NOW())
ON CONFLICT (card_id) DO NOTHING;

COMMIT;

-- ============================================================================
-- 验证查询
-- ============================================================================

SELECT 'Seed data loaded' AS status;

SELECT
    'categories'   AS table_name, COUNT(*) AS row_count FROM categories
UNION ALL
SELECT 'cards',        COUNT(*) FROM cards
UNION ALL
SELECT 'card_edges',   COUNT(*) FROM card_edges
UNION ALL
SELECT 'card_metrics', COUNT(*) FROM card_metrics
UNION ALL
SELECT 'card_layouts', COUNT(*) FROM card_layouts;
