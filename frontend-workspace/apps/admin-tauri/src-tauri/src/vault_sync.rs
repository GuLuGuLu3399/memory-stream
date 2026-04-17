//! # Vault 双向同步模块
//!
//! 云端卡片 ↔ 本地 .md 文件的双向同步：
//! - `sync_cloud_to_vault`: 增量拉取云端卡片，写入本地 .md（按分类子目录组织）
//! - `process_local_changes`: 从 Watcher 获取本地变更，上传到云端

use crate::api::AppHttpClient;
use crate::auth::AuthState;
use crate::config;
use futures_util::stream::{self, StreamExt};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tauri::State;
use md_parser::parse_markdown;
use temple_core::error::{ErrorCode, TempleError};

const CLOUD_SYNC_CONCURRENCY: usize = 8;

// ============================================================================
// 数据结构
// ============================================================================

/// 云端 → 本地同步结果
#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export_to = ".")]
pub struct SyncResult {
    pub synced: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
}

/// 本地 → 云端同步结果
#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export_to = ".")]
pub struct LocalSyncResult {
    pub uploaded: u32,
    pub created: u32,
    pub errors: Vec<String>,
}

/// API 返回的卡片列表项
#[derive(serde::Deserialize)]
struct CardListItem {
    id: String,
    title: String,
    updated_at: String,
    category_id: Option<u64>,
}

/// API 返回的卡片列表响应（匹配 Go PaginatedResult）
#[derive(serde::Deserialize)]
struct CardListResponse {
    data: Vec<CardListItem>,
    next_cursor: Option<String>,
}

/// API 返回的卡片详情（含 raw_md）
///
/// 注：虽然 id, title, category_id, updated_at 等字段在 fetch_card_detail 返回的 detail 对象中
/// 没有被显式使用（只使用了 raw_md），但这些字段必须存在以正确反序列化 API 响应的 JSON。
#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct CardDetail {
    id: String,
    title: String,
    raw_md: String,
    category_id: Option<u64>,
    updated_at: String,
}

/// API 返回的分类
#[derive(Clone, serde::Deserialize)]
struct CategoryItem {
    id: u64,
    name: String,
    parent_id: Option<u64>,
}

/// API 返回的分类列表
#[derive(serde::Deserialize)]
struct CategoryListResponse {
    categories: Vec<CategoryItem>,
}

#[derive(Debug)]
struct CloudSyncTask {
    card_id: String,
    title: String,
    file_path: PathBuf,
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 将卡片 title 转为安全的文件名
fn title_to_filename(title: &str) -> String {
    let sanitized: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = sanitized.trim();
    if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        format!("{}.md", trimmed)
    }
}

/// 构建 Authorization header
fn auth_header(auth: &Arc<AuthState>) -> Option<String> {
    auth.get_access_token()
        .map(|token| format!("Bearer {}", token))
}

/// GET 请求封装
async fn api_get(
    client: &reqwest::Client,
    base_url: &str,
    path: &str,
    auth: &Arc<AuthState>,
) -> Result<serde_json::Value, TempleError> {
    let url = format!("{}{}", base_url, path);
    let mut req = client.get(&url);
    if let Some(header) = auth_header(auth) {
        req = req.header("Authorization", header);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| TempleError::new(ErrorCode::NetworkUnreachable, format!("请求失败: {}", e)))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(TempleError::new(
            ErrorCode::ApiError,
            format!("HTTP {}: {}", status, body),
        ));
    }
    resp.json()
        .await
        .map_err(|e| TempleError::new(ErrorCode::InternalError, format!("解析响应失败: {}", e)))
}

/// PUT 请求封装
async fn api_put(
    client: &reqwest::Client,
    base_url: &str,
    path: &str,
    body: &serde_json::Value,
    auth: &Arc<AuthState>,
) -> Result<(), TempleError> {
    let url = format!("{}{}", base_url, path);
    let mut req = client.put(&url).json(body);
    if let Some(header) = auth_header(auth) {
        req = req.header("Authorization", header);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| TempleError::new(ErrorCode::NetworkUnreachable, format!("请求失败: {}", e)))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(TempleError::new(
            ErrorCode::ApiError,
            format!("HTTP {}: {}", status, body),
        ));
    }
    Ok(())
}

/// POST 请求封装
async fn api_post(
    client: &reqwest::Client,
    base_url: &str,
    path: &str,
    body: &serde_json::Value,
    auth: &Arc<AuthState>,
) -> Result<serde_json::Value, TempleError> {
    let url = format!("{}{}", base_url, path);
    let mut req = client.post(&url).json(body);
    if let Some(header) = auth_header(auth) {
        req = req.header("Authorization", header);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| TempleError::new(ErrorCode::NetworkUnreachable, format!("请求失败: {}", e)))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(TempleError::new(
            ErrorCode::ApiError,
            format!("HTTP {}: {}", status, body),
        ));
    }
    resp.json()
        .await
        .map_err(|e| TempleError::new(ErrorCode::InternalError, format!("解析响应失败: {}", e)))
}

/// 获取本地文件的修改时间（UNIX 秒）
fn file_mtime(path: &Path) -> Option<u64> {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

/// 递归收集 Vault 下所有 Markdown 文件
fn collect_markdown_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), TempleError> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| TempleError::new(ErrorCode::DirectoryNotFound, format!("读取目录失败 '{}': {}", dir.display(), e)))?;

    for entry in entries {
        let entry = entry
            .map_err(|e| TempleError::new(ErrorCode::DirectoryNotFound, format!("遍历目录失败 '{}': {}", dir.display(), e)))?;
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(&path, out)?;
            continue;
        }

        let is_md = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or(false);
        if is_md {
            out.push(path);
        }
    }

    Ok(())
}

// ============================================================================
// 云端 → 本地
// ============================================================================

/// 增量同步：将云端卡片导出为本地 .md 文件
///
/// 策略：
/// 1. 拉取分类列表构建 ID→名称映射
/// 2. 拉取全部卡片列表（标题 + updated_at）
/// 3. 对比本地文件 mtime 与 updated_at，跳过未变化的
/// 4. 只对有差异的卡片获取完整 raw_md 并写入
/// 5. 按分类组织子目录
#[tauri::command]
pub async fn sync_cloud_to_vault(
    app: tauri::AppHandle,
    client: State<'_, AppHttpClient>,
    auth: State<'_, Arc<AuthState>>,
) -> Result<SyncResult, TempleError> {
    let sync_start = Instant::now();
    let cfg = config::get_config(&app)
        .await
        .map_err(|e| TempleError::new(ErrorCode::InternalError, e))?;
    let vault_path = cfg
        .vault_path
        .ok_or_else(|| TempleError::new(ErrorCode::DirectoryNotFound, "未配置知识库目录".to_string()))?;

    let vault_dir = PathBuf::from(&vault_path);
    if !vault_dir.exists() {
        std::fs::create_dir_all(&vault_dir)
            .map_err(|e| TempleError::new(ErrorCode::DirectoryNotFound, format!("创建目录失败: {}", e)))?;
    }

    let base_url = client.get_base_url();
    let http = &client.client;

    // 1. 获取分类映射
    let fetch_categories_start = Instant::now();
    let categories = fetch_categories(http, &base_url, &auth).await.unwrap_or_default();
    let category_path_map = build_category_path_map(&categories);
    log::info!(
        "[VaultSync] fetched categories in {} ms",
        fetch_categories_start.elapsed().as_millis()
    );

    // 2. 获取全部卡片列表（分页拉取）
    let fetch_cards_start = Instant::now();
    let all_cards = fetch_all_cards(http, &base_url, &auth).await?;
    log::info!(
        "[VaultSync] fetched {} cards in {} ms",
        all_cards.len(),
        fetch_cards_start.elapsed().as_millis()
    );

    let mut synced: u32 = 0;
    let mut skipped: u32 = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut tasks: Vec<CloudSyncTask> = Vec::new();

    // 3. 增量对比 + 写入
    for card in &all_cards {
        let category_path = card
            .category_id
            .and_then(|cid| category_path_map.get(&cid).cloned());
        let filename = title_to_filename(&card.title);
        let file_path = build_file_path(&vault_dir, category_path.as_ref(), &filename);

        // 检查是否需要更新：文件不存在 或 云端 updated_at 更新
        if file_path.exists() && !should_update(&file_path, &card.updated_at) {
            skipped += 1;
            continue;
        }

        tasks.push(CloudSyncTask {
            card_id: card.id.clone(),
            title: card.title.clone(),
            file_path,
        });
    }

    let http = http.clone();
    let base_url = base_url.clone();
    let auth = auth.inner().clone();

    let fetch_details_start = Instant::now();
    let sync_results = stream::iter(tasks.into_iter().map(|task| {
        let http = http.clone();
        let base_url = base_url.clone();
        let auth = auth.clone();
        async move {
            let detail = fetch_card_detail(&http, &base_url, &task.card_id, &auth)
                .await
                .map_err(|e| format!("获取失败 '{}': {}", task.title, e))?;

            if let Some(parent) = task.file_path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| format!("创建目录失败 '{}': {}", task.title, e))?;
            }

            tokio::fs::write(&task.file_path, detail.raw_md)
                .await
                .map_err(|e| format!("写入失败 '{}': {}", task.title, e))
        }
    }))
    .buffer_unordered(CLOUD_SYNC_CONCURRENCY)
    .collect::<Vec<Result<(), String>>>()
    .await;
    log::info!(
        "[VaultSync] fetched details and wrote files in {} ms",
        fetch_details_start.elapsed().as_millis()
    );

    for result in sync_results {
        match result {
            Ok(_) => synced += 1,
            Err(e) => errors.push(e),
        }
    }

    // 4. 更新同步时间戳
    let _ = save_last_sync_time(&app);

    log::info!(
        "[VaultSync] cloud_to_vault finished: synced={}, skipped={}, errors={}, total={} ms",
        synced,
        skipped,
        errors.len(),
        sync_start.elapsed().as_millis()
    );

    Ok(SyncResult {
        synced,
        skipped,
        errors,
    })
}

/// 判断本地文件是否需要更新
///
/// 简单比较：解析云端 ISO 时间戳为 UNIX 秒，对比文件 mtime。
/// 云端较新 → 需要更新。
fn should_update(file_path: &Path, cloud_updated_at: &str) -> bool {
    let local_mtime = match file_mtime(file_path) {
        Some(t) => t,
        None => return true, // 无法获取 mtime，保守更新
    };

    // 简易 ISO 时间戳解析（截取 "2026-04-13T12:00:00" 部分）
    let cloud_unix = parse_iso_to_unix(cloud_updated_at).unwrap_or(0);

    // 云端比本地新 → 需要更新（加 1 秒容差）
    cloud_unix > local_mtime.saturating_sub(1)
}

/// 简易 ISO 8601 → UNIX 时间戳转换
fn parse_iso_to_unix(iso: &str) -> Option<u64> {
    // 格式: "2026-04-13T12:00:00..." 或 "2026-04-13T12:00:00Z"
    let s = iso.get(..19)?; // "YYYY-MM-DDTHH:MM:SS"
    let year: u64 = s.get(..4)?.parse().ok()?;
    let month: u64 = s.get(5..7)?.parse().ok()?;
    let day: u64 = s.get(8..10)?.parse().ok()?;
    let hour: u64 = s.get(11..13)?.parse().ok()?;
    let minute: u64 = s.get(14..16)?.parse().ok()?;
    let second: u64 = s.get(17..19)?.parse().ok()?;

    // 简易天数累计（不考虑闰年的完整精确计算，足够用于对比）
    let days_since_epoch = days_from_civil(year, month, day)?;
    Some(days_since_epoch * 86400 + hour * 3600 + minute * 60 + second)
}

/// 公历年月日 → 自 1970-01-01 以来的天数（Howard Hinnant 算法）
fn days_from_civil(year: u64, month: u64, day: u64) -> Option<u64> {
    let y = if month <= 2 { year - 1 } else { year };
    let m = if month <= 2 { month + 9 } else { month - 3 };
    let era = y / 400;
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + day;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe - 719468)
}

/// 目录名安全化
fn sanitize_dir_name(name: &str) -> String {
    name.chars()
        .map(|c| if c == '/' || c == '\\' || c == ':' { '_' } else { c })
        .collect()
}

fn card_lookup_key(title: &str, category_id: Option<u64>) -> String {
    format!(
        "{}::{}",
        title.trim().to_lowercase(),
        category_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "null".to_string())
    )
}

fn build_card_lookup(cards: &[CardListItem]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for card in cards {
        let key = card_lookup_key(&card.title, card.category_id);
        map.entry(key).or_insert_with(|| card.id.clone());
    }
    map
}

/// 获取分类列表
async fn fetch_categories(
    client: &reqwest::Client,
    base_url: &str,
    auth: &Arc<AuthState>,
) -> Result<Vec<CategoryItem>, TempleError> {
    let data = api_get(client, base_url, "/categories", auth).await?;
    let resp: CategoryListResponse =
        serde_json::from_value(data).map_err(|e| TempleError::new(ErrorCode::InternalError, e.to_string()))?;
    Ok(resp.categories)
}

fn build_category_path_map(categories: &[CategoryItem]) -> HashMap<u64, Vec<String>> {
    let mut items_by_id = HashMap::new();
    for item in categories {
        items_by_id.insert(item.id, item.clone());
    }

    let mut cache: HashMap<u64, Vec<String>> = HashMap::new();
    let ids: Vec<u64> = items_by_id.keys().copied().collect();
    for id in ids {
        let _ = resolve_category_path(id, &items_by_id, &mut cache);
    }

    cache
}

fn resolve_category_path(
    category_id: u64,
    items_by_id: &HashMap<u64, CategoryItem>,
    cache: &mut HashMap<u64, Vec<String>>,
) -> Vec<String> {
    if let Some(path) = cache.get(&category_id) {
        return path.clone();
    }

    let Some(item) = items_by_id.get(&category_id) else {
        return vec![];
    };

    let mut path = match item.parent_id {
        Some(parent_id) => resolve_category_path(parent_id, items_by_id, cache),
        None => vec![],
    };
    path.push(item.name.clone());
    cache.insert(category_id, path.clone());
    path
}

fn build_file_path(vault_dir: &Path, category_path: Option<&Vec<String>>, filename: &str) -> PathBuf {
    let mut path = vault_dir.to_path_buf();
    if let Some(segments) = category_path {
        for segment in segments {
            path.push(sanitize_dir_name(segment));
        }
    }
    path.push(filename);
    path
}

/// 分页获取全部卡片列表
async fn fetch_all_cards(
    client: &reqwest::Client,
    base_url: &str,
    auth: &Arc<AuthState>,
) -> Result<Vec<CardListItem>, TempleError> {
    let mut all = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let path = match &cursor {
            Some(c) => format!("/cards?limit=200&cursor={}", c),
            None => "/cards?limit=200".to_string(),
        };
        let data = api_get(client, base_url, &path, auth).await?;
        let resp: CardListResponse = serde_json::from_value(data)
            .map_err(|e| TempleError::new(ErrorCode::InternalError, e.to_string()))?;
        let count = resp.data.len();
        all.extend(resp.data);
        cursor = resp.next_cursor;
        if cursor.is_none() || count == 0 {
            break;
        }
    }

    Ok(all)
}

/// 获取单张卡片完整内容
async fn fetch_card_detail(
    client: &reqwest::Client,
    base_url: &str,
    card_id: &str,
    auth: &Arc<AuthState>,
) -> Result<CardDetail, TempleError> {
    let path = format!("/cards/{}", card_id);
    let data = api_get(client, base_url, &path, auth).await?;
    serde_json::from_value(data).map_err(|e| TempleError::new(ErrorCode::InternalError, e.to_string()))
}

/// 保存最后同步时间到 config
fn save_last_sync_time(app: &tauri::AppHandle) -> Result<(), TempleError> {
    let now = chrono_now_iso();
    // 通过 tauri_plugin_store 直接更新
    use tauri_plugin_store::StoreExt;
    let store = app
        .store("sysconfig.json")
        .map_err(|e| TempleError::new(ErrorCode::InternalError, e.to_string()))?;
    if let Some(mut val) = store.get("config") {
        if let Some(obj) = val.as_object_mut() {
            obj.insert("last_cloud_sync_at".to_string(), serde_json::Value::String(now));
            store.set("config", val);
            let _ = store.save();
        }
    }
    Ok(())
}

fn chrono_now_iso() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // 简易 UTC ISO 格式
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let (y, m, d) = civil_from_days(days);
    let h = time_of_day / 3600;
    let min = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, h, min, s)
}

/// UNIX 天数 → 年月日
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let z = days + 719468;
    let era = z / 146097;
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

// ============================================================================
// 本地 → 云端
// ============================================================================

/// 处理本地文件变更，上传到云端
///
/// 从 MarkdownWatcher 获取 pending 变更事件，逐个处理：
/// - created/modified → 读取文件内容，匹配已有卡片则更新，否则创建
/// - removed → 仅记录，不自动删除云端卡片
#[tauri::command]
pub async fn process_local_changes(
    app: tauri::AppHandle,
    client: State<'_, AppHttpClient>,
    auth: State<'_, Arc<AuthState>>,
    state: State<'_, std::sync::Mutex<crate::AppState>>,
) -> Result<LocalSyncResult, TempleError> {
    // 1. 从 watcher 获取变更
    let events = {
        let app_state = state.lock().map_err(|e| {
            TempleError::new(ErrorCode::CacheLockFailed, e.to_string())
        })?;
        match &app_state.watcher {
            Some(watcher) => watcher.poll_changes(),
            None => return Ok(LocalSyncResult {
                uploaded: 0,
                created: 0,
                errors: vec!["Watcher 未启动".to_string()],
            }),
        }
    };

    if events.is_empty() {
        return Ok(LocalSyncResult {
            uploaded: 0,
            created: 0,
            errors: vec![],
        });
    }

    let base_url = client.get_base_url();
    let http = &client.client;
    let cfg = config::get_config(&app)
        .await
        .map_err(|e| TempleError::new(ErrorCode::InternalError, e))?;
    let vault_path = cfg.vault_path.unwrap_or_default();

    // 构建分类列表 + 卡片查重索引（避免每文件一次远程查找）
    let lookup_start = Instant::now();
    let mut categories = fetch_categories(http, &base_url, &auth).await.unwrap_or_default();
    let all_cards = fetch_all_cards(http, &base_url, &auth).await.unwrap_or_default();
    let mut card_lookup = build_card_lookup(&all_cards);
    log::info!(
        "[VaultSync] built local lookup: categories={}, cards={}, cost={} ms",
        categories.len(),
        all_cards.len(),
        lookup_start.elapsed().as_millis()
    );

    let mut uploaded: u32 = 0;
    let mut created: u32 = 0;
    let mut errors: Vec<String> = Vec::new();

    for event in &events {
        let path = PathBuf::from(&event.path);

        match event.kind.as_str() {
            "created" | "modified" => {
                match sync_markdown_file_to_cloud(
                    &path,
                    &vault_path,
                    http,
                    &base_url,
                    &auth,
                    &mut categories,
                    &mut card_lookup,
                    "auto",
                    None,
                )
                .await
                {
                    Ok((up, cr)) => {
                        uploaded += up;
                        created += cr;
                    }
                    Err(e) => errors.push(e),
                }
            }
            "removed" => {
                // 不自动删除云端卡片，仅记录
                log::info!("文件已删除（不自动删除云端）: {}", event.path);
            }
            _ => {}
        }
    }

    Ok(LocalSyncResult {
        uploaded,
        created,
        errors,
    })
}

/// 全量导入：将本地 Vault 中全部 .md 文件导入当前云端
#[tauri::command]
pub async fn import_local_vault_to_cloud(
    app: tauri::AppHandle,
    client: State<'_, AppHttpClient>,
    auth: State<'_, Arc<AuthState>>,
    category_mode: Option<String>,
    selected_category_id: Option<u64>,
) -> Result<LocalSyncResult, TempleError> {
    let cfg = config::get_config(&app)
        .await
        .map_err(|e| TempleError::new(ErrorCode::InternalError, e))?;
    let vault_path = cfg
        .vault_path
        .ok_or_else(|| TempleError::new(ErrorCode::DirectoryNotFound, "未配置知识库目录".to_string()))?;

    let vault_dir = PathBuf::from(&vault_path);
    if !vault_dir.exists() {
        return Err(TempleError::new(
            ErrorCode::DirectoryNotFound,
            format!("知识库目录不存在: {}", vault_dir.display()),
        ));
    }

    let mut md_files = Vec::new();
    collect_markdown_files(&vault_dir, &mut md_files)?;

    let base_url = client.get_base_url();
    let http = &client.client;
    let lookup_start = Instant::now();
    let mut categories = fetch_categories(http, &base_url, &auth).await.unwrap_or_default();
    let all_cards = fetch_all_cards(http, &base_url, &auth).await.unwrap_or_default();
    let mut card_lookup = build_card_lookup(&all_cards);
    log::info!(
        "[VaultSync] built import lookup: categories={}, cards={}, cost={} ms",
        categories.len(),
        all_cards.len(),
        lookup_start.elapsed().as_millis()
    );

    let mut uploaded: u32 = 0;
    let mut created: u32 = 0;
    let mut errors: Vec<String> = Vec::new();
    let mode = category_mode.unwrap_or_else(|| "auto".to_string());

    for path in md_files {
        match sync_markdown_file_to_cloud(
            &path,
            &vault_path,
            http,
            &base_url,
            &auth,
            &mut categories,
            &mut card_lookup,
            &mode,
            selected_category_id,
        )
        .await
        {
            Ok((up, cr)) => {
                uploaded += up;
                created += cr;
            }
            Err(e) => errors.push(e),
        }
    }

    Ok(LocalSyncResult {
        uploaded,
        created,
        errors,
    })
}

#[allow(clippy::too_many_arguments)]
async fn sync_markdown_file_to_cloud(
    path: &Path,
    vault_path: &str,
    http: &reqwest::Client,
    base_url: &str,
    auth: &Arc<AuthState>,
    categories: &mut Vec<CategoryItem>,
    card_lookup: &mut HashMap<String, String>,
    category_mode: &str,
    selected_category_id: Option<u64>,
) -> Result<(u32, u32), String> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("读取失败 '{}': {}", path.display(), e))?;

    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();

    let category_id = match category_mode {
        "none" => None,
        "selected" => selected_category_id,
        _ => resolve_or_create_category_chain(path, vault_path, http, base_url, auth, categories).await?,
    };
    let key = card_lookup_key(&title, category_id);

    let ast = parse_markdown(&content)
        .map_err(|e| format!("AST解析失败 '{}': {}", path.display(), e))?;
    let ast_data = serde_json::to_value(&ast).unwrap_or(serde_json::json!({}));

    match card_lookup.get(&key).cloned() {
        Some(card_id) => {
            let mut body = serde_json::json!({
                "title": title,
                "raw_md": content,
                "excerpt": "",
                "ast_data": ast_data,
                "toc_data": serde_json::json!({}),
            });
            if let Some(cid) = category_id {
                body["category_id"] = serde_json::json!(cid);
            }
            api_put(http, base_url, &format!("/cards/{}", card_id), &body, auth)
                .await
                .map_err(|e| format!("更新失败 '{}': {}", path.display(), e))?;
            Ok((1, 0))
        }
        None => {
            let mut body = serde_json::json!({
                "title": title,
                "raw_md": content,
                "excerpt": "",
                "ast_data": ast_data,
            });
            if let Some(cid) = category_id {
                body["category_id"] = serde_json::json!(cid);
            }
            let created = api_post(http, base_url, "/cards", &body, auth)
                .await
                .map_err(|e| format!("创建失败 '{}': {}", path.display(), e))?;

            if let Some(card_id) = created.get("card_id").and_then(|v| v.as_str()) {
                card_lookup.insert(key, card_id.to_string());
            }

            Ok((0, 1))
        }
    }
}

/// 根据文件路径提取目录分类段
fn extract_category_segments(file_path: &Path, vault_path: &str) -> Vec<String> {
    let vault = PathBuf::from(vault_path);
    let Ok(relative) = file_path.strip_prefix(&vault) else {
        return vec![];
    };

    let mut segments = Vec::new();
    for component in relative.iter() {
        let part = component.to_string_lossy();
        if part.ends_with(".md") {
            break;
        }
        if !part.trim().is_empty() {
            segments.push(part.to_string());
        }
    }

    segments
}

fn find_category_by_name_and_parent(
    categories: &[CategoryItem],
    name: &str,
    parent_id: Option<u64>,
) -> Option<CategoryItem> {
    categories
        .iter()
        .find(|category| {
            category.name.trim().to_lowercase() == name.trim().to_lowercase()
                && category.parent_id == parent_id
        })
        .cloned()
}

async fn resolve_or_create_category_chain(
    file_path: &Path,
    vault_path: &str,
    http: &reqwest::Client,
    base_url: &str,
    auth: &Arc<AuthState>,
    categories: &mut Vec<CategoryItem>,
) -> Result<Option<u64>, String> {
    let segments = extract_category_segments(file_path, vault_path);
    if segments.is_empty() {
        return Ok(None);
    }

    let mut parent_id: Option<u64> = None;
    for segment in segments {
        let existing = find_category_by_name_and_parent(categories, &segment, parent_id);
        if let Some(category) = existing {
            parent_id = Some(category.id);
            continue;
        }

        let mut body = serde_json::json!({
            "name": segment,
            "description": "",
        });
        if let Some(pid) = parent_id {
            body["parent_id"] = serde_json::json!(pid);
        }

        let created = api_post(http, base_url, "/categories", &body, auth)
            .await
            .map_err(|e| format!("创建分类失败 '{}': {}", file_path.display(), e))?;

        let category_id = created
            .get("category")
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_u64())
            .ok_or_else(|| format!("创建分类失败 '{}': missing id", file_path.display()))?;

        categories.push(CategoryItem {
            id: category_id,
            name: segment,
            parent_id,
        });
        parent_id = Some(category_id);
    }

    Ok(parent_id)
}

