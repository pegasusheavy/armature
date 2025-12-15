//! Timeout pattern for operations.
//!
//! ## Example
//!
//! ```rust,ignore
//! use armature::resilience::{Timeout, TimeoutConfig};
//! use std::time::Duration;
//!
//! let timeout = Timeout::new(TimeoutConfig {
//!     duration: Duration::from_secs(5),
//!     ..Default::default()
//! });
//!
//! let result = timeout.call(|| async {
//!     slow_operation().await
//! }).await;
//! ```

use std::future::Future;
use std::time::Duration;
use tracing::warn;

/// Timeout configuration.
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Name for logging/metrics.
    pub name: String,
    /// Timeout duration.
    pub duration: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            duration: Duration::from_secs(30),
        }
    }
}

impl TimeoutConfig {
    /// Create a new timeout configuration.
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            ..Default::default()
        }
    }

    /// Set the name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

/// Timeout error.
#[derive(Debug)]
pub enum TimeoutError<E> {
    /// Operation timed out.
    Timeout(Duration),
    /// Operation failed.
    Execution(E),
}

impl<E: std::fmt::Display> std::fmt::Display for TimeoutError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout(d) => write!(f, "Operation timed out after {:?}", d),
            Self::Execution(e) => write!(f, "Operation failed: {}", e),
        }
    }
}

impl<E: std::fmt::Debug + std::fmt::Display> std::error::Error for TimeoutError<E> {}

/// Timeout executor.
#[derive(Clone)]
pub struct Timeout {
    config: TimeoutConfig,
}

impl Timeout {
    /// Create a new timeout executor.
    pub fn new(config: TimeoutConfig) -> Self {
        Self { config }
    }

    /// Create with a duration.
    pub fn with_duration(duration: Duration) -> Self {
        Self::new(TimeoutConfig::new(duration))
    }

    /// Get the timeout duration.
    pub fn duration(&self) -> Duration {
        self.config.duration
    }

    /// Execute with timeout.
    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, TimeoutError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        match tokio::time::timeout(self.config.duration, f()).await {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(TimeoutError::Execution(e)),
            Err(_) => {
                warn!(
                    name = %self.config.name,
                    duration = ?self.config.duration,
                    "Operation timed out"
                );
                Err(TimeoutError::Timeout(self.config.duration))
            }
        }
    }

    /// Execute with timeout, returning the result directly.
    pub async fn call_infallible<F, Fut, T>(&self, f: F) -> Result<T, Duration>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        match tokio::time::timeout(self.config.duration, f()).await {
            Ok(result) => Ok(result),
            Err(_) => Err(self.config.duration),
        }
    }
}

/// Execute a future with a timeout.
pub async fn with_timeout<F, Fut, T>(duration: Duration, f: F) -> Result<T, Duration>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    match tokio::time::timeout(duration, f()).await {
        Ok(result) => Ok(result),
        Err(_) => Err(duration),
    }
}

/// Execute a fallible future with a timeout.
pub async fn with_timeout_result<F, Fut, T, E>(
    duration: Duration,
    f: F,
) -> Result<T, TimeoutError<E>>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    Timeout::with_duration(duration).call(f).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timeout_completes() {
        let timeout = Timeout::with_duration(Duration::from_secs(1));

        let result: Result<i32, TimeoutError<&str>> = timeout.call(|| async {
            Ok(42)
        }).await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_timeout_expires() {
        let timeout = Timeout::with_duration(Duration::from_millis(10));

        let result: Result<i32, TimeoutError<&str>> = timeout.call(|| async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(42)
        }).await;

        assert!(matches!(result, Err(TimeoutError::Timeout(_))));
    }
}

