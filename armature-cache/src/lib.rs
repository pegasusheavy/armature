//! Cache management for Armature framework.
//!
//! Provides a unified interface for working with various cache backends
//! including Redis and Memcached, with advanced features like tag-based
//! invalidation and multi-tier caching.
//!
//! # Features
//!
//! - `redis` - Enable Redis cache support (enabled by default)
//! - `memcached` - Enable Memcached cache support (requires explicit opt-in)
//! - **Tag-based invalidation** - Invalidate multiple cache entries by tag
//! - **Multi-tier caching** - L1 (in-memory) + L2 (distributed) layers
//! - **Cache decorators** - `#[cache]` attribute for automatic caching
//!
//! # Examples
//!
//! ## Redis Cache
//!
//! ```no_run
//! use armature_cache::*;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), CacheError> {
//!     let redis_config = CacheConfig::redis("redis://localhost:6379")?;
//!     let redis_cache = RedisCache::new(redis_config).await?;
//!
//!     redis_cache.set_json("key", "value".to_string(), Some(Duration::from_secs(60))).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Tag-based Invalidation
//!
//! ```no_run
//! use armature_cache::*;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), CacheError> {
//! # let redis_cache = Arc::new(InMemoryCache::new());
//! let tagged = TaggedCache::new(redis_cache);
//!
//! // Set with tags
//! tagged.set_with_tags(
//!     "user:123",
//!     r#"{"name":"Alice"}"#.to_string(),
//!     &["users", "active-users"],
//!     None,
//! ).await?;
//!
//! // Invalidate all entries with "users" tag
//! tagged.invalidate_tag("users").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Multi-tier Caching
//!
//! ```no_run
//! use armature_cache::*;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), CacheError> {
//! # let redis_cache = Arc::new(InMemoryCache::new());
//! let l1 = Arc::new(InMemoryCache::new());
//! let l2 = redis_cache;
//!
//! let tiered = TieredCache::new(l1, l2);
//!
//! // Automatically uses L1 (fast) and falls back to L2
//! tiered.set("key", "value".to_string(), None).await?;
//! let value = tiered.get("key").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Memcached Cache (requires `memcached` feature)
//!
//! ```ignore
//! use armature_cache::*;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), CacheError> {
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
pub mod invalidation;
pub mod manager;
pub mod parallel;
pub mod tiered;
pub mod traits;

#[cfg(feature = "redis")]
pub mod redis_cache;

#[cfg(feature = "memcached")]
pub mod memcached_cache;

pub use config::CacheConfig;
pub use error::{CacheError, CacheResult};
pub use helpers::*;
pub use invalidation::TaggedCache;
pub use manager::CacheManager;
pub use tiered::{InMemoryCache, TieredCache, TieredCacheConfig};
pub use traits::CacheStore;

#[cfg(feature = "redis")]
pub use redis_cache::RedisCache;

#[cfg(feature = "memcached")]
pub use memcached_cache::MemcachedCache;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::config::CacheConfig;
    pub use crate::error::{CacheError, CacheResult};
    pub use crate::invalidation::TaggedCache;
    pub use crate::manager::CacheManager;
    pub use crate::tiered::{InMemoryCache, TieredCache, TieredCacheConfig};
    pub use crate::traits::CacheStore;

    #[cfg(feature = "redis")]
    pub use crate::redis_cache::RedisCache;

    #[cfg(feature = "memcached")]
    pub use crate::memcached_cache::MemcachedCache;
}
