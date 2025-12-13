//! Redis session storage implementation.

use crate::config::SessionConfig;
use crate::error::{SessionError, SessionResult};
use crate::traits::{Session, SessionStore, generate_session_id};
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::time::Duration;

/// Redis-backed session store.
///
/// # ⚠️ Important: Prefer Stateless Architecture
///
/// **Armature strongly recommends stateless architecture using JWT tokens.**
/// Use sessions only when absolutely necessary.
///
/// # Examples
///
/// ```no_run
/// use armature_session::{RedisSessionStore, SessionConfig, SessionStore};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = SessionConfig::redis("redis://localhost:6379")?
///         .with_namespace("myapp:session")
///         .with_default_ttl(Duration::from_secs(3600));
///
///     let store = RedisSessionStore::new(config).await?;
///
///     // Create a session
///     let mut session = store.create(None).await?;
///     session.set("user_id", 123)?;
///     store.save(&session).await?;
///
///     Ok(())
/// }
/// ```
pub struct RedisSessionStore {
    conn: ConnectionManager,
    config: SessionConfig,
}

impl RedisSessionStore {
    /// Create a new Redis session store.
    ///
    /// # Arguments
    ///
    /// * `config` - Session configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use armature_session::{RedisSessionStore, SessionConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = SessionConfig::redis("redis://localhost:6379")?;
    /// let store = RedisSessionStore::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: SessionConfig) -> SessionResult<Self> {
        let client = redis::Client::open(config.url.as_str())
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| SessionError::Connection(e.to_string()))?;

        Ok(Self { conn, config })
    }

    /// Get the session key for a given session ID.
    fn session_key(&self, session_id: &str) -> String {
        self.config.session_key(session_id)
    }
}

#[async_trait]
impl SessionStore for RedisSessionStore {
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
        let mut conn = self.conn.clone();

        let data: Option<String> = conn.get(&key).await?;

        match data {
            Some(json) => {
                let session: Session = serde_json::from_str(&json)
                    .map_err(|e| SessionError::Deserialization(e.to_string()))?;

                // Check if expired
                if session.is_expired() {
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
        let mut conn = self.conn.clone();

        let json = serde_json::to_string(session)
            .map_err(|e| SessionError::Serialization(e.to_string()))?;

        // Calculate remaining TTL
        let now = chrono::Utc::now();
        let remaining = (session.expires_at - now).num_seconds().max(0) as u64;

        if remaining > 0 {
            let _: () = conn.set_ex(&key, json, remaining).await?;
        }

        Ok(())
    }

    async fn delete(&self, session_id: &str) -> SessionResult<()> {
        let key = self.session_key(session_id);
        let mut conn = self.conn.clone();

        let _: () = conn.del(&key).await?;

        Ok(())
    }

    async fn exists(&self, session_id: &str) -> SessionResult<bool> {
        let key = self.session_key(session_id);
        let mut conn = self.conn.clone();

        let exists: bool = conn.exists(&key).await?;

        if exists {
            // Also check if not expired
            if let Some(session) = self.get(session_id).await? {
                return Ok(!session.is_expired());
            }
        }

        Ok(false)
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
        let mut conn = self.conn.clone();
        let pattern = format!("{}:*", self.config.namespace);

        // Use SCAN to find all session keys
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await?;

        if !keys.is_empty() {
            let _: () = conn.del(keys).await?;
        }

        Ok(())
    }

    async fn count(&self) -> SessionResult<usize> {
        let mut conn = self.conn.clone();
        let pattern = format!("{}:*", self.config.namespace);

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await?;

        Ok(keys.len())
    }

    async fn cleanup_expired(&self) -> SessionResult<usize> {
        // Redis automatically expires keys with TTL
        // This method is a no-op for Redis but returns 0 for consistency
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_key_generation() {
        let config = SessionConfig::redis("redis://localhost:6379").unwrap();
        assert!(config.session_key("test-id").starts_with("session:"));
    }
}

