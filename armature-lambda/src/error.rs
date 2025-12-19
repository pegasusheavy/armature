//! Lambda error types.

use thiserror::Error;

/// Result type for Lambda operations.
pub type Result<T> = std::result::Result<T, LambdaError>;

/// Lambda runtime errors.
#[derive(Debug, Error)]
pub enum LambdaError {
    /// Application error.
    #[error("Application error: {0}")]
    Application(String),

    /// Request conversion error.
    #[error("Request conversion error: {0}")]
    Request(String),

    /// Response conversion error.
    #[error("Response conversion error: {0}")]
    Response(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Lambda runtime error.
    #[error("Lambda runtime error: {0}")]
    Runtime(String),
}

impl From<lambda_runtime::Error> for LambdaError {
    fn from(err: lambda_runtime::Error) -> Self {
        Self::Runtime(err.to_string())
    }
}
