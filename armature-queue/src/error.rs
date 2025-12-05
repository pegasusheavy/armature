//! Error types for queue operations.

use thiserror::Error;

/// Result type for queue operations.
pub type QueueResult<T> = Result<T, QueueError>;

/// Queue-specific errors.
#[derive(Debug, Error)]
pub enum QueueError {
    /// Redis error
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Job not found
    #[error("Job not found: {0}")]
    JobNotFound(String),

    /// Job execution failed
    #[error("Job execution failed: {0}")]
    ExecutionFailed(String),

    /// No handler registered for job type
    #[error("No handler registered for job type: {0}")]
    NoHandler(String),

    /// Worker not running
    #[error("Worker not running")]
    WorkerNotRunning,

    /// Worker already running
    #[error("Worker already running")]
    WorkerAlreadyRunning,

    /// Queue is full
    #[error("Queue is full")]
    QueueFull,

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Timeout error
    #[error("Operation timeout")]
    Timeout,

    /// Generic error
    #[error("Queue error: {0}")]
    Other(String),
}
