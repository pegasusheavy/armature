//! Rate limit storage backends
//!
//! This module provides different storage backends for rate limiting:
//!
//! - **Memory**: In-memory storage using DashMap (default, single-instance)
//! - **Redis**: Distributed storage for multi-instance deployments

mod memory;
#[cfg(feature = "redis")]
mod redis;

pub use memory::MemoryStore;
#[cfg(feature = "redis")]
pub use redis::RedisStore;

use crate::error::RateLimitResult;
use async_trait::async_trait;
use std::time::Duration;

/// Store type for rate limiting
#[derive(Debug, Clone, Default)]
pub enum StoreType {
    /// In-memory store (single instance only)
    #[default]
    Memory,
    /// Redis store (distributed)
    Redis,
}

/// Trait for rate limit storage backends
#[async_trait]
pub trait RateLimitStore: Send + Sync {
    /// Check and consume a token using token bucket algorithm
    /// Returns (allowed, remaining_tokens)
    async fn token_bucket_check(
        &self,
        key: &str,
        capacity: u64,
        refill_rate: f64,
    ) -> RateLimitResult<(bool, u64)>;

    /// Check and record a request using sliding window log algorithm
    /// Returns (allowed, remaining_requests)
    async fn sliding_window_check(
        &self,
        key: &str,
        max_requests: u64,
        window: Duration,
    ) -> RateLimitResult<(bool, u64)>;

    /// Check and increment counter using fixed window algorithm
    /// Returns (allowed, remaining_requests)
    async fn fixed_window_check(
        &self,
        key: &str,
        max_requests: u64,
        window: Duration,
    ) -> RateLimitResult<(bool, u64)>;

    /// Reset rate limit state for a key
    async fn reset(&self, key: &str) -> RateLimitResult<()>;

    /// Get the current remaining count for a key
    /// This is implementation-specific
    async fn remaining(&self, key: &str) -> RateLimitResult<u64>;

    /// Clean up expired entries (optional, for memory optimization)
    async fn cleanup(&self) -> RateLimitResult<()> {
        Ok(())
    }

    /// Get store type name for debugging
    fn store_type(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_type_default() {
        let store_type = StoreType::default();
        assert!(matches!(store_type, StoreType::Memory));
    }
}
