//! 结构化错误契约 — IPC 友好，前端可据此渲染差异化 UI
//!
//! ms-graph crate 统一使用 `GraphError`，通过 `serde::Serialize`
//! 穿透 Tauri IPC 边界，前端收到结构化 JSON：
//! ```json
//! { "code": "MARKDOWN_PARSE_FAILED", "message": "...", "details": "..." }
//! ```

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// ErrorCode — 前端据此匹配 UI 状态
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // ── 文件系统 ──
    FileNotFound,
    FileReadFailed,
    FileWriteFailed,
    DirectoryNotFound,

    // ── Markdown / AST 解析 ──
    MarkdownParseFailed,
    AstRenderFailed,
    AstSerializeFailed,
    AstDeserializeFailed,
    UnknownNodeType,

    // ── Frontmatter ──
    FrontmatterParseFailed,

    // ── 图谱 ──
    GraphCycleDetected,
    GraphNodeNotFound,
    GraphEdgeCreationFailed,

    // ── 网络 ──
    NetworkUnreachable,
    ApiError,
    AuthExpired,

    // ── 存储 ──
    StorageConfigMissing,
    StorageConfigError,
    S3UploadFailed,
    S3DeleteFailed,
    S3HeadFailed,
    S3UrlError,

    // ── 图片 ──
    ImageDecodeFailed,
    ImageEncodeFailed,
    ImageResizeFailed,

    // ── 导出 ──
    ExportFileCreateFailed,
    ExportZipFailed,
    ExportFetchFailed,

    // ── 系统 ──
    TaskPanic,
    InternalError,
    NotImplemented,
}

// ============================================================================
// GraphError — IPC 友好的结构化错误
// ============================================================================

#[derive(Debug, Clone, Serialize, Error)]
#[error("{message}")]
pub struct GraphError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl GraphError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

pub type GraphResult<T> = Result<T, GraphError>;

// ============================================================================
// From 转换：底层 crate 错误 → GraphError
// ============================================================================

impl From<std::io::Error> for GraphError {
    fn from(e: std::io::Error) -> Self {
        let code = match e.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::FileNotFound,
            std::io::ErrorKind::PermissionDenied => ErrorCode::FileWriteFailed,
            _ => ErrorCode::FileReadFailed,
        };
        GraphError::new(code, e.to_string())
    }
}

impl From<serde_json::Error> for GraphError {
    fn from(e: serde_json::Error) -> Self {
        if e.is_data() {
            GraphError::new(ErrorCode::AstDeserializeFailed, e.to_string())
        } else {
            GraphError::new(ErrorCode::AstSerializeFailed, e.to_string())
        }
    }
}


// ── ms-io: unified IoError ──
#[cfg(feature = "io-conversions")]
impl From<ms_io::error::IoError> for GraphError {
    fn from(e: ms_io::error::IoError) -> Self {
        use ms_io::error::IoError;
        match e {
            IoError::AtomicWriteFailed(msg) | IoError::UnsafePath(msg) => {
                GraphError::new(ErrorCode::FileWriteFailed, msg)
            }
            IoError::VaultNotFound(msg) => {
                GraphError::new(ErrorCode::DirectoryNotFound, msg)
            }
            IoError::ImageDecodeFailed(msg) | IoError::PayloadTooLarge(msg) => {
                GraphError::new(ErrorCode::ImageDecodeFailed, msg)
            }
            IoError::ImageEncodeFailed(msg) => {
                GraphError::new(ErrorCode::ImageEncodeFailed, msg)
            }
            IoError::ImageResizeFailed(msg) => {
                GraphError::new(ErrorCode::ImageResizeFailed, msg)
            }
            IoError::CloudConfigError(msg) => {
                GraphError::new(ErrorCode::StorageConfigError, msg)
            }
            IoError::CloudUploadFailed(msg) => {
                GraphError::new(ErrorCode::S3UploadFailed, msg)
            }
            IoError::CloudDeleteFailed(msg) => {
                GraphError::new(ErrorCode::S3DeleteFailed, msg)
            }
            IoError::CloudHeadFailed(msg) => {
                GraphError::new(ErrorCode::S3HeadFailed, msg)
            }
            IoError::ExportCreateFailed(msg) => {
                GraphError::new(ErrorCode::ExportFileCreateFailed, msg)
            }
            IoError::ExportZipFailed(msg) => {
                GraphError::new(ErrorCode::ExportZipFailed, msg)
            }
            IoError::ExportFetchFailed(msg) => {
                GraphError::new(ErrorCode::ExportFetchFailed, msg)
            }
            IoError::Io(io) => GraphError::from(io),
            IoError::TaskPanic { reason } => {
                GraphError::new(ErrorCode::TaskPanic, format!("线程执行崩溃: {reason}"))
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_serde() -> Result<(), Box<dyn std::error::Error>> {
        let code = ErrorCode::MarkdownParseFailed;
        let json = serde_json::to_string(&code)?;
        assert_eq!(json, "\"MARKDOWN_PARSE_FAILED\"");
        let parsed: ErrorCode = serde_json::from_str(&json)?;
        assert_eq!(parsed, code);
        Ok(())
    }

    #[test]
    fn test_temple_error_serialize() -> Result<(), Box<dyn std::error::Error>> {
        let err = GraphError::new(ErrorCode::FileNotFound, "test.md 不存在")
            .with_details("path: /vault/test.md");
        let json = serde_json::to_string(&err)?;
        assert!(json.contains("\"code\":\"FILE_NOT_FOUND\""));
        assert!(json.contains("\"message\":\"test.md 不存在\""));
        assert!(json.contains("\"details\":\"path: /vault/test.md\""));
        Ok(())
    }

    #[test]
    fn test_temple_error_skip_none_details() -> Result<(), Box<dyn std::error::Error>> {
        let err = GraphError::new(ErrorCode::FileNotFound, "文件不存在");
        let json = serde_json::to_string(&err)?;
        assert!(!json.contains("details"));
        Ok(())
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let temple_err: GraphError = io_err.into();
        assert_eq!(temple_err.code, ErrorCode::FileNotFound);
        assert!(temple_err.message.contains("file missing"));
    }


    #[test]
    #[cfg(feature = "io-conversions")]
    fn test_from_io_error_cloud() {
        let err = ms_io::error::IoError::CloudUploadFailed("timeout".into());
        let graph_err: GraphError = err.into();
        assert_eq!(graph_err.code, ErrorCode::S3UploadFailed);
    }

    #[test]
    #[cfg(feature = "io-conversions")]
    fn test_from_io_error_media() {
        let err = ms_io::error::IoError::ImageDecodeFailed("corrupt".into());
        let graph_err: GraphError = err.into();
        assert_eq!(graph_err.code, ErrorCode::ImageDecodeFailed);
    }

    #[test]
    #[cfg(feature = "io-conversions")]
    fn test_from_io_error_export() {
        let err = ms_io::error::IoError::ExportZipFailed("crc mismatch".into());
        let graph_err: GraphError = err.into();
        assert_eq!(graph_err.code, ErrorCode::ExportZipFailed);
    }

    #[test]
    fn test_display_format() {
        let err = GraphError::new(ErrorCode::AuthExpired, "JWT 过期");
        let display = format!("{err}");
        assert!(display.contains("JWT 过期"));
    }
}
