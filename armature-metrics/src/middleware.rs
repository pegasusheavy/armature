//! Request metrics middleware
//!
//! Automatically collect HTTP request metrics.

use armature_core::{Error, HttpRequest, HttpResponse, Middleware};
use once_cell::sync::Lazy;
use prometheus::{CounterVec, GaugeVec, HistogramVec};
use std::time::Instant;

/// HTTP request metrics
static HTTP_REQUEST_COUNTER: Lazy<CounterVec> = Lazy::new(|| {
    crate::register_counter_vec(
        "http_requests_total",
        "Total number of HTTP requests",
        &["method", "path", "status"],
    )
    .expect("Failed to register http_requests_total")
});

static HTTP_REQUEST_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    crate::register_histogram_vec_with_buckets(
        "http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "path", "status"],
        vec![
            0.001, 0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0, 7.5, 10.0,
        ],
    )
    .expect("Failed to register http_request_duration_seconds")
});

static HTTP_REQUESTS_IN_FLIGHT: Lazy<GaugeVec> = Lazy::new(|| {
    crate::register_gauge_vec(
        "http_requests_in_flight",
        "Number of HTTP requests currently being processed",
        &["method", "path"],
    )
    .expect("Failed to register http_requests_in_flight")
});

static HTTP_REQUEST_SIZE_BYTES: Lazy<HistogramVec> = Lazy::new(|| {
    crate::register_histogram_vec_with_buckets(
        "http_request_size_bytes",
        "HTTP request size in bytes",
        &["method", "path"],
        vec![
            100.0,
            1_000.0,
            10_000.0,
            100_000.0,
            1_000_000.0,
            10_000_000.0,
        ],
    )
    .expect("Failed to register http_request_size_bytes")
});

static HTTP_RESPONSE_SIZE_BYTES: Lazy<HistogramVec> = Lazy::new(|| {
    crate::register_histogram_vec_with_buckets(
        "http_response_size_bytes",
        "HTTP response size in bytes",
        &["method", "path", "status"],
        vec![
            100.0,
            1_000.0,
            10_000.0,
            100_000.0,
            1_000_000.0,
            10_000_000.0,
        ],
    )
    .expect("Failed to register http_response_size_bytes")
});

/// Request metrics middleware
///
/// Automatically collects the following metrics:
/// - `http_requests_total` - Total number of requests
/// - `http_request_duration_seconds` - Request duration histogram
/// - `http_requests_in_flight` - Active requests gauge
/// - `http_request_size_bytes` - Request size histogram
/// - `http_response_size_bytes` - Response size histogram
///
/// # Examples
///
/// ```no_run
/// use armature_core::*;
/// use armature_metrics::*;
/// use std::sync::Arc;
///
/// let middleware = Arc::new(RequestMetricsMiddleware::new());
/// ```
pub struct RequestMetricsMiddleware {
    /// Whether to include path in metrics (can lead to high cardinality)
    include_path: bool,
}

impl RequestMetricsMiddleware {
    /// Create new request metrics middleware
    pub fn new() -> Self {
        Self { include_path: true }
    }

    /// Create middleware without path labels (to reduce cardinality)
    pub fn without_path() -> Self {
        Self {
            include_path: false,
        }
    }

    /// Get sanitized path for metrics
    fn sanitize_path(&self, path: &str) -> String {
        if !self.include_path {
            return "/".to_string();
        }

        // Limit path length to prevent cardinality explosion
        if path.len() > 100 {
            return format!("{}...", &path[..97]);
        }

        path.to_string()
    }
}

impl Default for RequestMetricsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Middleware for RequestMetricsMiddleware {
    async fn handle(
        &self,
        request: HttpRequest,
        next: armature_core::middleware::Next,
    ) -> Result<HttpResponse, Error> {
        let start = Instant::now();
        let method = request.method.clone();
        let path = self.sanitize_path(&request.path);
        let request_size = request.body.len() as f64;

        // Record request size
        HTTP_REQUEST_SIZE_BYTES
            .with_label_values(&[&method, &path])
            .observe(request_size);

        // Increment in-flight requests
        HTTP_REQUESTS_IN_FLIGHT
            .with_label_values(&[&method, &path])
            .inc();

        // Process request
        let result = next(request).await;

        // Decrement in-flight requests
        HTTP_REQUESTS_IN_FLIGHT
            .with_label_values(&[&method, &path])
            .dec();

        // Record metrics
        let duration = start.elapsed().as_secs_f64();

        match &result {
            Ok(response) => {
                let status = response.status.to_string();
                let response_size = response.body.len() as f64;

                // Record request count
                HTTP_REQUEST_COUNTER
                    .with_label_values(&[&method, &path, &status])
                    .inc();

                // Record request duration
                HTTP_REQUEST_DURATION
                    .with_label_values(&[&method, &path, &status])
                    .observe(duration);

                // Record response size
                HTTP_RESPONSE_SIZE_BYTES
                    .with_label_values(&[&method, &path, &status])
                    .observe(response_size);
            }
            Err(err) => {
                // Record error
                let status = err.status_code().to_string();

                HTTP_REQUEST_COUNTER
                    .with_label_values(&[&method, &path, &status])
                    .inc();

                HTTP_REQUEST_DURATION
                    .with_label_values(&[&method, &path, &status])
                    .observe(duration);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_metrics_middleware_new() {
        let middleware = RequestMetricsMiddleware::new();
        assert!(middleware.include_path);
    }

    #[test]
    fn test_request_metrics_middleware_without_path() {
        let middleware = RequestMetricsMiddleware::without_path();
        assert!(!middleware.include_path);
    }

    #[test]
    fn test_sanitize_path() {
        let middleware = RequestMetricsMiddleware::new();
        assert_eq!(middleware.sanitize_path("/api/users"), "/api/users");

        let long_path = "/".to_string() + &"a".repeat(150);
        let sanitized = middleware.sanitize_path(&long_path);
        assert!(sanitized.len() <= 103); // 100 + "..."
    }

    #[test]
    fn test_sanitize_path_without() {
        let middleware = RequestMetricsMiddleware::without_path();
        assert_eq!(middleware.sanitize_path("/api/users"), "/");
    }
}
