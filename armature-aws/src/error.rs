//! AWS error types.

use thiserror::Error;

/// Result type for AWS operations.
pub type Result<T> = std::result::Result<T, AwsError>;

/// AWS service errors.
#[derive(Debug, Error)]
pub enum AwsError {
    /// Service not enabled.
    #[error("Service '{0}' is not enabled. Enable the feature flag in Cargo.toml")]
    ServiceNotEnabled(&'static str),

    /// Service not configured.
    #[error("Service '{0}' is not configured. Call enable_{0}() on AwsConfig")]
    ServiceNotConfigured(&'static str),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Authentication error.
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Region not specified.
    #[error("AWS region not specified")]
    RegionNotSpecified,

    /// Service error.
    #[error("AWS service error: {0}")]
    Service(String),

    /// Network error.
    #[error("Network error: {0}")]
    Network(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl AwsError {
    /// Create a service not enabled error.
    pub fn not_enabled(service: &'static str) -> Self {
        Self::ServiceNotEnabled(service)
    }

    /// Create a service not configured error.
    pub fn not_configured(service: &'static str) -> Self {
        Self::ServiceNotConfigured(service)
    }
}

