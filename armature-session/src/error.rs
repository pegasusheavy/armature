//! Error types for session operations.

use thiserror::Error;

/// Result type for session operations.
pub type SessionResult<T> = Result<T, SessionError>;

/// Session-specific errors.
#[derive(Debug, Error)]
pub enum SessionError {
    /// Redis-specific error
    #[cfg(feature = "redis")]
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Memcached-specific error
    #[cfg(feature = "memcached")]
    #[error("Memcached error: {0}")]
    Memcached(#[from] memcache::MemcacheError),

    /// CouchDB-specific error
    #[cfg(feature = "couchdb")]
    #[error("CouchDB error: {0}")]
    CouchDb(String),

    /// HTTP request error (CouchDB)
    #[cfg(feature = "couchdb")]
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Session not found
    #[error("Session not found: {0}")]
    NotFound(String),

    /// Session expired
    #[error("Session expired: {0}")]
    Expired(String),

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

    /// Invalid session ID
    #[error("Invalid session ID: {0}")]
    InvalidSessionId(String),

    /// Generic error
    #[error("Session error: {0}")]
    Other(String),
}

