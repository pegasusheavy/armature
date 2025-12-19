//! Error types for Ferron integration
//!
//! This module provides error types for Ferron operations including
//! configuration, process management, and health checking.

use thiserror::Error;

/// Result type alias for Ferron operations
pub type Result<T> = std::result::Result<T, FerronError>;

/// Errors that can occur during Ferron operations
#[derive(Debug, Error)]
pub enum FerronError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid configuration value
    #[error("Invalid configuration value for '{field}': {message}")]
    InvalidConfig { field: String, message: String },

    /// Process management error
    #[error("Process error: {0}")]
    Process(String),

    /// Ferron process not found
    #[error("Ferron process not found at path: {0}")]
    ProcessNotFound(String),

    /// Process failed to start
    #[error("Failed to start Ferron: {0}")]
    StartFailed(String),

    /// Process failed to stop
    #[error("Failed to stop Ferron: {0}")]
    StopFailed(String),

    /// Configuration reload failed
    #[error("Failed to reload configuration: {0}")]
    ReloadFailed(String),

    /// Health check error
    #[error("Health check failed: {0}")]
    HealthCheck(String),

    /// Backend unreachable
    #[error("Backend unreachable: {url}")]
    BackendUnreachable { url: String },

    /// Service registry error
    #[error("Service registry error: {0}")]
    Registry(String),

    /// Service not found in registry
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// URL parsing error
    #[error("Invalid URL: {0}")]
    UrlParse(#[from] url::ParseError),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Template rendering error
    #[error("Template error: {0}")]
    Template(String),

    /// File watching error
    #[error("File watch error: {0}")]
    Watch(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Already running
    #[error("Ferron is already running with PID {0}")]
    AlreadyRunning(u32),

    /// Not running
    #[error("Ferron is not currently running")]
    NotRunning,
}

impl FerronError {
    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create an invalid configuration error
    pub fn invalid_config(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidConfig {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a process error
    pub fn process(msg: impl Into<String>) -> Self {
        Self::Process(msg.into())
    }

    /// Create a health check error
    pub fn health_check(msg: impl Into<String>) -> Self {
        Self::HealthCheck(msg.into())
    }

    /// Create a registry error
    pub fn registry(msg: impl Into<String>) -> Self {
        Self::Registry(msg.into())
    }

    /// Create a template error
    pub fn template(msg: impl Into<String>) -> Self {
        Self::Template(msg.into())
    }

    /// Create a timeout error
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::BackendUnreachable { .. }
                | Self::HealthCheck(_)
                | Self::Timeout(_)
                | Self::ReloadFailed(_)
        )
    }

    /// Check if the error is a configuration error
    pub fn is_config_error(&self) -> bool {
        matches!(self, Self::Config(_) | Self::InvalidConfig { .. })
    }

    /// Check if the error is a process error
    pub fn is_process_error(&self) -> bool {
        matches!(
            self,
            Self::Process(_)
                | Self::ProcessNotFound(_)
                | Self::StartFailed(_)
                | Self::StopFailed(_)
                | Self::AlreadyRunning(_)
                | Self::NotRunning
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = FerronError::config("invalid domain");
        assert_eq!(err.to_string(), "Configuration error: invalid domain");

        let err = FerronError::invalid_config("port", "must be between 1 and 65535");
        assert_eq!(
            err.to_string(),
            "Invalid configuration value for 'port': must be between 1 and 65535"
        );
    }

    #[test]
    fn test_error_is_recoverable() {
        assert!(FerronError::BackendUnreachable {
            url: "http://localhost:3000".into()
        }
        .is_recoverable());
        assert!(FerronError::HealthCheck("timeout".into()).is_recoverable());
        assert!(!FerronError::Config("bad config".into()).is_recoverable());
    }

    #[test]
    fn test_error_categories() {
        assert!(FerronError::Config("test".into()).is_config_error());
        assert!(FerronError::invalid_config("field", "msg").is_config_error());
        assert!(!FerronError::Process("test".into()).is_config_error());

        assert!(FerronError::Process("test".into()).is_process_error());
        assert!(FerronError::NotRunning.is_process_error());
        assert!(!FerronError::Config("test".into()).is_process_error());
    }
}

