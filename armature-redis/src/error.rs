//! Redis error types.

use thiserror::Error;

/// Result type for Redis operations.
pub type Result<T> = std::result::Result<T, RedisError>;

/// Redis errors.
#[derive(Debug, Error)]
pub enum RedisError {
    /// Connection error.
    #[error("Connection error: {0}")]
    Connection(String),

    /// Pool error.
    #[error("Pool error: {0}")]
    Pool(String),

    /// Command error.
    #[error("Command error: {0}")]
    Command(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Authentication error.
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Timeout error.
    #[error("Operation timed out")]
    Timeout,

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Pub/Sub error.
    #[error("Pub/Sub error: {0}")]
    PubSub(String),

    /// Cluster error.
    #[error("Cluster error: {0}")]
    Cluster(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Underlying Redis error.
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
}

impl RedisError {
    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Connection(_) | Self::Timeout | Self::Pool(_)
        )
    }

    /// Check if this error indicates connection loss.
    pub fn is_connection_error(&self) -> bool {
        matches!(self, Self::Connection(_))
    }
}

impl<E> From<bb8::RunError<E>> for RedisError
where
    E: std::error::Error + 'static,
{
    fn from(err: bb8::RunError<E>) -> Self {
        Self::Pool(err.to_string())
    }
}

impl From<serde_json::Error> for RedisError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

