// Error types for authentication

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("User inactive")]
    InactiveUser,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Password hashing error: {0}")]
    PasswordHashError(String),

    #[error("Password verification error: {0}")]
    PasswordVerifyError(String),

    #[error("JWT error: {0}")]
    JwtError(#[from] armature_jwt::JwtError),

    #[error("Missing required role: {0}")]
    MissingRole(String),

    #[error("Missing permission: {0}")]
    MissingPermission(String),
}

pub type Result<T> = std::result::Result<T, AuthError>;
