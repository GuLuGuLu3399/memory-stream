//! # 草稿后台同步 Worker
//!
//! 独立的 tokio 异步循环，每 5 秒轮询本地 SQLite 中 `sync_status = 'pending'` 的草稿，
//! 将其 AST 数据推送到 Go 后端，成功后通过乐观锁标记为 `'synced'`。
//!
//! 设计要点：
//! - 独立 `reqwest::Client`，不依赖 `AppHttpClient` 的 Arc 改造
//! - `mark_synced_if_unchanged` 乐观锁防止旧数据覆盖新数据后误标记
//! - 网络失败静默重试（下一个 cycle 自动处理）
//! - 无 pending 草稿时 skip，零开销

use crate::auth::AuthState;
use ms_local_draft::DraftDb;
use std::sync::Arc;
use std::time::Duration;

/// 后台同步循环入口
///
/// 在 Tauri `setup()` 中通过 `tokio::spawn` 启动，持续运行直到进程退出。
pub async fn run_sync_loop(
    db: Arc<DraftDb>,
    base_url: String,
    auth: Arc<AuthState>,
) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|e| {
            log::error!("[draft-sync] failed to build HTTP client: {}", e);
            panic!("draft sync client build failed");
        });

    let mut interval = tokio::time::interval(Duration::from_secs(5));

    log::info!(
        "[draft-sync] worker started, base_url={}",
        if base_url.is_empty() {
            "(empty)"
        } else {
            &base_url
        }
    );

    loop {
        interval.tick().await;

        // 无 API 地址时跳过（未配置后端）
        let base = base_url.trim_end_matches('/');
        if base.is_empty() {
            continue;
        }

        // 1. 从 SQLite 捞出所有 pending 草稿
        let unsynced = match db.list_unsynced().await {
            Ok(drafts) => drafts,
            Err(e) => {
                log::warn!("[draft-sync] list_unsynced failed: {:?}", e);
                continue;
            }
        };

        if unsynced.is_empty() {
            continue;
        }

        log::debug!("[draft-sync] found {} pending drafts", unsynced.len());

        for draft in unsynced {
            // 跳过没有 AST 数据的草稿（理论上不应出现）
            let Some(ast_data) = &draft.ast_data else {
                log::warn!(
                    "[draft-sync] skipping draft {} without AST data",
                    draft.card_id
                );
                continue;
            };

            // 2. 构建 PUT 请求体（与前端 saveCard 的 UpdateCardBody 对齐）
            let extracted_links: serde_json::Value = draft
                .extracted_links
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or(serde_json::json!([]));

            let toc_data: Option<serde_json::Value> = draft
                .toc_data
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok());

            let body = serde_json::json!({
                "title": draft.title,
                "raw_md": draft.raw_md,
                "ast_data": ast_data,
                "excerpt": draft.excerpt,
                "toc_data": toc_data,
                "extracted_links": extracted_links,
                "category_id": draft.category_id,
            });

            let url = format!("{}/cards/{}", base, draft.card_id);
            let saved_at = draft.updated_at;

            // 3. PUT /cards/{id} → Go 后端
            let mut req = client.put(&url).json(&body);

            if let Some(token) = auth.get_access_token() {
                req = req.header("Authorization", format!("Bearer {}", token));
            }

            match req.send().await {
                Ok(resp) if resp.status().is_success() => {
                    // 4. 乐观锁标记 — 只在 updated_at 未变时才标记为 synced
                    match db
                        .mark_synced_if_unchanged(&draft.card_id, saved_at)
                        .await
                    {
                        Ok(true) => {
                            log::info!("[draft-sync] synced: {}", draft.card_id);
                        }
                        Ok(false) => {
                            log::debug!(
                                "[draft-sync] draft {} was updated during sync, will retry next cycle",
                                draft.card_id
                            );
                        }
                        Err(e) => {
                            log::warn!(
                                "[draft-sync] mark_synced failed for {}: {:?}",
                                draft.card_id,
                                e
                            );
                        }
                    }
                }
                Ok(resp) => {
                    let status = resp.status();
                    // 404 = 卡片已被删除，清理本地草稿
                    if status == reqwest::StatusCode::NOT_FOUND {
                        log::info!(
                            "[draft-sync] card {} not found on server, deleting local draft",
                            draft.card_id
                        );
                        let _ = db.delete_draft(&draft.card_id).await;
                    } else {
                        log::warn!(
                            "[draft-sync] PUT {} failed: HTTP {}",
                            draft.card_id,
                            status
                        );
                    }
                }
                Err(e) => {
                    log::debug!(
                        "[draft-sync] network error for {}: {}",
                        draft.card_id,
                        e
                    );
                }
            }
        }
    }
}
