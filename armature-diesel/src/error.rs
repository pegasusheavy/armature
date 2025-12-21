//! Error types for Diesel integration.

use thiserror::Error;

/// Errors that can occur when using the Diesel integration.
#[derive(Error, Debug)]
pub enum DieselError {
    /// Database connection error.
    #[error("Connection error: {0}")]
    Connection(String),

    /// Connection pool error.
    #[error("Pool error: {0}")]
    Pool(String),

    /// Query execution error.
    #[error("Query error: {0}")]
    Query(#[from] diesel::result::Error),

    /// Transaction error.
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Timeout error.
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Migration error.
    #[cfg(feature = "migrations")]
    #[error("Migration error: {0}")]
    Migration(String),
}

/// Result type alias for Diesel operations.
pub type DieselResult<T> = Result<T, DieselError>;

impl From<std::io::Error> for DieselError {
    fn from(err: std::io::Error) -> Self {
        DieselError::Connection(err.to_string())
    }
}
