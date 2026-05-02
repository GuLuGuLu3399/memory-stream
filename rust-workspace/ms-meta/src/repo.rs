use rusqlite::params;

use crate::db::MetaDb;
use crate::error::MetaResult;
use crate::types::{CardIndex, RelationRecord, SyncStatus};

// ── card_index ──

impl MetaDb {
    /// 插入或更新卡片索引记录。
    ///
    /// 使用 `INSERT OR REPLACE` 语义，如果 UUID 已存在则覆盖所有字段。
    pub fn upsert_card(&self, card: &CardIndex) -> MetaResult<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO card_index (uuid, file_path, file_hash, version, sync_status, last_synced_hash)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    &card.uuid,
                    &card.file_path,
                    &card.file_hash,
                    &card.version,
                    &card.sync_status,
                    &card.last_synced_hash,
                ],
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// 根据 UUID 查询卡片索引。
    ///
    /// 返回 `Ok(Some(card))` 如果找到，`Ok(None)` 如果不存在。
    pub fn get_card(&self, uuid: &str) -> MetaResult<Option<CardIndex>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT uuid, file_path, file_hash, version, sync_status, last_synced_hash
                 FROM card_index WHERE uuid = ?1",
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let mut rows = stmt
            .query(params![uuid])
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?
        {
            Ok(Some(CardIndex {
                uuid: row.get(0)?,
                file_path: row.get(1)?,
                file_hash: row.get(2)?,
                version: row.get(3)?,
                sync_status: row.get(4)?,
                last_synced_hash: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// 根据文件路径查询卡片索引。
    ///
    /// 返回 `Ok(Some(card))` 如果找到，`Ok(None)` 如果不存在。
    pub fn get_by_path(&self, file_path: &str) -> MetaResult<Option<CardIndex>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT uuid, file_path, file_hash, version, sync_status, last_synced_hash
                 FROM card_index WHERE file_path = ?1",
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let mut rows = stmt
            .query(params![file_path])
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?
        {
            Ok(Some(CardIndex {
                uuid: row.get(0)?,
                file_path: row.get(1)?,
                file_hash: row.get(2)?,
                version: row.get(3)?,
                sync_status: row.get(4)?,
                last_synced_hash: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// 根据 UUID 删除卡片索引记录。
    ///
    /// 注意：此操作不清理关联的关系索引，通常应配合 `delete_relations_for_uuid` 使用。
    pub fn delete_card(&self, uuid: &str) -> MetaResult<()> {
        self.conn
            .execute("DELETE FROM card_index WHERE uuid = ?1", params![uuid])
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// 删除指定 UUID 的所有关系记录（作为源或目标）。
    ///
    /// 删除条件：`source_uuid = uuid` 或 `target_uuid_or_tag = uuid`。
    pub fn delete_relations_for_uuid(&self, uuid: &str) -> MetaResult<()> {
        self.conn
            .execute(
                "DELETE FROM relation_index WHERE source_uuid = ?1 OR target_uuid_or_tag = ?1",
                params![uuid],
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// 根据文件路径删除卡片及其所有关联数据。
    ///
    /// 级联删除：卡片索引 + 关系索引 + FTS 索引。
    /// 返回 `Ok(true)` 如果找到并删除，`Ok(false)` 如果文件路径不存在。
    pub fn delete_card_by_path(&self, file_path: &str) -> MetaResult<bool> {
        let card = self.get_by_path(file_path)?;
        let Some(card) = card else { return Ok(false) };
        let uuid = &card.uuid;

        self.delete_relations_for_uuid(uuid)?;
        self.delete_fts(uuid)?;

        self.delete_card(uuid)?;
        Ok(true)
    }

    /// 列出所有卡片索引记录。
    ///
    /// 返回完整的 `Vec<CardIndex>`，顺序取决于数据库存储顺序。
    pub fn list_all(&self) -> MetaResult<Vec<CardIndex>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT uuid, file_path, file_hash, version, sync_status, last_synced_hash
                 FROM card_index",
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(CardIndex {
                    uuid: row.get(0)?,
                    file_path: row.get(1)?,
                    file_hash: row.get(2)?,
                    version: row.get(3)?,
                    sync_status: row.get(4)?,
                    last_synced_hash: row.get(5)?,
                })
            })
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let mut cards = Vec::new();
        for card in rows {
            cards.push(card.map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?);
        }
        Ok(cards)
    }

    /// 更新指定卡片的同步状态。
    ///
    /// 直接设置 `sync_status` 字段，不修改其他字段。
    pub fn set_sync_status(&self, uuid: &str, status: SyncStatus) -> MetaResult<()> {
        self.conn
            .execute(
                "UPDATE card_index SET sync_status = ?1 WHERE uuid = ?2",
                params![status, uuid],
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// 列出所有待删除卡片的 UUID。
    ///
    /// 筛选条件：`sync_status = PendingDelete`。
    pub fn list_pending_deletes(&self) -> MetaResult<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT uuid FROM card_index WHERE sync_status = ?1")
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let rows = stmt
            .query_map(params![SyncStatus::PendingDelete], |row| row.get::<_, String>(0))
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let mut uuids = Vec::new();
        for uuid in rows {
            uuids.push(uuid.map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?);
        }
        Ok(uuids)
    }

    /// 根据同步状态筛选卡片 UUID 列表。
    ///
    /// 支持任意 `SyncStatus` 值作为筛选条件。
    pub fn list_uuids_by_sync_status(&self, status: SyncStatus) -> MetaResult<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT uuid FROM card_index WHERE sync_status = ?1")
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let rows = stmt
            .query_map(params![status], |row| row.get::<_, String>(0))
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let mut uuids = Vec::new();
        for uuid in rows {
            uuids.push(uuid.map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?);
        }
        Ok(uuids)
    }

    /// 递增卡片的版本号并返回新值。
    ///
    /// 原子操作：`version = version + 1`。
    pub fn bump_version(&self, uuid: &str) -> MetaResult<i64> {
        self.conn
            .execute(
                "UPDATE card_index SET version = version + 1 WHERE uuid = ?1",
                params![uuid],
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let version: i64 = self
            .conn
            .query_row(
                "SELECT version FROM card_index WHERE uuid = ?1",
                params![uuid],
                |row| row.get(0),
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(version)
    }

    /// 原子标记卡片为脏：更新哈希、设置为 PendingPush、递增版本号。
    pub fn mark_dirty(&self, uuid: &str, file_hash: &str) -> MetaResult<()> {
        self.conn
            .execute(
                "UPDATE card_index SET file_hash = ?1, sync_status = 'pending_push', version = version + 1 WHERE uuid = ?2",
                params![file_hash, uuid],
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }
}

// ── relation_index ──

impl MetaDb {
    /// 替换指定源卡片的所有 link 类型关系（事务操作）。
    ///
    /// 仅管理 link 类型关系，trunk 关系通过专用方法管理，不会被触碰。
    pub fn replace_relations(&self, source: &str, edges: &[RelationRecord]) -> MetaResult<()> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        // Only delete link relations — trunk relations are managed exclusively
        // through insert_trunk_relation / delete_relation and must never be
        // touched by AST-derived link extraction.
        tx.execute(
            "DELETE FROM relation_index WHERE source_uuid = ?1 AND relation_type = 'link'",
            params![source],
        )
        .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        for edge in edges {
            // Skip trunk entries — this function only manages links
            if edge.relation_type == "trunk" {
                continue;
            }
            tx.execute(
                "INSERT INTO relation_index (source_uuid, target_uuid_or_tag, relation_type)
                 VALUES (?1, ?2, ?3)",
                params![&edge.source_uuid, &edge.target_uuid_or_tag, &edge.relation_type],
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// 查询指定源卡片的所有出边关系。
    pub fn get_outbound(&self, source: &str) -> MetaResult<Vec<RelationRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT source_uuid, target_uuid_or_tag, relation_type
                 FROM relation_index WHERE source_uuid = ?1",
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let rows = stmt
            .query_map(params![source], |row| {
                Ok(RelationRecord {
                    source_uuid: row.get(0)?,
                    target_uuid_or_tag: row.get(1)?,
                    relation_type: row.get(2)?,
                })
            })
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let mut relations = Vec::new();
        for rel in rows {
            relations.push(rel.map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?);
        }
        Ok(relations)
    }

    /// List all trunk relations in the database (for trunk-only sync).
    pub fn list_all_trunks(&self) -> MetaResult<Vec<RelationRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT source_uuid, target_uuid_or_tag, relation_type
                 FROM relation_index WHERE relation_type = 'trunk'",
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(RelationRecord {
                    source_uuid: row.get(0)?,
                    target_uuid_or_tag: row.get(1)?,
                    relation_type: row.get(2)?,
                })
            })
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let mut relations = Vec::new();
        for rel in rows {
            relations.push(rel.map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?);
        }
        Ok(relations)
    }

    /// 插入一条 trunk（父子主干）关系，使用 INSERT OR REPLACE 语义。
    pub fn insert_trunk_relation(&self, source: &str, target: &str) -> MetaResult<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO relation_index (source_uuid, target_uuid_or_tag, relation_type)
                 VALUES (?1, ?2, 'trunk')",
                params![source, target],
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// 替换指定源卡片的所有 trunk 关系（事务操作）。
    ///
    /// 同步拉取时使用，使本地 trunk 关系与云端状态一致。
    pub fn replace_trunk_relations(&self, source: &str, targets: &[String]) -> MetaResult<()> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        tx.execute(
            "DELETE FROM relation_index WHERE source_uuid = ?1 AND relation_type = 'trunk'",
            params![source],
        )
        .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        for target in targets {
            tx.execute(
                "INSERT OR REPLACE INTO relation_index (source_uuid, target_uuid_or_tag, relation_type)
                 VALUES (?1, ?2, 'trunk')",
                params![source, target],
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// 插入一条 link 关系，仅在关系不存在时插入（INSERT OR IGNORE 语义）。
    pub fn insert_link_relation(&self, source: &str, target: &str) -> MetaResult<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO relation_index (source_uuid, target_uuid_or_tag, relation_type)
                 SELECT ?1, ?2, 'link'
                 WHERE NOT EXISTS (
                     SELECT 1 FROM relation_index WHERE source_uuid = ?1 AND target_uuid_or_tag = ?2
                 )",
                params![source, target],
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// 查询指向指定目标卡片的所有入边关系。
    pub fn get_inbound(&self, target: &str) -> MetaResult<Vec<RelationRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT source_uuid, target_uuid_or_tag, relation_type
                 FROM relation_index WHERE target_uuid_or_tag = ?1",
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let rows = stmt
            .query_map(params![target], |row| {
                Ok(RelationRecord {
                    source_uuid: row.get(0)?,
                    target_uuid_or_tag: row.get(1)?,
                    relation_type: row.get(2)?,
                })
            })
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let mut relations = Vec::new();
        for rel in rows {
            relations.push(rel.map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?);
        }
        Ok(relations)
    }

    /// 删除指定的源→目标关系记录。
    pub fn delete_relation(&self, source: &str, target: &str) -> MetaResult<()> {
        self.conn
            .execute(
                "DELETE FROM relation_index WHERE source_uuid = ?1 AND target_uuid_or_tag = ?2",
                params![source, target],
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// 根据 UUID 查询卡片标题（从 FTS 索引读取）。
    pub fn get_card_title(&self, uuid: &str) -> MetaResult<Option<String>> {
        self.conn
            .query_row(
                "SELECT title FROM card_fts WHERE uuid = ?1 LIMIT 1",
                params![uuid],
                |row| row.get(0),
            )
            .ok()
            .map_or(Ok(None), |v| Ok(Some(v)))
    }

    /// 根据标题查询卡片 UUID（精确匹配，从 FTS 索引读取）。
    pub fn get_uuid_by_title(&self, title: &str) -> MetaResult<Option<String>> {
        self.conn
            .query_row(
                "SELECT uuid FROM card_fts WHERE title = ?1 LIMIT 1",
                params![title],
                |row| row.get(0),
            )
            .ok()
            .map_or(Ok(None), |v| Ok(Some(v)))
    }
}

// ── asset_refs ──

impl MetaDb {
    /// 增加资产引用计数（首次引用时插入，已存在时递增）。
    pub fn ref_asset(&self, path: &str) -> MetaResult<()> {
        self.conn
            .execute(
                "INSERT INTO asset_refs (local_path, ref_count)
                 VALUES (?1, 1)
                 ON CONFLICT(local_path) DO UPDATE SET ref_count = ref_count + 1",
                params![path],
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// 减少资产引用计数，计数归零时自动删除记录。
    pub fn deref_asset(&self, path: &str) -> MetaResult<()> {
        self.conn
            .execute(
                "UPDATE asset_refs SET ref_count = ref_count - 1 WHERE local_path = ?1",
                params![path],
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        self.conn
            .execute(
                "DELETE FROM asset_refs WHERE local_path = ?1 AND ref_count <= 0",
                params![path],
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// 列出所有引用计数为零的孤立资产路径（可供垃圾回收）。
    pub fn orphan_assets(&self) -> MetaResult<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT local_path FROM asset_refs WHERE ref_count = 0")
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let mut assets = Vec::new();
        for asset in rows {
            assets.push(asset.map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?);
        }
        Ok(assets)
    }

    // ── Atomic trunk operations (transaction-wrapped) ──

    /// 事务操作：验证双方卡片存在后插入 trunk 关系。
    ///
    /// 整个操作在单一事务中完成，防止验证与插入之间的竞态。
    pub fn create_trunk_atomic(&self, source: &str, target: &str) -> MetaResult<()> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        let src_exists: bool = tx
            .query_row(
                "SELECT COUNT(*) > 0 FROM card_index WHERE uuid = ?1",
                params![source],
                |row| row.get(0),
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        if !src_exists {
            return Err(crate::error::MetaDbError::QueryFailed(format!(
                "Source card not found: {source}"
            )));
        }

        let tgt_exists: bool = tx
            .query_row(
                "SELECT COUNT(*) > 0 FROM card_index WHERE uuid = ?1",
                params![target],
                |row| row.get(0),
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        if !tgt_exists {
            return Err(crate::error::MetaDbError::QueryFailed(format!(
                "Target card not found: {target}"
            )));
        }

        tx.execute(
            "INSERT OR REPLACE INTO relation_index (source_uuid, target_uuid_or_tag, relation_type)
             VALUES (?1, ?2, 'trunk')",
            params![source, target],
        )
        .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        tx.commit()
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// 事务操作：删除关系后按需回插为 link 类型。
    ///
    /// 用于 trunk 断开时，若源卡片内容仍包含指向目标卡片的 wiki link，
    /// 则自动降级为 link 关系。`fallback_link` 参数由调用方预先判定。
    pub fn delete_trunk_with_fallback_atomic(
        &self,
        source: &str,
        target: &str,
        fallback_link: bool,
    ) -> MetaResult<()> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        tx.execute(
            "DELETE FROM relation_index WHERE source_uuid = ?1 AND target_uuid_or_tag = ?2",
            params![source, target],
        )
        .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        if fallback_link {
            tx.execute(
                "INSERT OR REPLACE INTO relation_index (source_uuid, target_uuid_or_tag, relation_type)
                 VALUES (?1, ?2, 'link')",
                params![source, target],
            )
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// 事务操作：反转 trunk 方向（删除旧方向，插入新方向）。
    ///
    /// 原子地完成方向反转，防止中间状态导致关系丢失。
    pub fn reverse_trunk_atomic(&self, source: &str, target: &str) -> MetaResult<()> {
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        tx.execute(
            "DELETE FROM relation_index WHERE source_uuid = ?1 AND target_uuid_or_tag = ?2",
            params![source, target],
        )
        .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        tx.execute(
            "INSERT OR REPLACE INTO relation_index (source_uuid, target_uuid_or_tag, relation_type)
             VALUES (?1, ?2, 'trunk')",
            params![target, source],
        )
        .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;

        tx.commit()
            .map_err(|e| crate::error::MetaDbError::QueryFailed(e.to_string()))?;
        Ok(())
    }
}

