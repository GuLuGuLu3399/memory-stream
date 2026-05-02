use std::collections::HashMap;

use tauri::http::{Request, Response, StatusCode};
use tauri::Manager;

use crate::db::with_db;
use crate::state::AppState;

pub fn handle_ms_protocol(
    ctx: tauri::UriSchemeContext<'_, tauri::Wry>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let uri = request.uri().to_string();
    let path = uri
        .trim_start_matches("ms://localhost")
        .split('?')
        .next()
        .unwrap_or("");

    let state = ctx.app_handle().state::<AppState>();

    match path {
        p if p.starts_with("/api/ast/") => handle_ast(&state, p),
        "/api/graph/full" => handle_full_graph(&state),
        p if p.starts_with("/api/graph/neighborhood/") => handle_neighborhood(&state, &uri),
        p if p.starts_with("/assets/") => handle_asset(&state, p),
        p if p.starts_with("/api/graph/neighborhood/") => {
            handle_neighborhood(&state, &uri)
        }
        p if p.starts_with("/assets/") => handle_asset(&state, p),
        _ => json_response(StatusCode::NOT_FOUND, &format!("{{\"error\":\"not found: {path}\"}}")),
    }
}

fn handle_ast(state: &AppState, path: &str) -> Response<Vec<u8>> {
    let uuid = path.trim_start_matches("/api/ast/").trim_end_matches('/');
    if uuid.is_empty() {
        return json_response(StatusCode::BAD_REQUEST, "{\"error\":\"missing uuid\"}");
    }

    let card = match with_db(state, |db| db.get_card(uuid)) {
        Ok(Some(c)) => c,
        Ok(None) => return json_response(StatusCode::NOT_FOUND, "{\"error\":\"card not found\"}"),
        Err(e) => return json_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("{{\"error\":\"{e}\"}}")),
    };

    let content = match std::fs::read_to_string(&card.file_path) {
        Ok(c) => c,
        Err(e) => return json_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("{{\"error\":\"read: {e}\"}}")),
    };

    let doc = match ms_ast::parse_document(&content) {
        Ok(d) => d,
        Err(e) => return json_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("{{\"error\":\"parse: {e}\"}}")),
    };

    match serde_json::to_string(&doc) {
        Ok(json) => json_response(StatusCode::OK, &json),
        Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("{{\"error\":\"serialize: {e}\"}}")),
    }
}

fn handle_full_graph(state: &AppState) -> Response<Vec<u8>> {
    let cards = match with_db(state, |db| db.list_all()) {
        Ok(c) => c,
        Err(e) => return json_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("{{\"error\":\"{e}\"}}")),
    };

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut title_map: HashMap<String, String> = HashMap::new();

    for card in &cards {
        // Try to get title from FTS
        let title = with_db(state, |db| db.search_fts(&card.uuid, 1))
            .ok()
            .and_then(|hits| hits.into_iter().next())
            .map(|h| h.title)
            .unwrap_or_else(|| card.uuid.clone());

        title_map.insert(card.uuid.clone(), title.clone());
        nodes.push(serde_json::json!({
            "uuid": card.uuid,
            "title": title,
        }));

        if let Ok(outbound) = with_db(state, |db| db.get_outbound(&card.uuid)) {
            for rel in outbound {
                edges.push(serde_json::json!({
                    "source": rel.source_uuid,
                    "target": rel.target_uuid_or_tag,
                    "relation": rel.relation_type,
                }));
            }
        }
    }

    let body = serde_json::json!({ "nodes": nodes, "edges": edges });
    match serde_json::to_string(&body) {
        Ok(json) => json_response(StatusCode::OK, &json),
        Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("{{\"error\":\"{e}\"}}")),
    }
}

fn handle_neighborhood(state: &AppState, uri: &str) -> Response<Vec<u8>> {
    let path_part = uri
        .trim_start_matches("ms://localhost/api/graph/neighborhood/");
    let (uuid_part, query) = path_part.split_once('?').unwrap_or((path_part, ""));
    let uuid = uuid_part.trim_end_matches('/');

    let depth: usize = query
        .split('&')
        .find_map(|p| p.strip_prefix("depth="))
        .and_then(|v| v.parse().ok())
        .unwrap_or(2);

    // Build full graph then extract subgraph
    let cards = match with_db(state, |db| db.list_all()) {
        Ok(c) => c,
        Err(e) => return json_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("{{\"error\":\"{e}\"}}")),
    };

    let mut graph_nodes = Vec::new();
    let mut graph_edges = Vec::new();

    for card in &cards {
        let title = with_db(state, |db| db.search_fts(&card.uuid, 1))
            .ok()
            .and_then(|hits| hits.into_iter().next())
            .map(|h| h.title)
            .unwrap_or_else(|| card.uuid.clone());

        graph_nodes.push(ms_graph::GraphNode { id: card.uuid.clone(), title, x: None, y: None });

        if let Ok(outbound) = with_db(state, |db| db.get_outbound(&card.uuid)) {
            for rel in outbound {
                graph_edges.push(ms_graph::GraphEdge {
                    source: rel.source_uuid,
                    target: rel.target_uuid_or_tag,
                    relation: rel.relation_type,
                });
            }
        }
    }

    let kg = match ms_graph::KnowledgeGraph::build_from(graph_nodes, graph_edges) {
        Ok(g) => g,
        Err(e) => return json_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("{{\"error\":\"graph: {e}\"}}")),
    };

    let result = match kg.subgraph(uuid, depth) {
        Ok(r) => r,
        Err(e) => return json_response(StatusCode::NOT_FOUND, &format!("{{\"error\":\"{e}\"}}")),
    };

    match serde_json::to_string(&result) {
        Ok(json) => json_response(StatusCode::OK, &json),
        Err(e) => json_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("{{\"error\":\"{e}\"}}")),
    }
}

fn handle_asset(state: &AppState, path: &str) -> Response<Vec<u8>> {
    let asset_rel = path.trim_start_matches("/assets/");
    if asset_rel.is_empty() {
        return json_response(StatusCode::BAD_REQUEST, "{\"error\":\"missing asset path\"}");
    }

    let vault_path = match state.config.lock() {
        Ok(cfg) => cfg.vault_path.clone(),
        Err(_) => return json_response(StatusCode::INTERNAL_SERVER_ERROR, "{\"error\":\"config lock\"}"),
    };

    let asset_path = match safe_asset_path(&vault_path, asset_rel) {
        Ok(p) => p,
        Err(e) => return json_response(StatusCode::BAD_REQUEST, &format!("{{\"error\":\"{e}\"}}")),
    };

    match std::fs::read(&asset_path) {
        Ok(bytes) => {
            let content_type = guess_content_type(asset_rel);
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .header("Cache-Control", "public, max-age=86400")
                .body(bytes)
                .unwrap_or_else(|_| json_response(StatusCode::INTERNAL_SERVER_ERROR, "{}"))
        }
        Err(_) => json_response(StatusCode::NOT_FOUND, "{\"error\":\"asset not found\"}"),
    }
}

/// Validate that `asset_rel` resolves inside `{vault_root}/assets/` without escaping.
fn safe_asset_path(vault_root: &str, asset_rel: &str) -> Result<std::path::PathBuf, String> {
    // Reject obviously malicious patterns before touching the filesystem
    if asset_rel.starts_with('/')
        || asset_rel.starts_with('\\')
        || asset_rel.contains("..")
        || asset_rel.contains(':')
    {
        return Err("invalid asset path".into());
    }

    let assets_dir = std::path::Path::new(vault_root).join("assets");
    let target = assets_dir.join(asset_rel);

    // canonicalize resolves symlinks and collapses `./` etc.
    // If the file doesn't exist yet, fall back to a prefix check on the naive path.
    match std::fs::canonicalize(&target) {
        Ok(canonical_target) => {
            let canonical_dir = std::fs::canonicalize(&assets_dir)
                .unwrap_or_else(|_| assets_dir.clone());
            if !canonical_target.starts_with(&canonical_dir) {
                return Err("path traversal blocked".into());
            }
            Ok(canonical_target)
        }
        Err(_) => {
            // File doesn't exist — use string-based prefix check as fallback
            let dir_str = assets_dir.to_string_lossy();
            let target_str = target.to_string_lossy();
            if !target_str.starts_with(dir_str.as_ref()) {
                return Err("path traversal blocked".into());
            }
            Err("asset not found".into())
        }
    }
}

fn json_response(status: StatusCode, body: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(body.as_bytes().to_vec())
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Vec::new())
                .unwrap()
        })
}

fn guess_content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}
