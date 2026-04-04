use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("文件创建失败: {0}")]
    FileCreateError(String),

    #[error("Zip 写入失败: {0}")]
    ZipError(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("数据拉取失败: {0}")]
    FetchError(String),

    #[error("线程执行崩溃")]
    TaskPanic,
}

pub type ExportResult<T> = Result<T, ExportError>;

impl From<zip::result::ZipError> for ExportError {
    fn from(e: zip::result::ZipError) -> Self {
        ExportError::ZipError(e.to_string())
    }
}
