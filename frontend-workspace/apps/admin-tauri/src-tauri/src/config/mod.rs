pub mod keyring_wrapper;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = ".")]
pub struct SysConfig {
    pub api_base_url: String,
    pub ws_url: String,
    pub s3_endpoint: String,
    pub s3_region: String,
    pub s3_bucket: String,
    #[serde(skip_serializing)]
    #[ts(skip)]
    pub s3_access_key: Option<String>,
    #[serde(skip_serializing)]
    #[ts(skip)]
    pub s3_secret_key: Option<String>,
    pub s3_public_url_base: Option<String>,
    #[serde(default)]
    pub s3_use_path_style: bool,
    /// 本地知识库 (Vault) 根目录路径
    #[serde(default)]
    pub vault_path: Option<String>,
}

impl Default for SysConfig {
    fn default() -> Self {
        Self {
            api_base_url: "http://localhost:8080/api/v1".to_string(),
            ws_url: "ws://localhost:8080/api/v1/ws".to_string(),
            s3_endpoint: String::new(),
            s3_region: "us-east-1".to_string(),
            s3_bucket: String::new(),
            s3_access_key: None,
            s3_secret_key: None,
            s3_public_url_base: None,
            s3_use_path_style: false,
            vault_path: None,
        }
    }
}

const STORE_NAME: &str = "sysconfig.json";

fn is_loopback_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("localhost") || lower.contains("127.0.0.1") || lower.contains("0.0.0.0")
}

fn derive_ws_url(api_base_url: &str) -> String {
    let trimmed = api_base_url.trim().trim_end_matches('/');
    if trimmed.starts_with("https://") {
        format!("wss://{}", trimmed.trim_start_matches("https://")) + "/ws"
    } else if trimmed.starts_with("http://") {
        format!("ws://{}", trimmed.trim_start_matches("http://")) + "/ws"
    } else if trimmed.starts_with("wss://") || trimmed.starts_with("ws://") {
        format!("{}/ws", trimmed)
    } else if trimmed.is_empty() {
        SysConfig::default().ws_url
    } else {
        format!("ws://{}/ws", trimmed)
    }
}

fn normalize_ws_url(config: &mut SysConfig) {
    let api_is_remote = !is_loopback_url(&config.api_base_url);
    let ws_is_default_local = config.ws_url.is_empty()
        || config.ws_url == SysConfig::default().ws_url
        || is_loopback_url(&config.ws_url);

    if api_is_remote && ws_is_default_local {
        config.ws_url = derive_ws_url(&config.api_base_url);
    }
}

pub async fn get_config(app: &AppHandle) -> Result<SysConfig, String> {
    use tauri_plugin_store::StoreExt;
    let store = app.store(STORE_NAME).map_err(|e| {
        log::error!("[config] Failed to get store: {}", e);
        e.to_string()
    })?;
    
    let config: Option<SysConfig> = store
        .get("config")
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    let mut config = match config {
        Some(config) => config,
        None => {
            let default_config = SysConfig::default();
            let value = serde_json::to_value(&default_config).map_err(|e| {
                log::error!("[config] Failed to serialize default config: {}", e);
                e.to_string()
            })?;

            store.set("config", value);
            store.save().map_err(|e| {
                log::error!("[config] Failed to initialize default config in store: {}", e);
                e.to_string()
            })?;

            log::warn!("[config] No config found in store, initialized defaults into {}", STORE_NAME);
            default_config
        }
    };

    normalize_ws_url(&mut config);
    
    log::debug!("[config] Loaded config: api_base_url={}, s3_endpoint={}, vault_path={:?}",
        config.api_base_url, config.s3_endpoint, config.vault_path);
    Ok(config)
}

pub async fn save_config(app: &AppHandle, config: &SysConfig) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    
    let store = app.store(STORE_NAME).map_err(|e| {
        log::error!("[config] Failed to get store for saving: {}", e);
        e.to_string()
    })?;
    
    let mut normalized = config.clone();
    normalize_ws_url(&mut normalized);

    let value = serde_json::to_value(&normalized).map_err(|e| {
        log::error!("[config] Failed to serialize config: {}", e);
        e.to_string()
    })?;
    
    store.set("config", value.clone());
    log::info!("[config] Config set in store, calling save()...");
    
    store.save().map_err(|e| {
        log::error!("[config] Failed to save config to file: {}", e);
        e.to_string()
    })?;
    
    log::info!("[config] ✓ Config saved successfully: api_base_url={}, s3_endpoint={}, vault_path={:?}",
        normalized.api_base_url, normalized.s3_endpoint, normalized.vault_path);
    
    // 验证保存是否成功：立即读回配置
    let store_verify = app.store(STORE_NAME).map_err(|e| {
        log::warn!("[config] Failed to verify save (store access): {}", e);
        e.to_string()
    })?;
    
    match store_verify.get("config") {
        Some(stored_value) => {
            if stored_value == value {
                log::debug!("[config] ✓ Verification: Saved config matches in-memory value");
            } else {
                log::warn!("[config] ⚠ Verification: Saved config DIFFERS from in-memory value!");
            }
        }
        None => {
            log::warn!("[config] ⚠ Verification: Config not found in store after save!");
            return Err("配置保存验证失败：存储中找不到配置".to_string());
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SysConfig::default();
        assert_eq!(config.api_base_url, "http://localhost:8080/api/v1");
        assert_eq!(config.ws_url, "ws://localhost:8080/api/v1/ws");
        assert_eq!(config.s3_region, "us-east-1");
        assert!(!config.s3_use_path_style);
        assert!(config.s3_access_key.is_none());
        assert!(config.vault_path.is_none());
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let config = SysConfig {
            api_base_url: "http://test:9090/api/v1".to_string(),
            s3_endpoint: "https://s3.test.com".to_string(),
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: SysConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.api_base_url, "http://test:9090/api/v1");
        assert_eq!(parsed.s3_endpoint, "https://s3.test.com");
    }

    #[test]
    fn test_sensitive_fields_not_serialized() {
        let config = SysConfig {
            s3_access_key: Some("secret123".to_string()),
            s3_secret_key: Some("secret456".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.contains("secret123"));
        assert!(!json.contains("secret456"));
    }
}
