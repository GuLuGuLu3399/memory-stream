/**
 * 🌟 SQLite 本地缓存层 — Local-First 零延迟秒开
 *
 * 启动时优先渲染本地坐标，同步成功后再平滑修正。
 * 缓存卡片布局坐标、边关系、热度数据，确保多端视觉体验一致。
 *
 * 2026 适配：增加 hot_score 字段 + cached_edges 表（CardEdge 关系缓存）
 */
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 卡片布局缓存记录（与 Go 后端 CardLayout + CardMetrics 对齐）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedLayout {
    pub card_id: String,
    pub x: f64,
    pub y: f64,
    pub title: String,
    pub category_id: Option<String>,
    pub hot_score: f64,
    pub updated_at: String,
}

/// 卡片边关系缓存（与 Go 后端 CardEdge 对齐）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedEdge {
    pub source_id: String,
    pub target_id: String,
    pub relation: String, // "sequence" | "reference"
}

/// 缓存管理器
pub struct CacheManager {
    conn: Connection,
}

impl CacheManager {
    /// 打开或创建本地 SQLite 缓存数据库
    pub fn open(db_path: &str) -> Result<Self> {
        let path = Path::new(db_path);

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(db_path)?;
        let manager = Self { conn };
        manager.init_tables()?;
        Ok(manager)
    }

    /// 创建内存缓存（用于测试）
    #[allow(dead_code)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let manager = Self { conn };
        manager.init_tables()?;
        Ok(manager)
    }

    /// 初始化缓存表结构（版本化迁移）
    ///
    /// 使用 SQLite `user_version` PRAGMA 管理迁移链：
    /// - v0 → v1: 全新数据库，创建所有表
    /// - 未来版本递增时添加新的迁移函数
    ///
    /// 不再使用 DROP TABLE，确保缓存数据跨启动保留。
    fn init_tables(&self) -> Result<()> {
        // 读取当前 schema 版本
        let current_version: i32 = self
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))?;

        match current_version {
            0 => {
                // v0 → v1：全新数据库，创建完整表结构
                self.conn.execute_batch(
                    "
                    CREATE TABLE IF NOT EXISTS cached_layouts (
                        card_id       TEXT PRIMARY KEY,
                        x             REAL NOT NULL DEFAULT 0,
                        y             REAL NOT NULL DEFAULT 0,
                        title         TEXT NOT NULL DEFAULT '',
                        category_id   TEXT,
                        hot_score     REAL NOT NULL DEFAULT 0,
                        updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
                    );

                    CREATE TABLE IF NOT EXISTS cached_edges (
                        source_id  TEXT NOT NULL,
                        target_id  TEXT NOT NULL,
                        relation   TEXT NOT NULL,
                        PRIMARY KEY (source_id, target_id)
                    );

                    CREATE TABLE IF NOT EXISTS sync_state (
                        key   TEXT PRIMARY KEY,
                        value TEXT NOT NULL DEFAULT ''
                    );

                    CREATE INDEX IF NOT EXISTS idx_cached_layouts_category
                    ON cached_layouts (category_id);

                    CREATE INDEX IF NOT EXISTS idx_cached_layouts_hot
                    ON cached_layouts (hot_score DESC);

                    CREATE INDEX IF NOT EXISTS idx_cached_edges_relation
                    ON cached_edges (relation);
                    ",
                )?;
                self.set_schema_version(1)?;
            }
            // 未来迁移示例：
            // 1 => { self.migrate_v1_to_v2()?; }
            _ => {
                // 已是最新版本，无需迁移
            }
        }

        Ok(())
    }

    /// 设置 SQLite user_version PRAGMA
    fn set_schema_version(&self, version: i32) -> Result<()> {
        self.conn.pragma_update(None, "user_version", version)?;
        Ok(())
    }

    /// 批量写入/更新布局缓存
    pub fn upsert_layouts(&self, layouts: &[CachedLayout]) -> Result<usize> {
        let tx = self.conn.unchecked_transaction()?;

        let mut count = 0;
        for layout in layouts {
            let rows = tx.execute(
                "INSERT INTO cached_layouts (card_id, x, y, title, category_id, hot_score, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(card_id) DO UPDATE SET
                    x = excluded.x,
                    y = excluded.y,
                    title = excluded.title,
                    category_id = excluded.category_id,
                    hot_score = excluded.hot_score,
                    updated_at = excluded.updated_at",
                params![
                    layout.card_id,
                    layout.x,
                    layout.y,
                    layout.title,
                    layout.category_id,
                    layout.hot_score,
                    layout.updated_at,
                ],
            )?;
            count += rows;
        }

        tx.commit()?;
        Ok(count)
    }

    /// 批量写入/更新边关系缓存
    pub fn upsert_edges(&self, edges: &[CachedEdge]) -> Result<usize> {
        let tx = self.conn.unchecked_transaction()?;

        let mut count = 0;
        for edge in edges {
            let rows = tx.execute(
                "INSERT INTO cached_edges (source_id, target_id, relation)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(source_id, target_id) DO UPDATE SET
                    relation = excluded.relation",
                params![edge.source_id, edge.target_id, edge.relation,],
            )?;
            count += rows;
        }

        tx.commit()?;
        Ok(count)
    }

    /// 获取全部缓存布局（启动时零延迟渲染，按热度排序）
    pub fn get_all_layouts(&self) -> Result<Vec<CachedLayout>> {
        let mut stmt = self.conn.prepare(
            "SELECT card_id, x, y, title, category_id, hot_score, updated_at
             FROM cached_layouts
             ORDER BY y ASC, x ASC",
        )?;

        let layouts = stmt.query_map([], |row| {
            Ok(CachedLayout {
                card_id: row.get(0)?,
                x: row.get(1)?,
                y: row.get(2)?,
                title: row.get(3)?,
                category_id: row.get(4)?,
                hot_score: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        layouts.collect()
    }

    /// 获取热门布局（按 hot_score 降序）
    #[allow(dead_code)]
    pub fn get_hot_layouts(&self, limit: i64) -> Result<Vec<CachedLayout>> {
        let mut stmt = self.conn.prepare(
            "SELECT card_id, x, y, title, category_id, hot_score, updated_at
             FROM cached_layouts
             WHERE hot_score > 0
             ORDER BY hot_score DESC
             LIMIT ?1",
        )?;

        let layouts = stmt.query_map(params![limit], |row| {
            Ok(CachedLayout {
                card_id: row.get(0)?,
                x: row.get(1)?,
                y: row.get(2)?,
                title: row.get(3)?,
                category_id: row.get(4)?,
                hot_score: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        layouts.collect()
    }

    /// 获取全部缓存边关系（启动时渲染连线）
    pub fn get_all_edges(&self) -> Result<Vec<CachedEdge>> {
        let mut stmt = self.conn.prepare(
            "SELECT source_id, target_id, relation
             FROM cached_edges",
        )?;

        let edges = stmt.query_map([], |row| {
            Ok(CachedEdge {
                source_id: row.get(0)?,
                target_id: row.get(1)?,
                relation: row.get(2)?,
            })
        })?;

        edges.collect()
    }

    /// 获取指定分类的布局
    #[allow(dead_code)]
    pub fn get_layouts_by_category(&self, category_id: &str) -> Result<Vec<CachedLayout>> {
        let mut stmt = self.conn.prepare(
            "SELECT card_id, x, y, title, category_id, hot_score, updated_at
             FROM cached_layouts
             WHERE category_id = ?1
             ORDER BY y ASC",
        )?;

        let layouts = stmt.query_map(params![category_id], |row| {
            Ok(CachedLayout {
                card_id: row.get(0)?,
                x: row.get(1)?,
                y: row.get(2)?,
                title: row.get(3)?,
                category_id: row.get(4)?,
                hot_score: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        layouts.collect()
    }

    /// 清除所有缓存（强制全量同步时使用）
    #[allow(dead_code)]
    pub fn clear_all(&self) -> Result<usize> {
        let layouts = self.conn.execute("DELETE FROM cached_layouts", [])?;
        let edges = self.conn.execute("DELETE FROM cached_edges", [])?;
        Ok(layouts + edges)
    }

    /// 获取最后同步时间
    pub fn get_last_sync_time(&self) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM sync_state WHERE key = 'last_sync'")?;

        let result = stmt.query_row([], |row| row.get::<_, String>(0)).ok();
        Ok(result)
    }

    /// 更新最后同步时间
    pub fn set_last_sync_time(&self, time: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sync_state (key, value) VALUES ('last_sync', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![time],
        )?;
        Ok(())
    }

    /// 获取缓存数量
    pub fn count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM cached_layouts", [], |row| row.get(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_roundtrip() {
        let cache = CacheManager::open_in_memory().unwrap();

        let layouts = vec![
            CachedLayout {
                card_id: "card-1".to_string(),
                x: 0.0,
                y: 0.0,
                title: "Test Card 1".to_string(),
                category_id: Some("cat-1".to_string()),
                hot_score: 3.5,
                updated_at: "2026-03-27T00:00:00".to_string(),
            },
            CachedLayout {
                card_id: "card-2".to_string(),
                x: 400.0,
                y: 200.0,
                title: "Test Card 2".to_string(),
                category_id: Some("cat-1".to_string()),
                hot_score: 7.2,
                updated_at: "2026-03-27T00:00:00".to_string(),
            },
        ];

        cache.upsert_layouts(&layouts).unwrap();
        assert_eq!(cache.count().unwrap(), 2);

        let cached = cache.get_all_layouts().unwrap();
        assert_eq!(cached.len(), 2);
        assert_eq!(cached[0].card_id, "card-1");
        assert_eq!(cached[1].hot_score, 7.2);

        // 测试边关系
        let edges = vec![CachedEdge {
            source_id: "card-1".to_string(),
            target_id: "card-2".to_string(),
            relation: "sequence".to_string(),
        }];
        cache.upsert_edges(&edges).unwrap();
        let cached_edges = cache.get_all_edges().unwrap();
        assert_eq!(cached_edges.len(), 1);
        assert_eq!(cached_edges[0].relation, "sequence");

        // 测试热度排序
        let hot = cache.get_hot_layouts(10).unwrap();
        assert_eq!(hot[0].card_id, "card-2"); // hot_score 7.2 > 3.5

        cache.set_last_sync_time("2026-03-27T21:00:00").unwrap();
        let sync_time = cache.get_last_sync_time().unwrap();
        assert_eq!(sync_time, Some("2026-03-27T21:00:00".to_string()));
    }
}
