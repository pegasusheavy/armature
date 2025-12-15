//! Azure Functions error types.

use thiserror::Error;

/// Result type for Azure Functions operations.
pub type Result<T> = std::result::Result<T, AzureFunctionsError>;

/// Azure Functions runtime errors.
#[derive(Debug, Error)]
pub enum AzureFunctionsError {
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

    /// Runtime error.
    #[error("Runtime error: {0}")]
    Runtime(String),

    /// Binding error.
    #[error("Binding error: {0}")]
    Binding(String),
}

