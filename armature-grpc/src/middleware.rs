//! gRPC middleware using Tower.

use std::time::Duration;
use tower::Layer;

/// gRPC middleware trait.
pub trait GrpcMiddleware<S> {
    /// The wrapped service type.
    type Service;

    /// Wrap the service with this middleware.
    fn wrap(self, service: S) -> Self::Service;
}

/// Middleware layer for Tower compatibility.
#[derive(Clone)]
pub struct MiddlewareLayer<M> {
    middleware: M,
}

impl<M> MiddlewareLayer<M> {
    /// Create a new middleware layer.
    pub fn new(middleware: M) -> Self {
        Self { middleware }
    }
}

impl<S, M> Layer<S> for MiddlewareLayer<M>
where
    M: GrpcMiddleware<S> + Clone,
{
    type Service = M::Service;

    fn layer(&self, service: S) -> Self::Service {
        self.middleware.clone().wrap(service)
    }
}

/// Timeout middleware for gRPC.
pub struct TimeoutMiddleware {
    timeout: Duration,
}

impl TimeoutMiddleware {
    /// Create a new timeout middleware.
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Get the timeout duration.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

/// Rate limiting middleware for gRPC.
pub struct RateLimitMiddleware {
    requests_per_second: u64,
}

impl RateLimitMiddleware {
    /// Create a new rate limit middleware.
    pub fn new(requests_per_second: u64) -> Self {
        Self {
            requests_per_second,
        }
    }

    /// Get the rate limit.
    pub fn rps(&self) -> u64 {
        self.requests_per_second
    }
}

/// Concurrency limit middleware.
pub struct ConcurrencyLimitMiddleware {
    max_concurrent: usize,
}

impl ConcurrencyLimitMiddleware {
    /// Create a new concurrency limit middleware.
    pub fn new(max_concurrent: usize) -> Self {
        Self { max_concurrent }
    }

    /// Get the concurrency limit.
    pub fn limit(&self) -> usize {
        self.max_concurrent
    }
}

/// Load shedding middleware that rejects requests when overloaded.
pub struct LoadSheddingMiddleware {
    enabled: bool,
}

impl LoadSheddingMiddleware {
    /// Create a new load shedding middleware.
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Enable or disable load shedding.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Default for LoadSheddingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Retry middleware for gRPC.
pub struct RetryMiddleware {
    max_attempts: u32,
    retry_codes: Vec<tonic::Code>,
}

impl RetryMiddleware {
    /// Create a new retry middleware.
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            retry_codes: vec![
                tonic::Code::Unavailable,
                tonic::Code::ResourceExhausted,
                tonic::Code::Aborted,
            ],
        }
    }

    /// Set the codes that should trigger a retry.
    pub fn with_retry_codes(mut self, codes: Vec<tonic::Code>) -> Self {
        self.retry_codes = codes;
        self
    }

    /// Get the maximum number of attempts.
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    /// Check if a code should trigger a retry.
    pub fn should_retry(&self, code: tonic::Code) -> bool {
        self.retry_codes.contains(&code)
    }
}

/// Compression middleware configuration.
pub struct CompressionMiddleware {
    encoding: CompressionEncoding,
}

/// Compression encoding types.
#[derive(Debug, Clone, Copy)]
pub enum CompressionEncoding {
    /// Gzip compression.
    Gzip,
    /// Zstd compression.
    Zstd,
    /// No compression.
    None,
}

impl CompressionMiddleware {
    /// Create a new compression middleware.
    pub fn new(encoding: CompressionEncoding) -> Self {
        Self { encoding }
    }

    /// Create a gzip compression middleware.
    pub fn gzip() -> Self {
        Self::new(CompressionEncoding::Gzip)
    }

    /// Create a zstd compression middleware.
    pub fn zstd() -> Self {
        Self::new(CompressionEncoding::Zstd)
    }

    /// Get the compression encoding.
    pub fn encoding(&self) -> CompressionEncoding {
        self.encoding
    }
}
