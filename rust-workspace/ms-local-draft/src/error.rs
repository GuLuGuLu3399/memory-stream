use thiserror::Error;

#[derive(Error, Debug)]
pub enum DraftError {
    #[error("数据库打开失败: {0}")]
    OpenError(String),

    #[error("SQL 执行失败: {0}")]
    SqlError(String),

    #[error("线程执行崩溃")]
    TaskPanic,
}

pub type DraftResult<T> = Result<T, DraftError>;

impl From<rusqlite::Error> for DraftError {
    fn from(e: rusqlite::Error) -> Self {
        DraftError::SqlError(e.to_string())
    }
}
