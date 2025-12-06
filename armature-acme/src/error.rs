/// Error types for ACME operations
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AcmeError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid account: {0}")]
    InvalidAccount(String),

    #[error("Order failed: {0}")]
    OrderFailed(String),

    #[error("Challenge failed: {0}")]
    ChallengeFailed(String),

    #[error("Certificate generation failed: {0}")]
    CertificateError(String),

    #[error("Invalid directory URL: {0}")]
    InvalidDirectory(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("PEM error: {0}")]
    PemError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Certificate not ready yet")]
    NotReady,

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, AcmeError>;
