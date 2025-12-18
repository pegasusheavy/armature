//! Redis connection pool.

use bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;
use redis::aio::MultiplexedConnection;
use std::ops::{Deref, DerefMut};
use tracing::info;

use crate::{RedisConfig, RedisError, Result};

/// Type alias for the connection pool.
pub type RedisPool = Pool<RedisConnectionManager>;

/// A pooled Redis connection.
pub struct RedisConnection<'a> {
    conn: PooledConnection<'a, RedisConnectionManager>,
}

impl<'a> RedisConnection<'a> {
    /// Create a new connection wrapper.
    pub fn new(conn: PooledConnection<'a, RedisConnectionManager>) -> Self {
        Self { conn }
    }
}

impl<'a> Deref for RedisConnection<'a> {
    type Target = MultiplexedConnection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl<'a> DerefMut for RedisConnection<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
}

/// Builder for creating Redis connection pools.
pub struct RedisPoolBuilder {
    config: RedisConfig,
}

impl RedisPoolBuilder {
    /// Create a new pool builder.
    pub fn new(config: RedisConfig) -> Self {
        Self { config }
    }

    /// Build the connection pool.
    pub async fn build(self) -> Result<RedisPool> {
        let url = self.config.connection_url();

        let manager = RedisConnectionManager::new(url.clone())
            .map_err(|e| RedisError::Connection(e.to_string()))?;

        let pool = Pool::builder()
            .max_size(self.config.pool_size)
            .min_idle(self.config.min_idle)
            .connection_timeout(self.config.connection_timeout)
            .build(manager)
            .await
            .map_err(|e| RedisError::Pool(e.to_string()))?;

        // Test the connection in a scope so the connection is dropped before returning pool
        {
            let mut conn = pool
                .get()
                .await
                .map_err(|e| RedisError::Pool(e.to_string()))?;
            let _: String = redis::cmd("PING")
                .query_async(&mut *conn)
                .await
                .map_err(|e| RedisError::Connection(e.to_string()))?;
        }

        info!(
            pool_size = self.config.pool_size,
            url = %self.config.url,
            "Redis connection pool created"
        );

        Ok(pool)
    }
}
