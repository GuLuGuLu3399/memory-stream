//! # Auth Module — JWT 凭据管理
//!
//! 管理 Tauri 桌面端的 JWT access/refresh token。
//! 所有 HTTP 请求和 WebSocket 连接都从这里读取 token。
//! Token 持久化到磁盘文件，应用重启后自动恢复。
//!
//! ## Proactive Refresh
//! 启动后台定时器，在 access token 过期前 5 分钟自动刷新。

use crate::api::AppHttpClient;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, State};
use tokio::sync::Notify;

/// Access token 默认有效期：2 小时（与 Go 后端一致）
const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(2 * 60 * 60);
/// Proactive refresh 提前时间：5 分钟
const REFRESH_MARGIN: Duration = Duration::from_secs(5 * 60);
/// 刷新失败重试间隔
const RETRY_INTERVAL: Duration = Duration::from_secs(30);
/// 无 token 时检查间隔
const NO_TOKEN_CHECK_INTERVAL: Duration = Duration::from_secs(30);
/// 最大连续刷新失败次数（超过后停止主动刷新，等 401 被动触发）
const MAX_CONSECUTIVE_FAILURES: u32 = 5;

/// 持久化到磁盘的 token 数据结构
#[derive(Serialize, Deserialize, Default)]
struct PersistedTokens {
    access_token: Option<String>,
    refresh_token: Option<String>,
}

/// 全局认证状态，存储 JWT 凭据及时间信息
pub struct AuthState {
    access_token: RwLock<Option<String>>,
    refresh_token: RwLock<Option<String>>,
    /// Token 创建时间（用于计算剩余有效期）
    created_at: RwLock<Option<Instant>>,
    /// Access token 有效期
    expires_in: Duration,
    /// 刷新操作进行中的标记（防止 thundering herd）
    refreshing: AtomicBool,
    /// 刷新完成通知（让并发请求等待“实际完成”而不是固定 sleep）
    refresh_notify: Notify,
    /// 连续刷新失败计数
    consecutive_failures: AtomicU32,
    /// Token 持久化文件路径
    persist_path: PathBuf,
}

impl AuthState {
    /// 创建新的 AuthState，并从磁盘恢复已保存的 token
    pub fn new(persist_dir: PathBuf) -> Self {
        let persist_path = persist_dir.join("tokens.json");
        let state = Self {
            access_token: RwLock::new(None),
            refresh_token: RwLock::new(None),
            created_at: RwLock::new(None),
            expires_in: ACCESS_TOKEN_TTL,
            refreshing: AtomicBool::new(false),
            refresh_notify: Notify::new(),
            consecutive_failures: AtomicU32::new(0),
            persist_path,
        };
        state.load_from_disk();
        state
    }

    /// 获取当前 access token（使用读锁）
    pub fn get_access_token(&self) -> Option<String> {
        self.access_token.read().ok().and_then(|t| t.clone())
    }

    /// 获取当前 refresh token（使用读锁）
    pub fn get_refresh_token(&self) -> Option<String> {
        self.refresh_token.read().ok().and_then(|t| t.clone())
    }

    /// 设置新的 token 对（登录或刷新成功后调用）
    fn set_tokens(&self, access_token: String, refresh_token: String) {
        if let Ok(mut t) = self.access_token.write() {
            *t = Some(access_token);
        }
        if let Ok(mut t) = self.refresh_token.write() {
            *t = Some(refresh_token);
        }
        if let Ok(mut t) = self.created_at.write() {
            *t = Some(Instant::now());
        }
        // Reset failure counter on successful token set
        self.consecutive_failures.store(0, Ordering::Release);
        self.save_to_disk();
    }

    /// 清除所有 token（登出时调用）
    #[allow(dead_code)]
    pub fn clear(&self) {
        if let Ok(mut t) = self.access_token.write() {
            *t = None;
        }
        if let Ok(mut t) = self.refresh_token.write() {
            *t = None;
        }
        if let Ok(mut t) = self.created_at.write() {
            *t = None;
        }
        self.save_to_disk();
    }

    /// 检查 access token 是否即将过期（在 REFRESH_MARGIN 内）
    #[allow(dead_code)]
    fn should_refresh_soon(&self) -> bool {
        let created = match self.created_at.read().ok() {
            Some(guard) => match *guard {
                Some(c) => c,
                None => return true,
            },
            None => return true,
        };
        let elapsed = created.elapsed();
        elapsed >= self.expires_in.saturating_sub(REFRESH_MARGIN)
    }

    /// 检查 access token 是否已完全过期
    #[allow(dead_code)]
    fn is_expired(&self) -> bool {
        let created = match self.created_at.read().ok() {
            Some(guard) => match *guard {
                Some(c) => c,
                None => return true,
            },
            None => return true,
        };
        created.elapsed() >= self.expires_in
    }

    /// 计算到下次需要刷新的剩余时间
    fn time_until_refresh(&self) -> Duration {
        let created = match self.created_at.read().ok() {
            Some(guard) => match *guard {
                Some(c) => c,
                None => return NO_TOKEN_CHECK_INTERVAL,
            },
            None => return NO_TOKEN_CHECK_INTERVAL,
        };
        let has_token = self.access_token.read().ok().is_some_and(|t| t.is_some());
        if !has_token {
            return NO_TOKEN_CHECK_INTERVAL;
        }
        let elapsed = created.elapsed();
        let refresh_at = self.expires_in.saturating_sub(REFRESH_MARGIN);
        if elapsed >= refresh_at {
            Duration::ZERO // 已经过了刷新时间
        } else {
            refresh_at - elapsed
        }
    }

    /// 尝试获取刷新锁（防 thundering herd）
    /// 返回 true 表示获得锁（应该执行刷新），false 表示其他线程正在刷新
    pub fn try_acquire_refresh_lock(&self) -> bool {
        !self.refreshing.swap(true, Ordering::AcqRel)
    }

    /// 释放刷新锁
    pub fn release_refresh_lock(&self) {
        self.refreshing.store(false, Ordering::Release);
        self.refresh_notify.notify_waiters();
    }

    /// 等待当前刷新操作完成（若当前无刷新，立即返回）
    pub async fn wait_for_refresh_completion(&self, timeout: Duration) -> bool {
        if !self.refreshing.load(Ordering::Acquire) {
            return true;
        }

        let notified = self.refresh_notify.notified();

        // 处理在 subscribed 之后立刻完成刷新的竞态
        if !self.refreshing.load(Ordering::Acquire) {
            return true;
        }

        tokio::time::timeout(timeout, notified).await.is_ok()
    }

    /// 记录一次刷新失败
    fn record_refresh_failure(&self) {
        self.consecutive_failures.fetch_add(1, Ordering::Release);
    }

    /// 是否超过最大连续失败次数
    fn too_many_failures(&self) -> bool {
        self.consecutive_failures.load(Ordering::Acquire) >= MAX_CONSECUTIVE_FAILURES
    }

    /// 持久化 token 到磁盘
    fn save_to_disk(&self) {
        let data = PersistedTokens {
            access_token: self.access_token.read().ok().and_then(|t| t.clone()),
            refresh_token: self.refresh_token.read().ok().and_then(|t| t.clone()),
        };

        // 两个 token 都没有时删除文件
        if data.access_token.is_none() && data.refresh_token.is_none() {
            let _ = std::fs::remove_file(&self.persist_path);
            return;
        }

        match serde_json::to_string_pretty(&data) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&self.persist_path, json) {
                    eprintln!("[Auth] failed to persist tokens: {}", e);
                }
            }
            Err(e) => eprintln!("[Auth] failed to serialize tokens: {}", e),
        }
    }

    /// 从磁盘加载已保存的 token
    fn load_from_disk(&self) {
        let data = match std::fs::read_to_string(&self.persist_path) {
            Ok(d) => d,
            Err(_) => return, // 文件不存在，首次启动正常
        };

        let tokens: PersistedTokens = match serde_json::from_str(&data) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[Auth] failed to parse persisted tokens: {}", e);
                let _ = std::fs::remove_file(&self.persist_path);
                return;
            }
        };

        if tokens.access_token.is_some() || tokens.refresh_token.is_some() {
            if let Ok(mut t) = self.access_token.write() {
                *t = tokens.access_token;
            }
            if let Ok(mut t) = self.refresh_token.write() {
                *t = tokens.refresh_token;
            }
            // 保守策略：假设恢复的 token 是新获取的
            // 如果实际已过期，401/WS rejected 会触发被动刷新
            if let Ok(mut t) = self.created_at.write() {
                *t = Some(Instant::now());
            }
            eprintln!("[Auth] restored tokens from disk");
        }
    }
}

// ============================================================================
// Proactive Refresh — 后台定时器
// ============================================================================

/// 启动 proactive token 刷新定时器
///
/// 在 access token 过期前 REFRESH_MARGIN（5分钟）自动刷新。
/// 使用 `tauri::async_runtime::spawn` 以兼容 Tauri 的 sync setup() 闭包。
pub fn spawn_proactive_refresh(auth: Arc<AuthState>, http_client: reqwest::Client, app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            // 计算下次刷新时间
            let sleep_duration = auth.time_until_refresh();
            tokio::time::sleep(sleep_duration).await;

            // 检查是否超过失败上限
            if auth.too_many_failures() {
                // 停止主动刷新，等 401 被动触发
                tokio::time::sleep(Duration::from_secs(5 * 60)).await;
                continue;
            }

            // 获取刷新锁
            if !auth.try_acquire_refresh_lock() {
                // 其他线程正在刷新，等一会
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }

            let api_base_url = match crate::config::get_config(&app).await {
                Ok(cfg) if !cfg.api_base_url.is_empty() => cfg.api_base_url,
                _ => crate::api::API_BASE_URL.to_string(),
            };

            let success = do_refresh(&http_client, &auth, &api_base_url).await;

            auth.release_refresh_lock();

            if !success {
                auth.record_refresh_failure();
                eprintln!("[Auth] proactive refresh failed, will retry in {}s", RETRY_INTERVAL.as_secs());
                tokio::time::sleep(RETRY_INTERVAL).await;
            }
        }
    });
}

// ============================================================================
// Token 刷新（内部使用）
// ============================================================================

/// Go 后端 refresh 响应结构
#[derive(Deserialize, Debug)]
struct RefreshResponse {
    access_token: String,
    refresh_token: String,
}

/// 执行 token 刷新（调用方负责加锁）
pub(crate) async fn do_refresh(client: &reqwest::Client, auth: &AuthState, api_base_url: &str) -> bool {
    let refresh_token = match auth.get_refresh_token() {
        Some(t) => t,
        None => {
            eprintln!("[Auth] no refresh token available");
            return false;
        }
    };

    let url = format!("{}/auth/refresh", api_base_url.trim_end_matches('/'));

    let resp = match client
        .post(&url)
        .json(&serde_json::json!({ "refresh_token": refresh_token }))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[Auth] refresh request failed: {}", e);
            return false;
        }
    };

    if !resp.status().is_success() {
        eprintln!("[Auth] refresh failed: {}", resp.status());
        return false;
    }

    let refresh_resp: RefreshResponse = match resp.json().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[Auth] parse refresh response failed: {}", e);
            return false;
        }
    };

    auth.set_tokens(refresh_resp.access_token, refresh_resp.refresh_token);
    eprintln!("[Auth] token refreshed successfully");
    true
}

/// 使用 refresh_token 换取新的 access_token（被 401/WS 断连调用）
///
/// 带刷新锁保护：同一时间只有一个刷新请求在进行。
/// 返回 true 表示刷新成功（或刚被其他请求刷新完），false 表示需要重新登录。
#[allow(dead_code)]
pub async fn try_refresh_token(client: &reqwest::Client, auth: &Arc<AuthState>) -> bool {
    // 尝试获取刷新锁
    if !auth.try_acquire_refresh_lock() {
        // 其他线程正在刷新 — 等待刷新完成，避免轮询忙等
        if !auth
            .wait_for_refresh_completion(Duration::from_secs(5))
            .await
        {
            eprintln!("[Auth] timed out waiting for concurrent refresh");
        }
        return auth.get_access_token().is_some();
    }

    let success = do_refresh(client, auth, crate::api::API_BASE_URL).await;

    auth.release_refresh_lock();

    if !success {
        auth.record_refresh_failure();
    }

    success
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Go 后端登录响应结构
#[derive(Deserialize, Debug)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    user: UserResponse,
}

#[derive(Deserialize, Debug)]
struct UserResponse {
    id: String,
    username: String,
    role: String,
}

/// 前端拿到的登录结果
#[derive(Serialize, Debug, ts_rs::TS)]
#[ts(export_to = ".")]
pub struct LoginResult {
    pub user_id: String,
    pub username: String,
    pub role: String,
}

/// 使用用户名密码登录 Go 后端
#[tauri::command]
pub async fn login(
    client: State<'_, AppHttpClient>,
    auth: State<'_, Arc<AuthState>>,
    username: String,
    password: String,
) -> Result<LoginResult, String> {
    let url = format!("{}/auth/login", client.get_base_url().trim_end_matches('/'));

    let resp = client
        .client
        .post(&url)
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .await
        .map_err(|e| format!("login request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("login failed: {} - {}", status, body));
    }

    let login_resp: LoginResponse = resp
        .json()
        .await
        .map_err(|e| format!("parse login response failed: {}", e))?;

    let result = LoginResult {
        user_id: login_resp.user.id,
        username: login_resp.user.username,
        role: login_resp.user.role,
    };

    auth.set_tokens(login_resp.access_token, login_resp.refresh_token);

    eprintln!(
        "[Auth] logged in: user={} role={}",
        result.username, result.role
    );

    Ok(result)
}

/// 创世接口 — 首次部署时创建 admin 账号
#[tauri::command]
pub async fn genesis(
    client: State<'_, AppHttpClient>,
    auth: State<'_, Arc<AuthState>>,
    username: String,
    password: String,
) -> Result<LoginResult, String> {
    let url = format!("{}/auth/init", client.get_base_url().trim_end_matches('/'));

    let resp = client
        .client
        .post(&url)
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .await
        .map_err(|e| format!("genesis request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("genesis failed: {} - {}", status, body));
    }

    let login_resp: LoginResponse = resp
        .json()
        .await
        .map_err(|e| format!("parse genesis response failed: {}", e))?;

    let result = LoginResult {
        user_id: login_resp.user.id,
        username: login_resp.user.username,
        role: login_resp.user.role,
    };

    auth.set_tokens(login_resp.access_token, login_resp.refresh_token);

    eprintln!(
        "[Auth] genesis complete: user={} role={}",
        result.username, result.role
    );

    Ok(result)
}

/// 手动注入 token（用于前端已有 token 的场景）
#[tauri::command]
pub fn set_auth_token(
    auth: State<'_, Arc<AuthState>>,
    access_token: String,
    refresh_token: String,
) -> Result<(), String> {
    auth.set_tokens(access_token, refresh_token);
    Ok(())
}

/// 获取当前认证状态（前端可轮询检查）
#[tauri::command]
pub fn get_auth_status(auth: State<'_, Arc<AuthState>>) -> Result<serde_json::Value, String> {
    let has_token = auth.get_access_token().is_some();
    Ok(serde_json::json!({ "authenticated": has_token }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let dir = tempfile::tempdir().unwrap();
        let state = AuthState::new(dir.path().to_path_buf());
        assert!(state.get_access_token().is_none());
        assert!(state.get_refresh_token().is_none());
    }

    #[test]
    fn test_set_and_get_tokens() {
        let dir = tempfile::tempdir().unwrap();
        let state = AuthState::new(dir.path().to_path_buf());
        state.set_tokens("access123".to_string(), "refresh456".to_string());
        assert_eq!(state.get_access_token(), Some("access123".to_string()));
        assert_eq!(state.get_refresh_token(), Some("refresh456".to_string()));
    }

    #[test]
    fn test_clear() {
        let dir = tempfile::tempdir().unwrap();
        let state = AuthState::new(dir.path().to_path_buf());
        state.set_tokens("access".to_string(), "refresh".to_string());
        state.clear();
        assert!(state.get_access_token().is_none());
        assert!(state.get_refresh_token().is_none());
    }

    #[test]
    fn test_clear_removes_disk_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let state = AuthState::new(path.clone());
        state.set_tokens("access".to_string(), "refresh".to_string());
        state.clear();
        let state2 = AuthState::new(path);
        assert!(state2.get_access_token().is_none());
    }

    #[test]
    fn test_persist_and_restore() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let state1 = AuthState::new(path.clone());
        state1.set_tokens("saved_access".to_string(), "saved_refresh".to_string());
        let state2 = AuthState::new(path);
        assert_eq!(state2.get_access_token(), Some("saved_access".to_string()));
        assert_eq!(state2.get_refresh_token(), Some("saved_refresh".to_string()));
    }

    #[test]
    fn test_should_refresh_soon_no_token() {
        let dir = tempfile::tempdir().unwrap();
        let state = AuthState::new(dir.path().to_path_buf());
        assert!(state.should_refresh_soon());
    }

    #[test]
    fn test_should_refresh_soon_fresh_token() {
        let dir = tempfile::tempdir().unwrap();
        let state = AuthState::new(dir.path().to_path_buf());
        state.set_tokens("access".to_string(), "refresh".to_string());
        assert!(!state.should_refresh_soon());
    }

    #[test]
    fn test_is_expired_no_token() {
        let dir = tempfile::tempdir().unwrap();
        let state = AuthState::new(dir.path().to_path_buf());
        assert!(state.is_expired());
    }

    #[test]
    fn test_is_expired_fresh_token() {
        let dir = tempfile::tempdir().unwrap();
        let state = AuthState::new(dir.path().to_path_buf());
        state.set_tokens("access".to_string(), "refresh".to_string());
        assert!(!state.is_expired());
    }

    #[test]
    fn test_time_until_refresh_no_token() {
        let dir = tempfile::tempdir().unwrap();
        let state = AuthState::new(dir.path().to_path_buf());
        assert_eq!(state.time_until_refresh(), NO_TOKEN_CHECK_INTERVAL);
    }

    #[test]
    fn test_time_until_refresh_fresh_token() {
        let dir = tempfile::tempdir().unwrap();
        let state = AuthState::new(dir.path().to_path_buf());
        state.set_tokens("access".to_string(), "refresh".to_string());
        let remaining = state.time_until_refresh();
        assert!(remaining > Duration::from_secs(6000));
        assert!(remaining < Duration::from_secs(7200));
    }

    #[test]
    fn test_refresh_lock() {
        let dir = tempfile::tempdir().unwrap();
        let state = AuthState::new(dir.path().to_path_buf());
        assert!(state.try_acquire_refresh_lock());
        assert!(!state.try_acquire_refresh_lock());
        state.release_refresh_lock();
        assert!(state.try_acquire_refresh_lock());
        state.release_refresh_lock();
    }

    #[test]
    fn test_consecutive_failures() {
        let dir = tempfile::tempdir().unwrap();
        let state = AuthState::new(dir.path().to_path_buf());
        assert!(!state.too_many_failures());
        for _ in 0..4 {
            state.record_refresh_failure();
        }
        assert!(!state.too_many_failures());
        state.record_refresh_failure();
        assert!(state.too_many_failures());
    }

    #[test]
    fn test_set_tokens_resets_failures() {
        let dir = tempfile::tempdir().unwrap();
        let state = AuthState::new(dir.path().to_path_buf());
        for _ in 0..5 {
            state.record_refresh_failure();
        }
        assert!(state.too_many_failures());
        state.set_tokens("access".to_string(), "refresh".to_string());
        assert!(!state.too_many_failures());
    }
}
