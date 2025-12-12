//! Error types for compression operations

use thiserror::Error;

/// Errors that can occur during compression
#[derive(Error, Debug)]
pub enum CompressionError {
    /// Compression operation failed
    #[error("Compression failed: {0}")]
    CompressionFailed(String),

    /// Decompression operation failed
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    /// Invalid compression level
    #[error("Invalid compression level: {0} (must be between {1} and {2})")]
    InvalidLevel(u32, u32, u32),

    /// Unsupported algorithm
    #[error("Unsupported compression algorithm: {0}")]
    UnsupportedAlgorithm(String),

    /// IO error during compression
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
}
