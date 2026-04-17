-- ============================================================================
-- Memory Stream v3.4 — Schema DDL（完整基线）
-- ============================================================================
-- 架构：categories → cards → card_edges / card_metrics / card_layouts
--        users（独立认证表）
-- 执行：psql -h localhost -U root -d devdb -f 001_schema.sql
--       (需先执行 000_reset.sql 或确认表不存在)
--
-- 说明：本文件是完整的基线 schema，包含所有已合并的增量迁移：
--   003_users.sql             → users 表已内联
--   007_category_hierarchy    → parent_id / sort_order 已内联
--   008_category_data_migr    → theme_color 已内联（移除冗余 ADD COLUMN）
--   009_cards_title_unique    → idx_cards_title_unique 已内联
--   010_search_vector         → search_vector + GIN 索引已内联
--
-- 决策记录：
--   D1: 移除 trees / tree_cards（已由 card_edges 图谱替代）
--   D2: 移除 deleted_at（全库物理删除 + CASCADE）
--   D3: relation_type 使用 CHECK 约束
--   D4: category_id 允许 NULL（Inbox 未分类）
--   D5: 移除 is_visible（显隐属客户端视图状态）
--   D6: excerpt 使用 TEXT（无长度限制）
--   D7: toc_data 预置（TOC 树 JSON）
--   D8: categories 支持层级（parent_id 自引用 + sort_order）
--   D9: cards 全文搜索（search_vector 生成列 + GIN 索引）
-- ============================================================================

-- ╔═══════════════════════════════════════════════════════════════════════════╗
-- ║  categories — 字典表 / 语义缩放 Topic 层                              ║
-- ╚═══════════════════════════════════════════════════════════════════════════╝

CREATE TABLE categories (
    id          SERIAL       PRIMARY KEY,
    name        VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    theme_color VARCHAR(20),
    parent_id   INTEGER      REFERENCES categories(id) ON DELETE SET NULL DEFAULT NULL,
    sort_order  INTEGER      NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_categories_no_self_ref CHECK (id != parent_id)
);

COMMENT ON TABLE  categories              IS '分类字典表 — 语义缩放 Topic 层，支持层级嵌套';
COMMENT ON COLUMN categories.name         IS '分类名称（唯一，如 Rust / 架构）';
COMMENT ON COLUMN categories.description  IS '分类描述';
COMMENT ON COLUMN categories.parent_id    IS '父分类 ID（NULL = 根级，最大深度 5 层）';
COMMENT ON COLUMN categories.sort_order   IS '同级排序（值越小越靠前）';

CREATE INDEX idx_categories_parent_id ON categories (parent_id);

-- ╔═══════════════════════════════════════════════════════════════════════════╗
-- ║  cards — 最小知识单元 / 图谱节点                                       ║
-- ╚═══════════════════════════════════════════════════════════════════════════╝

CREATE TABLE cards (
    id             UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    title          VARCHAR(255),
    excerpt        TEXT         NOT NULL DEFAULT '',
    raw_md         TEXT         NOT NULL,
    ast_data       JSONB        NOT NULL,
    toc_data       JSONB,
    search_vector  tsvector     GENERATED ALWAYS AS (
        setweight(to_tsvector('simple', coalesce(title, '')), 'A') ||
        setweight(to_tsvector('simple', coalesce(raw_md, '')), 'B')
    ) STORED,
    category_id    INTEGER      REFERENCES categories(id) ON DELETE SET NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE  cards              IS '知识卡片 — 图谱的最小节点单元';
COMMENT ON COLUMN cards.title        IS '卡片标题（允许 NULL，用于未命名的闪念笔记）';
COMMENT ON COLUMN cards.excerpt      IS '卡片摘要（自动截取或手动填写，无长度限制）';
COMMENT ON COLUMN cards.raw_md       IS '原始 Markdown 内容';
COMMENT ON COLUMN cards.ast_data     IS 'AST 抽象语法树（JSONB，由前端解析器生成）';
COMMENT ON COLUMN cards.toc_data     IS '目录树 JSON（由 ms-toc-extractor 预计算）';
COMMENT ON COLUMN cards.search_vector IS '全文搜索向量（title 权重 A，content 权重 B，simple 配置支持中文）';
COMMENT ON COLUMN cards.category_id  IS '所属分类 ID（NULL = 未分类 / Inbox）';

CREATE INDEX idx_cards_created_at_desc  ON cards (created_at DESC);
CREATE INDEX idx_cards_updated_at_desc  ON cards (updated_at DESC);
CREATE INDEX idx_cards_category_id      ON cards (category_id);
CREATE INDEX idx_cards_title            ON cards (title);
CREATE UNIQUE INDEX idx_cards_title_unique ON cards (title) WHERE title IS NOT NULL AND title != '';
CREATE INDEX idx_cards_search_vector    ON cards USING GIN (search_vector);

-- ╔═══════════════════════════════════════════════════════════════════════════╗
-- ║  card_edges — 图谱关系表 / 双线脊椎 (sequence + reference)             ║
-- ╚═══════════════════════════════════════════════════════════════════════════╝

CREATE TABLE card_edges (
    source_id     UUID        NOT NULL,
    target_id     UUID        NOT NULL,
    relation_type VARCHAR(20) NOT NULL
        CHECK (relation_type IN ('sequence', 'reference')),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (source_id, target_id),

    CONSTRAINT fk_card_edges_source
        FOREIGN KEY (source_id) REFERENCES cards(id) ON DELETE CASCADE,
    CONSTRAINT fk_card_edges_target
        FOREIGN KEY (target_id) REFERENCES cards(id) ON DELETE CASCADE
);

COMMENT ON TABLE  card_edges                IS '卡片关系表 — 支撑双线脊椎图谱 (DAG)';
COMMENT ON COLUMN card_edges.relation_type  IS '关系类型: sequence=主干演进, reference=关联网格';

CREATE INDEX idx_card_edges_source_type ON card_edges (source_id, relation_type);
CREATE INDEX idx_card_edges_target_type ON card_edges (target_id, relation_type);
CREATE INDEX idx_card_edges_target_id   ON card_edges (target_id);

-- ╔═══════════════════════════════════════════════════════════════════════════╗
-- ║  card_metrics — 热度量量表 / 高频 UPSERT                               ║
-- ╚═══════════════════════════════════════════════════════════════════════════╝

CREATE TABLE card_metrics (
    card_id    UUID             PRIMARY KEY,
    view_count BIGINT           NOT NULL DEFAULT 0,
    hot_score  DOUBLE PRECISION NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ      NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_card_metrics_card
        FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
);

COMMENT ON TABLE  card_metrics             IS '卡片热度量量表 — 支持 UPSERT 高频更新';
COMMENT ON COLUMN card_metrics.view_count  IS '累计浏览次数';
COMMENT ON COLUMN card_metrics.hot_score   IS '热度评分（加权衰减值）';

CREATE INDEX idx_card_metrics_hot_score ON card_metrics (hot_score DESC NULLS LAST);

-- ╔═══════════════════════════════════════════════════════════════════════════╗
-- ║  card_layouts — 坐标缓存表 / 前端图谱绝对定位                          ║
-- ╚═══════════════════════════════════════════════════════════════════════════╝

CREATE TABLE card_layouts (
    card_id    UUID             PRIMARY KEY,
    x          DOUBLE PRECISION NOT NULL DEFAULT 0,
    y          DOUBLE PRECISION NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ      NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_card_layouts_card
        FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
);

COMMENT ON TABLE  card_layouts          IS '卡片坐标缓存 — 前端图谱引擎绝对定位渲染';

-- ╔═══════════════════════════════════════════════════════════════════════════╗
-- ║  users — 用户表 / JWT 认证                                              ║
-- ╚═══════════════════════════════════════════════════════════════════════════╝

CREATE TABLE users (
    id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    username      VARCHAR(50)  NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role          VARCHAR(20)  NOT NULL DEFAULT 'guest'
        CHECK (role IN ('admin', 'user', 'guest')),
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE  users               IS '用户表 — 支持双 Token 认证';
COMMENT ON COLUMN users.username       IS '用户名（唯一）';
COMMENT ON COLUMN users.password_hash  IS 'bcrypt 密码哈希';
COMMENT ON COLUMN users.role           IS '角色: admin=管理员, user=普通用户, guest=访客';
