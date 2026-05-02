//! # ms-io — 文件与媒体控制器
//!
//! 纯粹的物理触手：原子写入、极速扫盘、WebP 压缩、ZIP 导出、S3 旁路上传。

pub mod error;
pub mod fs;
pub mod scanner;

#[cfg(feature = "media")]
pub mod media;

#[cfg(feature = "export")]
pub mod export;

#[cfg(feature = "storage")]
pub mod cloud;

pub use error::{IoError, IoResult};
