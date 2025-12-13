//! Memcached session storage implementation.
//!
//! This module requires the `memcached` feature flag.

use crate::config::SessionConfig;
use crate::error::{SessionError, SessionResult};
use crate::traits::{Session, SessionStore, generate_session_id};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Memcached-backed session store.
///
/// # ⚠️ Important: Prefer Stateless Architecture
///
/// **Armature strongly recommends stateless architecture using JWT tokens.**
/// Use sessions only when absolutely necessary.
///
/// # Feature Flag
///
/// This requires the `memcached` feature:
///
/// ```toml
/// [dependencies]
/// armature-session = { version = "0.1", features = ["memcached"] }
/// ```
///
/// # Examples
///
/// ```ignore
/// use armature_session::{MemcachedSessionStore, SessionConfig, SessionStore};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = SessionConfig::memcached("memcache://localhost:11211")?
///         .with_namespace("myapp:session")
///         .with_default_ttl(Duration::from_secs(3600));
///
///     let store = MemcachedSessionStore::new(config).await?;
///
///     // Create a session
///     let mut session = store.create(None).await?;
///     session.set("user_id", 123)?;
///     store.save(&session).await?;
///
///     Ok(())
/// }
/// ```
pub struct MemcachedSessionStore {
    client: Arc<Mutex<memcache::Client>>,
    config: SessionConfig,
}

impl MemcachedSessionStore {
    /// Create a new Memcached session store.
    ///
    /// # Arguments
    ///
    /// * `config` - Session configuration
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use armature_session::{MemcachedSessionStore, SessionConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = SessionConfig::memcached("memcache://localhost:11211")?;
    /// let store = MemcachedSessionStore::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: SessionConfig) -> SessionResult<Self> {
        let client = memcache::connect(config.url.as_str())
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            config,
        })
    }

    /// Get the session key for a given session ID.
    fn session_key(&self, session_id: &str) -> String {
        self.config.session_key(session_id)
    }
}

#[async_trait]
impl SessionStore for MemcachedSessionStore {
    async fn create(&self, ttl: Option<Duration>) -> SessionResult<Session> {
        let session_id = generate_session_id();
        let ttl = ttl.unwrap_or(self.config.default_ttl);

        // Enforce max TTL
        let ttl = if ttl > self.config.max_ttl {
            self.config.max_ttl
        } else {
            ttl
        };

        let session = Session::new(&session_id, ttl);

        // Save the session
        self.save(&session).await?;

        Ok(session)
    }

    async fn get(&self, session_id: &str) -> SessionResult<Option<Session>> {
        let key = self.session_key(session_id);
        let client = self.client.lock().await;

        let data: Option<String> = client
            .get(&key)
            .map_err(|e| SessionError::Other(e.to_string()))?;

        match data {
            Some(json) => {
                let session: Session = serde_json::from_str(&json)
                    .map_err(|e| SessionError::Deserialization(e.to_string()))?;

                // Check if expired
                if session.is_expired() {
                    drop(client); // Release lock before calling delete
                    self.delete(session_id).await?;
                    return Ok(None);
                }

                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    async fn save(&self, session: &Session) -> SessionResult<()> {
        let key = self.session_key(&session.id);
        let client = self.client.lock().await;

        let json = serde_json::to_string(session)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        // Calculate remaining TTL
        let now = chrono::Utc::now();
        let remaining = (session.expires_at - now).num_seconds().max(0) as u32;

        if remaining > 0 {
            client
                .set(&key, json.as_str(), remaining)
                .map_err(|e| SessionError::Other(e.to_string()))?;
        }

        Ok(())
    }

    async fn delete(&self, session_id: &str) -> SessionResult<()> {
        let key = self.session_key(session_id);
        let client = self.client.lock().await;

        // Memcached delete returns false if key doesn't exist, which is fine
        let _ = client.delete(&key);

        Ok(())
    }

    async fn exists(&self, session_id: &str) -> SessionResult<bool> {
        match self.get(session_id).await? {
            Some(session) => Ok(!session.is_expired()),
            None => Ok(false),
        }
    }

    async fn extend(&self, session_id: &str, ttl: Duration) -> SessionResult<()> {
        if let Some(mut session) = self.get(session_id).await? {
            // Enforce max TTL
            let ttl = if ttl > self.config.max_ttl {
                self.config.max_ttl
            } else {
                ttl
            };

            session.extend(ttl);
            self.save(&session).await?;
        }

        Ok(())
    }

    async fn touch(&self, session_id: &str) -> SessionResult<()> {
        if let Some(mut session) = self.get(session_id).await? {
            session.touch();
            self.save(&session).await?;
        }

        Ok(())
    }

    async fn clear_all(&self) -> SessionResult<()> {
        let client = self.client.lock().await;

        client
            .flush()
            .map_err(|e| SessionError::Other(e.to_string()))?;

        Ok(())
    }

    async fn count(&self) -> SessionResult<usize> {
        // Memcached doesn't support counting keys with a prefix
        // This is a limitation of the protocol
        Err(SessionError::Other(
            "Memcached does not support counting sessions by prefix".to_string(),
        ))
    }

    async fn cleanup_expired(&self) -> SessionResult<usize> {
        // Memcached automatically expires keys
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_key_generation() {
        let config = SessionConfig::memcached("memcache://localhost:11211").unwrap();
        assert!(config.session_key("test-id").starts_with("session:"));
    }
}

