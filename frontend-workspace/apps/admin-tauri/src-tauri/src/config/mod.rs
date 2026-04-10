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
    pub s3_access_key: Option<String>,
    #[serde(skip_serializing)]
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

pub async fn get_config(app: &AppHandle) -> Result<SysConfig, String> {
    use tauri_plugin_store::StoreExt;
    let store = app.store(STORE_NAME).map_err(|e| e.to_string())?;
    let config: SysConfig = store
        .get("config")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    Ok(config)
}

pub async fn save_config(app: &AppHandle, config: &SysConfig) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    let store = app.store(STORE_NAME).map_err(|e| e.to_string())?;
    let value = serde_json::to_value(config).map_err(|e| e.to_string())?;
    store.set("config", value);
    store.save().map_err(|e| e.to_string())?;
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
