//! Session storage for Armature framework.
//!
//! # ⚠️ Important: Stateless Architecture is Preferred
//!
//! **Armature strongly recommends stateless architecture using JWT tokens
//! instead of server-side sessions.** This module is provided for cases
//! where sessions are absolutely necessary (e.g., legacy system integration,
//! specific compliance requirements).
//!
//! ## Why Stateless?
//!
//! 1. **Horizontal Scalability** - Any server can handle any request
//! 2. **Cloud-Native** - Works seamlessly with containers and Kubernetes
//! 3. **Reliability** - No session store to fail or synchronize
//! 4. **Performance** - No session lookups on every request
//! 5. **Security** - No session hijacking or fixation attacks
//!
//! ## Preferred Approach: JWT Authentication
//!
//! ```rust,ignore
//! use armature_jwt::JwtManager;
//!
//! // Create token at login
//! let token = jwt_manager.create_token(UserClaims {
//!     user_id: user.id,
//!     email: user.email,
//!     roles: user.roles,
//! })?;
//!
//! // Client stores token
//! // Server validates on each request - no session lookup
//! let claims = jwt_manager.verify_token(&token)?;
//! ```
//!
//! See the [Stateless Architecture Guide](../docs/STATELESS_ARCHITECTURE.md).
//!
//! # When to Use Sessions
//!
//! Sessions may be appropriate when:
//! - Integrating with legacy systems that require sessions
//! - Compliance requirements mandate server-side session tracking
//! - You need immediate session invalidation (logout all devices)
//! - Storing large amounts of user-specific temporary data
//!
//! # Features
//!
//! - `redis` - Redis session storage (enabled by default)
//! - `memcached` - Memcached session storage (requires opt-in)
//! - `couchdb` - CouchDB session storage
//!
//! # Examples
//!
//! ## Redis Session Store (Default)
//!
//! ```no_run
//! use armature_session::*;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), SessionError> {
//!     // Configure Redis session store
//!     let config = SessionConfig::redis("redis://localhost:6379")?
//!         .with_namespace("myapp:session")
//!         .with_default_ttl(Duration::from_secs(3600));
//!
//!     let store = RedisSessionStore::new(config).await?;
//!
//!     // Create a new session
//!     let mut session = store.create(None).await?;
//!
//!     // Store data in session
//!     session.set("user_id", 123)?;
//!     session.set("username", "alice")?;
//!     store.save(&session).await?;
//!
//!     // Retrieve session later
//!     if let Some(session) = store.get(&session.id).await? {
//!         let user_id: Option<i32> = session.get("user_id");
//!         println!("User ID: {:?}", user_id);
//!     }
//!
//!     // Delete session (logout)
//!     store.delete(&session.id).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Memcached Session Store (requires `memcached` feature)
//!
//! ```ignore
//! use armature_session::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), SessionError> {
//!     let config = SessionConfig::memcached("memcache://localhost:11211")?;
//!     let store = MemcachedSessionStore::new(config).await?;
//!
//!     // Use same API as Redis
//!     let session = store.create(None).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## CouchDB Session Store (requires `couchdb` feature)
//!
//! ```ignore
//! use armature_session::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), SessionError> {
//!     let config = SessionConfig::couchdb("http://localhost:5984", "sessions")?
//!         .with_auth("admin", "password");
//!     let store = CouchDbSessionStore::new(config).await?;
//!
//!     // Use same API as Redis
//!     let session = store.create(None).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod traits;

#[cfg(feature = "redis")]
pub mod redis_session;

#[cfg(feature = "memcached")]
pub mod memcached_session;

#[cfg(feature = "couchdb")]
pub mod couchdb_session;

pub use config::{SessionBackend, SessionConfig};
pub use error::{SessionError, SessionResult};
pub use traits::{Session, SessionStore, generate_session_id};

#[cfg(feature = "redis")]
pub use redis_session::RedisSessionStore;

#[cfg(feature = "memcached")]
pub use memcached_session::MemcachedSessionStore;

#[cfg(feature = "couchdb")]
pub use couchdb_session::CouchDbSessionStore;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::config::{SessionBackend, SessionConfig};
    pub use crate::error::{SessionError, SessionResult};
    pub use crate::traits::{Session, SessionStore, generate_session_id};

    #[cfg(feature = "redis")]
    pub use crate::redis_session::RedisSessionStore;

    #[cfg(feature = "memcached")]
    pub use crate::memcached_session::MemcachedSessionStore;

    #[cfg(feature = "couchdb")]
    pub use crate::couchdb_session::CouchDbSessionStore;
}

