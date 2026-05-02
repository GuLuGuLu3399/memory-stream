use thiserror::Error;

pub type MetaResult<T> = Result<T, MetaDbError>;

#[derive(Error, Debug)]
pub enum MetaDbError {
    #[error("Failed to open database: {0}")]
    OpenFailed(#[from] rusqlite::Error),

    #[error("Query execution failed: {0}")]
    QueryFailed(String),

    #[error("Schema migration failed: {0}")]
    MigrationFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Concurrency conflict: {0}")]
    ConcurrencyConflict(String),
}
