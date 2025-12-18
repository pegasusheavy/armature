//! Health check utilities for Cloud Run.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Service is healthy.
    Healthy,
    /// Service is degraded but operational.
    Degraded,
    /// Service is unhealthy.
    Unhealthy,
}

impl HealthStatus {
    /// Check if the service is healthy or degraded (can accept traffic).
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }

    /// Check if the service is fully healthy.
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }
}

/// Health check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Overall status.
    pub status: HealthStatus,
    /// Service name.
    pub service: Option<String>,
    /// Revision name.
    pub revision: Option<String>,
    /// Uptime in seconds.
    pub uptime_seconds: u64,
    /// Individual check results.
    #[serde(default)]
    pub checks: Vec<CheckResult>,
}

/// Individual check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Check name.
    pub name: String,
    /// Check status.
    pub status: HealthStatus,
    /// Optional message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Check duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// Health check manager.
#[derive(Clone)]
pub struct HealthCheck {
    start_time: std::time::Instant,
    checks: Arc<RwLock<Vec<Box<dyn HealthChecker>>>>,
    status_override: Arc<RwLock<Option<HealthStatus>>>,
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthCheck {
    /// Create a new health check manager.
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            checks: Arc::new(RwLock::new(Vec::new())),
            status_override: Arc::new(RwLock::new(None)),
        }
    }

    /// Register a health checker.
    pub async fn register(&self, checker: impl HealthChecker + 'static) {
        let mut checks = self.checks.write().await;
        checks.push(Box::new(checker));
    }

    /// Override the health status (useful during shutdown).
    pub async fn set_status(&self, status: HealthStatus) {
        let mut override_status = self.status_override.write().await;
        *override_status = Some(status);
    }

    /// Clear the status override.
    pub async fn clear_status(&self) {
        let mut override_status = self.status_override.write().await;
        *override_status = None;
    }

    /// Mark as unhealthy (for graceful shutdown).
    pub async fn mark_unhealthy(&self) {
        self.set_status(HealthStatus::Unhealthy).await;
    }

    /// Run all health checks.
    pub async fn check(&self) -> HealthCheckResult {
        // Check for status override
        if let Some(status) = *self.status_override.read().await {
            return HealthCheckResult {
                status,
                service: std::env::var("K_SERVICE").ok(),
                revision: std::env::var("K_REVISION").ok(),
                uptime_seconds: self.start_time.elapsed().as_secs(),
                checks: vec![],
            };
        }

        let checks = self.checks.read().await;
        let mut results = Vec::with_capacity(checks.len());
        let mut overall_status = HealthStatus::Healthy;

        for checker in checks.iter() {
            let start = std::time::Instant::now();
            let result = checker.check().await;
            let duration_ms = start.elapsed().as_millis() as u64;

            // Update overall status
            match result.status {
                HealthStatus::Unhealthy => overall_status = HealthStatus::Unhealthy,
                HealthStatus::Degraded if overall_status == HealthStatus::Healthy => {
                    overall_status = HealthStatus::Degraded;
                }
                _ => {}
            }

            results.push(CheckResult {
                name: result.name,
                status: result.status,
                message: result.message,
                duration_ms: Some(duration_ms),
            });
        }

        HealthCheckResult {
            status: overall_status,
            service: std::env::var("K_SERVICE").ok(),
            revision: std::env::var("K_REVISION").ok(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            checks: results,
        }
    }

    /// Simple liveness check (always healthy unless overridden).
    pub async fn liveness(&self) -> bool {
        if let Some(status) = *self.status_override.read().await {
            return status.is_ready();
        }
        true
    }

    /// Readiness check (runs all checks).
    pub async fn readiness(&self) -> bool {
        self.check().await.status.is_ready()
    }
}

/// Trait for health checkers.
#[async_trait::async_trait]
pub trait HealthChecker: Send + Sync {
    /// Run the health check.
    async fn check(&self) -> CheckResult;
}

/// Simple function-based health checker.
pub struct FnHealthChecker<F> {
    name: String,
    check_fn: F,
}

impl<F, Fut> FnHealthChecker<F>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<(), String>> + Send,
{
    /// Create a new function-based health checker.
    pub fn new(name: impl Into<String>, check_fn: F) -> Self {
        Self {
            name: name.into(),
            check_fn,
        }
    }
}

#[async_trait::async_trait]
impl<F, Fut> HealthChecker for FnHealthChecker<F>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<(), String>> + Send,
{
    async fn check(&self) -> CheckResult {
        match (self.check_fn)().await {
            Ok(()) => CheckResult {
                name: self.name.clone(),
                status: HealthStatus::Healthy,
                message: None,
                duration_ms: None,
            },
            Err(msg) => CheckResult {
                name: self.name.clone(),
                status: HealthStatus::Unhealthy,
                message: Some(msg),
                duration_ms: None,
            },
        }
    }
}
