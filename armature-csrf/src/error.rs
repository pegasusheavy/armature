use thiserror::Error;

#[derive(Error, Debug)]
pub enum CsrfError {
    #[error("Invalid CSRF token")]
    InvalidToken,

    #[error("Missing CSRF token")]
    MissingToken,

    #[error("CSRF token expired")]
    TokenExpired,

    #[error("Token generation failed: {0}")]
    GenerationFailed(String),

    #[error("Token validation failed: {0}")]
    ValidationFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, CsrfError>;

