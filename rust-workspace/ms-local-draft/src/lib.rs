pub mod error;

use error::{DraftError, DraftResult};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::task;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draft {
    pub card_id: String,
    pub raw_md: String,
    pub ast_data: Option<String>,
    pub updated_at: i64,
}

pub struct DraftDb {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl DraftDb {
    pub async fn new(db_path: &Path) -> DraftResult<Self> {
        let path = db_path.to_path_buf();
        task::spawn_blocking(move || {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| DraftError::OpenError(format!("创建目录失败: {}", e)))?;
            }

            let conn = rusqlite::Connection::open(&path)
                .map_err(|e| DraftError::OpenError(format!("打开数据库失败: {}", e)))?;

            conn.execute_batch(
                "PRAGMA journal_mode=WAL;
                 PRAGMA synchronous=NORMAL;
                 PRAGMA foreign_keys=ON;",
            )
            .map_err(|e| DraftError::OpenError(format!("设置 PRAGMA 失败: {}", e)))?;

            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS drafts (
                    card_id TEXT PRIMARY KEY,
                    raw_md TEXT NOT NULL,
                    ast_data TEXT,
                    updated_at INTEGER NOT NULL
                );",
            )
            .map_err(|e| DraftError::OpenError(format!("建表失败: {}", e)))?;

            Ok(Self {
                conn: Arc::new(Mutex::new(conn)),
            })
        })
        .await
        .unwrap_or_else(|_| Err(DraftError::TaskPanic))
    }

    pub async fn save_draft(
        &self,
        card_id: &str,
        raw_md: &str,
        ast_data: Option<&str>,
    ) -> DraftResult<()> {
        let conn = self.conn.clone();
        let card_id = card_id.to_string();
        let raw_md = raw_md.to_string();
        let ast_data = ast_data.map(|s| s.to_string());
        let now = chrono::Utc::now().timestamp();

        task::spawn_blocking(move || {
            let lock = conn.lock().unwrap();
            lock.execute(
                "INSERT OR REPLACE INTO drafts (card_id, raw_md, ast_data, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![card_id, raw_md, ast_data, now],
            )?;
            Ok(())
        })
        .await
        .unwrap_or_else(|_| Err(DraftError::TaskPanic))
    }

    pub async fn load_draft(&self, card_id: &str) -> DraftResult<Option<Draft>> {
        let conn = self.conn.clone();
        let card_id = card_id.to_string();

        task::spawn_blocking(move || {
            let lock = conn.lock().unwrap();
            let mut stmt = lock.prepare(
                "SELECT card_id, raw_md, ast_data, updated_at FROM drafts WHERE card_id = ?1",
            )?;

            let mut rows = stmt.query(params![card_id])?;
            match rows.next()? {
                Some(row) => Ok(Some(Draft {
                    card_id: row.get(0)?,
                    raw_md: row.get(1)?,
                    ast_data: row.get(2)?,
                    updated_at: row.get(3)?,
                })),
                None => Ok(None),
            }
        })
        .await
        .unwrap_or_else(|_| Err(DraftError::TaskPanic))
    }

    pub async fn list_all(&self) -> DraftResult<Vec<Draft>> {
        let conn = self.conn.clone();

        task::spawn_blocking(move || {
            let lock = conn.lock().unwrap();
            let mut stmt = lock.prepare(
                "SELECT card_id, raw_md, ast_data, updated_at FROM drafts ORDER BY updated_at DESC",
            )?;

            let drafts = stmt
                .query_map([], |row| {
                    Ok(Draft {
                        card_id: row.get(0)?,
                        raw_md: row.get(1)?,
                        ast_data: row.get(2)?,
                        updated_at: row.get(3)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(drafts)
        })
        .await
        .unwrap_or_else(|_| Err(DraftError::TaskPanic))
    }

    pub async fn list_unsynced(&self) -> DraftResult<Vec<Draft>> {
        self.list_all().await
    }

    pub async fn mark_synced(&self, card_id: &str) -> DraftResult<()> {
        self.delete_draft(card_id).await
    }

    pub async fn delete_draft(&self, card_id: &str) -> DraftResult<()> {
        let conn = self.conn.clone();
        let card_id = card_id.to_string();

        task::spawn_blocking(move || {
            let lock = conn.lock().unwrap();
            lock.execute("DELETE FROM drafts WHERE card_id = ?1", params![card_id])?;
            Ok(())
        })
        .await
        .unwrap_or_else(|_| Err(DraftError::TaskPanic))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup() -> (TempDir, DraftDb) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test_drafts.db");
        let db = DraftDb::new(&db_path).await.unwrap();
        (dir, db)
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let (_dir, db) = setup().await;
        db.save_draft("card-1", "# Hello", Some(r#"{"type":"Root"}"#))
            .await
            .unwrap();

        let draft = db.load_draft("card-1").await.unwrap();
        assert!(draft.is_some());
        let d = draft.unwrap();
        assert_eq!(d.card_id, "card-1");
        assert_eq!(d.raw_md, "# Hello");
        assert_eq!(d.ast_data, Some(r#"{"type":"Root"}"#.to_string()));
    }

    #[tokio::test]
    async fn test_load_nonexistent() {
        let (_dir, db) = setup().await;
        let draft = db.load_draft("ghost").await.unwrap();
        assert!(draft.is_none());
    }

    #[tokio::test]
    async fn test_upsert_overwrites() {
        let (_dir, db) = setup().await;
        db.save_draft("card-1", "v1", None).await.unwrap();
        db.save_draft("card-1", "v2", Some("ast2")).await.unwrap();

        let d = db.load_draft("card-1").await.unwrap().unwrap();
        assert_eq!(d.raw_md, "v2");
        assert_eq!(d.ast_data, Some("ast2".to_string()));
    }

    #[tokio::test]
    async fn test_list_all() {
        let (_dir, db) = setup().await;
        db.save_draft("a", "md-a", None).await.unwrap();
        db.save_draft("b", "md-b", None).await.unwrap();
        db.save_draft("c", "md-c", None).await.unwrap();

        let all = db.list_all().await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_delete() {
        let (_dir, db) = setup().await;
        db.save_draft("card-1", "content", None).await.unwrap();
        db.delete_draft("card-1").await.unwrap();
        assert!(db.load_draft("card-1").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_mark_synced_removes() {
        let (_dir, db) = setup().await;
        db.save_draft("card-1", "content", None).await.unwrap();
        db.mark_synced("card-1").await.unwrap();
        assert!(db.load_draft("card-1").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_concurrent_writes() {
        let (_dir, db) = setup().await;
        let mut handles = Vec::new();
        for i in 0..10 {
            let db = DraftDb {
                conn: db.conn.clone(),
            };
            handles.push(tokio::spawn(async move {
                db.save_draft(&format!("card-{}", i), &format!("md-{}", i), None)
                    .await
            }));
        }
        for h in handles {
            h.await.unwrap().unwrap();
        }
        let all = db.list_all().await.unwrap();
        assert_eq!(all.len(), 10);
    }
}
