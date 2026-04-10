use thiserror::Error;

///`AstNode的错误枚举类型`
#[derive(Error, Debug)]
pub enum MSError {
    #[error("Markdown 解析失败: {0}")]
    ParseError(String),

    #[error("AST 渲染失败: {0}")]
    RenderError(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("未知的 AST 节点类型: {0}")]
    UnknownNodeType(String),
}

pub type MSResult<T> = Result<T, MSError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_parse_error_display() {
        let err = MSError::ParseError("unexpected token".to_string());
        assert_eq!(format!("{err}"), "Markdown 解析失败: unexpected token");
    }

    #[test]
    fn test_render_error_display() {
        let err = MSError::RenderError("buffer overflow".to_string());
        assert_eq!(format!("{err}"), "AST 渲染失败: buffer overflow");
    }

    #[test]
    fn test_io_error_from_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
        let err: MSError = io_err.into();
        assert!(matches!(err, MSError::IoError(_)));
        assert!(format!("{err}").contains("file missing"));
    }

    #[test]
    fn test_unknown_node_type_display() {
        let err = MSError::UnknownNodeType("FancyNode".to_string());
        assert_eq!(format!("{err}"), "未知的 AST 节点类型: FancyNode");
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::unnecessary_literal_unwrap)]
    fn test_msresult_ok() {
        let result: MSResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::unnecessary_literal_unwrap)]
    fn test_msresult_err() {
        let result: MSResult<i32> = Err(MSError::ParseError("bad input".to_string()));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, MSError::ParseError(_)));
    }

    #[test]
    fn test_error_debug_format() {
        let err = MSError::ParseError("detail".to_string());
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("ParseError"));
    }
}
