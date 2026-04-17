//! # API Gateway Module
//!
//! 统一的 HTTP 客户端层，充当 Tauri 桌面端与 Go 后端之间的唯一网络通道。
//! 所有 Vue 前端的 HTTP 请求必须通过此模块发出，前端不再直接访问网络。

use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tauri::State;

use crate::auth::AuthState;

/// Go 后端 API 基础 URL，编译时注入，默认指向本地开发服务器
pub const API_BASE_URL: &str = match option_env!("API_BASE_URL") {
    Some(url) => url,
    None => "http://localhost:8080/api/v1",
};

/// 全局 HTTP 客户端包装器
///
/// `base_url` 使用 `RwLock` 保护，支持运行时通过 Settings 热更新，
/// 无需重启应用即可切换后端地址。
pub struct AppHttpClient {
    pub client: Client,
    base_url: std::sync::RwLock<String>,
}

impl AppHttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("failed to build HTTP client"),
            base_url: std::sync::RwLock::new(API_BASE_URL.to_string()),
        }
    }

    /// 获取当前 API 基础 URL
    pub fn get_base_url(&self) -> String {
        match self.base_url.read() {
            Ok(guard) => guard.clone(),
            Err(_) => API_BASE_URL.to_string(),
        }
    }

    /// 更新 API 基础 URL（由 reload_sys_config 调用）
    pub fn set_base_url(&self, url: String) {
        if let Ok(mut guard) = self.base_url.write() {
            *guard = url;
        }
    }
}

#[tauri::command]
pub async fn api_request(
    client: State<'_, AppHttpClient>,
    auth: State<'_, Arc<AuthState>>,
    method: String,
    endpoint: String,
    body: Option<Value>,
) -> Result<Value, String> {
    let url = format!("{}{}", client.get_base_url(), endpoint);

    let request_builder = match method.to_uppercase().as_str() {
        "GET" => client.client.get(&url),
        "POST" => client.client.post(&url),
        "PUT" => client.client.put(&url),
        "PATCH" => client.client.patch(&url),
        "DELETE" => client.client.delete(&url),
        _ => return Err(format!("unsupported HTTP method: {}", method)),
    };

    let request_builder = if let Some(token) = auth.get_access_token() {
        request_builder.header("Authorization", format!("Bearer {}", token))
    } else {
        request_builder
    };

    let body_clone = body.clone();

    let request_builder = match body {
        Some(data) => request_builder.json(&data),
        None => request_builder,
    };

    let response = request_builder
        .send()
        .await
        .map_err(|e| format!("request failed: {}", e))?;

    // ── 401 自动刷新 ──
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        eprintln!("[API] got 401, attempting token refresh...");

        // 防止并发刷新（thundering herd）
        let should_refresh = auth.try_acquire_refresh_lock();

        if !should_refresh {
            // 其他线程正在刷新，等待其完成后再重试
            let _ = auth
                .wait_for_refresh_completion(std::time::Duration::from_secs(5))
                .await;
            return send_request(&client.client, &auth, &method, &url, body_clone).await;
        }

        let refreshed = crate::auth::do_refresh(&client.client, &auth, &client.get_base_url()).await;
        auth.release_refresh_lock();

        if refreshed {
            return send_request(&client.client, &auth, &method, &url, body_clone).await;
        } else {
            return Err("认证已过期，请重新登录".to_string());
        }
    }

    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status, error_body));
    }

    response
        .json::<Value>()
        .await
        .map_err(|e| format!("response parse failed: {}", e))
}

/// 构建+发送一次请求（带 Authorization header）
async fn send_request(
    client: &Client,
    auth: &AuthState,
    method: &str,
    url: &str,
    body: Option<Value>,
) -> Result<Value, String> {
    let mut builder = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        "DELETE" => client.delete(url),
        _ => return Err(format!("unsupported HTTP method: {}", method)),
    };

    if let Some(token) = auth.get_access_token() {
        builder = builder.header("Authorization", format!("Bearer {}", token));
    }

    let builder = match body {
        Some(data) => builder.json(&data),
        None => builder,
    };

    let response = builder
        .send()
        .await
        .map_err(|e| format!("request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status, error_body));
    }

    response
        .json::<Value>()
        .await
        .map_err(|e| format!("response parse failed: {}", e))
}
