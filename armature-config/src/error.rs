// Error types for configuration management

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration key not found: {0}")]
    KeyNotFound(String),

    #[error("Failed to load configuration: {0}")]
    LoadError(String),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Environment variable error: {0}")]
    EnvError(#[from] std::env::VarError),
}

pub type Result<T> = std::result::Result<T, ConfigError>;
