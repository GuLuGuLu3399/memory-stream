// ────────────────────────────────────────────────────────────────
// Config — mirrors admin-tauri Rust AppConfig
// ────────────────────────────────────────────────────────────────

export interface StorageConfig {
  provider: string;
  endpoint: string;
  region: string;
  bucket: string;
  access_key: string;
  secret_key: string;
  public_domain: string;
  force_path_style: boolean;
}

export interface AppConfig {
  api_base_url: string;
  vault_path: string;
  theme: string;
  storage: StorageConfig | null;
}
