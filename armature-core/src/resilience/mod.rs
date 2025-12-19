//! # Resilience Patterns
//!
//! Production-ready resilience patterns for building fault-tolerant applications.
//!
//! ## Patterns Included
//!
//! - **Circuit Breaker**: Prevent cascade failures by failing fast
//! - **Retry**: Automatic retry with configurable backoff strategies
//! - **Bulkhead**: Resource isolation to prevent overload
//! - **Timeout**: Configurable timeout policies
//! - **Fallback**: Graceful degradation when operations fail
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use armature::prelude::*;
//! use armature::resilience::{CircuitBreaker, CircuitBreakerConfig, Retry, RetryConfig};
//!
//! // Circuit breaker that opens after 5 failures
//! let circuit = CircuitBreaker::new(CircuitBreakerConfig {
//!     failure_threshold: 5,
//!     reset_timeout: Duration::from_secs(30),
//!     ..Default::default()
//! });
//!
//! // Execute with circuit breaker protection
//! let result = circuit.call(|| async {
//!     external_api_call().await
//! }).await;
//! ```

mod bulkhead;
mod circuit_breaker;
mod fallback;
mod retry;
mod timeout;

pub use bulkhead::*;
pub use circuit_breaker::*;
pub use fallback::*;
pub use retry::*;
pub use timeout::*;
