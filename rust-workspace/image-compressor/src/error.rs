use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressError {
    #[error("图片解码失败: {0}")]
    DecodeError(String),

    #[error("WebP 编码失败: {0}")]
    EncodeError(String),

    #[error("图片缩放失败: {0}")]
    ResizeError(String),

    #[error("线程执行崩溃")]
    TaskPanic,
}

pub type CompressResult<T> = Result<T, CompressError>;

impl From<image::ImageError> for CompressError {
    fn from(e: image::ImageError) -> Self {
        CompressError::DecodeError(e.to_string())
    }
}
