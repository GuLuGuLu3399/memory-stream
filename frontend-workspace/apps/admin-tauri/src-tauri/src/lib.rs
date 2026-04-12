//! # Memory Stream — Tauri 桌面端 Rust 核心
//!
//! 本 crate 是 memory-stream 知识管理系统的桌面端核心，负责：
//! - Markdown AST 解析与 HTML 渲染（调用 md-parser / ast-renderer）
//! - 统一 HTTP API 网关（通过 api 模块代理所有前端网络请求）
//! - 本地 SQLite 缓存管理（离线布局、图谱数据）
//! - 文件系统监控（Markdown Vault 目录变更检测）
//! - WebSocket 实时通信（边创建/删除操作）

mod api;
mod auth;
mod cache;
pub mod config;
mod draft;
mod export;
mod image;
mod importer;
mod toc;
mod vault_scanner;
mod watcher;
mod wikilink_replacer;
mod ws_client;

use api::AppHttpClient;
use auth::AuthState;
use cache::CacheManager;
use watcher::{FileChangeEvent, MarkdownWatcher};
use ws_client::WsSender;

use md_parser::{extract_wikilinks, parse_markdown};
use ast_renderer::render_to_html;
use temple_core::error::{ErrorCode, TempleError};
use temple_cache::{ChangeNotification, DocumentPool, VaultWatcher, preheat_vault};
use temple_graph::{KnowledgeGraph, GraphNode, GraphEdge, compute_hierarchical_layout};
use temple_search::SearchEngine;

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Listener, Manager, State};

/// Arc<AuthState> 类型别名，方便 Tauri command 使用
type AuthArc = Arc<AuthState>;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::WindowEvent;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// 应用全局状态，在启动时注入 Tauri State 管理
struct AppState {
    /// SQLite 缓存管理器（线程安全）
    cache: Arc<Mutex<Option<CacheManager>>>,
    /// 零 I/O 内存文档池
    doc_pool: DocumentPool,
    /// 文件系统监控器（新版 VaultWatcher → 内存池同步）
    vault_watcher: Option<VaultWatcher>,
    /// 文件系统监控器（旧版，兼容过渡）
    watcher: Option<MarkdownWatcher>,
    /// WebSocket 发送端（用于实时边操作）
    ws_sender: Option<WsSender>,
    /// 知识图谱引擎（Petgraph 有向图）
    knowledge_graph: Arc<Mutex<Option<KnowledgeGraph>>>,
    /// 全文搜索引擎（Tantivy + jieba 中文分词）
    search_engine: Arc<Mutex<Option<SearchEngine>>>,
}

/// 可重载的全局状态（运行时热更新配置）
pub struct ReloadableState {
    /// S3 存储配置（从 tauri-plugin-store 读取，支持热重载）
    pub s3_config: Mutex<Option<ms_storage::StorageConfig>>,
}

// ============================================================================
// 内部数据结构
// ============================================================================

/// Markdown 渲染结果，返回给前端
#[derive(Serialize, ts_rs::TS)]
#[ts(export_to = ".")]
struct RenderResult {
    /// 渲染后的 HTML 字符串
    html: String,
    /// AST 结构化 JSON 字符串
    ast_json: String,
    /// 从原文提取的纯文本摘要
    excerpt: String,
    /// 从原文提取的 wikilink 链接（提取自 Markdown 的链接目标）
    extracted_links: Vec<String>,
}

/// 本地布局缓存查询结果
#[derive(Serialize, ts_rs::TS)]
#[ts(export_to = ".")]
struct LayoutCacheResult {
    layouts: Vec<cache::CachedLayout>,
    edges: Vec<cache::CachedEdge>,
    #[ts(type = "number")]
    count: i64,
    last_sync: Option<String>,
}

/// 服务端同步操作结果
#[derive(Serialize, ts_rs::TS)]
#[ts(export_to = ".")]
struct SyncResult {
    success: bool,
    message: String,
    #[ts(type = "number")]
    synced_count: i64,
}

/// 创建卡片请求载荷（发送到 Go 后端）
#[derive(Serialize)]
struct CreateCardRequest {
    title: String,
    raw_md: String,
    excerpt: String,
    ast_data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    toc_data: Option<serde_json::Value>,
    parent_id: Option<String>,
    relation_type: Option<String>,
}

/// Go 后端创建卡片响应
#[derive(Deserialize)]
struct CreateCardResponse {
    card_id: String,
}

/// Go 后端图谱节点响应
#[derive(Deserialize)]
struct GraphNodeResponse {
    id: String,
    title: String,
}

/// Go 后端图谱边响应
#[derive(Deserialize)]
struct GraphEdgeResponse {
    source: String,
    target: String,
    relation: String,
}

/// Go 后端图谱概览响应
#[derive(Deserialize)]
struct OutlineResponse {
    nodes: Vec<GraphNodeResponse>,
    edges: Vec<GraphEdgeResponse>,
}

// ============================================================================
// Tauri IPC 命令
// ============================================================================

/// Markdown 解析与渲染命令
///
/// 接收原始 Markdown 文本，通过本地 AST 解析器生成结构化数据，
/// 再渲染为 HTML。同时提取纯文本摘要用于列表展示。
///
/// # 参数
/// - `content`: 原始 Markdown 文本（可能包含转义的换行符）
///
/// # 返回
/// `RenderResult` 包含 html、ast_json 和 excerpt
#[tauri::command]
async fn process_markdown(content: String) -> Result<RenderResult, TempleError> {
    tauri::async_runtime::spawn_blocking(move || {
    // 修复：将可能被转义的字面量 "\\n" 还原为真实换行符
    // 在 Go(JSON) → Vue(JS) → Tauri IPC → Rust(String) 的传输中，\n 可能变成 \\n
    let clean = content
        .replace("\\r\\n", "\n")
        .replace("\\n", "\n");

    let extracted_links = extract_wikilinks(&clean);

    let ast = parse_markdown(&clean)?;
    let html = render_to_html(&ast)?;
    let ast_json = serde_json::to_string(&ast)?;
    let excerpt = extract_plain_text(&clean, 150);

    Ok(RenderResult {
        html,
        ast_json,
        excerpt,
        extracted_links,
    })
    }).await.map_err(|e| TempleError::new(ErrorCode::TaskPanic, e.to_string()))?
}

/// 图片压缩为 WebP 格式
///
/// 将指定路径的图片转换为 WebP 格式，用于减少存储空间和传输带宽。
///
/// # 参数
/// - `input_path`: 源图片文件路径
/// - `output_path`: 输出 WebP 文件路径（为空时自动替换扩展名）
/// - `_quality`: 压缩质量（当前使用无损模式，此参数预留）
#[tauri::command]
async fn compress_image_to_webp(
    input_path: String,
    output_path: String,
    _quality: u8,
) -> Result<String, TempleError> {
    tauri::async_runtime::spawn_blocking(move || {
    let img =
        ::image::open(&input_path).map_err(|e| TempleError::new(ErrorCode::ImageDecodeFailed, e.to_string()))?;
    let output_path = if output_path.is_empty() {
        let mut p = std::path::PathBuf::from(&input_path);
        p.set_extension("webp");
        p.to_string_lossy().to_string()
    } else {
        output_path
    };
    let mut buf = std::io::BufWriter::new(
        std::fs::File::create(&output_path)?,
    );
    let encoder = ::image::codecs::webp::WebPEncoder::new_lossless(&mut buf);
    img.write_with_encoder(encoder)
        .map_err(|e| TempleError::new(ErrorCode::ImageEncodeFailed, e.to_string()))?;
    Ok(output_path)
    }).await.map_err(|e| TempleError::new(ErrorCode::TaskPanic, e.to_string()))?
}

/// 获取本地 SQLite 中缓存的所有布局数据
///
/// 用于离线模式或快速加载图谱视图，无需请求后端。
#[tauri::command]
fn get_cached_layouts(state: State<'_, Mutex<AppState>>) -> Result<LayoutCacheResult, TempleError> {
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    let cg = app_state
        .cache
        .lock()
        .map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    match &*cg {
        Some(cache) => {
            let layouts = cache.get_all_layouts().map_err(|e| TempleError::new(ErrorCode::CacheQueryFailed, e.to_string()))?;
            let edges = cache.get_all_edges().map_err(|e| TempleError::new(ErrorCode::CacheQueryFailed, e.to_string()))?;
            let count = cache.count().map_err(|e| TempleError::new(ErrorCode::CacheQueryFailed, e.to_string()))?;
            let last_sync = cache.get_last_sync_time().map_err(|e| TempleError::new(ErrorCode::CacheQueryFailed, e.to_string()))?;
            Ok(LayoutCacheResult {
                layouts,
                edges,
                count,
                last_sync,
            })
        }
        None => Err(TempleError::new(ErrorCode::CacheNotInitialized, "cache not initialized")),
    }
}

/// 创建卡片并建立与父卡片的关联关系
///
/// 这是专用的复合命令，一次调用完成：创建卡片 → 建立边关系。
/// 使用全局 HTTP 客户端连接池，避免重复创建。
#[allow(clippy::too_many_arguments)]
#[tauri::command]
async fn create_card_with_relation(
    client: State<'_, AppHttpClient>,
    auth_state: State<'_, AuthArc>,
    title: String,
    content: String,
    ast_data: String,
    excerpt: String,
    toc_data: Option<String>,
    parent_id: Option<String>,
    relation: String,
) -> Result<String, TempleError> {
    let parsed_toc: Option<serde_json::Value> = toc_data
        .and_then(|s| if s.is_empty() { None } else { serde_json::from_str(&s).ok() });
    let body = CreateCardRequest {
        title,
        raw_md: content,
        excerpt,
        ast_data,
        toc_data: parsed_toc,
        parent_id: parent_id.clone(),
        relation_type: Some(relation),
    };
    let mut req = client
        .0
        .post(format!("{}/cards", api::API_BASE_URL))
        .json(&body);
    if let Some(token) = auth_state.get_access_token() {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let resp = req.send().await
        .map_err(|e| TempleError::new(ErrorCode::NetworkUnreachable, e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(TempleError::new(ErrorCode::ApiError, format!("create card failed: {} - {}", status, body)));
    }

    let card_resp: CreateCardResponse = resp
        .json()
        .await
        .map_err(|e| TempleError::new(ErrorCode::AstDeserializeFailed, e.to_string()))?;
    Ok(card_resp.card_id)
}

/// 删除指定卡片（级联删除关联边和布局数据）
#[tauri::command]
async fn delete_card(
    client: State<'_, AppHttpClient>,
    auth_state: State<'_, AuthArc>,
    id: String,
) -> Result<(), TempleError> {
    let mut req = client
        .0
        .delete(format!("{}/cards/{}", api::API_BASE_URL, id));
    if let Some(token) = auth_state.get_access_token() {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let resp = req.send().await
        .map_err(|e| TempleError::new(ErrorCode::NetworkUnreachable, e.to_string()))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(TempleError::new(ErrorCode::ApiError, format!("delete failed: {}", resp.status())))
    }
}

/// 获取卡片详情（包含完整 raw_md 内容）
///
/// 用于编辑器加载卡片时进行数据水合（hydration）。
#[tauri::command]
async fn get_card_detail(
    client: State<'_, AppHttpClient>,
    auth_state: State<'_, AuthArc>,
    id: String,
) -> Result<serde_json::Value, TempleError> {
    let mut req = client
        .0
        .get(format!("{}/cards/{}", api::API_BASE_URL, id));
    if let Some(token) = auth_state.get_access_token() {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let resp = req.send().await
        .map_err(|e| TempleError::new(ErrorCode::NetworkUnreachable, e.to_string()))?;

    if !resp.status().is_success() {
        return Err(TempleError::new(ErrorCode::GraphNodeNotFound, format!("card not found: {}", resp.status())));
    }

    resp.json::<serde_json::Value>()
        .await
        .map_err(|e| TempleError::new(ErrorCode::AstDeserializeFailed, e.to_string()))
}

/// 从 Go 后端同步图谱数据到本地 SQLite 缓存
///
/// 异步版本：先发起 HTTP 请求（不持有锁），拿到数据后再锁定写入缓存。
#[tauri::command]
async fn sync_from_server(
    state: State<'_, Mutex<AppState>>,
    client: State<'_, AppHttpClient>,
    auth_state: State<'_, AuthArc>,
) -> Result<SyncResult, TempleError> {
    // Phase 1: 网络请求（不持有任何锁）
    let mut req = client
        .0
        .get(format!("{}/graph/outline", api::API_BASE_URL))
        .timeout(std::time::Duration::from_secs(10));
    if let Some(token) = auth_state.get_access_token() {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let resp = req.send().await
        .map_err(|e| TempleError::new(ErrorCode::NetworkUnreachable, e.to_string()))?;

    if !resp.status().is_success() {
        return Err(TempleError::new(ErrorCode::ApiError, format!("sync failed: {}", resp.status())));
    }

    let outline: OutlineResponse = resp.json().await
        .map_err(|e| TempleError::new(ErrorCode::AstDeserializeFailed, e.to_string()))?;

    // Phase 2: 数据转换（CPU 密集但无需锁）
    let layouts: Vec<cache::CachedLayout> = outline
        .nodes
        .iter()
        .map(|n| cache::CachedLayout {
            card_id: n.id.clone(),
            x: 0.0,
            y: 0.0,
            title: n.title.clone(),
            category_id: None,
            hot_score: 0.0,
            updated_at: format_sync_time(),
        })
        .collect();

    let edges: Vec<cache::CachedEdge> = outline
        .edges
        .iter()
        .map(|e| cache::CachedEdge {
            source_id: e.source.clone(),
            target_id: e.target.clone(),
            relation: e.relation.clone(),
        })
        .collect();

    let layout_count = layouts.len();
    let edge_count = edges.len();

    // Phase 3: 锁定 + 写入（持锁时间最短化）
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    let cg = app_state
        .cache
        .lock()
        .map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    let cache = match &*cg {
        Some(c) => c,
        None => return Err(TempleError::new(ErrorCode::CacheNotInitialized, "cache not initialized")),
    };

    cache.upsert_layouts(&layouts).map_err(|e| TempleError::new(ErrorCode::CacheQueryFailed, e.to_string()))?;
    cache.upsert_edges(&edges).map_err(|e| TempleError::new(ErrorCode::CacheQueryFailed, e.to_string()))?;
    cache.set_last_sync_time(&format_sync_time()).map_err(|e| TempleError::new(ErrorCode::CacheQueryFailed, e.to_string()))?;
    let count = cache.count().map_err(|e| TempleError::new(ErrorCode::CacheQueryFailed, e.to_string()))?;

    Ok(SyncResult {
        success: true,
        message: format!("sync: {} nodes, {} edges", layout_count, edge_count),
        synced_count: count,
    })
}

// ============================================================================
// 零 I/O 文档池命令 (temple_cache)
// ============================================================================

/// 从内存池获取单个文档（零 I/O）
#[tauri::command]
fn get_document(state: State<'_, Mutex<AppState>>, path: String) -> Result<Option<temple_cache::Document>, TempleError> {
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    Ok(app_state.doc_pool.get(&path).map(|arc| arc.as_ref().clone()))
}

/// 获取所有文档的轻量元数据（列表页使用，零 I/O）
#[tauri::command]
fn list_documents(state: State<'_, Mutex<AppState>>) -> Result<Vec<temple_cache::DocumentMeta>, TempleError> {
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    Ok(app_state.doc_pool.list_metadata())
}

/// 按标题搜索文档（简单的子串匹配，后续由 temple_search 替代）
#[tauri::command]
fn search_documents(state: State<'_, Mutex<AppState>>, query: String) -> Result<Vec<temple_cache::DocumentMeta>, TempleError> {
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    Ok(app_state.doc_pool.search_by_title(&query))
}

/// 启动 Vault 监听器 + 内存池同步
#[tauri::command]
fn start_vault_watcher(state: State<'_, Mutex<AppState>>, vault_path: String) -> Result<String, TempleError> {
    let mut app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    let mut watcher = VaultWatcher::new(&vault_path)?;
    watcher.start()?;
    app_state.vault_watcher = Some(watcher);
    Ok(format!("vault watcher started: {}", vault_path))
}

/// 轮询 Vault 变更 → 同步内存池 → 返回变更通知
#[tauri::command]
fn poll_vault_changes(state: State<'_, Mutex<AppState>>) -> Result<Vec<serde_json::Value>, TempleError> {
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    match &app_state.vault_watcher {
        Some(watcher) => {
            let notifications = watcher.poll_and_sync(&app_state.doc_pool);
            Ok(notifications.into_iter().map(|n| {
                match n {
                    ChangeNotification::Created(meta) => serde_json::json!({"kind": "created", "path": meta.path, "title": meta.title}),
                    ChangeNotification::Updated(meta) => serde_json::json!({"kind": "updated", "path": meta.path, "title": meta.title}),
                    ChangeNotification::Removed { path } => serde_json::json!({"kind": "removed", "path": path}),
                }
            }).collect())
        }
        None => Ok(vec![]),
    }
}

/// 预热 Vault — 并行扫描所有 Markdown 文件加载到内存
#[tauri::command]
fn preheat_vault_cmd(state: State<'_, Mutex<AppState>>, vault_path: String) -> Result<serde_json::Value, TempleError> {
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    let stats = preheat_vault(&app_state.doc_pool, &vault_path)?;
    Ok(serde_json::json!({
        "total_files": stats.total_files,
        "parsed_ok": stats.parsed_ok,
        "parse_errors": stats.parse_errors,
        "elapsed_ms": stats.elapsed_ms,
    }))
}

// ============================================================================
// 知识图谱命令 (temple_graph)
// ============================================================================

/// 从 Go 后端同步图谱数据并构建内存知识图谱
///
/// 异步版本：先发起 HTTP 请求（不持有锁），拿到数据后构建图谱再锁定写入。
#[tauri::command]
async fn build_knowledge_graph(
    state: State<'_, Mutex<AppState>>,
    client: State<'_, AppHttpClient>,
    auth_state: State<'_, AuthArc>,
) -> Result<serde_json::Value, TempleError> {
    // Phase 1: 网络请求（不持有任何锁）
    let mut req = client
        .0
        .get(format!("{}/graph/outline", api::API_BASE_URL))
        .timeout(std::time::Duration::from_secs(15));
    if let Some(token) = auth_state.get_access_token() {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let resp = req.send().await
        .map_err(|e| TempleError::new(ErrorCode::NetworkUnreachable, e.to_string()))?;

    if !resp.status().is_success() {
        return Err(TempleError::new(ErrorCode::ApiError, format!("sync graph failed: {}", resp.status())));
    }

    let outline: OutlineResponse = resp.json().await
        .map_err(|e| TempleError::new(ErrorCode::AstDeserializeFailed, e.to_string()))?;

    // Phase 2: 数据转换 + 图谱构建（CPU 密集但无需锁）
    let nodes: Vec<GraphNode> = outline.nodes.iter().map(|n| GraphNode {
        id: n.id.clone(),
        title: n.title.clone(),
    }).collect();

    let edges: Vec<GraphEdge> = outline.edges.iter().map(|e| GraphEdge {
        source: e.source.clone(),
        target: e.target.clone(),
        relation: e.relation.clone(),
    }).collect();

    let graph = KnowledgeGraph::build_from(nodes, edges);
    let node_count = graph.node_count();
    let edge_count = graph.edge_count();

    // Phase 3: 锁定 + 写入（持锁时间最短化）
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    *app_state.knowledge_graph.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))? = Some(graph);

    Ok(serde_json::json!({
        "node_count": node_count,
        "edge_count": edge_count,
    }))
}

/// 获取中心节点 N 度 BFS 子图（局部星图）
///
/// 前端请求图谱聚焦时调用，返回子图节点和边，
/// 无需加载完整图谱数据。
#[tauri::command]
fn get_subgraph(
    state: State<'_, Mutex<AppState>>,
    center_id: String,
    depth: usize,
) -> Result<temple_graph::SubgraphResult, TempleError> {
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    let kg = app_state.knowledge_graph.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    match &*kg {
        Some(graph) => graph.subgraph(&center_id, depth),
        None => Err(TempleError::new(ErrorCode::GraphNodeNotFound, "knowledge graph not built")),
    }
}

/// 计算层级布局（替代前端 Dagre）
///
/// 使用 Sugiyama 风格拓扑排序，返回节点坐标。
/// 前端直接应用坐标渲染，无需在 JS 侧计算布局。
#[tauri::command]
async fn compute_graph_layout(
    state: State<'_, Mutex<AppState>>,
    node_ids: Option<Vec<String>>,
) -> Result<temple_graph::LayoutResult, TempleError> {
    // 读取图谱快照（短暂持锁）
    let graph_snapshot = {
        let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
        let kg = app_state.knowledge_graph.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
        match &*kg {
            Some(graph) => graph.clone(),
            None => return Err(TempleError::new(ErrorCode::GraphNodeNotFound, "knowledge graph not built")),
        }
    };

    // CPU 密集的布局计算放入阻塞线程池
    tauri::async_runtime::spawn_blocking(move || {
        match node_ids {
            Some(ids) => Ok(compute_hierarchical_layout(&graph_snapshot, &ids)),
            None => Ok(compute_hierarchical_layout(&graph_snapshot, &[])),
        }
    }).await.map_err(|e| TempleError::new(ErrorCode::TaskPanic, e.to_string()))?
}

// ============================================================================
// 全文搜索命令 (temple_search)
// ============================================================================

/// 初始化全文搜索索引
///
/// 在指定目录创建或打开 Tantivy 索引。
/// 前端首次使用搜索功能时调用。
#[tauri::command]
fn init_search_index(state: State<'_, Mutex<AppState>>, index_dir: String) -> Result<String, TempleError> {
    let engine = SearchEngine::open(&index_dir)?;
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    *app_state.search_engine.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))? = Some(engine);
    Ok(format!("search index opened: {}", index_dir))
}

/// 将内存池中的文档批量索引到搜索引擎
///
/// 从 DocumentPool 读取所有文档，写入 Tantivy 索引。
/// 应在 preheat_vault 后调用。
#[tauri::command]
async fn rebuild_search_index(state: State<'_, Mutex<AppState>>) -> Result<serde_json::Value, TempleError> {
    // 短暂持锁：读取快照
    let (engine_ptr, docs_to_index): (Arc<Mutex<Option<SearchEngine>>>, Vec<(String, String, String)>) = {
        let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
        let metas = app_state.doc_pool.list_metadata();
        let mut docs = Vec::with_capacity(metas.len());
        for meta in &metas {
            if let Some(doc) = app_state.doc_pool.get(&meta.path) {
                docs.push((meta.path.clone(), meta.title.clone(), doc.raw_md.clone()));
            }
        }
        (app_state.search_engine.clone(), docs)
    };

    // CPU 密集的索引构建放入阻塞线程池
    let indexed = tauri::async_runtime::spawn_blocking(move || {
        let se = engine_ptr.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
        let engine = match &*se {
            Some(e) => e,
            None => return Err(TempleError::new(ErrorCode::IndexNotReady, "search index not initialized")),
        };
        let mut writer = engine.writer()?;
        let mut count = 0u32;
        for (path, title, raw_md) in &docs_to_index {
            let tags: Vec<String> = Vec::new();
            let wikilinks: Vec<String> = Vec::new();
            SearchEngine::add_document(&mut writer, engine.fields(), path, title, raw_md, &tags, &wikilinks)?;
            count += 1;
        }
        SearchEngine::commit(writer)?;
        Ok(count)
    }).await.map_err(|e| TempleError::new(ErrorCode::TaskPanic, e.to_string()))??;

    Ok(serde_json::json!({
        "indexed_docs": indexed,
    }))
}

/// 执行全文搜索
///
/// 使用 jieba 中文分词 + Tantivy BM25 评分。
#[tauri::command]
fn fulltext_search(
    state: State<'_, Mutex<AppState>>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<temple_search::SearchResult>, TempleError> {
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    let se = app_state.search_engine.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    match &*se {
        Some(engine) => engine.search(&query, limit.unwrap_or(20)),
        None => Err(TempleError::new(ErrorCode::IndexNotReady, "search index not initialized")),
    }
}

/// 获取搜索索引统计
#[tauri::command]
fn search_index_stats(state: State<'_, Mutex<AppState>>) -> Result<temple_search::IndexStats, TempleError> {
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    let se = app_state.search_engine.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    match &*se {
        Some(engine) => engine.stats(),
        None => Err(TempleError::new(ErrorCode::IndexNotReady, "search index not initialized")),
    }
}

/// 轮询文件系统变更事件
///
/// 从 MarkdownWatcher 的事件队列中取出自上次轮询以来的所有文件变更。
#[tauri::command]
fn poll_file_changes(state: State<'_, Mutex<AppState>>) -> Result<Vec<FileChangeEvent>, TempleError> {
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    match &app_state.watcher {
        Some(watcher) => Ok(watcher.poll_changes()),
        None => Ok(vec![]),
    }
}

/// 启动文件系统监控器
///
/// 监控指定目录下的 Markdown 文件变更（创建、修改、删除）。
#[tauri::command]
fn start_watcher(
    state: State<'_, Mutex<AppState>>,
    watch_dir: String,
) -> Result<String, TempleError> {
    let mut app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    let mut watcher = MarkdownWatcher::new(&watch_dir).map_err(|e| TempleError::new(ErrorCode::DirectoryNotFound, e.to_string()))?;
    watcher.start().map_err(|e| TempleError::new(ErrorCode::InternalError, e.to_string()))?;
    app_state.watcher = Some(watcher);
    Ok(format!("watching: {}", watch_dir))
}

/// 通过 WebSocket 创建图谱边
///
/// 使用实时 WebSocket 通道而非 HTTP 请求，确保多客户端同步。
#[tauri::command]
fn create_edge_cmd(
    state: State<'_, Mutex<AppState>>,
    source: String,
    target: String,
    rel: String,
) -> Result<(), TempleError> {
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    match &app_state.ws_sender {
        Some(sender) => sender.send_action(
            "CREATE_EDGE",
            serde_json::json!({"source_id": source, "target_id": target, "relation_type": rel}),
        ).map_err(|e| TempleError::new(ErrorCode::GraphEdgeCreationFailed, e.to_string())),
        None => Err(TempleError::new(ErrorCode::WsConnectionFailed, "WS not connected")),
    }
}

/// 通过 WebSocket 删除图谱边
#[tauri::command]
fn delete_edge_cmd(
    state: State<'_, Mutex<AppState>>,
    source: String,
    target: String,
) -> Result<(), TempleError> {
    let app_state = state.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    match &app_state.ws_sender {
        Some(sender) => sender.send_action(
            "DELETE_EDGE",
            serde_json::json!({"source_id": source, "target_id": target}),
        ).map_err(|e| TempleError::new(ErrorCode::GraphEdgeCreationFailed, e.to_string())),
        None => Err(TempleError::new(ErrorCode::WsConnectionFailed, "WS not connected")),
    }
}

#[tauri::command]
fn import_markdown_files(paths: Vec<String>) -> Result<Vec<importer::ImportCard>, TempleError> {
    let mut cards = vec![];
    for path_str in paths {
        let path = std::path::PathBuf::from(&path_str);
        match importer::parse_markdown_file(&path) {
            Ok(card) => cards.push(card),
            Err(e) => return Err(TempleError::new(ErrorCode::MarkdownParseFailed, format!("Failed to parse {}: {}", path_str, e))),
        }
    }
    Ok(cards)
}

#[tauri::command]
fn import_zip_archive(path: String) -> Result<Vec<importer::ImportCard>, TempleError> {
    let zip_path = std::path::PathBuf::from(&path);
    let (cards, _images) = importer::extract_zip_archive(&zip_path)
        .map_err(|e| TempleError::new(ErrorCode::ExportZipFailed, format!("Failed to extract zip: {}", e)))?;
    Ok(cards)
}

/// 热重载系统配置
///
/// 从 tauri-plugin-store 读取配置，从 keyring 读取敏感信息，
/// 更新全局可重载状态（S3 配置等），无需重启应用。
#[tauri::command]
async fn reload_sys_config(app: tauri::AppHandle) -> Result<(), TempleError> {
    let config = config::get_config(&app).await.map_err(|e| TempleError::new(ErrorCode::InternalError, e))?;

    let access_key = config::keyring_wrapper::get_secret("S3_ACCESS_KEY").map_err(|e| TempleError::new(ErrorCode::StorageConfigError, e))?;
    let secret_key = config::keyring_wrapper::get_secret("S3_SECRET_KEY").map_err(|e| TempleError::new(ErrorCode::StorageConfigError, e))?;

    let s3_config = ms_storage::StorageConfig {
        endpoint: config.s3_endpoint.clone(),
        region: config.s3_region.clone(),
        bucket: config.s3_bucket.clone(),
        access_key: access_key.unwrap_or_default(),
        secret_key: secret_key.unwrap_or_default(),
        public_url_base: config.s3_public_url_base.clone(),
        use_path_style: config.s3_use_path_style,
    };

    let state = app.state::<ReloadableState>();
    let mut guard = state.s3_config.lock().map_err(|e| TempleError::new(ErrorCode::CacheLockFailed, e.to_string()))?;
    *guard = Some(s3_config);

    app.emit("config-reloaded", serde_json::json!({}))
        .map_err(|e| TempleError::new(ErrorCode::InternalError, e.to_string()))?;

    log::info!("System config reloaded successfully");
    Ok(())
}

/// 获取系统配置
///
/// 从 tauri-plugin-store 读取配置，从 keyring 读取敏感信息，
/// 合并后返回完整的 SysConfig 给前端。
#[tauri::command]
async fn get_sys_config(app: tauri::AppHandle) -> Result<config::SysConfig, TempleError> {
    let mut config = config::get_config(&app).await.map_err(|e| TempleError::new(ErrorCode::InternalError, e))?;
    config.s3_access_key = config::keyring_wrapper::get_secret("S3_ACCESS_KEY").map_err(|e| TempleError::new(ErrorCode::StorageConfigError, e))?;
    config.s3_secret_key = config::keyring_wrapper::get_secret("S3_SECRET_KEY").map_err(|e| TempleError::new(ErrorCode::StorageConfigError, e))?;
    Ok(config)
}

/// 保存系统配置
///
/// 非敏感字段写入 tauri-plugin-store，敏感字段（access_key / secret_key）
/// 写入 OS keyring，然后自动热重载内存中的 S3 配置。
#[tauri::command]
async fn save_sys_config(app: tauri::AppHandle, config: config::SysConfig) -> Result<(), TempleError> {
    if let Some(ref key) = config.s3_access_key {
        config::keyring_wrapper::store_secret("S3_ACCESS_KEY", key).map_err(|e| TempleError::new(ErrorCode::StorageConfigError, e))?;
    }
    if let Some(ref key) = config.s3_secret_key {
        config::keyring_wrapper::store_secret("S3_SECRET_KEY", key).map_err(|e| TempleError::new(ErrorCode::StorageConfigError, e))?;
    }
    config::save_config(&app, &config).await.map_err(|e| TempleError::new(ErrorCode::InternalError, e))?;
    reload_sys_config(app).await
}

/// 设置本地知识库 (Vault) 目录路径
///
/// 接收前端通过 Dialog 选定的目录路径，持久化到 SysConfig 中，
/// 并通过事件通知前端状态变更。
#[tauri::command]
async fn set_vault_path(app: tauri::AppHandle, path: String) -> Result<config::SysConfig, TempleError> {
    let mut config = config::get_config(&app).await.map_err(|e| TempleError::new(ErrorCode::InternalError, e))?;
    config.vault_path = Some(path);
    config::save_config(&app, &config).await.map_err(|e| TempleError::new(ErrorCode::InternalError, e))?;
    app.emit("vault-path-changed", &config.vault_path)
        .map_err(|e| TempleError::new(ErrorCode::InternalError, e.to_string()))?;
    log::info!("Vault path set to: {:?}", config.vault_path);
    Ok(config)
}

#[tauri::command]
async fn test_api_connection(app: tauri::AppHandle) -> Result<(), TempleError> {
    let config = config::get_config(&app).await.map_err(|e| TempleError::new(ErrorCode::InternalError, e))?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| TempleError::new(ErrorCode::InternalError, format!("HTTP client init failed: {}", e)))?;
    let url = format!("{}/health", config.api_base_url.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| TempleError::new(ErrorCode::NetworkUnreachable, format!("API unreachable: {}", e)))?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(TempleError::new(ErrorCode::ApiError, format!("API returned HTTP {}", resp.status())))
    }
}

#[tauri::command]
async fn test_s3_connection(app: tauri::AppHandle) -> Result<(), TempleError> {
    let config = config::get_config(&app).await.map_err(|e| TempleError::new(ErrorCode::InternalError, e))?;
    let access_key = config::keyring_wrapper::get_secret("S3_ACCESS_KEY").map_err(|e| TempleError::new(ErrorCode::StorageConfigError, e))?;
    let secret_key = config::keyring_wrapper::get_secret("S3_SECRET_KEY").map_err(|e| TempleError::new(ErrorCode::StorageConfigError, e))?;
    let storage_config = ms_storage::StorageConfig {
        endpoint: config.s3_endpoint.clone(),
        region: config.s3_region.clone(),
        bucket: config.s3_bucket.clone(),
        access_key: access_key.unwrap_or_default(),
        secret_key: secret_key.unwrap_or_default(),
        public_url_base: config.s3_public_url_base.clone(),
        use_path_style: config.s3_use_path_style,
    };
    let backend = ms_storage::S3Backend::new(&storage_config).map_err(|e| TempleError::new(ErrorCode::StorageConfigError, e.to_string()))?;
    let provider: Box<dyn ms_storage::StorageProvider> = Box::new(backend);
    match provider.exists("__health_check_probe__").await {
        Ok(_) => Ok(()),
        Err(e) => Err(TempleError::new(ErrorCode::S3UploadFailed, format!("S3 connection failed: {}", e))),
    }
}

// ============================================================================
// 内部工具函数
// ============================================================================

/// 从 Markdown 纯文本中提取摘要：去除标记符号 → 拼接段落 → 截断
///
/// # 参数
/// - `content`: 原始 Markdown 文本
/// - `max_len`: 摘要最大字符数
fn extract_plain_text(content: &str, max_len: usize) -> String {
    let stripped = strip_wikilinks(content);

    let lines: Vec<&str> = stripped.lines().collect();
    let mut plain_parts: Vec<String> = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "---" {
            continue;
        }
        let cleaned = trimmed
            .replace(['*', '_', '`'], "")
            .replace(['[', ']', '('], "")
            .trim_start_matches('-')
            .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.')
            .trim()
            .to_string();
        if !cleaned.is_empty() {
            plain_parts.push(cleaned);
        }
        if plain_parts.join(" ").chars().count() >= max_len {
            break;
        }
    }

    let text = plain_parts.join(" ");
    let char_count = text.chars().count();
    if char_count > max_len {
        let chars_vec: Vec<char> = text.chars().take(max_len).collect();
        let truncated: String = chars_vec.into_iter().collect();
        let byte_cutoff = truncated.len();
        if let Some(byte_pos) = text[..byte_cutoff].rfind(' ') {
            if byte_pos > text.len() / 3 {
                return text[..byte_pos].to_string() + "...";
            }
        }
        truncated + "..."
    } else {
        text
    }
}

fn strip_wikilinks(text: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        if i + 1 < len && chars[i] == '[' && chars[i + 1] == '[' {
            let start = i + 2;
            let mut end = start;
            while end + 1 < len && !(chars[end] == ']' && chars[end + 1] == ']') {
                end += 1;
            }
            if end + 1 < len {
                for &ch in &chars[start..end] {
                    result.push(ch);
                }
                i = end + 2;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

/// 生成基于 Unix 时间戳的同步时间字符串
fn format_sync_time() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!(
        "{}.{:03}",
        duration.as_secs(),
        duration.subsec_millis()
    )
}

// ============================================================================
// 应用入口
// ============================================================================

/// Tauri 应用主入口
///
/// 初始化并注册：
/// 1. 全局 HTTP 客户端（`AppHttpClient`）— 复用连接池
/// 2. 应用状态（`AppState`）— 缓存、监控器、WebSocket
/// 3. 所有 IPC 命令处理器
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // ── 插件注册 ──────────────────────────────────────────────
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_store::Builder::new().build())

        // ── 应用初始化 ────────────────────────────────────────────
        .setup(|app| {
            // ── 1. 本地 SQLite 缓存 ──
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("cannot get app data dir");
            std::fs::create_dir_all(&data_dir).ok();
            let db_path = data_dir.join("memory-stream-cache.db");
            let cache =
                CacheManager::open(db_path.to_str().expect("cache db path must be valid UTF-8")).expect("SQLite init failed");
            let cache_arc = Arc::new(Mutex::new(Some(cache)));

            // ── 2. AuthState + Proactive Refresh ──
            let auth_arc = Arc::new(AuthState::new(data_dir.clone()));
            
            // Load config from store and initialize reloadable state
            let config = tauri::async_runtime::block_on(config::get_config(app.handle()))
                .expect("Failed to load config");
            
            let ws_url = if config.ws_url.is_empty() {
                "ws://localhost:8080/api/v1/ws".to_string()
            } else {
                config.ws_url.clone()
            };

            let access_key = config::keyring_wrapper::get_secret("S3_ACCESS_KEY")?;
            let secret_key = config::keyring_wrapper::get_secret("S3_SECRET_KEY")?;

            let s3_config = ms_storage::StorageConfig {
                endpoint: config.s3_endpoint.clone(),
                region: config.s3_region.clone(),
                bucket: config.s3_bucket.clone(),
                access_key: access_key.unwrap_or_default(),
                secret_key: secret_key.unwrap_or_default(),
                public_url_base: config.s3_public_url_base.clone(),
                use_path_style: config.s3_use_path_style,
            };

            app.manage(ReloadableState {
                s3_config: Mutex::new(Some(s3_config)),
            });

            let ws_sender = ws_client::start_ws_client(
                app.handle().clone(),
                cache_arc.clone(),
                auth_arc.clone(),
                ws_url,
            );

            // 注入应用状态
            app.manage(Mutex::new(AppState {
                cache: cache_arc,
                doc_pool: DocumentPool::new(),
                vault_watcher: None,
                watcher: None,
                ws_sender: Some(ws_sender),
                knowledge_graph: Arc::new(Mutex::new(None)),
                search_engine: Arc::new(Mutex::new(None)),
            }));

            // 注入全局 HTTP 客户端（连接池复用）
            app.manage(AppHttpClient::new());

            // ── AuthState（JWT 凭据管理） ── 共享 Arc，WS 和 Command 都能读取
            app.manage(auth_arc.clone());

            // ── Proactive Token 刷新 ── 启动后台定时器
            let refresh_client = reqwest::Client::new();
            auth::spawn_proactive_refresh(auth_arc.clone(), refresh_client);

            // ── DraftDb 本地草稿数据库（独立于缓存 DB） ──
            let draft_db_path = data_dir.join("drafts.db");
            let draft_manager = tauri::async_runtime::block_on(
                draft::DraftManager::new(&draft_db_path)
            ).expect("Draft DB init failed");
            app.manage(draft_manager.into_inner());

            // ── 3. 系统托盘 ──
            let show_item = MenuItemBuilder::with_id("show", "显示 Memory Stream").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().expect("app icon must be configured").clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ── 4. 关闭窗口时隐藏到托盘而非退出 ──
            if let Some(main_window) = app.get_webview_window("main") {
                let window = main_window.clone();
                main_window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                });
            }

            // ── 5. 全局快捷键 Alt+Space 唤起窗口 ──
            let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
            app.global_shortcut().register(shortcut)?;

            // ── 6. Deep Link 事件监听 ──
            // 监听 memory-stream:// 协议唤醒，将 URL 转发给前端路由
            let handle = app.handle().clone();
            app.listen("deep-link://request", move |event| {
                let payload = event.payload();
                // payload 可能是 JSON 数组格式: ["memory-stream://card/xxx"]
                let url = payload
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .trim_matches('"');
                log::info!("Deep link received: {}", url);
                let _ = handle.emit("deep-link-navigate", url);
            });

            // ── 7. 静默检查更新 ──
            let updater_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // 延迟 5 秒检查，避免阻塞启动
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                check_for_update(&updater_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth（JWT 认证）
            auth::login,
            auth::genesis,
            auth::set_auth_token,
            auth::get_auth_status,
            // API 网关
            api::api_request,
            // Markdown 处理
            process_markdown,
            compress_image_to_webp,
            // 缓存管理
            get_cached_layouts,
            sync_from_server,
            // 卡片 CRUD（update 走 api_request 网关，create/delete/detail 保留专用命令）
            create_card_with_relation,
            delete_card,
            get_card_detail,
            // 文件监控
            poll_file_changes,
            // 零 I/O 文档池 (temple_cache)
            get_document,
            list_documents,
            search_documents,
            start_vault_watcher,
            poll_vault_changes,
            preheat_vault_cmd,
            start_watcher,
            // 图谱边操作（WebSocket）
            create_edge_cmd,
            delete_edge_cmd,
            // 知识图谱引擎 (temple_graph)
            build_knowledge_graph,
            get_subgraph,
            compute_graph_layout,
            // 全文搜索引擎 (temple_search)
            init_search_index,
            rebuild_search_index,
            fulltext_search,
            search_index_stats,
            // 导入功能
            import_markdown_files,
            import_zip_archive,
            // 系统配置
            get_sys_config,
            save_sys_config,
            reload_sys_config,
            set_vault_path,
            test_api_connection,
            test_s3_connection,
            // 本地草稿（ms-local-draft）
            draft::save_draft,
            draft::load_draft,
            draft::list_drafts,
            draft::delete_draft,
            // 知识库导出（ms-kb-exporter）
            export::export_knowledge_base,
            // 目录树提取（ms-toc-extractor）
            toc::extract_toc,
            // 图片压缩 + S3 上传管道
            image::compress_and_upload_image,
            image::upload_clipboard_image,
            // Vault 配置扫描
            vault_scanner::scan_config,
            // Wikilink 替换预览
            wikilink_replacer::preview_merge_impact,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 检查应用更新
///
/// 后台静默轮询更新服务器，如果发现新版本则通知前端。
/// 实际的下载和安装由前端通过 `@tauri-apps/plugin-updater` 处理。
async fn check_for_update(app: &tauri::AppHandle) {
    use tauri_plugin_updater::UpdaterExt;

    match app.updater() {
        Ok(updater) => match updater.check().await {
            Ok(Some(update)) => {
                log::info!(
                    "Update available: {} → {}",
                    update.current_version,
                    update.version
                );
                // 通知前端有可用更新
                let date_str = update.date.map(|d| d.to_string());
                let _ = app.emit(
                    "update-available",
                    serde_json::json!({
                        "current_version": update.current_version,
                        "version": update.version,
                        "date": date_str,
                        "body": update.body,
                    }),
                );
            }
            Ok(None) => {
                log::info!("App is up to date");
            }
            Err(e) => {
                log::warn!("Update check failed: {}", e);
            }
        },
        Err(e) => {
            log::warn!("Updater not available: {}", e);
        }
    }
}

// ============================================================================
// ts-rs 类型导出测试
// ============================================================================

#[cfg(test)]
mod export_ts_types {
    use ts_rs::TS;

    /// 导出所有 IPC 类型到 bindings/ 目录
    #[test]
    fn export_all_ipc_types() {
        // lib.rs 内部类型
        super::RenderResult::export().unwrap();
        super::LayoutCacheResult::export().unwrap();
        super::SyncResult::export().unwrap();

        // 子模块类型
        super::auth::LoginResult::export().unwrap();
        super::draft::DraftDto::export().unwrap();
        super::export::ExportSummaryDto::export().unwrap();
        super::image::ImageUploadResult::export().unwrap();
        super::toc::TocNodeDto::export().unwrap();
        super::cache::CachedLayout::export().unwrap();
        super::cache::CachedEdge::export().unwrap();
        super::watcher::FileChangeEvent::export().unwrap();
        super::config::SysConfig::export().unwrap();
        super::vault_scanner::IssueSeverity::export().unwrap();
        super::vault_scanner::ConfigIssue::export().unwrap();
        super::vault_scanner::ScanResult::export().unwrap();
        super::wikilink_replacer::MergePreview::export().unwrap();
        super::wikilink_replacer::FileImpact::export().unwrap();
    }
}
