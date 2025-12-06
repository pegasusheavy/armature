//! Cache management for Armature framework.
//!
//! Provides a unified interface for working with various cache backends
//! including Redis and Memcached.
//!
//! # Features
//!
//! - `redis` - Enable Redis cache support (enabled by default)
//! - `memcached` - Enable Memcached cache support (enabled by default)
//!
//! # Examples
//!
//! ```no_run
//! use armature_cache::*;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), CacheError> {
//!     // Redis cache
//!     let redis_config = CacheConfig::redis("redis://localhost:6379")?;
//!     let redis_cache = RedisCache::new(redis_config).await?;
//!
//!     redis_cache.set_json("key", "value".to_string(), Some(Duration::from_secs(60))).await?;
//!
//!     // Memcached cache
//!     let memcached_config = CacheConfig::memcached("memcache://localhost:11211")?;
//!     let memcached_cache = MemcachedCache::new(memcached_config).await?;
//!
//!     memcached_cache.set_json("key", "value".to_string(), Some(Duration::from_secs(60))).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod helpers;
pub mod manager;
pub mod parallel;
pub mod traits;

#[cfg(feature = "redis")]
pub mod redis_cache;

#[cfg(feature = "memcached")]
pub mod memcached_cache;

pub use config::CacheConfig;
pub use error::{CacheError, CacheResult};
pub use helpers::*;
pub use manager::CacheManager;
pub use traits::CacheStore;

#[cfg(feature = "redis")]
pub use redis_cache::RedisCache;

#[cfg(feature = "memcached")]
pub use memcached_cache::MemcachedCache;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::config::CacheConfig;
    pub use crate::error::{CacheError, CacheResult};
    pub use crate::manager::CacheManager;
    pub use crate::traits::CacheStore;

    #[cfg(feature = "redis")]
    pub use crate::redis_cache::RedisCache;

    #[cfg(feature = "memcached")]
    pub use crate::memcached_cache::MemcachedCache;
}
