#![cfg(feature = "native")]

pub mod error;

use error::{DraftError, DraftResult};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::path::Path;
use parking_lot::Mutex;
use tokio::task;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draft {
    pub card_id: String,
    pub title: String,
    pub raw_md: String,
    pub ast_data: Option<String>,
    pub toc_data: Option<String>,
    pub excerpt: Option<String>,
    pub category_id: Option<i64>,
    pub extracted_links: Option<String>,
    pub sync_status: String,
    pub updated_at: i64,
}

fn extract_panic_source(join_err: tokio::task::JoinError) -> DraftError {
    let reason = if join_err.is_panic() {
        let panic_err = join_err.into_panic();
        if let Some(s) = panic_err.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_err.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        }
    } else {
        "Task cancelled".to_string()
    };
    DraftError::TaskPanic { reason }
}

pub struct DraftDb {
    conn: std::sync::Arc<Mutex<rusqlite::Connection>>,
}

impl DraftDb {
    /// 创建新的草稿数据库实例。
    ///
    /// # Errors
    /// 返回错误如果数据库打开失败或初始化失败。
    pub async fn new(db_path: &Path) -> DraftResult<Self> {
        let path = db_path.to_path_buf();
        task::spawn_blocking(move || {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| DraftError::OpenError(format!("创建目录失败: {e}")))?;
            }

            let conn = rusqlite::Connection::open(&path)
                .map_err(|e| DraftError::OpenError(format!("打开数据库失败: {e}")))?;

            conn.execute_batch(
                "PRAGMA journal_mode=WAL;
                 PRAGMA synchronous=NORMAL;
                 PRAGMA foreign_keys=ON;",
            )
            .map_err(|e| DraftError::OpenError(format!("设置 PRAGMA 失败: {e}")))?;

            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS drafts (
                    card_id TEXT PRIMARY KEY,
                    title TEXT NOT NULL DEFAULT '',
                    raw_md TEXT NOT NULL,
                    ast_data TEXT,
                    toc_data TEXT,
                    excerpt TEXT,
                    category_id INTEGER,
                    extracted_links TEXT,
                    sync_status TEXT NOT NULL DEFAULT 'pending',
                    updated_at INTEGER NOT NULL
                );",
            )
            .map_err(|e| DraftError::OpenError(format!("建表失败: {e}")))?;

            // ── Schema migration: 为旧表添加新列（忽略已存在错误） ──
            let migrations = [
                "ALTER TABLE drafts ADD COLUMN title TEXT NOT NULL DEFAULT ''",
                "ALTER TABLE drafts ADD COLUMN toc_data TEXT",
                "ALTER TABLE drafts ADD COLUMN excerpt TEXT",
                "ALTER TABLE drafts ADD COLUMN category_id INTEGER",
                "ALTER TABLE drafts ADD COLUMN extracted_links TEXT",
                "ALTER TABLE drafts ADD COLUMN sync_status TEXT NOT NULL DEFAULT 'pending'",
            ];
            for sql in &migrations {
                let _ = conn.execute_batch(sql); // 忽略 "duplicate column" 错误
            }

            Ok(Self {
                conn: std::sync::Arc::new(Mutex::new(conn)),
            })
        })
        .await
        .map_err(extract_panic_source)?
    }

    /// 保存草稿到数据库。
    ///
    /// # Errors
    /// 返回错误如果数据库操作失败。
    pub async fn save_draft(
        &self,
        card_id: &str,
        title: &str,
        raw_md: &str,
        ast_data: Option<&str>,
        toc_data: Option<&str>,
        excerpt: Option<&str>,
        category_id: Option<i64>,
        extracted_links: Option<&str>,
        sync_status: &str,
    ) -> DraftResult<()> {
        let conn = self.conn.clone();
        let card_id = card_id.to_string();
        let title = title.to_string();
        let raw_md = raw_md.to_string();
        let ast_data = ast_data.map(std::string::ToString::to_string);
        let toc_data = toc_data.map(std::string::ToString::to_string);
        let excerpt = excerpt.map(std::string::ToString::to_string);
        let extracted_links = extracted_links.map(std::string::ToString::to_string);
        let sync_status = sync_status.to_string();
        let now = chrono::Utc::now().timestamp();

        task::spawn_blocking(move || {
            let lock = conn.lock();
            lock.execute(
                "INSERT OR REPLACE INTO drafts (card_id, title, raw_md, ast_data, toc_data, excerpt, category_id, extracted_links, sync_status, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![card_id, title, raw_md, ast_data, toc_data, excerpt, category_id, extracted_links, sync_status, now],
            )?;
            Ok(())
        })
        .await
        .map_err(extract_panic_source)?
    }

    /// 从数据库加载草稿。
    ///
    /// # Errors
    /// 返回错误如果数据库查询失败。
    pub async fn load_draft(&self, card_id: &str) -> DraftResult<Option<Draft>> {
        let conn = self.conn.clone();
        let card_id = card_id.to_string();

        task::spawn_blocking(move || {
            let lock = conn.lock();
            let mut stmt = lock.prepare(
                "SELECT card_id, title, raw_md, ast_data, toc_data, excerpt, category_id, extracted_links, sync_status, updated_at FROM drafts WHERE card_id = ?1",
            )?;

            let mut rows = stmt.query(params![card_id])?;
            match rows.next()? {
                Some(row) => Ok(Some(row_to_draft(row)?)),
                None => Ok(None),
            }
        })
        .await
        .map_err(extract_panic_source)?
    }

    /// 列出所有草稿。
    ///
    /// # Errors
    /// 返回错误如果数据库查询失败。
    pub async fn list_all(&self) -> DraftResult<Vec<Draft>> {
        let conn = self.conn.clone();

        task::spawn_blocking(move || {
            let lock = conn.lock();
            let mut stmt = lock.prepare(
                "SELECT card_id, title, raw_md, ast_data, toc_data, excerpt, category_id, extracted_links, sync_status, updated_at FROM drafts ORDER BY updated_at DESC",
            )?;

            let drafts = stmt
                .query_map([], |row| row_to_draft(row))?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(drafts)
        })
        .await
        .map_err(extract_panic_source)?
    }

    /// 列出未同步的草稿。
    ///
    /// # Errors
    /// 返回错误如果数据库查询失败。
    pub async fn list_unsynced(&self) -> DraftResult<Vec<Draft>> {
        let conn = self.conn.clone();

        task::spawn_blocking(move || {
            let lock = conn.lock();
            let mut stmt = lock.prepare(
                "SELECT card_id, title, raw_md, ast_data, toc_data, excerpt, category_id, extracted_links, sync_status, updated_at FROM drafts WHERE sync_status = 'pending' ORDER BY updated_at ASC",
            )?;

            let drafts = stmt
                .query_map([], |row| row_to_draft(row))?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(drafts)
        })
        .await
        .map_err(extract_panic_source)?
    }

    /// 标记草稿为已同步。
    ///
    /// # Errors
    /// 返回错误如果数据库操作失败。
    pub async fn mark_synced(&self, card_id: &str) -> DraftResult<()> {
        let conn = self.conn.clone();
        let card_id = card_id.to_string();

        task::spawn_blocking(move || {
            let lock = conn.lock();
            lock.execute(
                "UPDATE drafts SET sync_status = 'synced' WHERE card_id = ?1",
                params![card_id],
            )?;
            Ok(())
        })
        .await
        .map_err(extract_panic_source)?
    }

    /// 乐观锁标记草稿为已同步。
    ///
    /// 仅当 `updated_at` 与 `expected_updated_at` 一致时才更新，
    /// 防止后台 worker 用旧数据覆盖新数据后误标记。
    ///
    /// 返回 `true` 表示实际更新了（即草稿未被修改），
    /// 返回 `false` 表示草稿在读取后被更新过，需要下次重新同步。
    pub async fn mark_synced_if_unchanged(
        &self,
        card_id: &str,
        expected_updated_at: i64,
    ) -> DraftResult<bool> {
        let conn = self.conn.clone();
        let card_id = card_id.to_string();

        task::spawn_blocking(move || {
            let lock = conn.lock();
            let rows = lock.execute(
                "UPDATE drafts SET sync_status = 'synced' WHERE card_id = ?1 AND updated_at = ?2",
                params![card_id, expected_updated_at],
            )?;
            Ok(rows > 0)
        })
        .await
        .map_err(extract_panic_source)?
    }

    /// 删除草稿。
    ///
    /// # Errors
    /// 返回错误如果数据库操作失败。
    pub async fn delete_draft(&self, card_id: &str) -> DraftResult<()> {
        let conn = self.conn.clone();
        let card_id = card_id.to_string();

        task::spawn_blocking(move || {
            let lock = conn.lock();
            lock.execute("DELETE FROM drafts WHERE card_id = ?1", params![card_id])?;
            Ok(())
        })
        .await
        .map_err(extract_panic_source)?
    }
}

fn row_to_draft(row: &rusqlite::Row<'_>) -> rusqlite::Result<Draft> {
    Ok(Draft {
        card_id: row.get(0)?,
        title: row.get(1)?,
        raw_md: row.get(2)?,
        ast_data: row.get(3)?,
        toc_data: row.get(4)?,
        excerpt: row.get(5)?,
        category_id: row.get(6)?,
        extracted_links: row.get(7)?,
        sync_status: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup() -> Result<(TempDir, DraftDb), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let db_path = dir.path().join("test_drafts.db");
        let db = DraftDb::new(&db_path).await?;
        Ok((dir, db))
    }

    #[tokio::test]
    async fn test_save_and_load() -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, db) = setup().await?;
        db.save_draft("card-1", "Test Title", "# Hello", Some(r#"{"type":"Root"}"#), None, Some("excerpt"), None, None, "pending")
            .await?;

        let draft = db.load_draft("card-1").await?;
        assert!(draft.is_some());
        let d = draft.ok_or("expected Some")?;
        assert_eq!(d.card_id, "card-1");
        assert_eq!(d.title, "Test Title");
        assert_eq!(d.raw_md, "# Hello");
        assert_eq!(d.ast_data, Some(r#"{"type":"Root"}"#.to_string()));
        assert_eq!(d.excerpt, Some("excerpt".to_string()));
        assert_eq!(d.sync_status, "pending");
        Ok(())
    }

    #[tokio::test]
    async fn test_load_nonexistent() -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, db) = setup().await?;
        let draft = db.load_draft("ghost").await?;
        assert!(draft.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_upsert_overwrites() -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, db) = setup().await?;
        db.save_draft("card-1", "t1", "v1", None, None, None, None, None, "pending").await?;
        db.save_draft("card-1", "t2", "v2", Some("ast2"), None, None, None, None, "pending").await?;

        let d = db.load_draft("card-1").await?.ok_or("expected Some")?;
        assert_eq!(d.title, "t2");
        assert_eq!(d.raw_md, "v2");
        assert_eq!(d.ast_data, Some("ast2".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_list_all() -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, db) = setup().await?;
        db.save_draft("a", "", "md-a", None, None, None, None, None, "pending").await?;
        db.save_draft("b", "", "md-b", None, None, None, None, None, "pending").await?;
        db.save_draft("c", "", "md-c", None, None, None, None, None, "pending").await?;

        let all = db.list_all().await?;
        assert_eq!(all.len(), 3);
        Ok(())
    }

    #[tokio::test]
    async fn test_delete() -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, db) = setup().await?;
        db.save_draft("card-1", "", "content", None, None, None, None, None, "pending").await?;
        db.delete_draft("card-1").await?;
        assert!(db.load_draft("card-1").await?.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_mark_synced_updates_status() -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, db) = setup().await?;
        db.save_draft("card-1", "", "content", None, None, None, None, None, "pending").await?;
        db.mark_synced("card-1").await?;
        let d = db.load_draft("card-1").await?.ok_or("expected Some")?;
        assert_eq!(d.sync_status, "synced");
        Ok(())
    }

    #[tokio::test]
    async fn test_list_unsynced_filters_by_status() -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, db) = setup().await?;
        db.save_draft("a", "", "md-a", None, None, None, None, None, "pending").await?;
        db.save_draft("b", "", "md-b", None, None, None, None, None, "synced").await?;
        db.save_draft("c", "", "md-c", None, None, None, None, None, "pending").await?;

        let unsynced = db.list_unsynced().await?;
        assert_eq!(unsynced.len(), 2);
        let ids: Vec<&str> = unsynced.iter().map(|d| d.card_id.as_str()).collect();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"c"));
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_writes() -> Result<(), Box<dyn std::error::Error>> {
        let (_dir, db) = setup().await?;
        let mut handles = Vec::new();
        for i in 0..10 {
            let db = DraftDb {
                conn: db.conn.clone(),
            };
            handles.push(tokio::spawn(async move {
                db.save_draft(&format!("card-{i}"), "", &format!("md-{i}"), None, None, None, None, None, "pending")
                    .await
            }));
        }
        for h in handles {
            h.await??;
        }
        let all = db.list_all().await?;
        assert_eq!(all.len(), 10);
        Ok(())
    }
}
