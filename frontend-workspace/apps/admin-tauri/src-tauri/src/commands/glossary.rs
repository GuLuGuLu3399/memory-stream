use std::collections::HashMap;
use std::path::Path;
use tauri::State;

use crate::commands::sync::{api_base, api_client, check_auth_expired, format_api_error, get_last_sync_cursor};
use crate::db::compute_hash;
use crate::state::AppState;

// ── Local glossary helpers ─────────────────────────────────────────

fn bunker_dir(vault_path: &str) -> std::path::PathBuf {
    Path::new(vault_path).join(".bunker")
}

fn glossary_path(vault_path: &str) -> std::path::PathBuf {
    bunker_dir(vault_path).join("glossary.json")
}

fn glossary_versions_path(vault_path: &str) -> std::path::PathBuf {
    bunker_dir(vault_path).join("glossary_versions.json")
}

fn read_local_glossary(vault_path: &str) -> Result<HashMap<String, String>, String> {
    let path = glossary_path(vault_path);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("Read glossary failed: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Parse glossary failed: {e}"))
}

fn save_local_glossary(glossary: &HashMap<String, String>, vault_path: &str) -> Result<(), String> {
    let dir = bunker_dir(vault_path);
    ms_io::fs::ensure_dir(&dir).map_err(|e| format!("Ensure .bunker dir failed: {e}"))?;
    let content = serde_json::to_string_pretty(glossary).map_err(|e| format!("Serialize glossary failed: {e}"))?;
    let path = glossary_path(vault_path);
    ms_io::fs::write_atomic(&path, &content).map_err(|e| format!("Write glossary failed: {e}"))
}

fn read_versions(vault_path: &str) -> HashMap<String, i64> {
    let path = glossary_versions_path(vault_path);
    if !path.exists() {
        return HashMap::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn save_versions(versions: &HashMap<String, i64>, vault_path: &str) -> Result<(), String> {
    let dir = bunker_dir(vault_path);
    ms_io::fs::ensure_dir(&dir).map_err(|e| format!("Ensure .bunker dir failed: {e}"))?;
    let content = serde_json::to_string_pretty(versions).map_err(|e| format!("Serialize versions failed: {e}"))?;
    let path = glossary_versions_path(vault_path);
    ms_io::fs::write_atomic(&path, &content).map_err(|e| format!("Write versions failed: {e}"))
}

// ── Go server API types ────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct GlossaryManifestResponse {
    #[allow(dead_code)]
    cursor: String,
    #[serde(default)]
    changes: Vec<GlossaryManifestItem>,
}

#[derive(serde::Deserialize)]
struct GlossaryManifestItem {
    term: String,
    version: i64,
    #[allow(dead_code)]
    hash: String,
    #[allow(dead_code)]
    updated_at: String,
    is_deleted: bool,
}

#[derive(serde::Serialize, Clone)]
struct GlossaryItemPayload {
    term: String,
    definition: String,
    version: i64,
    hash: String,
}

#[derive(serde::Serialize)]
struct GlossaryBatchRequest {
    items: Vec<GlossaryItemPayload>,
}

#[derive(serde::Deserialize)]
struct GlossaryBatchResponse {
    #[serde(default)]
    accepted: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    conflicts: Vec<GlossaryConflictItem>,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct GlossaryConflictItem {
    term: String,
    server_version: i64,
    server_hash: String,
}

// ── Pull ───────────────────────────────────────────────────────────

pub async fn sync_glossary_pull_inner(state: &tauri::State<'_, AppState>) -> Result<(), String> {
    let base = api_base(state)?;
    let client = api_client(state)?;
    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.vault_path.clone()
    };
    if vault_path.is_empty() {
        return Ok(());
    }

    let since = get_last_sync_cursor(state);
    let url = match since {
        Some(s) => format!("{base}/sync/glossary/manifest?since={s}"),
        None => format!("{base}/sync/glossary/manifest"),
    };

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Glossary manifest fetch failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let _ = check_auth_expired(status);
        let body = resp.text().await.unwrap_or_default();
        eprintln!("[glossary_sync] manifest fetch failed: {}", format_api_error(status, &body));
        return Ok(()); // non-fatal
    }

    let manifest = resp
        .json::<GlossaryManifestResponse>()
        .await
        .map_err(|e| format!("Glossary manifest parse failed: {e}"))?;

    if manifest.changes.is_empty() {
        return Ok(());
    }

    // Fetch full glossary once to get definitions for changed items
    let server_glossary: HashMap<String, String> = {
        let resp = client
            .get(format!("{base}/glossary"))
            .send()
            .await
            .map_err(|e| format!("Glossary fetch failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let _ = check_auth_expired(status);
            let body = resp.text().await.unwrap_or_default();
            eprintln!("[glossary_sync] full glossary fetch failed: {}", format_api_error(status, &body));
            return Ok(()); // non-fatal
        }

        let full: serde_json::Value = resp.json().await.map_err(|e| format!("Parse failed: {e}"))?;
        full.get("data")
            .and_then(|d| d.get("glossary"))
            .and_then(|g| g.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|entry| {
                        let term = entry.get("term")?.as_str()?.to_string();
                        let def = entry.get("definition")?.as_str()?.to_string();
                        Some((term, def))
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let mut glossary = read_local_glossary(&vault_path)?;
    let mut versions = read_versions(&vault_path);

    for item in &manifest.changes {
        if item.is_deleted {
            glossary.remove(&item.term);
            versions.remove(&item.term);
        } else if let Some(def) = server_glossary.get(&item.term) {
            glossary.insert(item.term.clone(), def.clone());
            versions.insert(item.term.clone(), item.version);
        }
    }

    save_local_glossary(&glossary, &vault_path)?;
    save_versions(&versions, &vault_path)?;

    Ok(())
}

// ── Push ───────────────────────────────────────────────────────────

pub async fn sync_glossary_push_inner(state: &tauri::State<'_, AppState>) -> Result<(), String> {
    let base = api_base(state)?;
    let client = api_client(state)?;
    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.vault_path.clone()
    };
    if vault_path.is_empty() {
        return Ok(());
    }

    let glossary = read_local_glossary(&vault_path)?;
    if glossary.is_empty() {
        return Ok(());
    }

    let versions = read_versions(&vault_path);

    let items: Vec<GlossaryItemPayload> = glossary
        .iter()
        .map(|(term, definition)| GlossaryItemPayload {
            term: term.clone(),
            definition: definition.clone(),
            version: versions.get(term).copied().unwrap_or(0),
            hash: compute_hash(definition),
        })
        .collect();

    let mut versions = versions;
    let mut accepted_total = 0;

    for chunk in items.chunks(100) {
        let batch_req = GlossaryBatchRequest { items: chunk.to_vec() };

        let resp = client
            .post(format!("{base}/sync/glossary/batch"))
            .json(&batch_req)
            .send()
            .await
            .map_err(|e| format!("Glossary batch push failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let _ = check_auth_expired(status);
            let body = resp.text().await.unwrap_or_default();
            eprintln!("[glossary_sync] push failed: {}", format_api_error(status, &body));
            return Ok(()); // non-fatal
        }

        let batch_resp = resp
            .json::<GlossaryBatchResponse>()
            .await
            .map_err(|e| format!("Glossary batch parse failed: {e}"))?;

        for term in &batch_resp.accepted {
            versions.insert(term.clone(), versions.get(term).copied().unwrap_or(0) + 1);
        }
        accepted_total += batch_resp.accepted.len();
    }

    if accepted_total > 0 {
        save_versions(&versions, &vault_path)?;
    }

    Ok(())
}

// ── Tauri Commands ─────────────────────────────────────────────────

#[tauri::command]
pub fn read_glossary(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        if cfg.vault_path.is_empty() {
            return Err("Vault path not configured".into());
        }
        cfg.vault_path.clone()
    };
    let path = glossary_path(&vault_path);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("Read glossary failed: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Parse glossary failed: {e}"))
}

#[tauri::command]
pub fn save_glossary(
    glossary: HashMap<String, String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        if cfg.vault_path.is_empty() {
            return Err("Vault path not configured".into());
        }
        cfg.vault_path.clone()
    };
    save_local_glossary(&glossary, &vault_path)
}

#[tauri::command]
pub async fn sync_glossary_pull(state: State<'_, AppState>) -> Result<(), String> {
    sync_glossary_pull_inner(&state).await
}

#[tauri::command]
pub async fn sync_glossary_push(state: State<'_, AppState>) -> Result<(), String> {
    sync_glossary_push_inner(&state).await
}
