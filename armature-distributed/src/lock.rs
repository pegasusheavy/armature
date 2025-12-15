//! Distributed locks using Redis

use async_trait::async_trait;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Distributed lock errors
#[derive(Debug, Error)]
pub enum LockError {
    #[error("Failed to acquire lock: {0}")]
    AcquireFailed(String),

    #[error("Failed to release lock: {0}")]
    ReleaseFailed(String),

    #[error("Lock timeout")]
    Timeout,

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Lock not held")]
    NotHeld,
}

/// Distributed lock trait
#[async_trait]
pub trait DistributedLock: Send + Sync {
    /// Acquire the lock
    async fn acquire(&self) -> Result<LockGuard, LockError>;

    /// Try to acquire the lock (non-blocking)
    async fn try_acquire(&self) -> Result<Option<LockGuard>, LockError>;

    /// Acquire with timeout
    async fn acquire_timeout(&self, timeout: Duration) -> Result<LockGuard, LockError>;
}

/// Lock guard that automatically releases on drop
pub struct LockGuard {
    key: String,
    token: String,
    conn: redis::aio::ConnectionManager,
}

impl LockGuard {
    /// Create new lock guard
    fn new(key: String, token: String, conn: redis::aio::ConnectionManager) -> Self {
        Self { key, token, conn }
    }

    /// Manually release the lock
    pub async fn release(mut self) -> Result<(), LockError> {
        self.release_internal().await
    }

    async fn release_internal(&mut self) -> Result<(), LockError> {
        // Use Lua script for atomic release
        let script = r#"
            if redis.call("get", KEYS[1]) == ARGV[1] then
                return redis.call("del", KEYS[1])
            else
                return 0
            end
        "#;

        let result: i32 = redis::Script::new(script)
            .key(&self.key)
            .arg(&self.token)
            .invoke_async(&mut self.conn)
            .await?;

        if result == 1 {
            debug!("Released lock: {}", self.key);
            Ok(())
        } else {
            warn!("Failed to release lock (not held or expired): {}", self.key);
            Err(LockError::NotHeld)
        }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // Best effort release on drop
        let key = self.key.clone();
        let token = self.token.clone();
        let mut conn = self.conn.clone();

        tokio::spawn(async move {
            let script = r#"
                if redis.call("get", KEYS[1]) == ARGV[1] then
                    return redis.call("del", KEYS[1])
                else
                    return 0
                end
            "#;

            let _: Result<i32, _> = redis::Script::new(script)
                .key(&key)
                .arg(&token)
                .invoke_async(&mut conn)
                .await;
        });
    }
}

/// Redis-based distributed lock
pub struct RedisLock {
    key: String,
    ttl: Duration,
    conn: redis::aio::ConnectionManager,
}

impl RedisLock {
    /// Create new Redis lock
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use armature_distributed::RedisLock;
    /// use std::time::Duration;
    ///
    /// let client = redis::Client::open("redis://127.0.0.1/")?;
    /// let conn = client.get_connection_manager().await?;
    /// let lock = RedisLock::new("my-resource", Duration::from_secs(30), conn);
    /// ```
    pub fn new(key: impl Into<String>, ttl: Duration, conn: redis::aio::ConnectionManager) -> Self {
        Self {
            key: key.into(),
            ttl,
            conn,
        }
    }

    /// Get the lock key
    pub fn key(&self) -> &str {
        &self.key
    }
}

#[async_trait]
impl DistributedLock for RedisLock {
    async fn acquire(&self) -> Result<LockGuard, LockError> {
        loop {
            match self.try_acquire().await? {
                Some(guard) => return Ok(guard),
                None => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    async fn try_acquire(&self) -> Result<Option<LockGuard>, LockError> {
        let token = Uuid::new_v4().to_string();
        let ttl_ms = self.ttl.as_millis() as usize;

        let mut conn = self.conn.clone();

        // Use SET NX PX for atomic acquire with TTL
        let result: Option<String> = redis::cmd("SET")
            .arg(&self.key)
            .arg(&token)
            .arg("NX")  // Only set if not exists
            .arg("PX")  // Set expiry in milliseconds
            .arg(ttl_ms)
            .query_async(&mut conn)
            .await?;

        if result.is_some() {
            info!("Acquired lock: {}", self.key);
            Ok(Some(LockGuard::new(
                self.key.clone(),
                token,
                conn,
            )))
        } else {
            debug!("Failed to acquire lock (already held): {}", self.key);
            Ok(None)
        }
    }

    async fn acquire_timeout(&self, timeout: Duration) -> Result<LockGuard, LockError> {
        let start = tokio::time::Instant::now();

        loop {
            match self.try_acquire().await? {
                Some(guard) => return Ok(guard),
                None => {
                    if start.elapsed() >= timeout {
                        return Err(LockError::Timeout);
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
}

/// Distributed lock builder
pub struct LockBuilder {
    key: String,
    ttl: Duration,
}

impl LockBuilder {
    /// Create new lock builder
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            ttl: Duration::from_secs(30),
        }
    }

    /// Set TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Build the lock
    pub fn build(self, conn: redis::aio::ConnectionManager) -> RedisLock {
        RedisLock::new(self.key, self.ttl, conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_builder() {
        let builder = LockBuilder::new("test-lock")
            .with_ttl(Duration::from_secs(60));

        assert_eq!(builder.key, "test-lock");
        assert_eq!(builder.ttl, Duration::from_secs(60));
    }
}

