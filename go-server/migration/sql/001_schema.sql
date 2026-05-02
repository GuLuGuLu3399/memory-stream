-- ============================================================================
-- Memory Stream V1 — Schema Baseline (Single Source of Truth)
-- 对齐 Rust: ms-meta CardIndex, RelationRecord; ms-graph GraphNode/GraphEdge
-- ============================================================================
-- 执行：psql -h localhost -U root -d devdb -f migration/sql/001_schema.sql
-- ============================================================================

CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ── users (auth middleware 依赖) ──────────────────────────────────────────────

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'guest' CHECK (role IN ('admin', 'user', 'guest')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── cards — 核心卡片表 ───────────────────────────────────────────────────────
-- 对齐 Rust: ms-meta CardIndex (uuid, file_hash → hash, version)
-- Go 不存 file_path / sync_status / last_synced_hash (本地专属)

CREATE TABLE cards (
    uuid UUID PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    category VARCHAR(255) DEFAULT '',

    content TEXT NOT NULL,                       -- 原始 Markdown (Tauri 推来的)
    ast_data JSONB NOT NULL,                     -- Rust 解析好的 AST (原样 JSONB)
    toc_data JSONB DEFAULT '[]'::jsonb,
    excerpt TEXT DEFAULT '',

    version BIGINT NOT NULL DEFAULT 1,
    hash VARCHAR(64) NOT NULL,                   -- 对齐 Rust file_hash

    deleted_at TIMESTAMPTZ,                      -- 软删除标记
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cards_ast ON cards USING GIN (ast_data);
CREATE INDEX idx_cards_updated_at ON cards(updated_at DESC);
CREATE INDEX idx_cards_category ON cards(category);
CREATE UNIQUE INDEX idx_cards_title_unique ON cards(title)
    WHERE deleted_at IS NULL AND title != '';

-- ── relations — 双向图谱关系 ─────────────────────────────────────────────────
-- 对齐 Rust: ms-meta RelationRecord
-- tag 关系为本地专属，不入云；Go 只存 trunk 和 link

CREATE TYPE relation_type_enum AS ENUM ('trunk', 'link');

CREATE TABLE relations (
    source_uuid UUID NOT NULL REFERENCES cards(uuid) ON DELETE CASCADE,
    target_uuid UUID,                            -- 允许悬空 (目标节点可能未创建)
    relation_type relation_type_enum NOT NULL DEFAULT 'link',
    PRIMARY KEY (source_uuid, target_uuid)
);

CREATE INDEX idx_relations_source ON relations(source_uuid);
CREATE INDEX idx_relations_target ON relations(target_uuid);

-- ── sync_change_log — 增量同步日志 ───────────────────────────────────────────

CREATE TABLE sync_change_log (
    seq BIGSERIAL PRIMARY KEY,
    card_uuid UUID NOT NULL,
    op VARCHAR(10) NOT NULL CHECK (op IN ('upsert', 'delete')),
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sync_log_changed_at ON sync_change_log(changed_at DESC);

-- ── triggers ──────────────────────────────────────────────────────────────────

CREATE OR REPLACE FUNCTION set_updated_at() RETURNS TRIGGER AS $$
BEGIN NEW.updated_at = NOW(); RETURN NEW; END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER cards_set_updated_at
    BEFORE UPDATE ON cards FOR EACH ROW EXECUTE FUNCTION set_updated_at();
