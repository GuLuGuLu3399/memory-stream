use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("上传失败: {0}")]
    UploadError(String),

    #[error("删除失败: {0}")]
    DeleteError(String),

    #[error("查询失败: {0}")]
    HeadError(String),

    #[error("URL 生成失败: {0}")]
    UrlError(String),
}

pub type StorageResult<T> = Result<T, StorageError>;
