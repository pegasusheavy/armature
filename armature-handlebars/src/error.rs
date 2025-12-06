//! Error types for Handlebars integration

use thiserror::Error;

/// Result type for Handlebars operations
pub type Result<T> = std::result::Result<T, HandlebarsError>;

/// Errors that can occur when using Handlebars templates
#[derive(Error, Debug)]
pub enum HandlebarsError {
    /// Template not found
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// Template rendering error
    #[error("Template rendering error: {0}")]
    RenderError(String),

    /// Template parsing error
    #[error("Template parsing error: {0}")]
    ParseError(String),

    /// IO error when loading templates
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Helper registration error
    #[error("Helper registration error: {0}")]
    HelperError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl From<handlebars::RenderError> for HandlebarsError {
    fn from(err: handlebars::RenderError) -> Self {
        HandlebarsError::RenderError(err.to_string())
    }
}

impl From<handlebars::TemplateError> for HandlebarsError {
    fn from(err: handlebars::TemplateError) -> Self {
        HandlebarsError::ParseError(err.to_string())
    }
}

