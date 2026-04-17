//! 结构化错误契约 — IPC 友好，前端可据此渲染差异化 UI
//!
//! 所有 temple_* crate 统一使用 `TempleError`，通过 `serde::Serialize`
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
    WsConnectionFailed,

    // ── 缓存 ──
    CacheNotInitialized,
    CacheQueryFailed,
    CacheLockFailed,

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

    // ── 草稿 ──
    DraftDbOpenFailed,
    DraftSqlFailed,

    // ── 搜索 ──
    IndexNotReady,
    SearchQueryFailed,

    // ── 系统 ──
    TaskPanic,
    InternalError,
    NotImplemented,
}

// ============================================================================
// TempleError — IPC 友好的结构化错误
// ============================================================================

#[derive(Debug, Clone, Serialize, Error)]
#[error("{message}")]
pub struct TempleError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl TempleError {
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

pub type TempleResult<T> = Result<T, TempleError>;

// ============================================================================
// From 转换：底层 crate 错误 → TempleError
// ============================================================================

impl From<std::io::Error> for TempleError {
    fn from(e: std::io::Error) -> Self {
        let code = match e.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::FileNotFound,
            std::io::ErrorKind::PermissionDenied => ErrorCode::FileWriteFailed,
            _ => ErrorCode::FileReadFailed,
        };
        TempleError::new(code, e.to_string())
    }
}

impl From<serde_json::Error> for TempleError {
    fn from(e: serde_json::Error) -> Self {
        if e.is_data() {
            TempleError::new(ErrorCode::AstDeserializeFailed, e.to_string())
        } else {
            TempleError::new(ErrorCode::AstSerializeFailed, e.to_string())
        }
    }
}

// ── ast-core: MSError ──
impl From<ast_core::error::MSError> for TempleError {
    fn from(e: ast_core::error::MSError) -> Self {
        match e {
            ast_core::error::MSError::ParseError(msg) => {
                TempleError::new(ErrorCode::MarkdownParseFailed, msg)
            }
            ast_core::error::MSError::RenderError(msg) => {
                TempleError::new(ErrorCode::AstRenderFailed, msg)
            }
            ast_core::error::MSError::IoError(io) => TempleError::from(io),
            ast_core::error::MSError::UnknownNodeType(ty) => {
                TempleError::new(ErrorCode::UnknownNodeType, format!("未知的 AST 节点类型: {ty}"))
            }
            ast_core::error::MSError::InvalidOperation(msg) => {
                TempleError::new(ErrorCode::AstRenderFailed, msg)
            }
        }
    }
}

// ── ms-storage: StorageError ──
#[cfg(feature = "native")]
impl From<ms_storage::StorageError> for TempleError {
    fn from(e: ms_storage::StorageError) -> Self {
        match e {
            ms_storage::StorageError::ConfigError(msg) => {
                TempleError::new(ErrorCode::StorageConfigError, msg)
            }
            ms_storage::StorageError::UploadError(msg) => {
                TempleError::new(ErrorCode::S3UploadFailed, msg)
            }
            ms_storage::StorageError::DeleteError(msg) => {
                TempleError::new(ErrorCode::S3DeleteFailed, msg)
            }
            ms_storage::StorageError::HeadError(msg) => {
                TempleError::new(ErrorCode::S3HeadFailed, msg)
            }
            ms_storage::StorageError::UrlError(msg) => {
                TempleError::new(ErrorCode::S3UrlError, msg)
            }
        }
    }
}

// ── ms-local-draft: DraftError ──
#[cfg(feature = "native")]
impl From<ms_local_draft::error::DraftError> for TempleError {
    fn from(e: ms_local_draft::error::DraftError) -> Self {
        match e {
            ms_local_draft::error::DraftError::OpenError(msg) => {
                TempleError::new(ErrorCode::DraftDbOpenFailed, msg)
            }
            ms_local_draft::error::DraftError::SqlError(msg) => {
                TempleError::new(ErrorCode::DraftSqlFailed, msg)
            }
            ms_local_draft::error::DraftError::TaskPanic { reason } => {
                TempleError::new(ErrorCode::TaskPanic, format!("线程执行崩溃: {reason}"))
            }
        }
    }
}

// ── image-compressor: CompressError ──
#[cfg(feature = "native")]
impl From<image_compressor::error::CompressError> for TempleError {
    fn from(e: image_compressor::error::CompressError) -> Self {
        match e {
            image_compressor::error::CompressError::DecodeError(msg) => {
                TempleError::new(ErrorCode::ImageDecodeFailed, msg)
            }
            image_compressor::error::CompressError::EncodeError(msg) => {
                TempleError::new(ErrorCode::ImageEncodeFailed, msg)
            }
            image_compressor::error::CompressError::ResizeError(msg) => {
                TempleError::new(ErrorCode::ImageResizeFailed, msg)
            }
            image_compressor::error::CompressError::TaskPanic { reason } => {
                TempleError::new(ErrorCode::TaskPanic, format!("线程执行崩溃: {reason}"))
            }
            image_compressor::error::CompressError::PayloadTooLarge(msg) => {
                TempleError::new(ErrorCode::ImageDecodeFailed, msg)
            }
        }
    }
}

// ── ms-kb-exporter: ExportError ──
#[cfg(feature = "native")]
impl From<ms_kb_exporter::error::ExportError> for TempleError {
    fn from(e: ms_kb_exporter::error::ExportError) -> Self {
        match e {
            ms_kb_exporter::error::ExportError::FileCreateError(msg) => {
                TempleError::new(ErrorCode::ExportFileCreateFailed, msg)
            }
            ms_kb_exporter::error::ExportError::ZipError(msg) => {
                TempleError::new(ErrorCode::ExportZipFailed, msg)
            }
            ms_kb_exporter::error::ExportError::IoError(io) => TempleError::from(io),
            ms_kb_exporter::error::ExportError::FetchError(msg) => {
                TempleError::new(ErrorCode::ExportFetchFailed, msg)
            }
            ms_kb_exporter::error::ExportError::TaskPanic { reason } => {
                TempleError::new(ErrorCode::TaskPanic, format!("线程执行崩溃: {reason}"))
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
        let err = TempleError::new(ErrorCode::FileNotFound, "test.md 不存在")
            .with_details("path: /vault/test.md");
        let json = serde_json::to_string(&err)?;
        assert!(json.contains("\"code\":\"FILE_NOT_FOUND\""));
        assert!(json.contains("\"message\":\"test.md 不存在\""));
        assert!(json.contains("\"details\":\"path: /vault/test.md\""));
        Ok(())
    }

    #[test]
    fn test_temple_error_skip_none_details() -> Result<(), Box<dyn std::error::Error>> {
        let err = TempleError::new(ErrorCode::CacheNotInitialized, "缓存未初始化");
        let json = serde_json::to_string(&err)?;
        assert!(!json.contains("details"));
        Ok(())
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let temple_err: TempleError = io_err.into();
        assert_eq!(temple_err.code, ErrorCode::FileNotFound);
        assert!(temple_err.message.contains("file missing"));
    }

    #[test]
    fn test_from_ms_error_parse() {
        let ms_err = ast_core::error::MSError::ParseError("unexpected token".into());
        let temple_err: TempleError = ms_err.into();
        assert_eq!(temple_err.code, ErrorCode::MarkdownParseFailed);
    }

    #[test]
    fn test_from_ms_error_render() {
        let ms_err = ast_core::error::MSError::RenderError("buffer overflow".into());
        let temple_err: TempleError = ms_err.into();
        assert_eq!(temple_err.code, ErrorCode::AstRenderFailed);
    }

    #[test]
    #[cfg(feature = "native")]
    fn test_from_storage_error() {
        let s_err = ms_storage::StorageError::UploadError("timeout".into());
        let temple_err: TempleError = s_err.into();
        assert_eq!(temple_err.code, ErrorCode::S3UploadFailed);
    }

    #[test]
    #[cfg(feature = "native")]
    fn test_from_draft_error() {
        let d_err = ms_local_draft::error::DraftError::SqlError("constraint".into());
        let temple_err: TempleError = d_err.into();
        assert_eq!(temple_err.code, ErrorCode::DraftSqlFailed);
    }

    #[test]
    #[cfg(feature = "native")]
    fn test_from_compress_error() {
        let c_err = image_compressor::error::CompressError::DecodeError("corrupt".into());
        let temple_err: TempleError = c_err.into();
        assert_eq!(temple_err.code, ErrorCode::ImageDecodeFailed);
    }

    #[test]
    #[cfg(feature = "native")]
    fn test_from_export_error() {
        let e_err = ms_kb_exporter::error::ExportError::ZipError("crc mismatch".into());
        let temple_err: TempleError = e_err.into();
        assert_eq!(temple_err.code, ErrorCode::ExportZipFailed);
    }

    #[test]
    fn test_display_format() {
        let err = TempleError::new(ErrorCode::AuthExpired, "JWT 过期");
        let display = format!("{err}");
        assert!(display.contains("JWT 过期"));
    }
}
