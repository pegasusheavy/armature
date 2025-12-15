//! Cloud Run configuration.

use std::net::SocketAddr;

/// Cloud Run configuration.
///
/// Reads configuration from environment variables set by Cloud Run.
#[derive(Debug, Clone)]
pub struct CloudRunConfig {
    /// Port to listen on (from PORT env var).
    pub port: u16,
    /// Host to bind to.
    pub host: String,
    /// Service name (from K_SERVICE env var).
    pub service: Option<String>,
    /// Revision name (from K_REVISION env var).
    pub revision: Option<String>,
    /// Configuration name (from K_CONFIGURATION env var).
    pub configuration: Option<String>,
    /// Project ID (from GOOGLE_CLOUD_PROJECT env var).
    pub project_id: Option<String>,
    /// Region (from GOOGLE_CLOUD_REGION env var).
    pub region: Option<String>,
    /// Memory limit in MB.
    pub memory_limit_mb: Option<u32>,
    /// CPU limit.
    pub cpu_limit: Option<f32>,
    /// Request timeout in seconds.
    pub timeout_seconds: u32,
    /// Maximum concurrent requests per instance.
    pub max_concurrent_requests: u32,
}

impl Default for CloudRunConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "0.0.0.0".to_string(),
            service: None,
            revision: None,
            configuration: None,
            project_id: None,
            region: None,
            memory_limit_mb: None,
            cpu_limit: None,
            timeout_seconds: 300,
            max_concurrent_requests: 80,
        }
    }
}

impl CloudRunConfig {
    /// Create configuration from environment variables.
    pub fn from_env() -> Self {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        Self {
            port,
            host: "0.0.0.0".to_string(),
            service: std::env::var("K_SERVICE").ok(),
            revision: std::env::var("K_REVISION").ok(),
            configuration: std::env::var("K_CONFIGURATION").ok(),
            project_id: std::env::var("GOOGLE_CLOUD_PROJECT").ok(),
            region: std::env::var("GOOGLE_CLOUD_REGION").ok(),
            memory_limit_mb: std::env::var("MEMORY_LIMIT_MB")
                .ok()
                .and_then(|m| m.parse().ok()),
            cpu_limit: std::env::var("CPU_LIMIT")
                .ok()
                .and_then(|c| c.parse().ok()),
            timeout_seconds: std::env::var("CLOUD_RUN_TIMEOUT_SECONDS")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(300),
            max_concurrent_requests: std::env::var("CLOUD_RUN_CONCURRENCY")
                .ok()
                .and_then(|c| c.parse().ok())
                .unwrap_or(80),
        }
    }

    /// Set the port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the host.
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Get the bind address.
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Get the socket address.
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(
            self.host.parse().unwrap_or_else(|_| "0.0.0.0".parse().unwrap()),
            self.port,
        )
    }

    /// Check if running on Cloud Run.
    pub fn is_cloud_run(&self) -> bool {
        self.service.is_some()
    }

    /// Get service URL (if deployed).
    pub fn service_url(&self) -> Option<String> {
        match (&self.service, &self.project_id, &self.region) {
            (Some(service), Some(project), Some(region)) => {
                Some(format!(
                    "https://{}-{}.{}.run.app",
                    service,
                    project.replace('_', "-").chars().take(10).collect::<String>(),
                    region
                ))
            }
            _ => None,
        }
    }
}

