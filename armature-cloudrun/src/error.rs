//! Cloud Run error types.

use thiserror::Error;

/// Result type for Cloud Run operations.
pub type Result<T> = std::result::Result<T, CloudRunError>;

/// Cloud Run errors.
#[derive(Debug, Error)]
pub enum CloudRunError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Metadata fetch error.
    #[error("Metadata error: {0}")]
    Metadata(String),

    /// Health check error.
    #[error("Health check error: {0}")]
    HealthCheck(String),

    /// Server error.
    #[error("Server error: {0}")]
    Server(String),
}

