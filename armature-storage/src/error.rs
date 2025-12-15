//! Storage error types.

use thiserror::Error;

/// Result type for storage operations.
pub type Result<T> = std::result::Result<T, StorageError>;

/// Storage and upload errors.
#[derive(Debug, Error)]
pub enum StorageError {
    /// File validation failed.
    #[error("Validation error: {0}")]
    Validation(#[from] crate::ValidationError),

    /// Multipart parsing error.
    #[error("Multipart error: {0}")]
    Multipart(String),

    /// File not found.
    #[error("File not found: {0}")]
    NotFound(String),

    /// Storage backend error.
    #[error("Storage error: {0}")]
    Storage(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Permission denied.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Quota exceeded.
    #[error("Storage quota exceeded")]
    QuotaExceeded,

    /// Upload too large.
    #[error("Upload too large: {size} bytes exceeds limit of {limit} bytes")]
    TooLarge {
        /// Actual size.
        size: u64,
        /// Maximum allowed size.
        limit: u64,
    },

    /// Invalid file name.
    #[error("Invalid file name: {0}")]
    InvalidFileName(String),

    /// Network error (for cloud storage).
    #[error("Network error: {0}")]
    Network(String),

    /// Timeout.
    #[error("Operation timed out")]
    Timeout,
}

impl StorageError {
    /// Check if this is a validation error.
    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Validation(_))
    }

    /// Check if this is a not found error.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    /// Check if this is a size limit error.
    pub fn is_too_large(&self) -> bool {
        matches!(self, Self::TooLarge { .. })
    }

    /// Convert to HTTP status code.
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Validation(_) => 400,
            Self::NotFound(_) => 404,
            Self::PermissionDenied(_) => 403,
            Self::QuotaExceeded | Self::TooLarge { .. } => 413,
            Self::InvalidFileName(_) => 400,
            Self::Timeout => 408,
            _ => 500,
        }
    }
}

impl From<multer::Error> for StorageError {
    fn from(err: multer::Error) -> Self {
        Self::Multipart(err.to_string())
    }
}

