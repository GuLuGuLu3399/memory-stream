-- ============================================================================
-- Memory Stream V4 — Glossary Items
-- 术语表：支持 Web 端概念引用悬浮提示 + Tauri 同步
-- ============================================================================

CREATE TABLE glossary_items (
    id BIGSERIAL PRIMARY KEY,
    term VARCHAR(255) NOT NULL UNIQUE,
    definition TEXT NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    hash VARCHAR(64) NOT NULL,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_glossary_updated_at ON glossary_items(updated_at ASC)
    WHERE deleted_at IS NULL;

CREATE TRIGGER glossary_set_updated_at
    BEFORE UPDATE ON glossary_items FOR EACH ROW EXECUTE FUNCTION set_updated_at();
