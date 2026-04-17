use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressError {
    #[error("图片解码失败: {0}")]
    DecodeError(String),

    #[error("WebP 编码失败: {0}")]
    EncodeError(String),

    #[error("图片缩放失败: {0}")]
    ResizeError(String),

    #[error("Blocking task panicked: {reason}")]
    TaskPanic { reason: String },

    #[error("输入数据过大: {0}")]
    PayloadTooLarge(String),
}

pub type CompressResult<T> = Result<T, CompressError>;

impl From<image::ImageError> for CompressError {
    fn from(e: image::ImageError) -> Self {
        CompressError::DecodeError(e.to_string())
    }
}
