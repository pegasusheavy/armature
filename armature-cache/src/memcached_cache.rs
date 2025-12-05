//! Memcached cache implementation.

use crate::config::CacheConfig;
use crate::error::{CacheError, CacheResult};
use crate::traits::CacheStore;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Memcached cache store.
///
/// Note: The `memcache` crate doesn't have native async support,
/// so we wrap it with tokio's Mutex and use spawn_blocking for operations.
#[derive(Clone)]
pub struct MemcachedCache {
    client: Arc<Mutex<memcache::Client>>,
    config: CacheConfig,
}

impl MemcachedCache {
    /// Create a new Memcached cache instance.
    ///
    /// # Arguments
    ///
    /// * `config` - Cache configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use armature_cache::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), CacheError> {
    ///     let config = CacheConfig::memcached("memcache://localhost:11211")?;
    ///     let cache = MemcachedCache::new(config).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(config: CacheConfig) -> CacheResult<Self> {
        // Parse the URL to extract the server address
        let url = config.url.clone();
        let server_url = Self::parse_memcached_url(&url)?;

        // Create client in blocking context
        let client = tokio::task::spawn_blocking(move || memcache::connect(server_url.as_str()))
            .await
            .map_err(|e| CacheError::Connection(format!("Failed to spawn task: {}", e)))?
            .map_err(|e| CacheError::Connection(format!("Failed to connect: {}", e)))?;

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            config,
        })
    }

    /// Parse Memcached URL to extract server address.
    ///
    /// Converts "memcache://localhost:11211" to "memcache://localhost:11211"
    /// or handles plain "localhost:11211" format.
    fn parse_memcached_url(url: &str) -> CacheResult<String> {
        if url.starts_with("memcache://") {
            Ok(url.to_string())
        } else if url.contains(':') {
            Ok(format!("memcache://{}", url))
        } else {
            Err(CacheError::InvalidUrl(format!(
                "Invalid Memcached URL: {}. Expected format: 'memcache://host:port' or 'host:port'",
                url
            )))
        }
    }

    /// Build the full key with prefix.
    fn build_key(&self, key: &str) -> String {
        self.config.build_key(key)
    }

    /// Convert Duration to Memcached expiration (in seconds).
    fn duration_to_expiration(ttl: Option<Duration>) -> u32 {
        ttl.map(|d| d.as_secs() as u32).unwrap_or(0)
    }
}

#[async_trait]
impl CacheStore for MemcachedCache {
    async fn get_json(&self, key: &str) -> CacheResult<Option<String>> {
        let key = self.build_key(key);
        let client = self.client.clone();

        let result = tokio::task::spawn_blocking(move || {
            let client = client.blocking_lock();
            client.get::<String>(&key)
        })
        .await
        .map_err(|e| CacheError::Other(format!("Task join error: {}", e)))?;

        match result {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) | Err(_) => Ok(None),
        }
    }

    async fn set_json(&self, key: &str, value: String, ttl: Option<Duration>) -> CacheResult<()> {
        let key = self.build_key(key);
        let client = self.client.clone();
        let ttl = ttl.or(self.config.default_ttl);
        let expiration = Self::duration_to_expiration(ttl);

        tokio::task::spawn_blocking(move || {
            let client = client.blocking_lock();
            client.set(&key, value, expiration)
        })
        .await
        .map_err(|e| CacheError::Other(format!("Task join error: {}", e)))??;

        Ok(())
    }

    async fn delete(&self, key: &str) -> CacheResult<()> {
        let key = self.build_key(key);
        let client = self.client.clone();

        tokio::task::spawn_blocking(move || {
            let client = client.blocking_lock();
            client.delete(&key)
        })
        .await
        .map_err(|e| CacheError::Other(format!("Task join error: {}", e)))??;

        Ok(())
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        // Memcached doesn't have a native "exists" command
        // We check by trying to get the key
        let result = self.get_json(key).await?;
        Ok(result.is_some())
    }

    async fn clear(&self) -> CacheResult<()> {
        let client = self.client.clone();

        tokio::task::spawn_blocking(move || {
            let client = client.blocking_lock();
            client.flush()
        })
        .await
        .map_err(|e| CacheError::Other(format!("Task join error: {}", e)))??;

        Ok(())
    }

    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        // Memcached doesn't support TTL retrieval
        // This is a limitation of the protocol
        let _ = key;
        Ok(None)
    }

    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<()> {
        // Memcached doesn't support updating expiration without resetting the value
        // We need to get the value first, then set it again with new TTL
        let value = self.get_json(key).await?;

        if let Some(value) = value {
            self.set_json(key, value, Some(ttl)).await?;
            Ok(())
        } else {
            Err(CacheError::NotFound(key.to_string()))
        }
    }

    async fn increment(&self, key: &str, delta: i64) -> CacheResult<i64> {
        let key = self.build_key(key);
        let key_clone = key.clone();
        let client = self.client.clone();

        if delta >= 0 {
            let delta = delta as u64;
            tokio::task::spawn_blocking(move || {
                let client = client.blocking_lock();
                client.increment(&key, delta)
            })
            .await
            .map_err(|e| CacheError::Other(format!("Task join error: {}", e)))??;
        } else {
            let delta = (-delta) as u64;
            tokio::task::spawn_blocking(move || {
                let client = client.blocking_lock();
                client.decrement(&key, delta)
            })
            .await
            .map_err(|e| CacheError::Other(format!("Task join error: {}", e)))??;
        }

        // Get the new value
        let new_value = self
            .get_json(&key_clone)
            .await?
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(delta.abs());

        Ok(new_value)
    }

    async fn decrement(&self, key: &str, delta: i64) -> CacheResult<i64> {
        self.increment(key, -delta).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_memcached_url() {
        assert_eq!(
            MemcachedCache::parse_memcached_url("memcache://localhost:11211").unwrap(),
            "memcache://localhost:11211"
        );

        assert_eq!(
            MemcachedCache::parse_memcached_url("localhost:11211").unwrap(),
            "memcache://localhost:11211"
        );

        assert!(MemcachedCache::parse_memcached_url("invalid").is_err());
    }

    #[test]
    fn test_duration_to_expiration() {
        assert_eq!(MemcachedCache::duration_to_expiration(None), 0);
        assert_eq!(
            MemcachedCache::duration_to_expiration(Some(Duration::from_secs(60))),
            60
        );
    }
}
