//! # Armature Redis
//!
//! Redis client integration with connection pooling and dependency injection.
//!
//! ## Features
//!
//! - **Connection Pooling**: Efficient connection management with bb8
//! - **Pub/Sub**: Redis pub/sub messaging
//! - **Cluster Support**: Redis Cluster integration
//! - **DI Integration**: Register Redis in your application's DI container
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use armature_redis::{RedisService, RedisConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Configure Redis
//!     let config = RedisConfig::builder()
//!         .url("redis://localhost:6379")
//!         .pool_size(10)
//!         .build();
//!
//!     // Create service (DI-ready)
//!     let redis = RedisService::new(config).await?;
//!
//!     // Get a connection from the pool
//!     let mut conn = redis.get().await?;
//!
//!     // Use Redis commands
//!     redis::cmd("SET")
//!         .arg("key")
//!         .arg("value")
//!         .query_async(&mut *conn)
//!         .await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## With Dependency Injection
//!
//! ```rust,ignore
//! use armature::prelude::*;
//! use armature_redis::{RedisService, RedisConfig};
//!
//! #[module]
//! struct RedisModule;
//!
//! #[module_impl]
//! impl RedisModule {
//!     #[provider(singleton)]
//!     async fn redis_service() -> Arc<RedisService> {
//!         let config = RedisConfig::from_env().build();
//!         Arc::new(RedisService::new(config).await.unwrap())
//!     }
//! }
//!
//! // In other crates that need Redis:
//! #[controller("/data")]
//! struct DataController;
//!
//! #[controller_impl]
//! impl DataController {
//!     #[get("/:key")]
//!     async fn get_data(
//!         &self,
//!         #[inject] redis: Arc<RedisService>,
//!         key: Path<String>,
//!     ) -> Result<Json<Value>, HttpError> {
//!         let mut conn = redis.get().await?;
//!         let value: Option<String> = redis::cmd("GET")
//!             .arg(&*key)
//!             .query_async(&mut *conn)
//!             .await?;
//!         Ok(Json(json!({ "value": value })))
//!     }
//! }
//! ```

mod config;
mod error;
mod pool;
mod pubsub;
mod service;

pub use config::{RedisConfig, RedisConfigBuilder};
pub use error::{RedisError, Result};
pub use pool::{RedisConnection, RedisPool};
pub use pubsub::{Message, PubSub, Subscription};
pub use service::RedisService;

// Re-export redis crate for convenience
pub use redis;
pub use redis::{AsyncCommands, Commands, RedisResult, Value};

/// Prelude for common imports.
///
/// ```
/// use armature_redis::prelude::*;
/// ```
pub mod prelude {
    pub use crate::config::{RedisConfig, RedisConfigBuilder};
    pub use crate::error::{RedisError, Result};
    pub use crate::pool::{RedisConnection, RedisPool};
    pub use crate::pubsub::{Message, PubSub, Subscription};
    pub use crate::service::RedisService;
    pub use redis::{AsyncCommands, Commands};
}
