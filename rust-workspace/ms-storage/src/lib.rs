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
    pub fn new(config: &StorageConfig) -> StorageResult<Self> {
        let credentials = Credentials::new(
            Some(&config.access_key),
            Some(&config.secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| StorageError::ConfigError(format!("凭证创建失败: {}", e)))?;

        let mut bucket = Bucket::new(
            &config.bucket,
            Region::Custom {
                region: config.region.clone(),
                endpoint: config.endpoint.clone(),
            },
            credentials,
        )
        .map_err(|e| StorageError::ConfigError(format!("Bucket 创建失败: {}", e)))?;

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

pub fn create_storage(config: &StorageConfig) -> StorageResult<Box<dyn StorageProvider>> {
    Ok(Box::new(S3Backend::new(config)?))
}

 
