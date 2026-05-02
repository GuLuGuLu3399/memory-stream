use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub use notify::RecommendedWatcher;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub provider: String,
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub public_domain: String,
    #[serde(default)]
    pub force_path_style: bool,
}

impl From<StorageConfig> for ms_io::cloud::StorageConfig {
    fn from(c: StorageConfig) -> Self {
        Self {
            endpoint: c.endpoint,
            region: c.region,
            bucket: c.bucket,
            access_key: c.access_key,
            secret_key: c.secret_key,
            public_url_base: None,
            use_path_style: c.force_path_style,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub api_base_url: String,
    pub vault_path: String,
    pub theme: String,
    #[serde(default)]
    pub storage: Option<StorageConfig>,
    #[serde(default)]
    pub last_sync_cursor: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_base_url: "http://localhost:8080/api/v1".into(),
            vault_path: String::new(),
            theme: "system".into(),
            storage: None,
            last_sync_cursor: None,
        }
    }
}

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub meta_db: Mutex<Option<ms_meta::MetaDb>>,
    pub config_dir: std::path::PathBuf,
    pub vault_watcher: Mutex<Option<RecommendedWatcher>>,
}

impl AppState {
    pub fn config_path(&self) -> std::path::PathBuf {
        self.config_dir.join("config.json")
    }

    pub fn save_config(&self) -> Result<(), String> {
        let cfg = self.config.lock().map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(&*cfg).map_err(|e| e.to_string())?;
        std::fs::write(self.config_path(), json).map_err(|e| format!("Config save failed: {e}"))
    }

    pub fn load_config(&self) {
        let path = self.config_path();
        if !path.exists() {
            return;
        }
        if let Ok(json) = std::fs::read_to_string(&path) {
            if let Ok(loaded) = serde_json::from_str::<AppConfig>(&json) {
                if let Ok(mut cfg) = self.config.lock() {
                    *cfg = loaded;
                }
            }
        }
    }
}
