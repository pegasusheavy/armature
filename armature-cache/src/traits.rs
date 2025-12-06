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

    // ========== Batch Operations (Parallel) ==========

    /// Get multiple keys in parallel.
    ///
    /// This operation fetches multiple cache keys concurrently, significantly
    /// reducing total latency compared to sequential gets.
    ///
    /// # Arguments
    ///
    /// * `keys` - Slice of cache keys to fetch
    ///
    /// # Returns
    ///
    /// Returns a vector of `Option<String>` in the same order as the input keys.
    /// `None` indicates the key was not found.
    ///
    /// # Performance
    ///
    /// - **Sequential:** O(n * network_latency)
    /// - **Parallel:** O(max(network_latencies)) ≈ O(network_latency)
    /// - **Speedup:** 10-100x for network-bound operations
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use armature_cache::*;
    /// # async fn example(cache: &impl CacheStore) -> CacheResult<()> {
    /// // Fetch 100 user profiles in parallel
    /// let keys: Vec<&str> = (1..=100).map(|i| format!("user:{}", i)).collect();
    /// let profiles = cache.get_many(&keys).await?;
    ///
    /// // Sequential: ~1000ms (10ms * 100)
    /// // Parallel:   ~15ms (max of all parallel requests)
    /// # Ok(())
    /// # }
    /// ```
    async fn get_many(&self, keys: &[&str]) -> CacheResult<Vec<Option<String>>> {
        use futures::future::try_join_all;

        let futures = keys.iter().map(|key| self.get_json(key));
        try_join_all(futures).await
    }

    /// Set multiple key-value pairs in parallel.
    ///
    /// # Arguments
    ///
    /// * `items` - Slice of (key, value) tuples
    /// * `ttl` - Optional time-to-live for all keys
    ///
    /// # Performance
    ///
    /// 10-100x faster than sequential sets for network-bound operations.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use armature_cache::*;
    /// # async fn example(cache: &impl CacheStore) -> CacheResult<()> {
    /// use std::time::Duration;
    ///
    /// let items = vec![
    ///     ("user:1", r#"{"name":"Alice"}"#.to_string()),
    ///     ("user:2", r#"{"name":"Bob"}"#.to_string()),
    /// ];
    ///
    /// cache.set_many(&items, Some(Duration::from_secs(3600))).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn set_many(
        &self,
        items: &[(&str, String)],
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        use futures::future::try_join_all;

        let futures = items
            .iter()
            .map(|(key, value)| self.set_json(key, value.clone(), ttl));

        try_join_all(futures).await?;
        Ok(())
    }

    /// Delete multiple keys in parallel.
    ///
    /// # Arguments
    ///
    /// * `keys` - Slice of cache keys to delete
    ///
    /// # Performance
    ///
    /// 10-100x faster than sequential deletes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use armature_cache::*;
    /// # async fn example(cache: &impl CacheStore) -> CacheResult<()> {
    /// // Bulk cache invalidation
    /// let keys = vec!["session:1", "session:2", "session:3"];
    /// cache.delete_many(&keys).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn delete_many(&self, keys: &[&str]) -> CacheResult<()> {
        use futures::future::try_join_all;

        let futures = keys.iter().map(|key| self.delete(key));
        try_join_all(futures).await?;
        Ok(())
    }

    /// Check existence of multiple keys in parallel.
    ///
    /// # Arguments
    ///
    /// * `keys` - Slice of cache keys to check
    ///
    /// # Returns
    ///
    /// Returns a vector of booleans in the same order as input keys.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use armature_cache::*;
    /// # async fn example(cache: &impl CacheStore) -> CacheResult<()> {
    /// let keys = vec!["user:1", "user:2", "user:3"];
    /// let exists = cache.exists_many(&keys).await?;
    ///
    /// for (key, exists) in keys.iter().zip(exists.iter()) {
    ///     println!("{}: {}", key, exists);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn exists_many(&self, keys: &[&str]) -> CacheResult<Vec<bool>> {
        use futures::future::try_join_all;

        let futures = keys.iter().map(|key| self.exists(key));
        try_join_all(futures).await
    }

    /// Get TTL for multiple keys in parallel.
    ///
    /// # Arguments
    ///
    /// * `keys` - Slice of cache keys
    ///
    /// # Returns
    ///
    /// Returns a vector of `Option<Duration>` for each key.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use armature_cache::*;
    /// # async fn example(cache: &impl CacheStore) -> CacheResult<()> {
    /// let keys = vec!["session:1", "session:2"];
    /// let ttls = cache.ttl_many(&keys).await?;
    ///
    /// for (key, ttl) in keys.iter().zip(ttls.iter()) {
    ///     match ttl {
    ///         Some(duration) => println!("{}: expires in {:?}", key, duration),
    ///         None => println!("{}: no expiration", key),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn ttl_many(&self, keys: &[&str]) -> CacheResult<Vec<Option<Duration>>> {
        use futures::future::try_join_all;

        let futures = keys.iter().map(|key| self.ttl(key));
        try_join_all(futures).await
    }
}
