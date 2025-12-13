//! Session configuration.

use crate::error::{SessionError, SessionResult};
use std::time::Duration;

/// Session backend type.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionBackend {
    /// Redis backend
    Redis,
    /// Memcached backend
    Memcached,
    /// CouchDB backend
    CouchDb,
}

/// Session configuration.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Backend type
    pub backend: SessionBackend,
    /// Connection URL
    pub url: String,
    /// Session namespace/prefix
    pub namespace: String,
    /// Default session TTL
    pub default_ttl: Duration,
    /// Maximum session TTL (for security)
    pub max_ttl: Duration,
    /// CouchDB database name (only for CouchDB backend)
    pub database: Option<String>,
    /// CouchDB username (only for CouchDB backend)
    pub username: Option<String>,
    /// CouchDB password (only for CouchDB backend)
    pub password: Option<String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            backend: SessionBackend::Redis,
            url: "redis://localhost:6379".to_string(),
            namespace: "session".to_string(),
            default_ttl: Duration::from_secs(3600), // 1 hour
            max_ttl: Duration::from_secs(86400 * 7), // 7 days
            database: None,
            username: None,
            password: None,
        }
    }
}

impl SessionConfig {
    /// Create a Redis session configuration.
    ///
    /// # Arguments
    ///
    /// * `url` - Redis connection URL (e.g., "redis://localhost:6379")
    ///
    /// # Examples
    ///
    /// ```
    /// use armature_session::SessionConfig;
    ///
    /// let config = SessionConfig::redis("redis://localhost:6379").unwrap();
    /// ```
    pub fn redis(url: &str) -> SessionResult<Self> {
        if !url.starts_with("redis://") && !url.starts_with("rediss://") {
            return Err(SessionError::InvalidUrl(
                "Redis URL must start with redis:// or rediss://".to_string(),
            ));
        }

        Ok(Self {
            backend: SessionBackend::Redis,
            url: url.to_string(),
            ..Default::default()
        })
    }

    /// Create a Memcached session configuration.
    ///
    /// # Arguments
    ///
    /// * `url` - Memcached connection URL (e.g., "memcache://localhost:11211")
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use armature_session::SessionConfig;
    ///
    /// let config = SessionConfig::memcached("memcache://localhost:11211").unwrap();
    /// ```
    pub fn memcached(url: &str) -> SessionResult<Self> {
        if !url.starts_with("memcache://") {
            return Err(SessionError::InvalidUrl(
                "Memcached URL must start with memcache://".to_string(),
            ));
        }

        Ok(Self {
            backend: SessionBackend::Memcached,
            url: url.to_string(),
            ..Default::default()
        })
    }

    /// Create a CouchDB session configuration.
    ///
    /// # Arguments
    ///
    /// * `url` - CouchDB connection URL (e.g., "http://localhost:5984")
    /// * `database` - Database name for sessions
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use armature_session::SessionConfig;
    ///
    /// let config = SessionConfig::couchdb("http://localhost:5984", "sessions").unwrap();
    /// ```
    pub fn couchdb(url: &str, database: &str) -> SessionResult<Self> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(SessionError::InvalidUrl(
                "CouchDB URL must start with http:// or https://".to_string(),
            ));
        }

        Ok(Self {
            backend: SessionBackend::CouchDb,
            url: url.to_string(),
            database: Some(database.to_string()),
            ..Default::default()
        })
    }

    /// Set the session namespace/prefix.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace to use for session keys
    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = namespace.to_string();
        self
    }

    /// Set the default session TTL.
    ///
    /// # Arguments
    ///
    /// * `ttl` - Default time-to-live for sessions
    pub fn with_default_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// Set the maximum session TTL.
    ///
    /// # Arguments
    ///
    /// * `ttl` - Maximum time-to-live for sessions
    pub fn with_max_ttl(mut self, ttl: Duration) -> Self {
        self.max_ttl = ttl;
        self
    }

    /// Set CouchDB authentication credentials.
    ///
    /// # Arguments
    ///
    /// * `username` - CouchDB username
    /// * `password` - CouchDB password
    pub fn with_auth(mut self, username: &str, password: &str) -> Self {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());
        self
    }

    /// Build the session key with namespace.
    pub fn session_key(&self, session_id: &str) -> String {
        format!("{}:{}", self.namespace, session_id)
    }
}

