//! Helper functions for common cache operations.

use crate::error::CacheResult;
use crate::traits::CacheStore;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

/// Get a typed value from the cache.
pub async fn get<S: CacheStore, T: DeserializeOwned>(
    store: &S,
    key: &str,
) -> CacheResult<Option<T>> {
    if let Some(json) = store.get_json(key).await? {
        let value: T = serde_json::from_str(&json)
            .map_err(|e| crate::error::CacheError::Deserialization(e.to_string()))?;
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

/// Set a typed value in the cache.
pub async fn set<S: CacheStore, T: Serialize>(
    store: &S,
    key: &str,
    value: &T,
    ttl: Option<Duration>,
) -> CacheResult<()> {
    let json = serde_json::to_string(value)
        .map_err(|e| crate::error::CacheError::Serialization(e.to_string()))?;
    store.set_json(key, json, ttl).await
}

/// Remember a value for a given duration.
///
/// If the key exists, returns the cached value.
/// If not, calls the factory function, caches the result, and returns it.
pub async fn remember<S: CacheStore, T, F, Fut>(
    store: &S,
    key: &str,
    ttl: Duration,
    factory: F,
) -> CacheResult<T>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = CacheResult<T>>,
{
    if let Some(value) = get(store, key).await? {
        return Ok(value);
    }

    let value = factory().await?;
    set(store, key, &value, Some(ttl)).await?;
    Ok(value)
}

/// Remember a value forever (no TTL).
pub async fn remember_forever<S: CacheStore, T, F, Fut>(
    store: &S,
    key: &str,
    factory: F,
) -> CacheResult<T>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = CacheResult<T>>,
{
    if let Some(value) = get(store, key).await? {
        return Ok(value);
    }

    let value = factory().await?;
    set(store, key, &value, None).await?;
    Ok(value)
}
