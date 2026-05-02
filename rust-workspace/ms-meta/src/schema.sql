PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;
PRAGMA busy_timeout=5000;

CREATE TABLE IF NOT EXISTS card_index (
    uuid TEXT PRIMARY KEY,
    file_path TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    sync_status TEXT NOT NULL DEFAULT 'synced',
    last_synced_hash TEXT
);
CREATE INDEX IF NOT EXISTS idx_card_file_path ON card_index(file_path);

CREATE TABLE IF NOT EXISTS relation_index (
    source_uuid TEXT NOT NULL,
    target_uuid_or_tag TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    UNIQUE(source_uuid, target_uuid_or_tag)
);
CREATE INDEX IF NOT EXISTS idx_relation_source ON relation_index(source_uuid);
CREATE INDEX IF NOT EXISTS idx_relation_target ON relation_index(target_uuid_or_tag);

CREATE TABLE IF NOT EXISTS asset_refs (
    local_path TEXT PRIMARY KEY,
    cloud_url TEXT,
    ref_count INTEGER NOT NULL DEFAULT 0
);

CREATE VIRTUAL TABLE IF NOT EXISTS card_fts USING fts5(
    uuid UNINDEXED,
    title,
    body_text,
    category_name
);
