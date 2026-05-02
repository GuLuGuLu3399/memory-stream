use rusqlite::params;

use crate::db::MetaDb;
use crate::error::MetaResult;
use crate::types::FtsHit;

impl MetaDb {
    pub fn upsert_fts(
        &self,
        uuid: &str,
        title: &str,
        body_text: &str,
        category_name: &str,
    ) -> MetaResult<()> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        tx.execute("DELETE FROM card_fts WHERE uuid = ?1", params![uuid])
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        tx.execute(
            "INSERT INTO card_fts (uuid, title, body_text, category_name) VALUES (?1, ?2, ?3, ?4)",
            params![uuid, title, body_text, category_name],
        )
        .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        tx.commit()
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    pub fn delete_fts(&self, uuid: &str) -> MetaResult<()> {
        self.conn
            .execute("DELETE FROM card_fts WHERE uuid = ?1", params![uuid])
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    pub fn search_fts(&self, query: &str, limit: usize) -> MetaResult<Vec<FtsHit>> {
        let query = if query.len() > 500 { &query[..500] } else { query };
        let mut stmt = self
            .conn
            .prepare(
                "SELECT uuid, title, snippet(card_fts, 1, '>>>', '<<<', '...', 32) as excerpt, rank
                 FROM card_fts WHERE card_fts MATCH ?1
                 ORDER BY rank LIMIT ?2",
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let rows = stmt
            .query_map(params![query, limit as i64], |row| {
                Ok(FtsHit {
                    uuid: row.get(0)?,
                    title: row.get(1)?,
                    excerpt: row.get(2)?,
                    rank: row.get(3)?,
                })
            })
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let mut hits = Vec::new();
        for hit in rows {
            hits.push(hit.map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?);
        }
        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    fn test_db() -> MetaDb {
        let id = std::sync::atomic::AtomicU64::fetch_add(
            &TEST_COUNTER,
            1,
            std::sync::atomic::Ordering::Relaxed,
        );
        let dir = std::env::temp_dir().join(format!("ms-meta-test-{}", id));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        MetaDb::open(&dir).unwrap()
    }

    #[test]
    fn test_fts_insert_and_search() {
        let db = test_db();

        db.upsert_fts(
            "uuid-1",
            "Rust 入门指南",
            "Rust 是一门系统编程语言，拥有所有权和借用机制",
            "programming",
        )
        .unwrap();
        db.upsert_fts(
            "uuid-2",
            "Go 并发模式",
            "Go 语言使用 goroutine 和 channel 实现并发",
            "programming",
        )
        .unwrap();
        db.upsert_fts("uuid-3", "数学分析笔记", "极限与连续性的基本定理", "math").unwrap();

        let hits = db.search_fts("Rust", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].uuid, "uuid-1");
        assert!(hits[0].excerpt.contains("Rust"));
    }

    #[test]
    fn test_fts_update() {
        let db = test_db();

        db.upsert_fts("uuid-1", "Old Title", "Old content", "cat").unwrap();
        db.upsert_fts("uuid-1", "New Title", "New content about Rust", "cat").unwrap();

        let hits = db.search_fts("Old", 10).unwrap();
        assert!(hits.is_empty(), "Old content should be gone after update");

        let hits = db.search_fts("Rust", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].uuid, "uuid-1");
    }

    #[test]
    fn test_fts_delete() {
        let db = test_db();

        db.upsert_fts("uuid-1", "Test", "Some content here", "").unwrap();
        db.delete_fts("uuid-1").unwrap();

        let hits = db.search_fts("content", 10).unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn test_fts_title_search() {
        let db = test_db();

        db.upsert_fts(
            "uuid-1",
            "Rust ownership model",
            "Ownership and borrowing in Rust",
            "programming",
        )
        .unwrap();
        db.upsert_fts("uuid-2", "Rust async await", "Async runtime and futures", "programming")
            .unwrap();

        let hits = db.search_fts("ownership", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].uuid, "uuid-1");
    }
}
