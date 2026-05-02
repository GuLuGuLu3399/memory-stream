use serde::Deserialize;

use crate::db::{compute_hash, with_db};
use crate::events::SyncProgressEvent;
use crate::models::{article_path, category_path, path_stem};
use crate::state::AppState;
use crate::commands::auth::{get_access_token, clear_access_token};

// ── Go server API types ────────────────────────────────────────────

#[derive(Deserialize)]
struct GoApiError {
    #[allow(dead_code)]
    code: i64,
    message: String,
}

pub fn format_api_error(status: reqwest::StatusCode, body: &str) -> String {
    if let Ok(err) = serde_json::from_str::<GoApiError>(body) {
        format!("{} (HTTP {})", err.message, status.as_u16())
    } else {
        format!("HTTP {status} — {body}")
    }
}

/// Check for 401 — clear keyring token and return AUTH_EXPIRED sentinel.
pub fn check_auth_expired(status: reqwest::StatusCode) -> Result<(), String> {
    if status == reqwest::StatusCode::UNAUTHORIZED {
        clear_access_token();
        return Err("AUTH_EXPIRED".to_string());
    }
    Ok(())
}

#[derive(Deserialize)]
struct SyncManifestResponse {
    #[allow(dead_code)]
    cursor: String,
    #[serde(default)]
    changes: Vec<SyncManifestItem>,
}

#[derive(Deserialize)]
struct SyncManifestItem {
    uuid: String,
    version: i64,
    hash: String,
    #[allow(dead_code)]
    updated_at: String,
    is_deleted: bool,
}

#[derive(serde::Serialize, Clone)]
struct SyncEdgePayload {
    target_uuid: String,
    relation_type: String,
}

#[derive(serde::Serialize, Clone)]
struct SyncCardPayload {
    uuid: String,
    title: String,
    category: String,
    content: String,
    ast_data: serde_json::Value,
    toc_data: serde_json::Value,
    excerpt: String,
    version: i64,
    hash: String,
    edges: Vec<SyncEdgePayload>,
}

#[derive(serde::Serialize)]
struct SyncBatchRequest {
    cards: Vec<SyncCardPayload>,
}

#[derive(Deserialize)]
struct SyncBatchResponse {
    #[serde(default)]
    accepted: Vec<SyncAcceptedItem>,
    #[serde(default)]
    conflicts: Vec<SyncConflictItem>,
    #[serde(default)]
    #[allow(dead_code)]
    rejected: Vec<SyncRejectedItem>,
}

#[derive(Deserialize)]
struct SyncAcceptedItem {
    uuid: String,
    version: i64,
    #[allow(dead_code)]
    updated_at: String,
}

#[derive(Deserialize)]
struct SyncConflictItem {
    uuid: String,
    #[allow(dead_code)]
    server_version: i64,
    #[allow(dead_code)]
    server_hash: String,
}

#[derive(Deserialize)]
struct SyncRejectedItem {
    #[allow(dead_code)]
    uuid: String,
    #[allow(dead_code)]
    reason: String,
}

// ── Trunk sync types ────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct RelationPayload {
    source_uuid: String,
    target_uuid: String,
    relation_type: String,
}

#[derive(serde::Serialize)]
struct RelationsSyncRequest {
    relations: Vec<RelationPayload>,
}

#[derive(Deserialize)]
struct RelationsSyncResponse {
    #[allow(dead_code)]
    accepted: i32,
}

// ── Public types ───────────────────────────────────────────────────

#[derive(serde::Serialize, Clone)]
pub struct SyncConflictEntry {
    pub uuid: String,
    pub title: String,
    pub local_path: String,
    pub conflict_copy_path: String,
    pub local_bytes: u64,
    pub remote_bytes: u64,
    pub local_updated_at: String,
    pub remote_updated_at: String,
}

#[derive(serde::Serialize, Clone, Default)]
pub struct SyncRunReport {
    pub downloaded: u32,
    pub uploaded: u32,
    pub deleted: u32,
    pub conflicts: Vec<SyncConflictEntry>,
}

#[derive(Default)]
struct PullReport {
    downloaded: u32,
    conflicts: Vec<SyncConflictEntry>,
}

// ── Helpers ────────────────────────────────────────────────────────

pub fn api_base(state: &AppState) -> Result<String, String> {
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    Ok(cfg.api_base_url.clone())
}

pub fn api_client(_state: &AppState) -> Result<reqwest::Client, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(token) = get_access_token() {
        let val = format!("Bearer {token}")
            .parse::<reqwest::header::HeaderValue>()
            .map_err(|e| format!("Invalid token: {e}"))?;
        headers.insert(reqwest::header::AUTHORIZATION, val);
    }
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| format!("Client build failed: {e}"))
}

pub fn get_last_sync_cursor(state: &AppState) -> Option<String> {
    state.config.lock().ok()?.last_sync_cursor.clone()
}

fn save_sync_cursor(state: &AppState, cursor: &str) {
    if let Ok(mut cfg) = state.config.lock() {
        let old = cfg.last_sync_cursor.take();
        cfg.last_sync_cursor = Some(cursor.to_string());
        drop(cfg);
        if state.save_config().is_err() {
            if let Ok(mut cfg) = state.config.lock() {
                cfg.last_sync_cursor = old;
            }
            eprintln!("[sync] failed to persist cursor");
        }
    }
}

fn format_system_time(ts: std::time::SystemTime) -> String {
    match ts.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => {
            let total_secs = d.as_secs();
            let days = total_secs / 86400;
            let tod = total_secs % 86400;
            let (y, m, d) = civil_from_days(days as i64);
            let h = tod / 3600;
            let min = (tod % 3600) / 60;
            let s = tod % 60;
            format!("{y:04}-{m:02}-{d:02}T{h:02}:{min:02}:{s:02}Z")
        }
        Err(_) => "1970-01-01T00:00:00Z".to_string(),
    }
}

/// Convert days since 1970-01-01 to (year, month, day) using civil calendar algorithm.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn modified_time_string(path: &std::path::Path) -> String {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(format_system_time)
        .unwrap_or_else(|| "unknown".to_string())
}

fn file_size(path: &std::path::Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

fn build_conflict_copy_path(local_path: &std::path::Path) -> std::path::PathBuf {
    let parent = local_path
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_default();

    let stem = local_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled");

    let ext = local_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("md");

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut candidate = parent.join(format!("{}(云端冲突副本 {}).{}", stem, stamp, ext));
    let mut i = 1u32;
    while candidate.exists() {
        candidate = parent.join(format!(
            "{}(云端冲突副本 {}-{}).{}",
            stem, stamp, i, ext
        ));
        i += 1;
    }
    candidate
}

async fn fetch_manifest(
    client: &reqwest::Client,
    base: &str,
    since: Option<&str>,
) -> Result<SyncManifestResponse, String> {
    let url = match since {
        Some(s) => format!("{base}/sync/manifest?since={s}"),
        None => format!("{base}/sync/manifest"),
    };
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Manifest fetch failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let _ = check_auth_expired(status);
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Manifest fetch failed: {}", format_api_error(status, &body)));
    }

    resp.json::<SyncManifestResponse>()
        .await
        .map_err(|e| format!("Manifest parse failed: {e}"))
}

async fn fetch_card_data(
    client: &reqwest::Client,
    base: &str,
    uuid: &str,
) -> Result<serde_json::Value, String> {
    let resp = client
        .get(format!("{base}/sync/card/{uuid}"))
        .send()
        .await
        .map_err(|e| format!("Card fetch failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let _ = check_auth_expired(status);
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Card fetch {uuid} failed: {}", format_api_error(status, &body)));
    }

    let payload = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Card parse failed: {e}"))?;

    payload
        .get("data")
        .cloned()
        .ok_or_else(|| "Card response missing data".to_string())
}

/// Extract trunk target UUIDs from the server card payload's `edges` array.
fn extract_trunk_targets(card_data: &serde_json::Value) -> Vec<String> {
    card_data
        .get("edges")
        .and_then(|e| e.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|edge| {
                    let rt = edge.get("relation_type").and_then(|v| v.as_str())?;
                    if rt == "trunk" {
                        edge.get("target_uuid").and_then(|v| v.as_str()).map(String::from)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

// ── Pull ───────────────────────────────────────────────────────────

async fn run_sync_pull(
    state: &tauri::State<'_, AppState>,
    app: &tauri::AppHandle,
) -> Result<PullReport, String> {
    let base = api_base(state)?;
    let client = api_client(state)?;

    let since = get_last_sync_cursor(state);
    let manifest_resp = fetch_manifest(&client, &base, since.as_deref()).await?;
    let manifest = manifest_resp.changes;

    if manifest.is_empty() {
        save_sync_cursor(state, &manifest_resp.cursor);
        return Ok(PullReport::default());
    }

    let _ = tauri::Emitter::emit(
        app,
        "sync:progress",
        SyncProgressEvent {
            status: "pulling".into(),
            progress: 0,
        },
    );

    let local_cards = with_db(state, |db| db.list_all())?;
    let mut local_map: std::collections::HashMap<String, ms_meta::CardIndex> = local_cards
        .into_iter()
        .map(|c| (c.uuid.clone(), c))
        .collect();

    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.vault_path.clone()
    };

    let mut report = PullReport::default();
    let total = manifest.len() as u32;

    for (index, item) in manifest.iter().enumerate() {
        if item.is_deleted {
            if let Some(card) = local_map.remove(&item.uuid) {
                let _ = std::fs::remove_file(&card.file_path);
                with_db(state, |db| {
                    db.delete_card(&item.uuid)?;
                    db.delete_fts(&item.uuid)
                })?;
            }

            if total > 0 {
                let _ = tauri::Emitter::emit(
                    app,
                    "sync:progress",
                    SyncProgressEvent {
                        status: "pulling".into(),
                        progress: ((index as u32 + 1) * 100).checked_div(total).unwrap_or(100),
                    },
                );
            }
            continue;
        }

        match local_map.get(&item.uuid) {
            None => {
                let card_data = fetch_card_data(&client, &base, &item.uuid).await?;
                let content = card_data
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let title = card_data
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let category = card_data
                    .get("category")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let dir = category_path(std::path::Path::new(&vault_path), &category)?;
                let _ = ms_io::fs::ensure_dir(&dir);
                let file_path = article_path(std::path::Path::new(&vault_path), &category, &title)?;
                ms_io::fs::write_atomic(&file_path, &content).map_err(|e| e.to_string())?;

                let card = ms_meta::CardIndex {
                    uuid: item.uuid.clone(),
                    file_path: file_path.to_string_lossy().to_string(),
                    file_hash: item.hash.clone(),
                    version: item.version,
                    sync_status: ms_meta::SyncStatus::Synced,
                    last_synced_hash: Some(item.hash.clone()),
                };
                with_db(state, |db| db.upsert_card(&card))?;

                // Sync trunk relations from server payload (before ingest so links see trunks)
                let trunks = extract_trunk_targets(&card_data);
                if !trunks.is_empty() {
                    if let Err(e) = with_db(state, |db| db.replace_trunk_relations(&item.uuid, &trunks)) {
                        eprintln!("[sync_pull] trunk sync failed for {}: {e}", item.uuid);
                    }
                }

                // FTS + wiki link extraction via shared ingestion pipeline
                if let Err(e) = super::vault::ingest_card_content(&item.uuid, &content, &title, &category, state) {
                    eprintln!("[sync_pull] ingest failed for {}: {e}", item.uuid);
                }

                report.downloaded += 1;
            }
            Some(local) => {
                let baseline = local
                    .last_synced_hash
                    .clone()
                    .unwrap_or_else(|| local.file_hash.clone());
                let local_changed = local.sync_status == ms_meta::SyncStatus::PendingPush
                    || local.file_hash != baseline;
                let remote_changed = item.hash != baseline;

                if local_changed && remote_changed {
                    let card_data = fetch_card_data(&client, &base, &item.uuid).await?;
                    let content = card_data
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let title = card_data
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&item.uuid)
                        .to_string();

                    let local_path = std::path::Path::new(&local.file_path);
                    let copy_path = build_conflict_copy_path(local_path);
                    ms_io::fs::write_atomic(&copy_path, &content).map_err(|e| e.to_string())?;

                    with_db(state, |db| {
                        db.set_sync_status(&item.uuid, ms_meta::SyncStatus::Conflict)
                    })?;

                    report.conflicts.push(SyncConflictEntry {
                        uuid: item.uuid.clone(),
                        title,
                        local_path: local.file_path.clone(),
                        conflict_copy_path: copy_path.to_string_lossy().to_string(),
                        local_bytes: file_size(local_path),
                        remote_bytes: content.len() as u64,
                        local_updated_at: modified_time_string(local_path),
                        remote_updated_at: card_data
                            .get("updated_at")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                    });
                } else if !local_changed && remote_changed {
                    // TOCTOU guard: re-check local file hash before overwriting
                    let current_hash = std::fs::read_to_string(&local.file_path)
                        .ok()
                        .map(|c| crate::db::compute_hash(&c));
                    if current_hash.as_ref() != Some(&local.file_hash) {
                        let card_data = fetch_card_data(&client, &base, &item.uuid).await?;
                        let content = card_data
                            .get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let title = card_data
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&item.uuid)
                            .to_string();
                        let local_path = std::path::Path::new(&local.file_path);
                        let copy_path = build_conflict_copy_path(local_path);
                        ms_io::fs::write_atomic(&copy_path, &content).map_err(|e| e.to_string())?;
                        with_db(state, |db| {
                            db.set_sync_status(&item.uuid, ms_meta::SyncStatus::Conflict)
                        })?;
                        report.conflicts.push(SyncConflictEntry {
                            uuid: item.uuid.clone(),
                            title,
                            local_path: local.file_path.clone(),
                            conflict_copy_path: copy_path.to_string_lossy().to_string(),
                            local_bytes: file_size(local_path),
                            remote_bytes: content.len() as u64,
                            local_updated_at: modified_time_string(local_path),
                            remote_updated_at: card_data
                                .get("updated_at")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                        });
                        if total > 0 {
                            let _ = tauri::Emitter::emit(
                                app,
                                "sync:progress",
                                SyncProgressEvent {
                                    status: "pulling".into(),
                                    progress: ((index as u32 + 1) * 100)
                                        .checked_div(total)
                                        .unwrap_or(100),
                                },
                            );
                        }
                        continue;
                    }

                    let card_data = fetch_card_data(&client, &base, &item.uuid).await?;
                    let content = card_data
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let title = card_data
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let category = card_data
                        .get("category")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let desired_path =
                        article_path(std::path::Path::new(&vault_path), &category, &title)?;
                    if desired_path == std::path::Path::new(&local.file_path) {
                        ms_io::fs::write_atomic(std::path::Path::new(&local.file_path), &content)
                            .map_err(|e| e.to_string())?;
                    } else {
                        ms_io::fs::write_atomic(&desired_path, &content)
                            .map_err(|e| e.to_string())?;
                        let _ = std::fs::remove_file(&local.file_path);
                    }

                    let updated = ms_meta::CardIndex {
                        uuid: local.uuid.clone(),
                        file_path: desired_path.to_string_lossy().to_string(),
                        file_hash: item.hash.clone(),
                        version: item.version,
                        sync_status: ms_meta::SyncStatus::Synced,
                        last_synced_hash: Some(item.hash.clone()),
                    };
                    with_db(state, |db| db.upsert_card(&updated))?;

                    // Sync trunk relations from server payload (before ingest so links see trunks)
                    let trunks = extract_trunk_targets(&card_data);
                    if !trunks.is_empty() {
                        if let Err(e) = with_db(state, |db| db.replace_trunk_relations(&local.uuid, &trunks)) {
                            eprintln!("[sync_pull] trunk sync failed for {}: {e}", local.uuid);
                        }
                    }

                    // FTS + wiki link extraction via shared ingestion pipeline
                    if let Err(e) = super::vault::ingest_card_content(&updated.uuid, &content, &title, &category, state) {
                        eprintln!("[sync_pull] ingest failed for {}: {e}", updated.uuid);
                    }

                    report.downloaded += 1;
                }
            }
        }

        if total > 0 {
            let _ = tauri::Emitter::emit(
                app,
                "sync:progress",
                SyncProgressEvent {
                    status: "pulling".into(),
                    progress: ((index as u32 + 1) * 100).checked_div(total).unwrap_or(100),
                },
            );
        }
    }

    // Persist cursor after successful pull
    save_sync_cursor(state, &manifest_resp.cursor);

    // ── Pull trunk relations from server ──────────────────────
    pull_trunk_relations(state, &client, &base).await?;

    Ok(report)
}

// ── Push (Batch) ──────────────────────────────────────────────────

async fn run_sync_push(
    state: &tauri::State<'_, AppState>,
    app: &tauri::AppHandle,
) -> Result<u32, String> {
    let base = api_base(state)?;
    let client = api_client(state)?;

    let cards = with_db(state, |db| db.list_all())?;
    let pending: Vec<_> = cards
        .into_iter()
        .filter(|c| c.sync_status == ms_meta::SyncStatus::PendingPush)
        .collect();

    if pending.is_empty() {
        return Ok(0);
    }

    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.vault_path.clone()
    };

    // Build payload map for hash reuse after push
    let mut payload_map: std::collections::HashMap<String, (SyncCardPayload, String)> =
        std::collections::HashMap::new();
    let mut payloads: Vec<SyncCardPayload> = Vec::new();

    for card in &pending {
        let content = std::fs::read_to_string(&card.file_path)
            .map_err(|e| format!("Read {} failed: {e}", card.uuid))?;

        let doc = ms_ast::parse_document(&content).map_err(|e| e.to_string())?;
        let ast_data: serde_json::Value =
            serde_json::from_str(&doc.ast_json).unwrap_or(serde_json::Value::Null);
        let toc_data: serde_json::Value =
            serde_json::to_value(&doc.toc).unwrap_or(serde_json::Value::Array(Vec::new()));

        let title = path_stem(std::path::Path::new(&card.file_path))
            .or_else(|| doc.toc.first().map(|h| h.text.clone()))
            .unwrap_or_else(|| card.uuid.clone());

        let category = std::path::Path::new(&card.file_path)
            .strip_prefix(&vault_path)
            .ok()
            .and_then(|rel| rel.parent())
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();

        let hash = compute_hash(&content);

        // Query real relation types from SQLite — only include links in card payload
        // (trunks are synced separately via POST /sync/relations)
        let relations = with_db(state, |db| db.get_outbound(&card.uuid)).unwrap_or_default();
        let edges: Vec<SyncEdgePayload> = relations
            .into_iter()
            .filter(|r| r.relation_type == "link")
            .map(|r| SyncEdgePayload {
                target_uuid: r.target_uuid_or_tag,
                relation_type: r.relation_type,
            })
            .collect();

        eprintln!(
            "[sync_push] card {} v={} link_edges={}",
            card.uuid, card.version,
            edges.len()
        );

        let payload = SyncCardPayload {
            uuid: card.uuid.clone(),
            title,
            category,
            content,
            ast_data,
            toc_data,
            excerpt: doc.excerpt,
            version: card.version,
            hash: hash.clone(),
            edges,
        };

        payload_map.insert(card.uuid.clone(), (payload.clone(), hash));
        payloads.push(payload);
    }

    let batch_req = SyncBatchRequest { cards: payloads };

    let _ = tauri::Emitter::emit(
        app,
        "sync:progress",
        SyncProgressEvent {
            status: "pushing".into(),
            progress: 50,
        },
    );

    let resp = client
        .post(format!("{base}/sync/batch"))
        .json(&batch_req)
        .send()
        .await
        .map_err(|e| format!("Batch push failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let _ = check_auth_expired(status);
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Batch push failed: {}", format_api_error(status, &body)));
    }

    let batch_resp = resp
        .json::<SyncBatchResponse>()
        .await
        .map_err(|e| format!("Batch response parse failed: {e}"))?;

    eprintln!(
        "[sync_push] server response: accepted={} conflicts={} rejected={}",
        batch_resp.accepted.len(),
        batch_resp.conflicts.len(),
        batch_resp.rejected.len(),
    );
    for c in &batch_resp.conflicts {
        eprintln!("[sync_push] CONFLICT {} server_v={}", c.uuid, c.server_version);
    }

    // Mark accepted cards as Synced — use SERVER-returned version
    for item in &batch_resp.accepted {
        if let Some((_, hash)) = payload_map.get(&item.uuid) {
            if let Some(card) = pending.iter().find(|c| c.uuid == item.uuid) {
                let updated = ms_meta::CardIndex {
                    uuid: card.uuid.clone(),
                    file_path: card.file_path.clone(),
                    file_hash: hash.clone(),
                    sync_status: ms_meta::SyncStatus::Synced,
                    last_synced_hash: Some(hash.clone()),
                    version: item.version, // server version, not local
                };
                with_db(state, |db| db.upsert_card(&updated))?;
            }
        }
    }

    // Handle conflicts — download server version as conflict copy
    for conflict in &batch_resp.conflicts {
        if let Some(card) = pending.iter().find(|c| c.uuid == conflict.uuid) {
            let card_data = fetch_card_data(&client, &base, &conflict.uuid).await?;
            let content = card_data
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let local_path = std::path::Path::new(&card.file_path);
            let copy_path = build_conflict_copy_path(local_path);
            ms_io::fs::write_atomic(&copy_path, &content).map_err(|e| e.to_string())?;

            with_db(state, |db| {
                db.set_sync_status(&conflict.uuid, ms_meta::SyncStatus::Conflict)
            })?;
        }
    }

    // Log rejected cards
    for rejected in &batch_resp.rejected {
        eprintln!(
            "[sync_push] rejected {}: {}",
            rejected.uuid, rejected.reason
        );
    }

    // ── Phase 2: Sync trunk relations independently ──────────────
    sync_trunk_relations(state, &client, &base).await?;

    let _ = tauri::Emitter::emit(
        app,
        "sync:progress",
        SyncProgressEvent {
            status: "done".into(),
            progress: 100,
        },
    );

    Ok(batch_resp.accepted.len() as u32)
}

// ── Delete tombstones ─────────────────────────────────────────────

async fn run_sync_delete_tombstones(
    state: &tauri::State<'_, AppState>,
) -> Result<u32, String> {
    let base = api_base(state)?;
    let client = api_client(state)?;
    let uuids = with_db(state, |db| db.list_pending_deletes())?;

    for uuid in &uuids {
        let resp = client
            .delete(format!("{base}/sync/card/{uuid}"))
            .send()
            .await
            .map_err(|e| format!("Delete {uuid} failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Delete {uuid} failed: {}", format_api_error(status, &body)));
        }

        if let Some(card) = with_db(state, |db| db.get_card(uuid))? {
            let _ = std::fs::remove_file(&card.file_path);
        }
        with_db(state, |db| {
            db.delete_card(uuid)?;
            db.delete_fts(uuid)
        })?;
    }

    Ok(uuids.len() as u32)
}

/// Pull all trunk relations from the server and update local SQLite.
/// Fetches the full trunk set via GET /sync/relations and replaces local trunks per-source.
async fn pull_trunk_relations(
    state: &tauri::State<'_, AppState>,
    client: &reqwest::Client,
    base: &str,
) -> Result<(), String> {
    let resp = client
        .get(format!("{base}/sync/relations"))
        .send()
        .await
        .map_err(|e| format!("Trunk pull request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        eprintln!("[sync_pull] trunk pull failed: {}", format_api_error(status, &body));
        return Ok(()); // Non-fatal
    }

    #[derive(Deserialize)]
    struct TrunkPullEnvelope {
        data: TrunkPullData,
    }
    #[derive(Deserialize)]
    struct TrunkPullData {
        #[serde(default)]
        #[allow(dead_code)]
        relations: Vec<ServerRelation>,
    }
    #[derive(Deserialize)]
    struct ServerRelation {
        source_uuid: String,
        target_uuid: String,
        #[allow(dead_code)]
        relation_type: String,
    }

    let envelope = resp
        .json::<TrunkPullEnvelope>()
        .await
        .map_err(|e| format!("Trunk pull response parse failed: {e}"))?;

    // Group by source UUID and replace local trunks
    let mut by_source: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for rel in &envelope.data.relations {
        by_source
            .entry(rel.source_uuid.clone())
            .or_default()
            .push(rel.target_uuid.clone());
    }

    for (source, targets) in &by_source {
        if let Err(e) = with_db(state, |db| db.replace_trunk_relations(source, targets)) {
            eprintln!("[sync_pull] trunk restore failed for {}: {e}", source);
        }
    }

    eprintln!("[sync_pull] restored trunks for {} sources", by_source.len());
    Ok(())
}

/// Push all local trunk relations to the server via the dedicated endpoint.
/// This is independent of card content version — trunks always sync.
async fn sync_trunk_relations(
    state: &tauri::State<'_, AppState>,
    client: &reqwest::Client,
    base: &str,
) -> Result<(), String> {
    let trunks: Vec<RelationPayload> = with_db(state, |db| {
        let all_relations = db.list_all_trunks()?;
        Ok(all_relations
            .into_iter()
            .map(|r| RelationPayload {
                source_uuid: r.source_uuid,
                target_uuid: r.target_uuid_or_tag,
                relation_type: r.relation_type,
            })
            .collect())
    })
    .map_err(|e| e.to_string())?;

    if trunks.is_empty() {
        return Ok(());
    }

    eprintln!("[sync_push] pushing {} trunk relations", trunks.len());

    let req = RelationsSyncRequest { relations: trunks };
    let resp = client
        .post(format!("{base}/sync/relations"))
        .json(&req)
        .send()
        .await
        .map_err(|e| format!("Trunk sync request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        eprintln!("[sync_push] trunk sync failed: {}", format_api_error(status, &body));
        // Non-fatal: trunk sync failure should not block the overall sync
        return Ok(());
    }

    let sync_resp = resp
        .json::<RelationsSyncResponse>()
        .await
        .map_err(|e| format!("Trunk sync response parse failed: {e}"))?;

    eprintln!("[sync_push] trunk sync accepted={}", sync_resp.accepted);
    Ok(())
}

// ── Commands ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn sync_pull(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let _ = run_sync_pull(&state, &app).await?;

    let _ = tauri::Emitter::emit(
        &app,
        "sync:progress",
        SyncProgressEvent {
            status: "done".into(),
            progress: 100,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn sync_push(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let _ = run_sync_push(&state, &app).await?;

    let _ = tauri::Emitter::emit(
        &app,
        "sync:progress",
        SyncProgressEvent {
            status: "done".into(),
            progress: 100,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn sync_delete_tombstones(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let _ = run_sync_delete_tombstones(&state).await?;

    let _ = tauri::Emitter::emit(
        &app,
        "sync:progress",
        SyncProgressEvent {
            status: "done".into(),
            progress: 100,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn sync_now(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<SyncRunReport, String> {
    let _ = tauri::Emitter::emit(
        &app,
        "sync:progress",
        SyncProgressEvent {
            status: "syncing".into(),
            progress: 0,
        },
    );

    let pull = run_sync_pull(&state, &app).await?;

    let uploaded = run_sync_push(&state, &app).await?;

    // Glossary sync (non-fatal)
    let _ = super::glossary::sync_glossary_pull_inner(&state).await;
    let _ = super::glossary::sync_glossary_push_inner(&state).await;

    let deleted = run_sync_delete_tombstones(&state).await?;

    let final_status = if pull.conflicts.is_empty() {
        "done"
    } else {
        "conflict"
    };
    let _ = tauri::Emitter::emit(
        &app,
        "sync:progress",
        SyncProgressEvent {
            status: final_status.into(),
            progress: 100,
        },
    );

    Ok(SyncRunReport {
        downloaded: pull.downloaded,
        uploaded,
        deleted,
        conflicts: pull.conflicts,
    })
}
