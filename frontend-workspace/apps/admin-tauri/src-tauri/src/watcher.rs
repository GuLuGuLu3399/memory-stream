use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{Emitter, Manager};

use crate::db;
use crate::events::FileChangeEvent;
use crate::state::AppState;

const DEBOUNCE_MS: u64 = 200;
const STALE_THRESHOLD_SECS: u64 = 10;
const SWEEP_INTERVAL: u32 = 100;

pub fn start_vault_watcher(app: &tauri::AppHandle) -> Option<RecommendedWatcher> {
    let state = app.state::<AppState>();
    let vault_path = {
        let cfg = state.config.lock().map_err(|e| {
            eprintln!("[watcher] config lock failed: {e}");
            e
        }).ok()?;
        if cfg.vault_path.is_empty() {
            return None;
        }
        std::path::PathBuf::from(&cfg.vault_path)
    };

    let app_handle = app.clone();
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if tx.send(event).is_err() {
                    eprintln!("[watcher] channel send failed — receiver dropped");
                }
            }
        },
        notify::Config::default(),
    )
    .map_err(|e| { eprintln!("[watcher] watcher creation failed: {e}"); e }).ok()?;

    watcher.watch(&vault_path, RecursiveMode::Recursive).map_err(|e| { eprintln!("[watcher] watch registration failed for {}: {e}", vault_path.display()); e }).ok()?;

    std::thread::spawn(move || {
        let mut debounce_map: std::collections::HashMap<String, std::time::Instant> =
            std::collections::HashMap::new();
        let mut total_received: u32 = 0;
        let mut total_emitted: u32 = 0;
        let mut total_debounced: u32 = 0;

        while let Ok(event) = rx.recv() {
            total_received += 1;

            let kind = match event.kind {
                EventKind::Create(_) => "create",
                EventKind::Modify(_) => "modify",
                EventKind::Remove(_) => "delete",
                _ => continue,
            };

            for path in event.paths {
                // Only emit for .md files
                if path.extension().is_none_or(|ext| ext != "md") {
                    continue;
                }

                let path_str = path.to_string_lossy().to_string();
                let now = std::time::Instant::now();

                // Debounce: skip if same path fired within threshold
                if let Some(last) = debounce_map.get(&path_str) {
                    if now.duration_since(*last).as_millis() < DEBOUNCE_MS as u128 {
                        total_debounced += 1;
                        continue;
                    }
                }
                debounce_map.insert(path_str.clone(), now);
                total_emitted += 1;

                // On file removal, immediately clean DB records — only emit on success
                if kind == "delete" {
                    let state = app_handle.state::<AppState>();
                    if let Err(e) = db::with_db(&state, |db| db.delete_card_by_path(&path_str)) {
                        eprintln!("[watcher] delete_card_by_path failed for {}: {e}", path_str);
                        continue; // skip emit on DB failure
                    }
                }

                // On file modification, recompute hash and mark dirty for sync
                if kind == "modify" && path.exists() {
                    let state = app_handle.state::<AppState>();
                    if let Ok(Some(card)) = db::with_db(&state, |db| db.get_by_path(&path_str)) {
                        let file_ok = std::fs::metadata(&path)
                            .map(|m| m.len() <= 10 * 1024 * 1024)
                            .unwrap_or(false);
                        if file_ok {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let hash = db::compute_hash(&content);
                                if let Err(e) = db::with_db(&state, |db| db.mark_dirty(&card.uuid, &hash)) {
                                    eprintln!("[watcher] mark_dirty failed for {}: {e}", card.uuid);
                                }
                            }
                        }
                    }
                }

                if let Err(e) = app_handle.emit(
                    "fs:change",
                    FileChangeEvent {
                        path: path_str,
                        kind: kind.to_string(),
                    },
                ) {
                    eprintln!("[watcher] emit failed: {e}");
                }
            }

            // Periodic sweep: remove stale entries to bound memory
            if total_received.is_multiple_of(SWEEP_INTERVAL) && debounce_map.len() > 200 {
                let cutoff = now_ref() - std::time::Duration::from_secs(STALE_THRESHOLD_SECS);
                debounce_map.retain(|_, instant| *instant > cutoff);

                if total_received.is_multiple_of(1000) {
                    eprintln!(
                        "[watcher] stats — received: {total_received}, emitted: {total_emitted}, debounced: {total_debounced}, cache: {}",
                        debounce_map.len()
                    );
                }
            }
        }
    });

    Some(watcher)
}

/// Helper to get a reference `Instant` for comparison in retain.
/// `Instant` has no `now() -> &Self`, so we use a subtraction trick.
fn now_ref() -> std::time::Instant {
    std::time::Instant::now()
}
