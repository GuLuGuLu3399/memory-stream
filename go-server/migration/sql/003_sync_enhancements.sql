-- ============================================================================
-- Memory Stream V3 — Sync Enhancements
-- Incremental cursor index + audit log debug columns
-- ============================================================================

-- Partial index for cursor-based manifest queries (skips soft-deleted rows)
CREATE INDEX IF NOT EXISTS idx_cards_sync_cursor
    ON cards (updated_at ASC)
    WHERE deleted_at IS NULL;

-- Audit log: track client-side version/hash for conflict debugging
ALTER TABLE sync_change_log ADD COLUMN IF NOT EXISTS client_version BIGINT DEFAULT 0;
ALTER TABLE sync_change_log ADD COLUMN IF NOT EXISTS client_hash VARCHAR(64) DEFAULT '';
