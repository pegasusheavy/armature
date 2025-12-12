//! In-memory rate limit store
//!
//! Uses DashMap for thread-safe concurrent access. Suitable for single-instance
//! deployments or testing. For distributed deployments, use the Redis store.

use crate::error::RateLimitResult;
use crate::stores::RateLimitStore;
use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tracing::{debug, trace};

/// Token bucket state
#[derive(Debug, Clone)]
struct TokenBucketState {
    tokens: f64,
    last_refill: Instant,
}

/// Fixed window state
#[derive(Debug, Clone)]
struct FixedWindowState {
    count: u64,
    window_start: Instant,
}

/// In-memory rate limit store
pub struct MemoryStore {
    /// Token bucket states
    token_buckets: DashMap<String, TokenBucketState>,
    /// Sliding window logs
    sliding_logs: DashMap<String, VecDeque<Instant>>,
    /// Fixed window states
    fixed_windows: DashMap<String, FixedWindowState>,
}

impl MemoryStore {
    /// Create a new in-memory store
    pub fn new() -> Self {
        debug!("Creating new in-memory rate limit store");
        Self {
            token_buckets: DashMap::new(),
            sliding_logs: DashMap::new(),
            fixed_windows: DashMap::new(),
        }
    }

    /// Get the number of tracked keys (for monitoring)
    pub fn key_count(&self) -> usize {
        self.token_buckets.len() + self.sliding_logs.len() + self.fixed_windows.len()
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RateLimitStore for MemoryStore {
    async fn token_bucket_check(
        &self,
        key: &str,
        capacity: u64,
        refill_rate: f64,
    ) -> RateLimitResult<(bool, u64)> {
        trace!(key = %key, capacity = capacity, refill_rate = refill_rate, "Token bucket check");

        let now = Instant::now();

        let mut entry = self
            .token_buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucketState {
                tokens: capacity as f64,
                last_refill: now,
            });

        // Refill based on elapsed time
        let elapsed = now.duration_since(entry.last_refill).as_secs_f64();
        let new_tokens = elapsed * refill_rate;
        entry.tokens = (entry.tokens + new_tokens).min(capacity as f64);
        entry.last_refill = now;

        if entry.tokens >= 1.0 {
            entry.tokens -= 1.0;
            let remaining = entry.tokens as u64;
            trace!(key = %key, remaining = remaining, "Token bucket: allowed");
            Ok((true, remaining))
        } else {
            trace!(key = %key, "Token bucket: denied");
            Ok((false, 0))
        }
    }

    async fn sliding_window_check(
        &self,
        key: &str,
        max_requests: u64,
        window: Duration,
    ) -> RateLimitResult<(bool, u64)> {
        trace!(key = %key, max_requests = max_requests, window = ?window, "Sliding window check");

        let now = Instant::now();
        let cutoff = now - window;

        let mut entry = self.sliding_logs.entry(key.to_string()).or_default();

        // Remove old timestamps
        while let Some(front) = entry.front() {
            if *front < cutoff {
                entry.pop_front();
            } else {
                break;
            }
        }

        let current_count = entry.len() as u64;

        if current_count < max_requests {
            entry.push_back(now);
            let remaining = max_requests - current_count - 1;
            trace!(key = %key, remaining = remaining, "Sliding window: allowed");
            Ok((true, remaining))
        } else {
            trace!(key = %key, "Sliding window: denied");
            Ok((false, 0))
        }
    }

    async fn fixed_window_check(
        &self,
        key: &str,
        max_requests: u64,
        window: Duration,
    ) -> RateLimitResult<(bool, u64)> {
        trace!(key = %key, max_requests = max_requests, window = ?window, "Fixed window check");

        let now = Instant::now();

        let mut entry = self
            .fixed_windows
            .entry(key.to_string())
            .or_insert_with(|| FixedWindowState {
                count: 0,
                window_start: now,
            });

        // Check if we're in a new window
        let elapsed = now.duration_since(entry.window_start);
        if elapsed >= window {
            entry.count = 0;
            entry.window_start = now;
        }

        if entry.count < max_requests {
            entry.count += 1;
            let remaining = max_requests - entry.count;
            trace!(key = %key, remaining = remaining, "Fixed window: allowed");
            Ok((true, remaining))
        } else {
            trace!(key = %key, "Fixed window: denied");
            Ok((false, 0))
        }
    }

    async fn reset(&self, key: &str) -> RateLimitResult<()> {
        debug!(key = %key, "Resetting rate limit state");
        self.token_buckets.remove(key);
        self.sliding_logs.remove(key);
        self.fixed_windows.remove(key);
        Ok(())
    }

    async fn remaining(&self, key: &str) -> RateLimitResult<u64> {
        // This is algorithm-specific; return 0 as a default
        // In practice, callers should use the algorithm-specific methods
        if let Some(entry) = self.token_buckets.get(key) {
            return Ok(entry.tokens as u64);
        }
        Ok(0)
    }

    async fn cleanup(&self) -> RateLimitResult<()> {
        debug!("Cleaning up expired entries");

        // For token buckets, we keep them indefinitely (they self-refill)

        // For sliding window, clean entries with no recent requests
        let now = Instant::now();
        let old_cutoff = now - Duration::from_secs(3600); // 1 hour

        self.sliding_logs.retain(|_, logs| {
            if let Some(last) = logs.back() {
                *last > old_cutoff
            } else {
                false
            }
        });

        // For fixed window, clean old windows
        self.fixed_windows
            .retain(|_, state| now.duration_since(state.window_start) < Duration::from_secs(3600));

        debug!(key_count = self.key_count(), "Cleanup complete");

        Ok(())
    }

    fn store_type(&self) -> &'static str {
        "memory"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_bucket() {
        let store = MemoryStore::new();

        // First 5 requests should be allowed
        for i in (0..5).rev() {
            let (allowed, remaining) = store.token_bucket_check("test", 5, 1.0).await.unwrap();
            assert!(allowed);
            assert_eq!(remaining, i);
        }

        // 6th should be denied
        let (allowed, _) = store.token_bucket_check("test", 5, 1.0).await.unwrap();
        assert!(!allowed);
    }

    #[tokio::test]
    async fn test_sliding_window() {
        let store = MemoryStore::new();
        let window = Duration::from_secs(60);

        // First 3 requests should be allowed
        for i in (0..3).rev() {
            let (allowed, remaining) = store.sliding_window_check("test", 3, window).await.unwrap();
            assert!(allowed);
            assert_eq!(remaining, i);
        }

        // 4th should be denied
        let (allowed, _) = store.sliding_window_check("test", 3, window).await.unwrap();
        assert!(!allowed);
    }

    #[tokio::test]
    async fn test_fixed_window() {
        let store = MemoryStore::new();
        let window = Duration::from_secs(60);

        // First 3 requests should be allowed
        for i in (0..3).rev() {
            let (allowed, remaining) = store.fixed_window_check("test", 3, window).await.unwrap();
            assert!(allowed);
            assert_eq!(remaining, i);
        }

        // 4th should be denied
        let (allowed, _) = store.fixed_window_check("test", 3, window).await.unwrap();
        assert!(!allowed);
    }

    #[tokio::test]
    async fn test_reset() {
        let store = MemoryStore::new();

        // Use up token bucket
        store.token_bucket_check("test", 1, 0.001).await.unwrap();
        let (allowed, _) = store.token_bucket_check("test", 1, 0.001).await.unwrap();
        assert!(!allowed);

        // Reset
        store.reset("test").await.unwrap();

        // Should be allowed again
        let (allowed, _) = store.token_bucket_check("test", 1, 0.001).await.unwrap();
        assert!(allowed);
    }

    #[tokio::test]
    async fn test_different_keys() {
        let store = MemoryStore::new();

        // Exhaust key1
        store.token_bucket_check("key1", 1, 0.001).await.unwrap();
        let (allowed, _) = store.token_bucket_check("key1", 1, 0.001).await.unwrap();
        assert!(!allowed);

        // key2 should still work
        let (allowed, _) = store.token_bucket_check("key2", 1, 0.001).await.unwrap();
        assert!(allowed);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let store = MemoryStore::new();

        // Add some entries
        store.token_bucket_check("test", 10, 1.0).await.unwrap();
        store
            .sliding_window_check("test", 10, Duration::from_secs(60))
            .await
            .unwrap();
        store
            .fixed_window_check("test", 10, Duration::from_secs(60))
            .await
            .unwrap();

        assert!(store.key_count() > 0);

        // Cleanup (won't remove recent entries)
        store.cleanup().await.unwrap();

        // Entries should still exist (they're recent)
        assert!(store.key_count() > 0);
    }

    #[test]
    fn test_store_type() {
        let store = MemoryStore::new();
        assert_eq!(store.store_type(), "memory");
    }
}
