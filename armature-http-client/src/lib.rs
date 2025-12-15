//! # Armature HTTP Client
//!
//! A robust HTTP client with built-in retry logic, circuit breaker pattern,
//! timeout management, and request/response interceptors.
//!
//! ## Features
//!
//! - **Retry with Backoff**: Configurable retry strategies (exponential, linear, constant)
//! - **Circuit Breaker**: Prevents cascade failures with automatic recovery
//! - **Timeouts**: Per-request and global timeout configuration
//! - **Interceptors**: Request/response transformation and logging
//! - **Connection Pooling**: Efficient connection reuse
//! - **Compression**: Automatic gzip/brotli support
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use armature_http_client::{HttpClient, HttpClientConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = HttpClient::new(HttpClientConfig::default());
//!
//!     let response = client
//!         .get("https://api.example.com/users")
//!         .send()
//!         .await?;
//!
//!     println!("Status: {}", response.status());
//!     Ok(())
//! }
//! ```
//!
//! ## With Retry and Circuit Breaker
//!
//! ```rust,no_run
//! use armature_http_client::{HttpClient, HttpClientConfig, RetryConfig, CircuitBreakerConfig};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = HttpClientConfig::builder()
//!         .timeout(Duration::from_secs(30))
//!         .retry(RetryConfig::exponential(3, Duration::from_millis(100)))
//!         .circuit_breaker(CircuitBreakerConfig::default())
//!         .build();
//!
//!     let client = HttpClient::new(config);
//!
//!     // Requests will automatically retry on failure
//!     // Circuit breaker will open after consecutive failures
//!     let response = client
//!         .post("https://api.example.com/orders")
//!         .json(&serde_json::json!({"item": "widget", "quantity": 5}))
//!         .send()
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

mod client;
mod config;
mod error;
mod retry;
mod circuit_breaker;
mod request;
mod response;
mod interceptor;
mod middleware;

pub use client::HttpClient;
pub use config::{HttpClientConfig, HttpClientConfigBuilder};
pub use error::{HttpClientError, Result};
pub use retry::{RetryConfig, RetryStrategy, BackoffStrategy};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
pub use request::RequestBuilder;
pub use response::Response;
pub use interceptor::{Interceptor, RequestInterceptor, ResponseInterceptor};
pub use middleware::{Middleware, MiddlewareChain};

// Re-export common types
pub use http::{Method, StatusCode, HeaderMap, HeaderValue, header};
pub use url::Url;
pub use bytes::Bytes;

/// Prelude for common imports.
///
/// ```
/// use armature_http_client::prelude::*;
/// ```
pub mod prelude {
    pub use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
    pub use crate::client::HttpClient;
    pub use crate::config::{HttpClientConfig, HttpClientConfigBuilder};
    pub use crate::error::{HttpClientError, Result};
    pub use crate::interceptor::{Interceptor, RequestInterceptor, ResponseInterceptor};
    pub use crate::middleware::{Middleware, MiddlewareChain};
    pub use crate::request::RequestBuilder;
    pub use crate::response::Response;
    pub use crate::retry::{BackoffStrategy, RetryConfig, RetryStrategy};
    pub use http::{header, HeaderMap, HeaderValue, Method, StatusCode};
}

