//! ms-io 统一错误类型

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IoError {
    // ── 文件系统 ──
    #[error("原子写入失败: {0}")]
    AtomicWriteFailed(String),

    #[error("路径不安全: {0}")]
    UnsafePath(String),

    // ── 扫描 ──
    #[error("Vault 目录不存在: {0}")]
    VaultNotFound(String),

    // ── 媒体 ──
    #[error("图片解码失败: {0}")]
    ImageDecodeFailed(String),

    #[error("图片编码失败: {0}")]
    ImageEncodeFailed(String),

    #[error("图片缩放失败: {0}")]
    ImageResizeFailed(String),

    #[error("输入数据过大: {0}")]
    PayloadTooLarge(String),

    // ── 云端 ──
    #[error("云存储配置错误: {0}")]
    CloudConfigError(String),

    #[error("上传失败: {0}")]
    CloudUploadFailed(String),

    #[error("删除失败: {0}")]
    CloudDeleteFailed(String),

    #[error("查询失败: {0}")]
    CloudHeadFailed(String),

    // ── 导出 ──
    #[error("导出文件创建失败: {0}")]
    ExportCreateFailed(String),

    #[error("ZIP 写入失败: {0}")]
    ExportZipFailed(String),

    #[error("数据拉取失败: {0}")]
    ExportFetchFailed(String),

    // ── 系统 ──
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("任务崩溃: {reason}")]
    TaskPanic { reason: String },
}

pub type IoResult<T> = Result<T, IoError>;
