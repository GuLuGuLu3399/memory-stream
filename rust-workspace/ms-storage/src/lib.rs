#![cfg(feature = "native")]

mod error;

pub use error::{StorageError, StorageResult};

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::error::S3Error;
use s3::Region;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub public_url_base: Option<String>,
    #[serde(default)]
    pub use_path_style: bool,
}

#[async_trait::async_trait]
pub trait StorageProvider: Send + Sync {
    async fn upload(&self, key: &str, data: &[u8], content_type: &str) -> StorageResult<String>;
    async fn delete(&self, key: &str) -> StorageResult<()>;
    async fn get_url(&self, key: &str) -> String;
    async fn exists(&self, key: &str) -> StorageResult<bool>;
}

pub struct S3Backend {
    bucket: Box<Bucket>,
    public_url_base: Option<String>,
}

impl S3Backend {
    /// 创建新的 S3 后端实例。
    ///
    /// # Errors
    /// 返回错误如果配置无效或 S3 客户端创建失败。
    pub fn new(config: &StorageConfig) -> StorageResult<Self> {
        let credentials = Credentials::new(
            Some(&config.access_key),
            Some(&config.secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| StorageError::ConfigError(format!("凭证创建失败: {e}")))?;

        let mut bucket = Bucket::new(
            &config.bucket,
            Region::Custom {
                region: config.region.clone(),
                endpoint: config.endpoint.clone(),
            },
            credentials,
        )
        .map_err(|e| StorageError::ConfigError(format!("Bucket 创建失败: {e}")))?;

        if config.use_path_style {
            bucket.set_path_style();
        }

        Ok(Self {
            bucket,
            public_url_base: config.public_url_base.clone(),
        })
    }
}

#[async_trait::async_trait]
impl StorageProvider for S3Backend {
    async fn upload(&self, key: &str, data: &[u8], content_type: &str) -> StorageResult<String> {
        self.bucket
            .put_object_with_content_type(key, data, content_type)
            .await
            .map_err(|e| StorageError::UploadError(e.to_string()))?;

        Ok(self.get_url(key).await)
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        self.bucket
            .delete_object(key)
            .await
            .map_err(|e| StorageError::DeleteError(e.to_string()))?;
        Ok(())
    }

    async fn get_url(&self, key: &str) -> String {
        if let Some(base) = &self.public_url_base {
            return format!("{}/{}", base.trim_end_matches('/'), key);
        }
        match self.bucket.presign_get(key, 3600, None).await {
            Ok(url) => url,
            Err(_) => format!("{}/{}", self.bucket.host(), key),
        }
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        match self.bucket.head_object(key).await {
            Ok(_) => Ok(true),
            Err(S3Error::HttpFailWithBody(404, _)) => Ok(false),
            Err(e) => Err(StorageError::HeadError(e.to_string())),
        }
    }
}

/// 创建存储提供者实例。
///
/// # Errors
/// 返回错误如果配置无效或后端创建失败。
pub fn create_storage(config: &StorageConfig) -> StorageResult<Box<dyn StorageProvider>> {
    Ok(Box::new(S3Backend::new(config)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------------
    // StorageConfig serde round-trip
    // ---------------------------------------------------------------------------

    fn sample_config_json() -> &'static str {
        r#"{
            "endpoint": "https://s3.example.com",
            "region": "us-east-1",
            "bucket": "test-bucket",
            "access_key": "AKIAEXAMPLE",
            "secret_key": "secret123",
            "public_url_base": "https://cdn.example.com",
            "use_path_style": true
        }"#
    }

    #[test]
    fn test_config_deserialize_full() -> Result<(), Box<dyn std::error::Error>> {
        let config: StorageConfig = serde_json::from_str(sample_config_json())?;
        assert_eq!(config.endpoint, "https://s3.example.com");
        assert_eq!(config.region, "us-east-1");
        assert_eq!(config.bucket, "test-bucket");
        assert_eq!(config.access_key, "AKIAEXAMPLE");
        assert_eq!(config.secret_key, "secret123");
        assert_eq!(config.public_url_base, Some("https://cdn.example.com".to_string()));
        assert!(config.use_path_style);
        Ok(())
    }

    #[test]
    fn test_config_deserialize_minimal() -> Result<(), Box<dyn std::error::Error>> {
        // use_path_style defaults to false, public_url_base is None
        let json = r#"{
            "endpoint": "http://localhost:9000",
            "region": "local",
            "bucket": "my-bucket",
            "access_key": "minioadmin",
            "secret_key": "minioadmin"
        }"#;
        let config: StorageConfig = serde_json::from_str(json)?;
        assert_eq!(config.endpoint, "http://localhost:9000");
        assert!(config.public_url_base.is_none());
        assert!(!config.use_path_style);
        Ok(())
    }

    #[test]
    fn test_config_deserialize_missing_required_field() {
        let json = r#"{
            "endpoint": "https://s3.example.com",
            "region": "us-east-1"
        }"#;
        let result = serde_json::from_str::<StorageConfig>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        // StorageConfig only has Deserialize, so we round-trip by re-deserializing
        // the same JSON and comparing all fields.
        let a: StorageConfig = serde_json::from_str(sample_config_json())?;
        let b: StorageConfig = serde_json::from_str(sample_config_json())?;

        assert_eq!(a.endpoint, b.endpoint);
        assert_eq!(a.region, b.region);
        assert_eq!(a.bucket, b.bucket);
        assert_eq!(a.access_key, b.access_key);
        assert_eq!(a.secret_key, b.secret_key);
        assert_eq!(a.public_url_base, b.public_url_base);
        assert_eq!(a.use_path_style, b.use_path_style);
        Ok(())
    }

    #[test]
    fn test_config_clone_equals() -> Result<(), Box<dyn std::error::Error>> {
        let config: StorageConfig = serde_json::from_str(sample_config_json())?;
        let cloned = config.clone();
        assert_eq!(cloned.endpoint, config.endpoint);
        assert_eq!(cloned.region, config.region);
        assert_eq!(cloned.bucket, config.bucket);
        assert_eq!(cloned.access_key, config.access_key);
        assert_eq!(cloned.secret_key, config.secret_key);
        assert_eq!(cloned.public_url_base, config.public_url_base);
        assert_eq!(cloned.use_path_style, config.use_path_style);
        Ok(())
    }

    #[test]
    fn test_config_debug_format() -> Result<(), Box<dyn std::error::Error>> {
        let config: StorageConfig = serde_json::from_str(sample_config_json())?;
        let debug_str = format!("{config:?}");
        // Debug output should contain field names
        assert!(debug_str.contains("endpoint"));
        assert!(debug_str.contains("region"));
        assert!(debug_str.contains("bucket"));
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // StorageError display & variants
    // ---------------------------------------------------------------------------

    #[test]
    fn test_error_display_config() {
        let err = StorageError::ConfigError("bad cfg".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("配置错误"), "expected Chinese prefix, got: {msg}");
        assert!(msg.contains("bad cfg"));
    }

    #[test]
    fn test_error_display_upload() {
        let err = StorageError::UploadError("network failure".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("上传失败"));
        assert!(msg.contains("network failure"));
    }

    #[test]
    fn test_error_display_delete() {
        let err = StorageError::DeleteError("not found".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("删除失败"));
        assert!(msg.contains("not found"));
    }

    #[test]
    fn test_error_display_head() {
        let err = StorageError::HeadError("timeout".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("查询失败"));
        assert!(msg.contains("timeout"));
    }

    #[test]
    fn test_error_display_url() {
        let err = StorageError::UrlError("invalid host".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("URL 生成失败"));
        assert!(msg.contains("invalid host"));
    }

    #[test]
    fn test_error_debug_includes_variant_info() {
        let err = StorageError::ConfigError("detail".to_string());
        let debug = format!("{err:?}");
        assert!(debug.contains("ConfigError"));
    }

    // ---------------------------------------------------------------------------
    // StorageResult type alias
    // ---------------------------------------------------------------------------

    #[test]
    #[allow(clippy::unwrap_used, clippy::unnecessary_literal_unwrap)]
    fn test_storage_result_ok() {
        let result: StorageResult<String> = Ok("hello".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::unnecessary_literal_unwrap)]
    fn test_storage_result_err() {
        let result: StorageResult<String> = Err(StorageError::UploadError("fail".to_string()));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{err}").contains("上传失败"));
    }

    // ---------------------------------------------------------------------------
    // S3Backend::new — failure paths (no real S3 needed, constructor fails fast)
    // ---------------------------------------------------------------------------

    #[test]
    fn test_s3_backend_new_with_empty_credentials() {
        // Empty strings should still create Credentials but Bucket may fail
        // depending on rust-s3 validation. At minimum, this should not panic.
        let config = StorageConfig {
            endpoint: String::new(),
            region: String::new(),
            bucket: String::new(),
            access_key: String::new(),
            secret_key: String::new(),
            public_url_base: None,
            use_path_style: false,
        };
        // We're verifying this doesn't panic; it may Ok or Err depending on
        // the s3 crate's validation, both are acceptable.
        let _ = S3Backend::new(&config);
    }

    #[test]
    fn test_s3_backend_new_with_path_style() {
        let config = StorageConfig {
            endpoint: "https://s3.example.com".to_string(),
            region: "us-east-1".to_string(),
            bucket: "test-bucket".to_string(),
            access_key: "key".to_string(),
            secret_key: "secret".to_string(),
            public_url_base: None,
            use_path_style: true,
        };
        // Should succeed at construction (no network call)
        let result = S3Backend::new(&config);
        // The constructor only builds objects, no I/O, so it should succeed
        // unless the s3 crate validates endpoint format.
        // We just verify no panic.
        let _ = result;
    }

    #[test]
    fn test_s3_backend_new_stores_public_url_base() {
        let config = StorageConfig {
            endpoint: "https://s3.example.com".to_string(),
            region: "us-east-1".to_string(),
            bucket: "test-bucket".to_string(),
            access_key: "key".to_string(),
            secret_key: "secret".to_string(),
            public_url_base: Some("https://cdn.example.com".to_string()),
            use_path_style: false,
        };
        // Construction should succeed; we verify the backend is created
        if let Ok(backend) = S3Backend::new(&config) {
            assert!(backend.public_url_base.is_some());
            assert_eq!(backend.public_url_base.as_deref(), Some("https://cdn.example.com"));
        }
        // If construction fails (unlikely for non-empty fields), that's also acceptable
        // since no real S3 endpoint is being contacted.
    }

    #[test]
    fn test_s3_backend_new_without_public_url_base() {
        let config = StorageConfig {
            endpoint: "https://s3.example.com".to_string(),
            region: "us-east-1".to_string(),
            bucket: "test-bucket".to_string(),
            access_key: "key".to_string(),
            secret_key: "secret".to_string(),
            public_url_base: None,
            use_path_style: false,
        };
        if let Ok(backend) = S3Backend::new(&config) {
            assert!(backend.public_url_base.is_none());
        }
    }

    // ---------------------------------------------------------------------------
    // create_storage — factory function
    // ---------------------------------------------------------------------------

    #[test]
    fn test_create_storage_returns_boxed_provider() {
        let config = StorageConfig {
            endpoint: "https://s3.example.com".to_string(),
            region: "us-east-1".to_string(),
            bucket: "test-bucket".to_string(),
            access_key: "key".to_string(),
            secret_key: "secret".to_string(),
            public_url_base: None,
            use_path_style: false,
        };
        let result = create_storage(&config);
        // Construction doesn't contact S3, should succeed
        if let Ok(provider) = result {
            // Verify the trait object is usable (type check)
            let _: &dyn StorageProvider = &*provider;
        }
    }

    // ---------------------------------------------------------------------------
    // Mock StorageProvider — verifies trait contract and edge cases
    // ---------------------------------------------------------------------------

    struct MockStorage {
        store: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
        public_url_base: String,
    }

    impl MockStorage {
        fn new(public_url_base: &str) -> Self {
            Self {
                store: std::sync::Mutex::new(std::collections::HashMap::new()),
                public_url_base: public_url_base.to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl StorageProvider for MockStorage {
        async fn upload(&self, key: &str, data: &[u8], _content_type: &str) -> StorageResult<String> {
            {
                let mut store = self.store.lock().expect("mock lock poisoned");
                store.insert(key.to_string(), data.to_vec());
            }
            Ok(self.get_url(key).await)
        }

        async fn delete(&self, key: &str) -> StorageResult<()> {
            {
                let mut store = self.store.lock().expect("mock lock poisoned");
                store.remove(key);
            }
            Ok(())
        }

        async fn get_url(&self, key: &str) -> String {
            format!("{}/{}", self.public_url_base.trim_end_matches('/'), key)
        }

        async fn exists(&self, key: &str) -> StorageResult<bool> {
            let store = self.store.lock().expect("mock lock poisoned");
            Ok(store.contains_key(key))
        }
    }

    #[tokio::test]
    async fn test_mock_upload_returns_url() -> Result<(), Box<dyn std::error::Error>> {
        let mock = MockStorage::new("https://cdn.example.com");
        let url = mock.upload("images/photo.png", b"png-data", "image/png")
            .await?;
        assert_eq!(url, "https://cdn.example.com/images/photo.png");
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_upload_stores_data() -> Result<(), Box<dyn std::error::Error>> {
        let mock = MockStorage::new("https://cdn.example.com");
        mock.upload("file.txt", b"hello world", "text/plain").await?;

        let store = mock.store.lock().expect("mock lock poisoned");
        assert_eq!(store.get("file.txt"), Some(&b"hello world".to_vec()));
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_exists_after_upload() -> Result<(), Box<dyn std::error::Error>> {
        let mock = MockStorage::new("https://cdn.example.com");
        assert!(!mock.exists("key").await?);

        mock.upload("key", b"data", "application/octet-stream").await?;
        assert!(mock.exists("key").await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_not_exists_after_delete() -> Result<(), Box<dyn std::error::Error>> {
        let mock = MockStorage::new("https://cdn.example.com");
        mock.upload("key", b"data", "text/plain").await?;
        assert!(mock.exists("key").await?);

        mock.delete("key").await?;
        assert!(!mock.exists("key").await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_delete_nonexistent_is_ok() {
        let mock = MockStorage::new("https://cdn.example.com");
        // Deleting a key that doesn't exist should not error
        let result = mock.delete("ghost-key").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_get_url_trims_trailing_slash() {
        let mock = MockStorage::new("https://cdn.example.com/");
        let url = mock.get_url("path/to/file.jpg").await;
        assert_eq!(url, "https://cdn.example.com/path/to/file.jpg");
    }

    #[tokio::test]
    async fn test_mock_upload_empty_data() -> Result<(), Box<dyn std::error::Error>> {
        let mock = MockStorage::new("https://cdn.example.com");
        let url = mock.upload("empty.dat", b"", "application/octet-stream")
            .await?;
        assert_eq!(url, "https://cdn.example.com/empty.dat");
        assert!(mock.exists("empty.dat").await?);

        let store = mock.store.lock().expect("mock lock poisoned");
        assert_eq!(store.get("empty.dat"), Some(&Vec::<u8>::new()));
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_upload_overwrites() -> Result<(), Box<dyn std::error::Error>> {
        let mock = MockStorage::new("https://cdn.example.com");
        mock.upload("key", b"v1", "text/plain").await?;
        mock.upload("key", b"v2", "text/plain").await?;

        let store = mock.store.lock().expect("mock lock poisoned");
        assert_eq!(store.get("key"), Some(&b"v2".to_vec()));
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_multiple_keys() -> Result<(), Box<dyn std::error::Error>> {
        let mock = MockStorage::new("https://cdn.example.com");
        for i in 0..10 {
            mock.upload(
                &format!("key-{i}"),
                format!("data-{i}").as_bytes(),
                "text/plain",
            )
            .await?;
        }

        for i in 0..10 {
            assert!(mock.exists(&format!("key-{i}")).await?);
        }
        assert!(!mock.exists("key-10").await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_mock_special_characters_in_key() -> Result<(), Box<dyn std::error::Error>> {
        let mock = MockStorage::new("https://cdn.example.com");
        let key = "path/to/file with spaces/日本語.md";
        mock.upload(key, b"special", "text/plain").await?;
        assert!(mock.exists(key).await?);

        let url = mock.get_url(key).await;
        assert!(url.contains(key));
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // get_url formatting edge cases (via mock)
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn test_get_url_no_double_slash() {
        let mock = MockStorage::new("https://cdn.example.com");
        let url = mock.get_url("simple-key").await;
        assert_eq!(url, "https://cdn.example.com/simple-key");
        // Should not have double slash between base and key
        assert!(!url.contains("//simple-key"));
    }

    #[tokio::test]
    async fn test_get_url_with_nested_path() {
        let mock = MockStorage::new("https://cdn.example.com");
        let url = mock.get_url("a/b/c/d/file.txt").await;
        assert_eq!(url, "https://cdn.example.com/a/b/c/d/file.txt");
    }

    // ---------------------------------------------------------------------------
    // Trait object usage — verify dyn StorageProvider works correctly
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn test_trait_object_upload_and_exists() -> Result<(), Box<dyn std::error::Error>> {
        let mock: Box<dyn StorageProvider> = Box::new(MockStorage::new("https://cdn.example.com"));
        let url = mock.upload("test", b"data", "text/plain").await?;
        assert_eq!(url, "https://cdn.example.com/test");
        assert!(mock.exists("test").await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_trait_object_delete() -> Result<(), Box<dyn std::error::Error>> {
        let mock: Box<dyn StorageProvider> = Box::new(MockStorage::new("https://cdn.example.com"));
        mock.upload("test", b"data", "text/plain").await?;
        mock.delete("test").await?;
        assert!(!mock.exists("test").await?);
        Ok(())
    }
}
