use rusqlite::Connection;
use std::path::Path;

use crate::error::{MetaDbError, MetaResult};

const SCHEMA_SQL: &str = include_str!("schema.sql");

pub struct MetaDb {
    pub(crate) conn: Connection,
}

impl MetaDb {
    pub fn open(vault_path: &Path) -> MetaResult<Self> {
        let bunker_dir = vault_path.join(".bunker");
        std::fs::create_dir_all(&bunker_dir)?;

        let new_path = bunker_dir.join("index.db");

        // Migrate legacy .vault_meta.db → .bunker/index.db
        let legacy_path = vault_path.join(".vault_meta.db");
        if legacy_path.exists() && !new_path.exists() {
            if let Err(e) = std::fs::rename(&legacy_path, &new_path) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(MetaDbError::MigrationFailed(format!(
                        "旧数据库迁移失败: {} -> {}: {e}",
                        legacy_path.display(),
                        new_path.display()
                    )));
                }
            }
        }

        let conn = Connection::open(&new_path)?;

        conn.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|e| MetaDbError::MigrationFailed(e.to_string()))?;
        conn.pragma_update(None, "journal_mode", "WAL".to_string())
            .map_err(|e| MetaDbError::MigrationFailed(e.to_string()))?;
        conn.pragma_update(None, "synchronous", "NORMAL".to_string())
            .map_err(|e| MetaDbError::MigrationFailed(e.to_string()))?;

        conn.execute_batch(SCHEMA_SQL)
            .map_err(|e| MetaDbError::MigrationFailed(e.to_string()))?;

        // Migrate: ensure unique index on relation_index (may not exist on older DBs)
        if conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_relation_unique ON relation_index(source_uuid, target_uuid_or_tag)",
            [],
        ).is_err() {
            // Deduplicate: keep 'trunk' over 'link' for same (source, target)
            conn.execute_batch(
                "CREATE TEMP TABLE _dedup AS
                 SELECT source_uuid, target_uuid_or_tag,
                     CASE WHEN MAX(CASE WHEN relation_type = 'trunk' THEN 1 ELSE 0 END) = 1
                          THEN 'trunk' ELSE 'link' END AS relation_type
                 FROM relation_index GROUP BY source_uuid, target_uuid_or_tag;
                 DELETE FROM relation_index;
                 INSERT INTO relation_index SELECT * FROM _dedup;
                 DROP TABLE _dedup;"
            ).ok();
            conn.execute(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_relation_unique ON relation_index(source_uuid, target_uuid_or_tag)",
                [],
            ).ok();
        }

        Ok(MetaDb { conn })
    }
}
