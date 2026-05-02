//! 云端旁路 — 极简 S3 发射器
//!
//! 无 trait 抽象，直接使用 `S3Backend` 结构体。
//! 上传失败不影响本地主干读写。

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::error::S3Error;
use s3::Region;
use serde::Deserialize;

use crate::error::{IoError, IoResult};

/// S3 连接配置
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

/// S3 后端 — 直接操作，无 trait 分发
pub struct S3Backend {
    bucket: Bucket,
    public_url_base: Option<String>,
}

impl S3Backend {
    pub fn new(config: &StorageConfig) -> IoResult<Self> {
        let credentials = Credentials::new(
            Some(&config.access_key),
            Some(&config.secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| IoError::CloudConfigError(format!("凭证创建失败: {e}")))?;

        let mut bucket = Bucket::new(
            &config.bucket,
            Region::Custom {
                region: config.region.clone(),
                endpoint: config.endpoint.clone(),
            },
            credentials,
        )
        .map_err(|e| IoError::CloudConfigError(format!("Bucket 创建失败: {e}")))?;

        if config.use_path_style {
            bucket.set_path_style();
        }

        Ok(Self {
            bucket,
            public_url_base: config.public_url_base.clone(),
        })
    }

    /// 上传文件，返回公开访问 URL
    pub async fn upload(&self, key: &str, data: &[u8], content_type: &str) -> IoResult<String> {
        self.bucket
            .put_object_with_content_type(key, data, content_type)
            .await
            .map_err(|e| IoError::CloudUploadFailed(e.to_string()))?;
        Ok(self.get_url(key))
    }

    /// 删除远端文件
    pub async fn delete(&self, key: &str) -> IoResult<()> {
        self.bucket
            .delete_object(key)
            .await
            .map_err(|e| IoError::CloudDeleteFailed(e.to_string()))?;
        Ok(())
    }

    /// 获取文件访问 URL
    pub fn get_url(&self, key: &str) -> String {
        if let Some(base) = &self.public_url_base {
            return format!("{}/{}", base.trim_end_matches('/'), key);
        }
        format!("{}/{}", self.bucket.host(), key)
    }

    /// 检查远端文件是否存在
    pub async fn exists(&self, key: &str) -> IoResult<bool> {
        match self.bucket.head_object(key).await {
            Ok(_) => Ok(true),
            Err(S3Error::HttpFailWithBody(404, _)) => Ok(false),
            Err(e) => Err(IoError::CloudHeadFailed(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(
            config.public_url_base,
            Some("https://cdn.example.com".to_string())
        );
        assert!(config.use_path_style);
        Ok(())
    }

    #[test]
    fn test_config_deserialize_minimal() -> Result<(), Box<dyn std::error::Error>> {
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
    fn test_config_missing_required_field() {
        let json = r#"{
            "endpoint": "https://s3.example.com",
            "region": "us-east-1"
        }"#;
        let result = serde_json::from_str::<StorageConfig>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_url_with_base() -> Result<(), Box<dyn std::error::Error>> {
        let config: StorageConfig = serde_json::from_str(sample_config_json())?;
        let backend = S3Backend::new(&config)?;
        let url = backend.get_url("images/photo.webp");
        assert_eq!(url, "https://cdn.example.com/images/photo.webp");
        Ok(())
    }
}
