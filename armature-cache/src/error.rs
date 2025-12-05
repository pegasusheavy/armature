//! Error types for cache operations.

use thiserror::Error;

/// Result type for cache operations.
pub type CacheResult<T> = Result<T, CacheError>;

/// Cache-specific errors.
#[derive(Debug, Error)]
pub enum CacheError {
    /// Redis-specific error
    #[cfg(feature = "redis")]
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Memcached-specific error
    #[cfg(feature = "memcached")]
    #[error("Memcached error: {0}")]
    Memcached(#[from] memcache::MemcacheError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Key not found
    #[error("Key not found: {0}")]
    NotFound(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Operation timeout
    #[error("Operation timeout")]
    Timeout,

    /// Generic error
    #[error("Cache error: {0}")]
    Other(String),
}
