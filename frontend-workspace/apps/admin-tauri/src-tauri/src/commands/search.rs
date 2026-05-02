use crate::db::{compute_hash, with_db, CardIndexDto, FtsHitDto};
use crate::models::{category_path, reserve_card_path};
use crate::state::AppState;

#[tauri::command]
pub fn search_fts(
    query: String,
    limit: i64,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<FtsHitDto>, String> {
    let hits = with_db(&state, |db| db.search_fts(&query, limit as usize))?;
    Ok(hits.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub fn materialize_ghost(
    title: String,
    category: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<CardIndexDto, String> {
    // Check if a card with this title already exists via FTS
    let existing = with_db(&state, |db| db.search_fts(&title, 1))?;
    if let Some(hit) = existing.first() {
        if hit.title == title {
            let card = with_db(&state, |db| db.get_card(&hit.uuid))?
                .ok_or_else(|| "FTS hit but no card_index entry".to_string())?;
            return Ok(card.into());
        }
    }

    // Not found — create it
    let uuid = uuid::Uuid::new_v4().to_string();
    let category = category.unwrap_or_default();

    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        if cfg.vault_path.is_empty() {
            return Err("Vault path not configured".into());
        }
        cfg.vault_path.clone()
    };

    let dir = category_path(std::path::Path::new(&vault_path), &category)?;
    ms_io::fs::ensure_dir(&dir).map_err(|e| e.to_string())?;
    let (title, file_path) = reserve_card_path(&dir, &title);
    let content = format!("---\nuuid: {uuid}\ntitle: {title}\n---\n\n# {title}\n");
    ms_io::fs::write_atomic(&file_path, &content).map_err(|e| e.to_string())?;

    let file_hash = compute_hash(&content);
    let card = ms_meta::CardIndex {
        uuid,
        file_path: file_path.to_string_lossy().to_string(),
        file_hash,
        version: 1,
        sync_status: ms_meta::SyncStatus::PendingPush,
        last_synced_hash: None,
    };

    with_db(&state, |db| db.upsert_card(&card))?;
    with_db(&state, |db| {
        db.upsert_fts(&card.uuid, &title, &content, &category)
    })?;

    Ok(card.into())
}
