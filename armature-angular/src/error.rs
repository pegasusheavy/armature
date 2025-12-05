// Error types for Angular SSR

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AngularError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Rendering error: {0}")]
    RenderError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Static file not found: {0}")]
    FileNotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Node.js error: {0}")]
    NodeError(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, AngularError>;
