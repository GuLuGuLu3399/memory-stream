use serde::Serialize;

use crate::state::AppState;

// ── DTO types (ms-meta types lack Serialize) ──────────────────────

#[derive(Serialize)]
pub struct CardIndexDto {
    pub uuid: String,
    pub file_path: String,
    pub file_hash: String,
    pub version: i64,
    pub sync_status: String,
    pub last_synced_hash: Option<String>,
}

impl From<ms_meta::CardIndex> for CardIndexDto {
    fn from(c: ms_meta::CardIndex) -> Self {
        Self {
            uuid: c.uuid,
            file_path: c.file_path,
            file_hash: c.file_hash,
            version: c.version,
            sync_status: c.sync_status.as_str().into(),
            last_synced_hash: c.last_synced_hash,
        }
    }
}

#[derive(Serialize)]
pub struct FtsHitDto {
    pub uuid: String,
    pub title: String,
    pub excerpt: String,
    pub rank: f64,
}

impl From<ms_meta::FtsHit> for FtsHitDto {
    fn from(h: ms_meta::FtsHit) -> Self {
        Self {
            uuid: h.uuid,
            title: h.title,
            excerpt: h.excerpt,
            rank: h.rank,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct TreeNodeDto {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub is_dir: bool,
    pub children: Vec<TreeNodeDto>,
}

// ── Lazy-load MetaDb helper ───────────────────────────────────────

pub fn with_db<F, R>(state: &AppState, f: F) -> Result<R, String>
where
    F: FnOnce(&ms_meta::MetaDb) -> ms_meta::MetaResult<R>,
{
    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        if cfg.vault_path.is_empty() {
            return Err("Vault path not configured".into());
        }
        cfg.vault_path.clone()
    };
    let mut guard = state.meta_db.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        let db = ms_meta::MetaDb::open(std::path::Path::new(&vault_path))
            .map_err(|e| format!("MetaDb open failed: {e}"))?;
        *guard = Some(db);
    }
    let db = guard.as_ref().ok_or("MetaDb not initialized")?;
    f(db).map_err(|e| e.to_string())
}

// ── Hashing helper ────────────────────────────────────────────────

pub fn compute_hash(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
