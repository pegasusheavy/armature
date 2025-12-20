//! Error types for TOON operations.

use thiserror::Error;

/// Errors that can occur during TOON operations.
#[derive(Debug, Error)]
pub enum ToonError {
    /// Serialization error.
    #[error("TOON serialization error: {0}")]
    SerializeError(String),

    /// Deserialization error.
    #[error("TOON deserialization error: {0}")]
    DeserializeError(String),

    /// UTF-8 encoding error.
    #[error("UTF-8 encoding error: {0}")]
    Utf8Error(String),

    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<serde_toon::Error> for ToonError {
    fn from(err: serde_toon::Error) -> Self {
        ToonError::SerializeError(err.to_string())
    }
}

