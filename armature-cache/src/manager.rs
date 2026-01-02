//! High-level cache manager with convenience methods.

use crate::error::CacheResult;
use crate::traits::CacheStore;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// High-level cache manager with type-safe operations.
pub struct CacheManager<S: CacheStore> {
    store: Arc<S>,
}

impl<S: CacheStore> CacheManager<S> {
    /// Create a new cache manager.
    pub fn new(store: S) -> Self {
        Self {
            store: Arc::new(store),
        }
    }

    /// Get a typed value from the cache.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> CacheResult<Option<T>> {
        if let Some(json) = self.store.get_json(key).await? {
            let value: T = serde_json::from_str(&json)
                .map_err(|e| crate::error::CacheError::Deserialization(e.to_string()))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Set a typed value in the cache.
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        let json = serde_json::to_string(value)
            .map_err(|e| crate::error::CacheError::Serialization(e.to_string()))?;
        self.store.set_json(key, json, ttl).await
    }

    /// Get or set a value using a factory function.
    ///
    /// If the key exists, returns the cached value.
    /// If not, calls the factory function, caches the result, and returns it.
    pub async fn get_or_set<T, F, Fut>(
        &self,
        key: &str,
        ttl: Option<Duration>,
        factory: F,
    ) -> CacheResult<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = CacheResult<T>>,
    {
        if let Some(value) = self.get(key).await? {
            return Ok(value);
        }

        let value = factory().await?;
        self.set(key, &value, ttl).await?;
        Ok(value)
    }

    /// Delete a key from the cache.
    pub async fn delete(&self, key: &str) -> CacheResult<()> {
        self.store.delete(key).await
    }

    /// Check if a key exists.
    pub async fn exists(&self, key: &str) -> CacheResult<bool> {
        self.store.exists(key).await
    }

    /// Clear all keys.
    pub async fn clear(&self) -> CacheResult<()> {
        self.store.clear().await
    }

    /// Get the TTL of a key.
    pub async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        self.store.ttl(key).await
    }

    /// Set the expiration of a key.
    pub async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<()> {
        self.store.expire(key, ttl).await
    }

    /// Create a namespaced cache manager.
    pub fn namespace(&self, prefix: &str) -> NamespacedCache<S> {
        NamespacedCache {
            store: self.store.clone(),
            prefix: prefix.to_string(),
        }
    }
}

/// Namespaced cache manager that automatically prefixes all keys.
pub struct NamespacedCache<S: CacheStore> {
    store: Arc<S>,
    prefix: String,
}

impl<S: CacheStore> NamespacedCache<S> {
    fn build_key(&self, key: &str) -> String {
        format!("{}:{}", self.prefix, key)
    }

    /// Get a typed value from the cache.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> CacheResult<Option<T>> {
        let key = self.build_key(key);
        if let Some(json) = self.store.get_json(&key).await? {
            let value: T = serde_json::from_str(&json)
                .map_err(|e| crate::error::CacheError::Deserialization(e.to_string()))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Set a typed value in the cache.
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        let key = self.build_key(key);
        let json = serde_json::to_string(value)
            .map_err(|e| crate::error::CacheError::Serialization(e.to_string()))?;
        self.store.set_json(&key, json, ttl).await
    }

    /// Delete a key from the cache.
    pub async fn delete(&self, key: &str) -> CacheResult<()> {
        let key = self.build_key(key);
        self.store.delete(&key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_build_key() {
        struct MockStore;

        #[async_trait::async_trait]
        impl CacheStore for MockStore {
            async fn get_json(&self, _key: &str) -> CacheResult<Option<String>> {
                Ok(None)
            }
            async fn set_json(
                &self,
                _key: &str,
                _value: String,
                _ttl: Option<Duration>,
            ) -> CacheResult<()> {
                Ok(())
            }
            async fn delete(&self, _key: &str) -> CacheResult<()> {
                Ok(())
            }
            async fn exists(&self, _key: &str) -> CacheResult<bool> {
                Ok(false)
            }
            async fn clear(&self) -> CacheResult<()> {
                Ok(())
            }
            async fn ttl(&self, _key: &str) -> CacheResult<Option<Duration>> {
                Ok(None)
            }
            async fn expire(&self, _key: &str, _ttl: Duration) -> CacheResult<()> {
                Ok(())
            }
            async fn increment(&self, _key: &str, _delta: i64) -> CacheResult<i64> {
                Ok(0)
            }
            async fn decrement(&self, _key: &str, _delta: i64) -> CacheResult<i64> {
                Ok(0)
            }
        }

        let namespaced = NamespacedCache {
            store: Arc::new(MockStore),
            prefix: "users".to_string(),
        };

        assert_eq!(namespaced.build_key("123"), "users:123");
    }

    #[test]
    fn test_namespace_build_key_empty() {
        struct MockStore;

        #[async_trait::async_trait]
        impl CacheStore for MockStore {
            async fn get_json(&self, _key: &str) -> CacheResult<Option<String>> {
                Ok(None)
            }
            async fn set_json(
                &self,
                _key: &str,
                _value: String,
                _ttl: Option<Duration>,
            ) -> CacheResult<()> {
                Ok(())
            }
            async fn delete(&self, _key: &str) -> CacheResult<()> {
                Ok(())
            }
            async fn exists(&self, _key: &str) -> CacheResult<bool> {
                Ok(false)
            }
            async fn clear(&self) -> CacheResult<()> {
                Ok(())
            }
            async fn ttl(&self, _key: &str) -> CacheResult<Option<Duration>> {
                Ok(None)
            }
            async fn expire(&self, _key: &str, _ttl: Duration) -> CacheResult<()> {
                Ok(())
            }
            async fn increment(&self, _key: &str, _delta: i64) -> CacheResult<i64> {
                Ok(0)
            }
            async fn decrement(&self, _key: &str, _delta: i64) -> CacheResult<i64> {
                Ok(0)
            }
        }

        let namespaced = NamespacedCache {
            store: Arc::new(MockStore),
            prefix: "app".to_string(),
        };

        assert_eq!(namespaced.build_key(""), "app:");
    }

    #[test]
    fn test_namespace_build_key_with_colons() {
        struct MockStore;

        #[async_trait::async_trait]
        impl CacheStore for MockStore {
            async fn get_json(&self, _key: &str) -> CacheResult<Option<String>> {
                Ok(None)
            }
            async fn set_json(
                &self,
                _key: &str,
                _value: String,
                _ttl: Option<Duration>,
            ) -> CacheResult<()> {
                Ok(())
            }
            async fn delete(&self, _key: &str) -> CacheResult<()> {
                Ok(())
            }
            async fn exists(&self, _key: &str) -> CacheResult<bool> {
                Ok(false)
            }
            async fn clear(&self) -> CacheResult<()> {
                Ok(())
            }
            async fn ttl(&self, _key: &str) -> CacheResult<Option<Duration>> {
                Ok(None)
            }
            async fn expire(&self, _key: &str, _ttl: Duration) -> CacheResult<()> {
                Ok(())
            }
            async fn increment(&self, _key: &str, _delta: i64) -> CacheResult<i64> {
                Ok(0)
            }
            async fn decrement(&self, _key: &str, _delta: i64) -> CacheResult<i64> {
                Ok(0)
            }
        }

        let namespaced = NamespacedCache {
            store: Arc::new(MockStore),
            prefix: "app".to_string(),
        };

        assert_eq!(namespaced.build_key("user:123"), "app:user:123");
    }

    #[test]
    fn test_namespace_multiple_prefixes() {
        struct MockStore;

        #[async_trait::async_trait]
        impl CacheStore for MockStore {
            async fn get_json(&self, _key: &str) -> CacheResult<Option<String>> {
                Ok(None)
            }
            async fn set_json(
                &self,
                _key: &str,
                _value: String,
                _ttl: Option<Duration>,
            ) -> CacheResult<()> {
                Ok(())
            }
            async fn delete(&self, _key: &str) -> CacheResult<()> {
                Ok(())
            }
            async fn exists(&self, _key: &str) -> CacheResult<bool> {
                Ok(false)
            }
            async fn clear(&self) -> CacheResult<()> {
                Ok(())
            }
            async fn ttl(&self, _key: &str) -> CacheResult<Option<Duration>> {
                Ok(None)
            }
            async fn expire(&self, _key: &str, _ttl: Duration) -> CacheResult<()> {
                Ok(())
            }
            async fn increment(&self, _key: &str, _delta: i64) -> CacheResult<i64> {
                Ok(0)
            }
            async fn decrement(&self, _key: &str, _delta: i64) -> CacheResult<i64> {
                Ok(0)
            }
        }

        let ns1 = NamespacedCache {
            store: Arc::new(MockStore),
            prefix: "app1".to_string(),
        };

        let ns2 = NamespacedCache {
            store: Arc::new(MockStore),
            prefix: "app2".to_string(),
        };

        assert_eq!(ns1.build_key("key"), "app1:key");
        assert_eq!(ns2.build_key("key"), "app2:key");
        assert_ne!(ns1.build_key("key"), ns2.build_key("key"));
    }
}
