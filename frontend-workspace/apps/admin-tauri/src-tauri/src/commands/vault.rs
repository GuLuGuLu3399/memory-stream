use std::collections::{HashMap, HashSet};

use crate::db::{compute_hash, with_db, CardIndexDto, TreeNodeDto};
use crate::events::FileChangeEvent;
use crate::models::{card_file_path, category_path, path_stem, reserve_card_path, CardMeta};
use crate::state::AppState;

const MAX_CARD_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

fn check_file_size(path: &std::path::Path) -> Result<(), String> {
    let len = std::fs::metadata(path)
        .map_err(|e| format!("Cannot read file metadata: {e}"))?
        .len();
    if len > MAX_CARD_FILE_SIZE {
        return Err(format!("File too large: {} bytes (max 10MB)", len));
    }
    Ok(())
}

#[tauri::command]
pub fn read_card_file(uuid: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        if cfg.vault_path.is_empty() {
            return Err("Vault path not configured".into());
        }
        cfg.vault_path.clone()
    };

    // Try MetaDb first
    if let Some(card) = with_db(&state, |db| db.get_card(&uuid))? {
        // File still at DB-recorded path — read and self-heal YAML
        check_file_size(std::path::Path::new(&card.file_path))?;
        if let Ok(raw_content) = std::fs::read_to_string(&card.file_path) {
        let parsed = crate::parse_markdown_document(&raw_content)?;
        let file_title =
            path_stem(std::path::Path::new(&card.file_path)).unwrap_or_else(|| uuid.clone());
        let category = derive_category(&card.file_path, &vault_path);

        if let Some(meta) = &parsed.meta {
            if meta.title != file_title || meta.category != category {
                let healed = compose_card_content(
                    &CardMeta {
                        uuid: meta.uuid.clone(),
                        title: file_title,
                        category: category.clone(),
                        created_at: meta.created_at.clone(),
                        updated_at: now_rfc3339(),
                    },
                    &parsed.content,
                )?;
                if healed != raw_content {
                    ms_io::fs::write_atomic(std::path::Path::new(&card.file_path), &healed)
                        .map_err(|e| e.to_string())?;
                }
                return Ok(healed);
            }
            return Ok(raw_content);
        } else {
            let healed = compose_card_content(
                &CardMeta {
                    uuid: uuid.clone(),
                    title: file_title.clone(),
                    category: category.clone(),
                    created_at: now_rfc3339(),
                    updated_at: now_rfc3339(),
                },
                &parsed.content,
            )?;
            if healed != raw_content {
                ms_io::fs::write_atomic(std::path::Path::new(&card.file_path), &healed)
                    .map_err(|e| e.to_string())?;
            }
            return Ok(healed);
        }
        } // file still at DB path
        // File moved externally — fall through to vault scan to relocate
    }

    // Not in MetaDb or file relocated — scan vault and auto-import
    let root = std::path::Path::new(&vault_path);
    let files = ms_io::scanner::scan_markdown_files(root).map_err(|e| e.to_string())?;
    let mut found = None;
    for file in &files {
        let raw = match std::fs::read_to_string(file) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("[read_card_file] scan read failed {}: {e}", file.display());
                continue;
            }
        };
        if let Ok(parsed) = crate::parse_markdown_document(&raw) {
            if parsed.meta.as_ref().is_some_and(|meta| meta.uuid == uuid) {
                found = Some(file.clone());
                break;
            }
        }
    }

    // Fallback: match by file stem for external files without frontmatter
    if found.is_none() {
        for file in &files {
            if path_stem(file).as_deref() == Some(uuid.as_str()) {
                found = Some(file.clone());
                break;
            }
        }
    }

    let found = found.ok_or_else(|| format!("Card file not found: {uuid}"))?;

    let raw_content = std::fs::read_to_string(&found).map_err(|e| format!("Read failed: {e}"))?;
    let parsed = crate::parse_markdown_document(&raw_content)?;
    let category = found
        .parent()
        .and_then(|p| p.strip_prefix(root).ok())
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string();

    let content = if let Some(meta) = &parsed.meta {
        let title = path_stem(&found).unwrap_or_else(|| meta.title.clone());
        if meta.title != title || meta.category != category {
            compose_card_content(
                &CardMeta {
                    uuid: meta.uuid.clone(),
                    title,
                    category: category.clone(),
                    created_at: meta.created_at.clone(),
                    updated_at: now_rfc3339(),
                },
                &parsed.content,
            )?
        } else {
            raw_content.clone()
        }
    } else {
        let title = path_stem(&found)
            .or_else(|| extract_title(&parsed.content))
            .unwrap_or_else(|| uuid.clone());
        let healed = compose_card_content(
            &CardMeta {
                uuid: uuid.clone(),
                title: title.clone(),
                category: category.clone(),
                created_at: now_rfc3339(),
                updated_at: now_rfc3339(),
            },
            &parsed.content,
        )?;
        if healed != raw_content {
            ms_io::fs::write_atomic(&found, &healed).map_err(|e| e.to_string())?;
        }
        healed
    };

    // Use the file stem as the card title when available.
    let title = path_stem(&found)
        .or_else(|| parsed.meta.as_ref().map(|m| m.title.clone()))
        .or_else(|| extract_title(&parsed.content))
        .unwrap_or_else(|| uuid.clone());
    let file_hash = compute_hash(&content);

    let existing = with_db(&state, |db| db.get_card(&uuid)).ok().flatten();
    let (version, sync_status, last_synced_hash) = match &existing {
        Some(prev) => (prev.version, prev.sync_status.clone(), prev.last_synced_hash.clone()),
        None => (1, ms_meta::SyncStatus::PendingPush, None),
    };
    let card = ms_meta::CardIndex {
        uuid: uuid.clone(),
        file_path: found.to_string_lossy().to_string(),
        file_hash,
        version,
        sync_status,
        last_synced_hash,
    };
    with_db(&state, |db| db.upsert_card(&card))?;
    if existing.is_none() {
        with_db(&state, |db| db.bump_version(&uuid))?;
    }
    with_db(&state, |db| {
        db.upsert_fts(&uuid, &title, &content, &category)
    })?;

    Ok(content)
}

#[tauri::command]
pub fn create_card(
    title: String,
    category: Option<String>,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<CardIndexDto, String> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let category = category.unwrap_or_default();

    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        if cfg.vault_path.is_empty() {
            return Err("Vault path not configured".into());
        }
        cfg.vault_path.clone()
    };

    // Create file on disk
    let dir = category_path(std::path::Path::new(&vault_path), &category)?;
    ms_io::fs::ensure_dir(&dir).map_err(|e| e.to_string())?;
    let (title, file_path) = reserve_card_path(&dir, &title);
    let now = now_rfc3339();
    let content = compose_card_content(
        &CardMeta {
            uuid: uuid.clone(),
            title: title.clone(),
            category: category.clone(),
            created_at: now.clone(),
            updated_at: now,
        },
        "",
    )
    .map_err(|e| e.to_string())?;
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

    if let Err(e) = tauri::Emitter::emit(
        &app,
        "fs:change",
        FileChangeEvent {
            path: file_path.to_string_lossy().to_string(),
            kind: "create".into(),
        },
    ) {
        eprintln!("[create_card] event emit failed: {e}");
    }

    Ok(card.into())
}

#[tauri::command]
pub fn delete_card(
    uuid: String,
    soft: bool,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    if soft {
        with_db(&state, |db| {
            db.set_sync_status(&uuid, ms_meta::SyncStatus::PendingDelete)
        })?;
    } else {
        let card = with_db(&state, |db| db.get_card(&uuid))?
            .ok_or_else(|| format!("Card not found: {uuid}"))?;
        let deleted_path = card.file_path.clone();
        std::fs::remove_file(&card.file_path).map_err(|e| format!("Delete file failed: {e}"))?;
        with_db(&state, |db| {
            db.delete_relations_for_uuid(&uuid)?;
            db.delete_card(&uuid)?;
            db.delete_fts(&uuid)?;
            Ok(())
        })?;
        if let Err(e) = tauri::Emitter::emit(
            &app,
            "fs:change",
            FileChangeEvent {
                path: deleted_path,
                kind: "delete".into(),
            },
        ) {
            eprintln!("[delete_card] event emit failed: {e}");
        }
    }
    Ok(())
}

#[tauri::command]
pub fn rename_card(
    uuid: String,
    new_title: String,
    state: tauri::State<'_, AppState>,
) -> Result<CardMeta, String> {
    let card = with_db(&state, |db| db.get_card(&uuid))?
        .ok_or_else(|| format!("Card not found: {uuid}"))?;

    let content =
        std::fs::read_to_string(&card.file_path).map_err(|e| format!("Read failed: {e}"))?;
    let parsed = crate::parse_markdown_document(&content)?;
    let new_title = new_title.trim().to_string();
    let old_path = std::path::PathBuf::from(&card.file_path);
    let parent_dir = old_path
        .parent()
        .ok_or_else(|| "Invalid file path".to_string())?
        .to_path_buf();
    let new_path = card_file_path(&parent_dir, &new_title);
    let new_meta = CardMeta {
        uuid: uuid.clone(),
        title: new_title.clone(),
        category: parsed
            .meta
            .as_ref()
            .map(|m| m.category.clone())
            .unwrap_or_default(),
        created_at: parsed
            .meta
            .as_ref()
            .map(|m| m.created_at.clone())
            .unwrap_or_else(now_rfc3339),
        updated_at: now_rfc3339(),
    };
    let new_content = compose_card_content(&new_meta, &parsed.content)?;
    let new_hash = compute_hash(&new_content);

    if new_path == old_path {
        ms_io::fs::write_atomic(&old_path, &new_content).map_err(|e| e.to_string())?;
    } else {
        ms_io::fs::write_atomic(&new_path, &new_content).map_err(|e| e.to_string())?;
        if let Err(e) = std::fs::remove_file(&old_path) {
            eprintln!("[rename_card] failed to remove old file {}: {e}", old_path.display());
        }
    }

    let updated = ms_meta::CardIndex {
        file_hash: new_hash,
        file_path: new_path.to_string_lossy().to_string(),
        sync_status: ms_meta::SyncStatus::PendingPush,
        ..card
    };
    with_db(&state, |db| db.upsert_card(&updated))?;
    with_db(&state, |db| {
        db.upsert_fts(&uuid, &new_title, &new_content, &updated.file_path)
    })?;

    Ok(new_meta)
}

#[tauri::command]
pub fn move_card(
    uuid: String,
    target_category: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let card = with_db(&state, |db| db.get_card(&uuid))?
        .ok_or_else(|| format!("Card not found: {uuid}"))?;

    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.vault_path.clone()
    };
    let dest_dir = category_path(std::path::Path::new(&vault_path), &target_category)?;
    ms_io::fs::ensure_dir(&dest_dir).map_err(|e| e.to_string())?;

    let old_path = std::path::Path::new(&card.file_path);
    let file_name = old_path
        .file_name()
        .ok_or_else(|| "Invalid file path".to_string())?;
    let new_path = dest_dir.join(file_name);

    let content = std::fs::read_to_string(old_path).map_err(|e| format!("Read failed: {e}"))?;
    let parsed = crate::parse_markdown_document(&content)?;
    let new_content = compose_card_content(
        &CardMeta {
            uuid: uuid.clone(),
            title: parsed
                .meta
                .as_ref()
                .map(|m| m.title.clone())
                .or_else(|| extract_title(&parsed.content))
                .unwrap_or_else(|| uuid.clone()),
            category: target_category.clone(),
            created_at: parsed
                .meta
                .as_ref()
                .map(|m| m.created_at.clone())
                .unwrap_or_else(now_rfc3339),
            updated_at: now_rfc3339(),
        },
        &parsed.content,
    )?;

    ms_io::fs::write_atomic(&new_path, &new_content).map_err(|e| e.to_string())?;
    if let Err(e) = std::fs::remove_file(old_path) {
        eprintln!("[move_card] failed to remove old file {}: {e}", old_path.display());
    }

    let updated = ms_meta::CardIndex {
        file_path: new_path.to_string_lossy().to_string(),
        sync_status: ms_meta::SyncStatus::PendingPush,
        ..card
    };
    with_db(&state, |db| db.upsert_card(&updated))?;

    if let Err(e) = tauri::Emitter::emit(
        &app,
        "fs:change",
        FileChangeEvent {
            path: new_path.to_string_lossy().to_string(),
            kind: "create".into(),
        },
    ) {
        eprintln!("[move_card] event emit failed: {e}");
    }

    Ok(())
}

#[tauri::command]
pub fn scan_vault_tree(state: tauri::State<'_, AppState>) -> Result<Vec<TreeNodeDto>, String> {
    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        if cfg.vault_path.is_empty() {
            return Err("Vault path not configured".into());
        }
        cfg.vault_path.clone()
    };

    let root = std::path::Path::new(&vault_path);
    let files = ms_io::scanner::scan_markdown_files(root).map_err(|e| e.to_string())?;

    // Pre-heal files without frontmatter so tree gets real UUIDs
    for file in &files {
        let fm = read_frontmatter_head(file, 4096);
        if fm.as_ref().is_none_or(|f| f.uuid.is_none()) {
            let raw = match std::fs::read_to_string(file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[scan_vault_tree] read failed {}: {e}", file.display());
                    continue;
                }
            };
            let parsed = match crate::parse_markdown_document(&raw) {
                Ok(d) => d,
                Err(_) => continue,
            };
            if parsed.meta.is_none() {
                let uuid = uuid::Uuid::new_v4().to_string();
                let title = path_stem(file)
                    .or_else(|| extract_title(&parsed.content))
                    .unwrap_or_else(|| uuid.clone());
                let category = derive_category(&file.to_string_lossy(), &vault_path);
                let healed = match compose_card_content(
                    &CardMeta {
                        uuid: uuid.clone(),
                        title: title.clone(),
                        category: category.clone(),
                        created_at: now_rfc3339(),
                        updated_at: now_rfc3339(),
                    },
                    &parsed.content,
                ) {
                    Ok(h) => h,
                    Err(_) => continue,
                };
                if healed != raw && ms_io::fs::write_atomic(file, &healed).is_err() {
                    eprintln!("[scan_vault_tree] write_atomic failed for {}", file.display());
                    continue;
                }
                let file_hash = compute_hash(&healed);
                let card = ms_meta::CardIndex {
                    uuid,
                    file_path: file.to_string_lossy().to_string(),
                    file_hash,
                    version: 1,
                    sync_status: ms_meta::SyncStatus::PendingPush,
                    last_synced_hash: None,
                };
                if let Err(e) = with_db(&state, |db| db.upsert_card(&card)) {
                    eprintln!("[scan_vault_tree] upsert_card failed: {e}");
                }
                if let Err(e) = with_db(&state, |db| db.bump_version(&card.uuid)) {
                    eprintln!("[scan_vault_tree] bump_version failed: {e}");
                }
                if let Err(e) = ingest_card_content(&card.uuid, &healed, &title, &category, &state) {
                    eprintln!("[scan_vault_tree] ingest failed for {}: {e}", card.uuid);
                }
            }
        }
    }

    // Extract wikilink relations for all scanned files via shared ingestion pipeline
    for file in &files {
        let fm = read_frontmatter_head(file, 4096);
        let Some(uuid) = fm.as_ref().and_then(|f| f.uuid.clone()) else {
            continue;
        };
        let Ok(raw) = std::fs::read_to_string(file) else { continue };
        let title = fm.as_ref().and_then(|f| f.title.clone()).unwrap_or_else(|| uuid.clone());
        let category = derive_category(&file.to_string_lossy(), &vault_path);
        if let Err(e) = ingest_card_content(&uuid, &raw, &title, &category, &state) {
            eprintln!("[scan_vault_tree] ingest failed for {uuid}: {e}");
        }
    }

    // Also discover empty directories so they appear as categories
    let mut dirs: Vec<std::path::PathBuf> = Vec::new();
    walk_dirs_recursive(root, &mut dirs);

    build_tree(&files, &dirs, root)
}

#[tauri::command]
pub fn scan_and_heal(state: tauri::State<'_, AppState>) -> Result<usize, String> {
    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        if cfg.vault_path.is_empty() {
            return Err("Vault path not configured".into());
        }
        cfg.vault_path.clone()
    };

    let root = std::path::Path::new(&vault_path);
    let files = ms_io::scanner::scan_markdown_files(root).map_err(|e| e.to_string())?;
    let mut healed = 0usize;

    for file in &files {
        let raw = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[scan_and_heal] read failed {}: {e}", file.display());
                continue;
            }
        };
        let category = file
            .parent()
            .and_then(|p| p.strip_prefix(root).ok())
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .replace('\\', "/")
            .trim_end_matches('/')
            .to_string();

        let parsed = match crate::parse_markdown_document(&raw) {
            Ok(doc) => doc,
            Err(e) => {
                eprintln!("[scan_and_heal] parse failed {}: {e}", file.display());
                continue;
            }
        };

        let uuid = parsed
            .meta
            .as_ref()
            .map(|m| m.uuid.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let title = path_stem(file)
            .or_else(|| parsed.meta.as_ref().map(|m| m.title.clone()))
            .or_else(|| extract_title(&parsed.content))
            .unwrap_or_else(|| uuid.clone());

        let healed_content = if parsed.meta.is_none() {
            let healed_document = compose_card_content(
                &CardMeta {
                    uuid: uuid.clone(),
                    title: title.clone(),
                    category: category.clone(),
                    created_at: now_rfc3339(),
                    updated_at: now_rfc3339(),
                },
                &parsed.content,
            )?;
            if healed_document != raw {
                if ms_io::fs::write_atomic(file, &healed_document).is_err() {
                    eprintln!("[scan_and_heal] write_atomic failed for {}", file.display());
                    continue;
                }
                healed += 1;
            }
            healed_document
        } else if let Some(meta) = parsed.meta.as_ref() {
            if meta.title != title || meta.category != category {
                let healed_document = compose_card_content(
                    &CardMeta {
                        uuid: meta.uuid.clone(),
                        title: title.clone(),
                        category: category.clone(),
                        created_at: meta.created_at.clone(),
                        updated_at: now_rfc3339(),
                    },
                    &parsed.content,
                )?;
                if healed_document != raw {
                    if ms_io::fs::write_atomic(file, &healed_document).is_err() {
                        eprintln!("[scan_and_heal] write_atomic failed for {}", file.display());
                        continue;
                    }
                    healed += 1;
                }
                healed_document
            } else {
                raw.clone()
            }
        } else {
            raw.clone()
        };

        // Ensure card_index entry exists — preserve version for existing cards
        let file_hash = compute_hash(&healed_content);
        let existing = with_db(&state, |db| db.get_card(&uuid)).ok().flatten();
        let (version, sync_status, last_synced_hash) = match &existing {
            Some(prev) => (prev.version, prev.sync_status.clone(), prev.last_synced_hash.clone()),
            None => (1, ms_meta::SyncStatus::PendingPush, None),
        };
        let card = ms_meta::CardIndex {
            uuid: uuid.clone(),
            file_path: file.to_string_lossy().to_string(),
            file_hash,
            version,
            sync_status,
            last_synced_hash,
        };
        if let Err(e) = with_db(&state, |db| db.upsert_card(&card)) {
            eprintln!("[scan_and_heal] upsert_card {} failed: {e}", uuid);
        }
        if existing.is_none() {
            if let Err(e) = with_db(&state, |db| db.bump_version(&uuid)) {
                eprintln!("[scan_and_heal] bump_version {} failed: {e}", uuid);
            }
        }
        if let Err(e) = with_db(&state, |db| {
            db.upsert_fts(&uuid, &title, &healed_content, &category)
        }) {
            eprintln!("[scan_and_heal] upsert_fts {} failed: {e}", uuid);
        }
    }

    // Clean stale DB entries for files that no longer exist on disk
    let all_cards = with_db(&state, |db| db.list_all())?;
    for card in &all_cards {
        if !std::path::Path::new(&card.file_path).exists() {
            if let Err(e) = with_db(&state, |db| db.delete_card_by_path(&card.file_path)) {
                eprintln!("[scan_and_heal] delete stale {} failed: {e}", card.uuid);
            } else {
                healed += 1;
            }
        }
    }

    Ok(healed)
}

// ── Helpers ────────────────────────────────────────────────────────

fn extract_title(content: &str) -> Option<String> {
    // Try frontmatter first
    let mut in_fm = false;
    for line in content.lines() {
        if line == "---" {
            if in_fm {
                break;
            }
            in_fm = true;
            continue;
        }
        if in_fm && line.starts_with("title:") {
            return Some(line.trim_start_matches("title:").trim().to_string());
        }
    }
    // Fallback: first markdown heading
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return Some(trimmed.trim_start_matches('#').trim().to_string());
        }
    }
    None
}

fn derive_category(file_path: &str, vault_root: &str) -> String {
    std::path::Path::new(file_path)
        .parent()
        .and_then(|p| p.strip_prefix(std::path::Path::new(vault_root)).ok())
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string()
}

fn compose_card_content(meta: &CardMeta, body: &str) -> Result<String, String> {
    crate::models::compose_document(meta, body)
}

/// Strip YAML frontmatter from raw markdown, returning only the body content.
fn strip_frontmatter(raw: &str) -> String {
    let trimmed = raw.trim_start();
    if !trimmed.starts_with("---") {
        return raw.to_string();
    }
    // After the opening ---, find a line that is exactly "---" (with optional trailing whitespace)
    let after_first = &trimmed[3..];
    let mut search_from = 0;
    for line in after_first.lines() {
        if line.trim_end() == "---" {
            let marker_in_source = after_first[search_from..].find(line).unwrap() + search_from;
            let body_start = marker_in_source + line.len();
            let body = after_first[body_start..]
                .trim_start_matches(|c| c == '\r' || c == '\n');
            return body.to_string();
        }
        // Advance search position past this line + its line ending
        search_from += line.len();
        // Skip the line ending (\n or \r\n)
        let rest = &after_first[search_from..];
        if rest.starts_with("\r\n") {
            search_from += 2;
        } else if rest.starts_with('\n') {
            search_from += 1;
        }
    }
    raw.to_string()
}

#[cfg(test)]
mod tests {
    use super::strip_frontmatter;

    #[test]
    fn strips_normal_frontmatter() {
        let input = "---\ntitle: Hello\ncategory: test\n---\n\nBody content";
        assert_eq!(strip_frontmatter(input), "Body content");
    }

    #[test]
    fn no_frontmatter_returns_original() {
        let input = "# Hello\n\nBody content";
        assert_eq!(strip_frontmatter(input), input);
    }

    #[test]
    fn windows_crlf_line_endings() {
        let input = "---\r\ntitle: Hello\r\ncategory: test\r\n---\r\n\r\nBody content";
        assert_eq!(strip_frontmatter(input), "Body content");
    }

    #[test]
    fn no_closing_delimiter_returns_original() {
        let input = "---\ntitle: Hello\nBody that goes on";
        assert_eq!(strip_frontmatter(input), input);
    }

    #[test]
    fn empty_frontmatter() {
        let input = "---\n---\n\nBody content";
        assert_eq!(strip_frontmatter(input), "Body content");
    }

    #[test]
    fn horizontal_rule_in_body_not_confused() {
        let input = "---\ntitle: Test\n---\n\nSome text\n\n---\n\nMore text";
        assert_eq!(strip_frontmatter(input), "Some text\n\n---\n\nMore text");
    }

    #[test]
    fn leading_whitespace_before_frontmatter() {
        let input = "  ---\ntitle: Test\n---\nBody";
        assert_eq!(strip_frontmatter(input), "Body");
    }

    #[test]
    fn frontmatter_with_no_body() {
        let input = "---\ntitle: Test\n---";
        assert_eq!(strip_frontmatter(input), "");
    }

    #[test]
    fn frontmatter_with_trailing_spaces_on_delimiter() {
        let input = "---\ntitle: Test\n---   \n\nBody";
        assert_eq!(strip_frontmatter(input), "Body");
    }

    #[test]
    fn body_with_dash_line() {
        let input = "---\ntitle: Test\n---\n\nLine before\n\n---\n\nLine after";
        assert_eq!(strip_frontmatter(input), "Line before\n\n---\n\nLine after");
    }

    #[test]
    fn frontmatter_with_colons_in_values() {
        let input = "---\ntitle: \"A: B\"\ncategory: \"x:y\"\n---\n\nContent";
        assert_eq!(strip_frontmatter(input), "Content");
    }

    #[test]
    fn content_starting_with_three_dashes_but_not_frontmatter() {
        let input = "---dash\nNot frontmatter";
        assert_eq!(strip_frontmatter(input), input);
    }

    #[test]
    fn frontmatter_value_containing_dashes() {
        let input = "---\ntitle: Some --- Title\n---\n\nBody";
        assert_eq!(strip_frontmatter(input), "Body");
    }
}

/// Lightweight frontmatter extraction — reads at most `max_bytes` from file head.
/// Avoids full file read + AST parse for tree building.
struct FrontmatterHead {
    uuid: Option<String>,
    title: Option<String>,
}

fn read_frontmatter_head(path: &std::path::Path, max_bytes: usize) -> Option<FrontmatterHead> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).ok()?;
    let mut buf = vec![0u8; max_bytes];
    let n = file.read(&mut buf).ok()?;
    buf.truncate(n);
    let head = String::from_utf8_lossy(&buf);

    let mut in_fm = false;
    let mut uuid = None;
    let mut title = None;

    for line in head.lines() {
        if line.trim() == "---" {
            if in_fm {
                break;
            }
            in_fm = true;
            continue;
        }
        if in_fm {
            let colon = line.find(':')?;
            let key = line[..colon].trim();
            let value = line[colon + 1..].trim().to_string();
            match key {
                "uuid" => uuid = Some(value),
                "title" => title = Some(value),
                _ => {}
            }
        }
    }

    Some(FrontmatterHead { uuid, title })
}

fn walk_dirs_recursive(root: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            // Skip hidden dirs (.bunker, .git, .obsidian, etc.)
            if name.to_str().is_some_and(|s| s.starts_with('.')) {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                out.push(path.clone());
                walk_dirs_recursive(&path, out);
            }
        }
    }
}

fn build_tree(
    files: &[std::path::PathBuf],
    dirs: &[std::path::PathBuf],
    root: &std::path::Path,
) -> Result<Vec<TreeNodeDto>, String> {
    let mut entries: HashMap<String, Vec<String>> = HashMap::new();
    let mut leaf_nodes: HashMap<String, TreeNodeDto> = HashMap::new();

    for file in files {
        let rel = file.strip_prefix(root).unwrap_or(file);
        // Normalize to forward slashes for cross-platform consistency
        let rel_str = rel.to_str().unwrap_or("").replace('\\', "/");
        let parent = if rel_str.contains('/') {
            rel_str.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("")
        } else {
            ""
        };
        let fm = read_frontmatter_head(file, 4096);
        let stem = rel
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());
        let uuid = fm
            .as_ref()
            .and_then(|f| f.uuid.clone())
            .or_else(|| stem.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let title = fm
            .as_ref()
            .and_then(|f| f.title.clone())
            .or(stem)
            .unwrap_or_else(|| "unknown".to_string());
        entries
            .entry(parent.to_string())
            .or_default()
            .push(uuid.clone());
        leaf_nodes.insert(
            uuid.clone(),
            TreeNodeDto {
                id: uuid,
                name: title,
                parent_id: Some(parent.to_string()).filter(|p| !p.is_empty()),
                is_dir: false,
                children: Vec::new(),
            },
        );
    }

    // Flat categories: only top-level directories (no '/' in path)
    let mut category_nodes: HashMap<String, TreeNodeDto> = HashMap::new();
    for cat in entries.keys() {
        if !cat.is_empty() && !cat.contains('/') {
            category_nodes
                .entry(cat.clone())
                .or_insert_with(|| TreeNodeDto {
                    id: cat.clone(),
                    name: cat.clone(),
                    parent_id: None,
                    is_dir: true,
                    children: Vec::new(),
                });
        }
    }

    // Merge disk directories — only top-level
    for dir in dirs {
        let rel = dir.strip_prefix(root).unwrap_or(dir);
        let cat = rel.to_str().unwrap_or("").replace('\\', "/").trim_end_matches('/').to_string();
        if !cat.is_empty() && !cat.contains('/') {
            category_nodes
                .entry(cat.clone())
                .or_insert_with(|| TreeNodeDto {
                    id: cat.clone(),
                    name: cat.clone(),
                    parent_id: None,
                    is_dir: true,
                    children: Vec::new(),
                });
        }
    }

    // Build flat tree: categories contain only immediate children
    let mut result: Vec<TreeNodeDto> = Vec::new();
    let mut nested_ids: HashSet<String> = HashSet::new();

    for cat_id in category_nodes.keys() {
        let mut cat_node = category_nodes[cat_id].clone();
        if let Some(files) = entries.get(cat_id) {
            for uuid in files {
                if let Some(leaf) = leaf_nodes.get(uuid) {
                    nested_ids.insert(uuid.clone());
                    cat_node.children.push(leaf.clone());
                }
            }
        }
        result.push(cat_node);
    }

    // Root-level files (no category or promoted from nested dirs)
    if let Some(root_files) = entries.get("") {
        for uuid in root_files {
            if nested_ids.contains(uuid) {
                continue;
            }
            if let Some(node) = leaf_nodes.get(uuid) {
                result.push(node.clone());
            }
        }
    }
    for (parent, uuids) in &entries {
        if parent.contains('/') {
            for uuid in uuids {
                if nested_ids.contains(uuid) {
                    continue;
                }
                if let Some(node) = leaf_nodes.get(uuid) {
                    result.push(node.clone());
                }
            }
        }
    }

    Ok(result)
}

fn now_rfc3339() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs() as i64;
    // Simple RFC3339-ish format without chrono dependency
    let days = secs / 86400;
    // Calculate year/month/day from days since epoch
    let (year, month, day) = days_to_ymd(days);
    let time_secs = secs % 86400;
    let h = time_secs / 3600;
    let m = (time_secs % 3600) / 60;
    let s = time_secs % 60;
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
}

fn days_to_ymd(mut days: i64) -> (i64, u32, u32) {
    // Shift to era starting March 1, 0000
    days += 719468;
    let era = (if days >= 0 { days } else { days - 146096 }) / 146097;
    let mut doe = days - era * 146097;
    if doe < 0 {
        doe += 146097;
    }
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

#[tauri::command]
pub fn rename_category(
    old_name: String,
    new_name: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let new_name = new_name.trim().to_string();
    if new_name.is_empty() {
        return Err("分类名不能为空".into());
    }

    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        if cfg.vault_path.is_empty() {
            return Err("Vault path not configured".into());
        }
        cfg.vault_path.clone()
    };

    let old_dir = category_path(std::path::Path::new(&vault_path), &old_name)?;
    let new_dir = category_path(std::path::Path::new(&vault_path), &new_name)?;

    if !old_dir.exists() {
        return Err(format!("分类不存在: {old_name}"));
    }
    if new_dir.exists() {
        return Err(format!("分类已存在: {new_name}"));
    }

    std::fs::rename(&old_dir, &new_dir).map_err(|e| format!("重命名失败: {e}"))?;

    // Update frontmatter category for all cards in renamed dir
    let files = ms_io::scanner::scan_markdown_files(&new_dir).map_err(|e| e.to_string())?;
    for file in &files {
        let raw = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[rename_category] read failed {}: {e}", file.display());
                continue;
            }
        };
        let parsed = match crate::parse_markdown_document(&raw) {
            Ok(doc) => doc,
            Err(_) => continue,
        };
        if let Some(meta) = parsed.meta {
            let healed = compose_card_content(
                &CardMeta {
                    uuid: meta.uuid.clone(),
                    title: meta.title.clone(),
                    category: new_name.clone(),
                    created_at: meta.created_at.clone(),
                    updated_at: meta.updated_at.clone(),
                },
                &parsed.content,
            )?;
            if healed != raw {
                ms_io::fs::write_atomic(file, &healed).map_err(|e| e.to_string())?;
            }
            let file_hash = compute_hash(&healed);
            let card = ms_meta::CardIndex {
                uuid: meta.uuid.clone(),
                file_path: file.to_string_lossy().to_string(),
                file_hash,
                version: 1,
                sync_status: ms_meta::SyncStatus::PendingPush,
                last_synced_hash: None,
            };
            if let Err(e) = with_db(&state, |db| db.upsert_card(&card)) {
                eprintln!("[rename_category] upsert_card {} failed: {e}", meta.uuid);
            }
            if let Err(e) = with_db(&state, |db| {
                db.upsert_fts(&meta.uuid, &meta.title, &healed, &new_name)
            }) {
                eprintln!("[rename_category] upsert_fts {} failed: {e}", meta.uuid);
            }
        }
    }

    if let Err(e) = tauri::Emitter::emit(
        &app,
        "fs:change",
        FileChangeEvent {
            path: new_dir.to_string_lossy().to_string(),
            kind: "create".into(),
        },
    ) {
        eprintln!("[rename_category] event emit failed: {e}");
    }

    Ok(())
}

#[tauri::command]
pub fn delete_category(
    category: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        if cfg.vault_path.is_empty() {
            return Err("Vault path not configured".into());
        }
        cfg.vault_path.clone()
    };

    let dir = category_path(std::path::Path::new(&vault_path), &category)?;
    if !dir.exists() {
        return Err(format!("分类不存在: {category}"));
    }

    // Remove DB entries for all cards inside
    let files = ms_io::scanner::scan_markdown_files(&dir).map_err(|e| e.to_string())?;
    for file in &files {
        let raw = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[delete_category] read failed {}: {e}", file.display());
                continue;
            }
        };
        if let Ok(parsed) = crate::parse_markdown_document(&raw) {
            if let Some(meta) = parsed.meta {
                if let Err(e) = with_db(&state, |db| {
                    db.delete_card(&meta.uuid)?;
                    db.delete_fts(&meta.uuid)?;
                    Ok(())
                }) {
                    eprintln!("[delete_category] delete {} failed: {e}", meta.uuid);
                }
            }
        }
    }

    std::fs::remove_dir_all(&dir).map_err(|e| format!("删除分类失败: {e}"))?;

    if let Err(e) = tauri::Emitter::emit(
        &app,
        "fs:change",
        FileChangeEvent {
            path: dir.to_string_lossy().to_string(),
            kind: "delete".into(),
        },
    ) {
        eprintln!("[delete_category] event emit failed: {e}");
    }

    Ok(())
}

#[tauri::command]
pub fn create_category(
    category: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        if cfg.vault_path.is_empty() {
            return Err("Vault path not configured".into());
        }
        cfg.vault_path.clone()
    };

    let dir = category_path(std::path::Path::new(&vault_path), &category)?;
    if dir.exists() {
        return Err(format!("分类已存在: {category}"));
    }

    ms_io::fs::ensure_dir(&dir).map_err(|e| format!("创建目录失败: {e}"))?;

    if let Err(e) = tauri::Emitter::emit(
        &app,
        "fs:change",
        FileChangeEvent {
            path: dir.to_string_lossy().to_string(),
            kind: "create".into(),
        },
    ) {
        eprintln!("[create_category] event emit failed: {e}");
    }

    Ok(())
}

/// Full ingestion pipeline: update FTS and extract wiki links from content.
/// Preserves trunk relations — only link-type relations are replaced.
pub fn ingest_card_content(
    uuid: &str,
    content: &str,
    title: &str,
    category: &str,
    state: &tauri::State<'_, AppState>,
) -> Result<(), String> {
    // 1. Update full-text search index
    with_db(state, |db| db.upsert_fts(uuid, title, content, category))?;

    // 2. Extract wiki links → replace link relations (preserve trunks)
    let pure = strip_frontmatter(content);
    if let Ok(links) = ms_ast::extract_links(&pure) {
        let existing = with_db(state, |db| db.get_outbound(uuid)).unwrap_or_default();
        let trunk_targets: std::collections::HashSet<String> = existing
            .iter()
            .filter(|r| r.relation_type == "trunk")
            .map(|r| r.target_uuid_or_tag.clone())
            .collect();
        let mut link_relations: Vec<ms_meta::RelationRecord> = Vec::new();
        for target_title in &links {
            if let Ok(Some(target_uuid)) = with_db(state, |db| db.get_uuid_by_title(target_title)) {
                if !trunk_targets.contains(&target_uuid) {
                    link_relations.push(ms_meta::RelationRecord {
                        source_uuid: uuid.into(),
                        target_uuid_or_tag: target_uuid,
                        relation_type: "link".into(),
                    });
                }
            }
        }
        if let Err(e) = with_db(state, |db| db.replace_relations(uuid, &link_relations)) {
            eprintln!("[ingest] relation update failed for {uuid}: {e}");
        }
    }
    Ok(())
}

#[tauri::command]
pub fn resolve_conflict_keep_local(uuid: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Local wins — mark for push, delete conflict copy file
    let card = with_db(&state, |db| db.get_card(&uuid))
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Card not found: {uuid}"))?;

    // Try to delete conflict copy file (non-fatal if missing)
    let path = std::path::Path::new(&card.file_path);
    let parent = path.parent().unwrap_or(std::path::Path::new("."));
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("untitled");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("md");
    // Match conflict copy naming: "stem(云端冲突副本 timestamp).ext"
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(stem) && name_str.contains("云端冲突副本") && name_str.ends_with(&format!(".{ext}")) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    with_db(&state, |db| {
        db.set_sync_status(&uuid, ms_meta::SyncStatus::PendingPush)?;
        db.bump_version(&uuid)?;
        Ok(())
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resolve_conflict_keep_remote(uuid: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Remote wins — read conflict copy, overwrite local, mark synced
    let card = with_db(&state, |db| db.get_card(&uuid))
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Card not found: {uuid}"))?;

    let path = std::path::Path::new(&card.file_path);
    let parent = path.parent().unwrap_or(std::path::Path::new("."));
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("untitled");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("md");

    // Find the conflict copy file
    let mut conflict_path: Option<std::path::PathBuf> = None;
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(stem) && name_str.contains("云端冲突副本") && name_str.ends_with(&format!(".{ext}")) {
                conflict_path = Some(entry.path());
                break;
            }
        }
    }

    let Some(conflict_file) = conflict_path else {
        // Idempotent: conflict copy already removed — just mark synced
        eprintln!("[resolve_remote] conflict copy already removed for {uuid}, marking synced");
        with_db(&state, |db| db.set_sync_status(&uuid, ms_meta::SyncStatus::Synced))
            .map_err(|e| e.to_string())?;
        return Ok(());
    };
    let content = std::fs::read_to_string(&conflict_file)
        .map_err(|e| format!("Read conflict copy failed: {e}"))?;

    // Overwrite local file with remote content
    ms_io::fs::write_atomic(path, &content)
        .map_err(|e| format!("Write local file failed: {e}"))?;

    // Delete conflict copy
    let _ = std::fs::remove_file(&conflict_file);

    // Update hash and mark as synced
    let hash = compute_hash(&content);
    let updated = ms_meta::CardIndex {
        uuid: uuid.clone(),
        file_path: card.file_path.clone(),
        file_hash: hash.clone(),
        version: card.version,
        sync_status: ms_meta::SyncStatus::Synced,
        last_synced_hash: Some(hash),
    };
    with_db(&state, |db| db.upsert_card(&updated))
        .map_err(|e| e.to_string())?;

    // Ingest: update FTS + extract wiki links from resolved content
    let parsed = crate::parse_markdown_document(&content);
    let title = parsed.as_ref().ok().and_then(|d| d.meta.as_ref().map(|m| m.title.clone())).unwrap_or_else(|| uuid.clone());
    let category = parsed.as_ref().ok().and_then(|d| d.meta.as_ref().map(|m| m.category.clone())).unwrap_or_default();
    if let Err(e) = ingest_card_content(&uuid, &content, &title, &category, &state) {
        eprintln!("[resolve_remote] ingest failed for {uuid}: {e}");
    }

    Ok(())
}

#[tauri::command]
pub fn get_conflicted_uuids(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    with_db(&state, |db| db.list_uuids_by_sync_status(ms_meta::SyncStatus::Conflict))
        .map_err(|e| e.to_string())
}
