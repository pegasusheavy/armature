//! Error types for SeaORM integration.

use thiserror::Error;

/// Errors that can occur when using the SeaORM integration.
#[derive(Error, Debug)]
pub enum SeaOrmError {
    /// Database connection error.
    #[error("Connection error: {0}")]
    Connection(String),

    /// Query execution error.
    #[error("Query error: {0}")]
    Query(String),

    /// Database error from SeaORM.
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// Transaction error.
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Entity not found.
    #[error("Entity not found: {0}")]
    NotFound(String),

    /// Validation error.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Migration error.
    #[error("Migration error: {0}")]
    Migration(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Result type alias for SeaORM operations.
pub type SeaOrmResult<T> = Result<T, SeaOrmError>;

impl From<serde_json::Error> for SeaOrmError {
    fn from(err: serde_json::Error) -> Self {
        SeaOrmError::Serialization(err.to_string())
    }
}
