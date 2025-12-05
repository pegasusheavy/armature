//! Cache store trait definition.

use crate::error::CacheResult;
use async_trait::async_trait;
use std::time::Duration;

/// Cache store trait for different cache backends.
#[async_trait]
pub trait CacheStore: Send + Sync {
    /// Get a JSON value from the cache.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(value))` if the key exists, `Ok(None)` if not found,
    /// or an error if the operation fails.
    async fn get_json(&self, key: &str) -> CacheResult<Option<String>>;

    /// Set a JSON value in the cache.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key
    /// * `value` - The JSON string value
    /// * `ttl` - Optional time-to-live duration
    async fn set_json(&self, key: &str, value: String, ttl: Option<Duration>) -> CacheResult<()>;

    /// Delete a key from the cache.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key to delete
    async fn delete(&self, key: &str) -> CacheResult<()>;

    /// Check if a key exists in the cache.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key to check
    async fn exists(&self, key: &str) -> CacheResult<bool>;

    /// Clear all keys from the cache.
    ///
    /// **Warning:** This operation may be destructive and affect all keys.
    async fn clear(&self) -> CacheResult<()>;

    /// Get the TTL (time-to-live) of a key.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(duration))` if the key has a TTL, `Ok(None)` if the key
    /// has no expiration or doesn't exist.
    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>>;

    /// Set or update the expiration time for a key.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key
    /// * `ttl` - The new time-to-live duration
    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<()>;

    /// Increment a numeric value.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key
    /// * `delta` - The amount to increment by
    ///
    /// # Returns
    ///
    /// Returns the new value after incrementing.
    async fn increment(&self, key: &str, delta: i64) -> CacheResult<i64>;

    /// Decrement a numeric value.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key
    /// * `delta` - The amount to decrement by
    ///
    /// # Returns
    ///
    /// Returns the new value after decrementing.
    async fn decrement(&self, key: &str, delta: i64) -> CacheResult<i64>;
}
