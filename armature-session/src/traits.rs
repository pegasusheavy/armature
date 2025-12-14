//! Session store trait definition.

use crate::error::SessionResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Session data structure.
///
/// Contains all session information including metadata and user data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: String,
    /// Session data as key-value pairs
    pub data: HashMap<String, serde_json::Value>,
    /// Session creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last access timestamp
    pub last_accessed_at: DateTime<Utc>,
    /// Session expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// User agent (optional)
    pub user_agent: Option<String>,
    /// IP address (optional)
    pub ip_address: Option<String>,
}

impl Session {
    /// Create a new session with the given ID and TTL.
    pub fn new(id: impl Into<String>, ttl: Duration) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            data: HashMap::new(),
            created_at: now,
            last_accessed_at: now,
            expires_at: now + chrono::Duration::from_std(ttl).unwrap_or_default(),
            user_agent: None,
            ip_address: None,
        }
    }

    /// Check if the session has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Get a value from the session data.
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Set a value in the session data.
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> SessionResult<()> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| crate::error::SessionError::Serialization(e.to_string()))?;
        self.data.insert(key.to_string(), json_value);
        Ok(())
    }

    /// Remove a value from the session data.
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.data.remove(key)
    }

    /// Check if a key exists in the session data.
    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Get all keys in the session data.
    pub fn keys(&self) -> Vec<&String> {
        self.data.keys().collect()
    }

    /// Clear all session data.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Update the last accessed timestamp.
    pub fn touch(&mut self) {
        self.last_accessed_at = Utc::now();
    }

    /// Extend the session expiration.
    pub fn extend(&mut self, ttl: Duration) {
        self.expires_at = Utc::now() + chrono::Duration::from_std(ttl).unwrap_or_default();
    }

    /// Set user agent.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set IP address.
    pub fn with_ip_address(mut self, ip_address: impl Into<String>) -> Self {
        self.ip_address = Some(ip_address.into());
        self
    }
}

/// Session store trait for different storage backends.
///
/// # ⚠️ Important: Prefer Stateless Architecture
///
/// **Armature strongly recommends stateless architecture using JWT tokens
/// instead of server-side sessions.** This module is provided for cases
/// where sessions are absolutely necessary (e.g., legacy system integration,
/// specific compliance requirements).
///
/// For most applications, use JWT-based authentication:
///
/// ```ignore
/// // Preferred: Stateless JWT
/// use armature_jwt::JwtManager;
///
/// let token = jwt_manager.create_token(user_claims)?;
/// // Client stores token, server validates on each request
/// ```
///
/// See the [Stateless Architecture Guide](../docs/stateless-architecture.md)
/// for why stateless is preferred.
///
/// # When to Use Sessions
///
/// Sessions may be appropriate when:
/// - Integrating with legacy systems that require sessions
/// - Compliance requirements mandate server-side session tracking
/// - You need immediate session invalidation (logout)
/// - Storing large amounts of user-specific data
///
/// # Examples
///
/// ```ignore
/// use armature_session::{SessionStore, SessionConfig, Session};
///
/// async fn example(store: &impl SessionStore) -> SessionResult<()> {
///     // Create a new session
///     let session = store.create(None).await?;
///
///     // Store data in session
///     let mut session = session;
///     session.set("user_id", 123)?;
///     store.save(&session).await?;
///
///     // Retrieve session
///     let session = store.get(&session.id).await?;
///
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Create a new session.
    ///
    /// # Arguments
    ///
    /// * `ttl` - Optional custom TTL (uses default if None)
    ///
    /// # Returns
    ///
    /// Returns a new session with a unique ID.
    async fn create(&self, ttl: Option<Duration>) -> SessionResult<Session>;

    /// Get a session by ID.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(session))` if found, `Ok(None)` if not found or expired.
    async fn get(&self, session_id: &str) -> SessionResult<Option<Session>>;

    /// Save/update a session.
    ///
    /// # Arguments
    ///
    /// * `session` - The session to save
    async fn save(&self, session: &Session) -> SessionResult<()>;

    /// Delete a session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to delete
    async fn delete(&self, session_id: &str) -> SessionResult<()>;

    /// Check if a session exists and is valid.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to check
    async fn exists(&self, session_id: &str) -> SessionResult<bool>;

    /// Extend a session's TTL.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to extend
    /// * `ttl` - The new TTL duration
    async fn extend(&self, session_id: &str, ttl: Duration) -> SessionResult<()>;

    /// Touch a session (update last accessed time).
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to touch
    async fn touch(&self, session_id: &str) -> SessionResult<()>;

    /// Clear all sessions (use with caution!).
    ///
    /// **Warning:** This will invalidate all user sessions.
    async fn clear_all(&self) -> SessionResult<()>;

    /// Get the number of active sessions.
    async fn count(&self) -> SessionResult<usize>;

    /// Cleanup expired sessions.
    ///
    /// This is called automatically by some backends, but can be
    /// called manually for maintenance.
    async fn cleanup_expired(&self) -> SessionResult<usize>;

    // ========== Convenience Methods ==========

    /// Get a session value by key.
    async fn get_value<T: for<'de> Deserialize<'de>>(
        &self,
        session_id: &str,
        key: &str,
    ) -> SessionResult<Option<T>> {
        if let Some(session) = self.get(session_id).await? {
            Ok(session.get(key))
        } else {
            Ok(None)
        }
    }

    /// Set a session value by key.
    async fn set_value<T: Serialize + Send>(
        &self,
        session_id: &str,
        key: &str,
        value: T,
    ) -> SessionResult<()> {
        if let Some(mut session) = self.get(session_id).await? {
            session.set(key, value)?;
            self.save(&session).await?;
        }
        Ok(())
    }

    /// Remove a session value by key.
    async fn remove_value(&self, session_id: &str, key: &str) -> SessionResult<()> {
        if let Some(mut session) = self.get(session_id).await? {
            session.remove(key);
            self.save(&session).await?;
        }
        Ok(())
    }
}

/// Generate a new unique session ID.
pub fn generate_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

