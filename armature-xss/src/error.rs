use thiserror::Error;

#[derive(Error, Debug)]
pub enum XssError {
    #[error("Potentially malicious content detected: {0}")]
    MaliciousContent(String),

    #[error("Sanitization failed: {0}")]
    SanitizationFailed(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, XssError>;

