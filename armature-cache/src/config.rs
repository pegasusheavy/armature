//! Cache configuration types.

use crate::error::CacheResult;
use std::time::Duration;

/// Cache backend type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheBackend {
    /// Redis backend
    Redis,
    /// Memcached backend
    Memcached,
}

/// Cache configuration.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Cache backend type
    pub backend: CacheBackend,

    /// Connection URL
    pub url: String,

    /// Key prefix for all cache keys
    pub key_prefix: Option<String>,

    /// Default TTL for cache entries
    pub default_ttl: Option<Duration>,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Operation timeout
    pub operation_timeout: Duration,

    /// Maximum number of connections (for connection pools)
    pub max_connections: usize,
}

impl CacheConfig {
    /// Create a new Redis cache configuration.
    ///
    /// # Arguments
    ///
    /// * `url` - Redis connection URL (e.g., "redis://localhost:6379")
    ///
    /// # Examples
    ///
    /// ```
    /// use armature_cache::CacheConfig;
    ///
    /// let config = CacheConfig::redis("redis://localhost:6379").unwrap();
    /// ```
    pub fn redis(url: impl Into<String>) -> CacheResult<Self> {
        Ok(Self {
            backend: CacheBackend::Redis,
            url: url.into(),
            key_prefix: None,
            default_ttl: None,
            connection_timeout: Duration::from_secs(5),
            operation_timeout: Duration::from_secs(3),
            max_connections: 10,
        })
    }

    /// Create a new Memcached cache configuration.
    ///
    /// # Arguments
    ///
    /// * `url` - Memcached connection URL (e.g., "memcache://localhost:11211")
    ///
    /// # Examples
    ///
    /// ```
    /// use armature_cache::CacheConfig;
    ///
    /// let config = CacheConfig::memcached("memcache://localhost:11211").unwrap();
    /// ```
    pub fn memcached(url: impl Into<String>) -> CacheResult<Self> {
        Ok(Self {
            backend: CacheBackend::Memcached,
            url: url.into(),
            key_prefix: None,
            default_ttl: None,
            connection_timeout: Duration::from_secs(5),
            operation_timeout: Duration::from_secs(3),
            max_connections: 10,
        })
    }

    /// Set the key prefix.
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }

    /// Set the default TTL.
    pub fn with_default_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = Some(ttl);
        self
    }

    /// Set the connection timeout.
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Set the operation timeout.
    pub fn with_operation_timeout(mut self, timeout: Duration) -> Self {
        self.operation_timeout = timeout;
        self
    }

    /// Set the maximum number of connections.
    pub fn with_max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    /// Build the final key with prefix if configured.
    pub fn build_key(&self, key: &str) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{}:{}", prefix, key),
            None => key.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_config() {
        let config = CacheConfig::redis("redis://localhost:6379").unwrap();
        assert_eq!(config.backend, CacheBackend::Redis);
        assert_eq!(config.url, "redis://localhost:6379");
    }

    #[test]
    fn test_memcached_config() {
        let config = CacheConfig::memcached("memcache://localhost:11211").unwrap();
        assert_eq!(config.backend, CacheBackend::Memcached);
        assert_eq!(config.url, "memcache://localhost:11211");
    }

    #[test]
    fn test_config_builder() {
        let config = CacheConfig::redis("redis://localhost:6379")
            .unwrap()
            .with_key_prefix("app")
            .with_default_ttl(Duration::from_secs(300))
            .with_max_connections(20);

        assert_eq!(config.key_prefix, Some("app".to_string()));
        assert_eq!(config.default_ttl, Some(Duration::from_secs(300)));
        assert_eq!(config.max_connections, 20);
    }

    #[test]
    fn test_build_key_with_prefix() {
        let config = CacheConfig::redis("redis://localhost:6379")
            .unwrap()
            .with_key_prefix("myapp");

        assert_eq!(config.build_key("user:123"), "myapp:user:123");
    }

    #[test]
    fn test_build_key_without_prefix() {
        let config = CacheConfig::redis("redis://localhost:6379").unwrap();
        assert_eq!(config.build_key("user:123"), "user:123");
    }
}
