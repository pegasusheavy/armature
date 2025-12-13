//! Error types for webhook operations

use thiserror::Error;

/// Errors that can occur during webhook operations
#[derive(Error, Debug)]
pub enum WebhookError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// Signature verification failed
    #[error("Signature verification failed: {0}")]
    SignatureInvalid(String),

    /// Signature missing from request
    #[error("Signature missing from request")]
    SignatureMissing,

    /// Timestamp validation failed
    #[error("Timestamp validation failed: {0}")]
    TimestampInvalid(String),

    /// Payload serialization/deserialization failed
    #[error("Payload error: {0}")]
    PayloadError(String),

    /// Delivery failed after all retries
    #[error("Delivery failed after {attempts} attempts: {message}")]
    DeliveryFailed { attempts: u32, message: String },

    /// Endpoint not found
    #[error("Endpoint not found: {0}")]
    EndpointNotFound(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded for endpoint: {0}")]
    RateLimitExceeded(String),

    /// Timeout error
    #[error("Request timed out after {0} seconds")]
    Timeout(u64),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for WebhookError {
    fn from(err: serde_json::Error) -> Self {
        WebhookError::PayloadError(err.to_string())
    }
}
