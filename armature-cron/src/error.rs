//! Error types for cron operations.

use thiserror::Error;

/// Result type for cron operations.
pub type CronResult<T> = Result<T, CronError>;

/// Cron-specific errors.
#[derive(Debug, Error)]
pub enum CronError {
    /// Invalid cron expression
    #[error("Invalid cron expression: {0}")]
    InvalidExpression(String),

    /// Job not found
    #[error("Job not found: {0}")]
    JobNotFound(String),

    /// Job already exists
    #[error("Job already exists: {0}")]
    JobAlreadyExists(String),

    /// Job execution failed
    #[error("Job execution failed: {0}")]
    ExecutionFailed(String),

    /// Scheduler not running
    #[error("Scheduler not running")]
    SchedulerNotRunning,

    /// Scheduler already running
    #[error("Scheduler already running")]
    SchedulerAlreadyRunning,

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Generic error
    #[error("Cron error: {0}")]
    Other(String),
}
