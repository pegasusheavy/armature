//! GCP error types.

use thiserror::Error;

/// Result type for GCP operations.
pub type Result<T> = std::result::Result<T, GcpError>;

/// GCP service errors.
#[derive(Debug, Error)]
pub enum GcpError {
    /// Service not enabled.
    #[error("Service '{0}' is not enabled. Enable the feature flag in Cargo.toml")]
    ServiceNotEnabled(&'static str),

    /// Service not configured.
    #[error("Service '{0}' is not configured. Call enable_{0}() on GcpConfig")]
    ServiceNotConfigured(&'static str),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Authentication error.
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Project ID not specified.
    #[error("GCP project ID not specified")]
    ProjectNotSpecified,

    /// Service error.
    #[error("GCP service error: {0}")]
    Service(String),

    /// Network error.
    #[error("Network error: {0}")]
    Network(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl GcpError {
    /// Create a service not enabled error.
    pub fn not_enabled(service: &'static str) -> Self {
        Self::ServiceNotEnabled(service)
    }

    /// Create a service not configured error.
    pub fn not_configured(service: &'static str) -> Self {
        Self::ServiceNotConfigured(service)
    }
}
