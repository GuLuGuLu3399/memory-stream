mod commands;
mod db;
mod events;
mod protocol;
mod models;
mod state;
mod watcher;

use gray_matter::{engine::YAML, Matter};
use serde::{Deserialize, Serialize};
use tauri::Manager;

use models::{article_path, CardMeta, ParsedDocument};
use state::{AppConfig, AppState, StorageConfig};

// ── Config commands ────────────────────────────────────────────────

#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    Ok(cfg.clone())
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct ConfigPatch {
    api_base_url: String,
    vault_path: String,
    theme: String,
}

#[tauri::command]
fn set_config(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    patch: ConfigPatch,
) -> Result<(), String> {
    let vault_changed = {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        if !patch.api_base_url.is_empty() {
            let url = patch.api_base_url.trim();
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err("API URL must start with http:// or https://".to_string());
            }
            cfg.api_base_url = url.to_string();
        }
        let changed = !patch.vault_path.is_empty() && patch.vault_path != cfg.vault_path;
        if changed {
            cfg.vault_path = patch.vault_path;
        }
        if !patch.theme.is_empty() {
            cfg.theme = patch.theme;
        }
        changed
    };
    if vault_changed {
        *state.meta_db.lock().map_err(|e| e.to_string())? = None;

        let mut watcher_guard = state.vault_watcher.lock().map_err(|e| e.to_string())?;
        *watcher_guard = None;
        *watcher_guard = watcher::start_vault_watcher(&app);
    }
    state.save_config()
}

#[tauri::command]
fn set_storage_config(
    state: tauri::State<'_, AppState>,
    storage: StorageConfig,
) -> Result<(), String> {
    let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
    cfg.storage = Some(storage);
    drop(cfg);
    state.save_config()
}

#[tauri::command]
fn clear_storage_config(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
    cfg.storage = None;
    drop(cfg);
    state.save_config()
}

// ── AST commands ───────────────────────────────────────────────────

#[tauri::command]
fn parse_markdown(raw_text: &str) -> Result<ParsedDocument, String> {
    parse_markdown_document(raw_text)
}

#[tauri::command]
fn parse_live_markdown(content: &str) -> Result<serde_json::Value, String> {
    let ast = ms_ast::parse_markdown(content).map_err(|e| e.to_string())?;
    serde_json::to_value(&ast).map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_document_io(
    meta: CardMeta,
    content: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<CardMeta, String> {
    let vault_path = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        if cfg.vault_path.is_empty() {
            return Err("Vault path not configured".into());
        }
        cfg.vault_path.clone()
    };

    let meta = CardMeta {
        updated_at: now_rfc3339(),
        ..meta
    };
    let full_content = models::compose_document(&meta, &content)?;
    let current_card = db::with_db(&state, |db| db.get_card(&meta.uuid))?;
    let current_path = current_card
        .as_ref()
        .map(|card| std::path::PathBuf::from(&card.file_path))
        .unwrap_or_else(|| std::path::PathBuf::from(&vault_path).join(format!("{}.md", meta.uuid)));
    let desired_path = article_path(std::path::Path::new(&vault_path), &meta.category, &meta.title)?;

    if desired_path != current_path && desired_path.exists() {
        return Err(format!(
            "Target file already exists: {}",
            desired_path.display()
        ));
    }

    if let Some(parent) = desired_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Create target directory failed: {e}"))?;
    }

    if desired_path == current_path {
        ms_io::fs::write_atomic(&desired_path, &full_content)
            .map_err(|e| format!("Write failed: {e}"))?;
    } else {
        ms_io::fs::write_atomic(&desired_path, &full_content)
            .map_err(|e| format!("Write failed: {e}"))?;
        if current_path.exists() {
            std::fs::remove_file(&current_path)
                .map_err(|e| format!("Remove old file failed: {e}"))?;
        }
    }

    let next_version = current_card.as_ref().map(|card| card.version.saturating_add(1)).unwrap_or(1);
    let updated = ms_meta::CardIndex {
        uuid: meta.uuid.clone(),
        file_path: desired_path.to_string_lossy().to_string(),
        file_hash: db::compute_hash(&full_content),
        version: next_version,
        sync_status: ms_meta::SyncStatus::PendingPush,
        last_synced_hash: current_card.and_then(|card| card.last_synced_hash),
    };

    db::with_db(&state, |db| db.upsert_card(&updated))?;

    // FTS + wiki link extraction via shared ingestion pipeline
    commands::vault::ingest_card_content(&meta.uuid, &content, &meta.title, &meta.category, &state)?;

    if let Err(e) = tauri::Emitter::emit(
        &app,
        "fs:change",
        events::FileChangeEvent {
            path: desired_path.to_string_lossy().to_string(),
            kind: "modify".into(),
        },
    ) {
        eprintln!("[save_document_io] event emit failed: {e}");
    }

    Ok(meta)
}

pub(crate) fn parse_markdown_document(raw_text: &str) -> Result<ParsedDocument, String> {
    let matter = Matter::<YAML>::new();
    let result = matter.parse(raw_text).map_err(|e| e.to_string())?;
    let meta: Option<CardMeta> = result
        .data
        .and_then(|data: gray_matter::Pod| data.deserialize::<CardMeta>().ok());
    let pure_content = result.content;
    let ast = ms_ast::parse_markdown(&pure_content).map_err(|e| e.to_string())?;

    // Compute analysis
    let lines = pure_content.lines().count();
    let chars = pure_content.chars().count();
    let cjk: usize = pure_content
        .chars()
        .filter(|c| ('\u{4E00}'..='\u{9FFF}').contains(c) || ('\u{3400}'..='\u{4DBF}').contains(c))
        .count();
    let lat_words = pure_content
        .replace(|c: char| ('\u{4E00}'..='\u{9FFF}').contains(&c) || ('\u{3400}'..='\u{4DBF}').contains(&c), " ")
        .split_whitespace()
        .count();
    let words = cjk + lat_words;
    let read_time = ((words as f32) / 300.0).ceil() as u32;

    let toc_flat = ms_ast::extract_toc_flat(&ast);
    let toc: Vec<models::TocItem> = toc_flat
        .into_iter()
        .map(|item| models::TocItem {
            level: item.level,
            text: item.text,
            id: item.slug,
        })
        .collect();

    let excerpt: String = pure_content
        .chars()
        .take(200)
        .collect::<String>()
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();

    let outbound_links = ms_ast::extract_links(&pure_content)
        .map(|v| v.len())
        .unwrap_or(0);

    Ok(ParsedDocument {
        meta,
        analysis: models::DocAnalysis {
            stats: models::DocumentStats {
                lines,
                chars,
                words,
                read_time,
            },
            toc,
            excerpt,
            outbound_links,
        },
        ast: serde_json::to_value(&ast).map_err(|e| e.to_string())?,
        content: pure_content,
    })
}

pub(crate) fn now_rfc3339() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs() as i64;
    let days = secs / 86400;
    let (year, month, day) = days_to_ymd(days);
    let time_secs = secs % 86400;
    let h = time_secs / 3600;
    let m = (time_secs % 3600) / 60;
    let s = time_secs % 60;
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
}

#[derive(Serialize)]
struct GraphSnapshot {
    nodes: Vec<ms_graph::GraphNode>,
    edges: Vec<ms_graph::GraphEdge>,
}

fn build_graph_snapshot(state: &AppState) -> Result<GraphSnapshot, String> {
    let cards = db::with_db(state, |db| db.list_all())?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for card in &cards {
        let title = db::with_db(state, |db| db.search_fts(&card.uuid, 1))
            .ok()
            .and_then(|hits| hits.into_iter().next())
            .map(|h| h.title)
            .unwrap_or_else(|| {
                std::path::Path::new(&card.file_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| card.uuid.clone())
            });

        nodes.push(ms_graph::GraphNode {
            id: card.uuid.clone(),
            title,
            x: None,
            y: None,
        });

        if let Ok(outbound) = db::with_db(state, |db| db.get_outbound(&card.uuid)) {
            for rel in outbound {
                edges.push(ms_graph::GraphEdge {
                    source: rel.source_uuid,
                    target: rel.target_uuid_or_tag,
                    relation: rel.relation_type,
                });
            }
        }
    }

    let valid_ids: std::collections::HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
    edges.retain(|e| valid_ids.contains(e.source.as_str()) && valid_ids.contains(e.target.as_str()));

    Ok(GraphSnapshot { nodes, edges })
}

#[tauri::command]
fn get_full_graph(state: tauri::State<'_, AppState>) -> Result<GraphSnapshot, String> {
    build_graph_snapshot(&state)
}

#[tauri::command]
fn get_graph_neighborhood(
    uuid: String,
    depth: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> Result<ms_graph::SubgraphResult, String> {
    let snapshot = build_graph_snapshot(&state)?;
    let graph = ms_graph::KnowledgeGraph::build_from(snapshot.nodes, snapshot.edges)
        .map_err(|e| format!("graph: {e}"))?;
    graph
        .subgraph(&uuid, depth.unwrap_or(2))
        .map_err(|e| e.to_string())
}

/// 预计算断开 trunk 后是否需要回退为 link（文件 I/O 在 DB 锁外完成）。
fn check_fallback_link(source: &str, target: &str, state: &AppState) -> Result<bool, String> {
    let source_card = db::with_db(state, |db| db.get_card(source))?;
    let Some(card) = source_card else { return Ok(false) };
    let Ok(content) = std::fs::read_to_string(&card.file_path) else { return Ok(false) };
    let Ok(links) = ms_ast::extract_links(&content) else { return Ok(false) };
    let target_title = db::with_db(state, |db| db.get_card_title(target))?;
    let Some(title) = target_title else { return Ok(false) };
    Ok(links.iter().any(|link| link == &title))
}

#[tauri::command]
fn create_trunk(source: String, target: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    db::with_db(&state, |db| {
        db.create_trunk_atomic(&source, &target)?;
        db.set_sync_status(&source, ms_meta::SyncStatus::PendingPush)
    })
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_trunk_with_fallback(source: String, target: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let fallback = check_fallback_link(&source, &target, &state)?;
    db::with_db(&state, |db| {
        db.delete_trunk_with_fallback_atomic(&source, &target, fallback)?;
        db.set_sync_status(&source, ms_meta::SyncStatus::PendingPush)
    })
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn create_link(source: String, target: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    db::with_db(&state, |db| db.insert_link_relation(&source, &target))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn reverse_trunk(source: String, target: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    db::with_db(&state, |db| {
        db.reverse_trunk_atomic(&source, &target)?;
        db.set_sync_status(&source, ms_meta::SyncStatus::PendingPush)?;
        db.set_sync_status(&target, ms_meta::SyncStatus::PendingPush)
    })
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct BacklinkItem {
    uuid: String,
    title: String,
    relation_type: String,
}

#[tauri::command]
fn get_backlinks(uuid: String, state: tauri::State<'_, AppState>) -> Result<Vec<BacklinkItem>, String> {
    let relations = db::with_db(&state, |db| db.get_inbound(&uuid))
        .map_err(|e| e.to_string())?;
    let mut backlinks = Vec::new();
    for rel in relations {
        let title = db::with_db(&state, |db| db.get_card_title(&rel.source_uuid))
            .ok()
            .flatten()
            .unwrap_or_else(|| rel.source_uuid.clone());
        backlinks.push(BacklinkItem {
            uuid: rel.source_uuid,
            title,
            relation_type: rel.relation_type,
        });
    }
    Ok(backlinks)
}

#[derive(Serialize)]
struct NavNode {
    uuid: String,
    title: String,
}

#[derive(Serialize)]
struct TrunkNav {
    parents: Vec<NavNode>,
    children: Vec<NavNode>,
}

#[tauri::command]
fn get_trunk_navigation(uuid: String, state: tauri::State<'_, AppState>) -> Result<TrunkNav, String> {
    let inbound = db::with_db(&state, |db| db.get_inbound(&uuid))
        .map_err(|e| e.to_string())?;
    let parents: Vec<NavNode> = inbound
        .iter()
        .filter(|r| r.relation_type == "trunk")
        .filter_map(|r| {
            let title = db::with_db(&state, |db| db.get_card_title(&r.source_uuid))
                .ok()
                .flatten()?;
            Some(NavNode {
                uuid: r.source_uuid.clone(),
                title,
            })
        })
        .collect();

    let outbound = db::with_db(&state, |db| db.get_outbound(&uuid))
        .map_err(|e| e.to_string())?;
    let children: Vec<NavNode> = outbound
        .iter()
        .filter(|r| r.relation_type == "trunk")
        .filter_map(|r| {
            let title = db::with_db(&state, |db| db.get_card_title(&r.target_uuid_or_tag))
                .ok()
                .flatten()?;
            Some(NavNode {
                uuid: r.target_uuid_or_tag.clone(),
                title,
            })
        })
        .collect();

    Ok(TrunkNav { parents, children })
}

#[tauri::command]
fn purge_orphan_cards(state: tauri::State<'_, AppState>) -> Result<usize, String> {
    let cards = db::with_db(&state, |db| db.list_all())?;
    let mut deleted = 0usize;
    for card in &cards {
        let title = db::with_db(&state, |db| db.get_card_title(&card.uuid))?;
        let Some(t) = title else { continue };
        if t != "未命名卡片" {
            continue;
        }
        let outbound = db::with_db(&state, |db| db.get_outbound(&card.uuid))?;
        if !outbound.is_empty() {
            continue;
        }
        let inbound = db::with_db(&state, |db| db.get_inbound(&card.uuid))?;
        if !inbound.is_empty() {
            continue;
        }
        db::with_db(&state, |db| db.delete_card(&card.uuid))?;
        if std::path::Path::new(&card.file_path).exists() {
            std::fs::remove_file(&card.file_path)
                .map_err(|e| format!("Delete file failed: {e}"))?;
        }
        deleted += 1;
    }
    Ok(deleted)
}

fn days_to_ymd(mut days: i64) -> (i64, u32, u32) {
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

// ── Misc commands ──────────────────────────────────────────────────

#[tauri::command]
fn check_title_exists(title: String, state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let hits = db::with_db(&state, |db| db.search_fts(&title, 1))?;
    Ok(hits.first().map(|h| h.title == title).unwrap_or(false))
}

#[tauri::command]
async fn upload_image(
    image_data: Vec<u8>,
    filename: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    const MAX_IMAGE_SIZE: usize = 50 * 1024 * 1024;
    if image_data.len() > MAX_IMAGE_SIZE {
        return Err(format!("图片过大: {} bytes (上限 50MB)", image_data.len()));
    }

    let (s3_config, public_domain) = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
        let sc = cfg
            .storage
            .clone()
            .ok_or_else(|| "Storage not configured".to_string())?;
        let domain = sc.public_domain.clone();
        (sc.into(), domain)
    };

    let extension = filename
        .rsplit('.')
        .next()
        .unwrap_or("webp")
        .to_string();

    let compressed = ms_io::media::compress_to_webp(
        image_data,
        ms_io::media::CompressOptions::new(75.0),
    )
    .await
    .map_err(|e| format!("Image compression failed: {e}"))?;

    let object_name = format!("{}.{}", uuid::Uuid::new_v4(), extension);
    let object_path = format!("assets/{object_name}");

    let backend = ms_io::cloud::S3Backend::new(&s3_config)
        .map_err(|e| format!("S3 init failed: {e}"))?;

    let content_type = format!("image/{extension}");
    backend
        .upload(&object_path, &compressed, &content_type)
        .await
        .map_err(|e| format!("Upload failed: {e}"))?;

    let public_url = format!("{}/{}", public_domain.trim_end_matches('/'), object_path);
    Ok(public_url)
}


// ── Entry point ────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .register_uri_scheme_protocol("ms", protocol::handle_ms_protocol)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to resolve app data dir: {e}"))?;
            std::fs::create_dir_all(&config_dir).map_err(|e| format!("Cannot create config dir: {e}"))?;

            let state = AppState {
                config: std::sync::Mutex::new(AppConfig::default()),
                meta_db: std::sync::Mutex::new(None),
                config_dir,
                vault_watcher: std::sync::Mutex::new(None),
            };
            state.load_config();
            app.manage(state);

            // Start vault filesystem watcher
            let w = watcher::start_vault_watcher(app.handle());
            if let Some(w) = w {
                let state = app.state::<AppState>();
                if let Ok(mut guard) = state.vault_watcher.lock() {
                    *guard = Some(w);
                } else {
                    eprintln!("[setup] vault_watcher lock poisoned — watcher not stored");
                };
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            set_config,
            set_storage_config,
            clear_storage_config,
            parse_markdown,
            parse_live_markdown,
            save_document_io,
            check_title_exists,
            upload_image,
            get_full_graph,
            get_graph_neighborhood,
            create_trunk,
            delete_trunk_with_fallback,
            create_link,
            reverse_trunk,
            get_backlinks,
            get_trunk_navigation,
            purge_orphan_cards,
            // Zone 1: Vault IO
            commands::vault::read_card_file,
            commands::vault::create_card,
            commands::vault::delete_card,
            commands::vault::rename_card,
            commands::vault::move_card,
            commands::vault::create_category,
            commands::vault::rename_category,
            commands::vault::delete_category,
            commands::vault::scan_vault_tree,
            commands::vault::scan_and_heal,
            commands::vault::resolve_conflict_keep_local,
            commands::vault::resolve_conflict_keep_remote,
            commands::vault::get_conflicted_uuids,
            // Zone 2: Search
            commands::search::search_fts,
            commands::search::materialize_ghost,
            // Zone 3: Sync
            commands::sync::sync_pull,
            commands::sync::sync_push,
            commands::sync::sync_delete_tombstones,
            commands::sync::sync_now,
            // Zone 3.1: Auth
            commands::auth::login_to_server,
            commands::auth::logout_from_server,

            // Zone 4: Glossary
            commands::glossary::read_glossary,
            commands::glossary::save_glossary,
            commands::glossary::sync_glossary_pull,
            commands::glossary::sync_glossary_push,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
