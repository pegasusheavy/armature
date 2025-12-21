//! Error types for i18n operations

use thiserror::Error;

/// Errors that can occur during i18n operations.
#[derive(Debug, Error)]
pub enum I18nError {
    /// Invalid locale string
    #[error("Invalid locale: {0}")]
    InvalidLocale(String),

    /// Message not found
    #[error("Message not found: {key} for locale {locale}")]
    MessageNotFound { key: String, locale: String },

    /// Bundle not found for locale
    #[error("No message bundle for locale: {0}")]
    BundleNotFound(String),

    /// Failed to parse message file
    #[error("Failed to parse message file: {0}")]
    ParseError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON parse error
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Fluent error
    #[cfg(feature = "fluent")]
    #[error("Fluent error: {0}")]
    FluentError(String),

    /// Format error
    #[error("Format error: {0}")]
    FormatError(String),

    /// Invalid plural category
    #[error("Invalid plural category: {0}")]
    InvalidPluralCategory(String),
}
