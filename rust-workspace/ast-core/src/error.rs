use thiserror::Error;

///AstNode的错误枚举类型
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
