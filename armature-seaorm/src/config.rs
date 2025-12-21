//! Configuration for SeaORM database connections.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for a SeaORM database connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL.
    pub database_url: String,

    /// Maximum number of connections in the pool.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Minimum number of connections in the pool.
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Connection timeout.
    #[serde(default = "default_connect_timeout")]
    #[serde(with = "humantime_serde")]
    pub connect_timeout: Duration,

    /// Idle timeout for connections.
    #[serde(default = "default_idle_timeout")]
    #[serde(with = "humantime_serde")]
    pub idle_timeout: Duration,

    /// Maximum lifetime of a connection.
    #[serde(default = "default_max_lifetime")]
    #[serde(with = "humantime_serde")]
    pub max_lifetime: Duration,

    /// Enable SQLx logging.
    #[serde(default)]
    pub sqlx_logging: bool,

    /// SQLx log level.
    #[serde(default = "default_sqlx_log_level")]
    pub sqlx_log_level: String,

    /// Schema name (for PostgreSQL).
    #[serde(default)]
    pub schema: Option<String>,

    /// Set SQLx statement cache capacity.
    #[serde(default = "default_statement_cache_capacity")]
    pub statement_cache_capacity: usize,
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    1
}

fn default_connect_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_idle_timeout() -> Duration {
    Duration::from_secs(10 * 60) // 10 minutes
}

fn default_max_lifetime() -> Duration {
    Duration::from_secs(30 * 60) // 30 minutes
}

fn default_sqlx_log_level() -> String {
    "debug".to_string()
}

fn default_statement_cache_capacity() -> usize {
    100
}

impl DatabaseConfig {
    /// Create a new configuration with the given database URL.
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connect_timeout: default_connect_timeout(),
            idle_timeout: default_idle_timeout(),
            max_lifetime: default_max_lifetime(),
            sqlx_logging: false,
            sqlx_log_level: default_sqlx_log_level(),
            schema: None,
            statement_cache_capacity: default_statement_cache_capacity(),
        }
    }

    /// Create configuration from environment variables.
    ///
    /// Uses the following environment variables:
    /// - `DATABASE_URL`: Required database URL
    /// - `DATABASE_MAX_CONNECTIONS`: Max connections (default: 10)
    /// - `DATABASE_MIN_CONNECTIONS`: Min connections (default: 1)
    /// - `DATABASE_CONNECT_TIMEOUT`: Connect timeout in seconds
    /// - `DATABASE_IDLE_TIMEOUT`: Idle timeout in seconds
    /// - `DATABASE_SQLX_LOGGING`: Enable SQLx logging (true/false)
    pub fn from_env() -> Result<Self, crate::SeaOrmError> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| crate::SeaOrmError::Config("DATABASE_URL not set".into()))?;

        let mut config = Self::new(database_url);

        if let Ok(max) = std::env::var("DATABASE_MAX_CONNECTIONS") {
            config.max_connections = max
                .parse()
                .map_err(|_| crate::SeaOrmError::Config("Invalid DATABASE_MAX_CONNECTIONS".into()))?;
        }

        if let Ok(min) = std::env::var("DATABASE_MIN_CONNECTIONS") {
            config.min_connections = min
                .parse()
                .map_err(|_| crate::SeaOrmError::Config("Invalid DATABASE_MIN_CONNECTIONS".into()))?;
        }

        if let Ok(timeout) = std::env::var("DATABASE_CONNECT_TIMEOUT") {
            config.connect_timeout = Duration::from_secs(
                timeout
                    .parse()
                    .map_err(|_| crate::SeaOrmError::Config("Invalid DATABASE_CONNECT_TIMEOUT".into()))?,
            );
        }

        if let Ok(logging) = std::env::var("DATABASE_SQLX_LOGGING") {
            config.sqlx_logging = logging == "true" || logging == "1";
        }

        Ok(config)
    }

    /// Set the maximum number of connections.
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Set the minimum number of connections.
    pub fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = min;
        self
    }

    /// Set the connection timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set the idle timeout.
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Set the maximum connection lifetime.
    pub fn max_lifetime(mut self, lifetime: Duration) -> Self {
        self.max_lifetime = lifetime;
        self
    }

    /// Enable or disable SQLx logging.
    pub fn sqlx_logging(mut self, enabled: bool) -> Self {
        self.sqlx_logging = enabled;
        self
    }

    /// Set the schema name (PostgreSQL).
    pub fn schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    /// Convert to SeaORM ConnectOptions.
    pub fn to_connect_options(&self) -> sea_orm::ConnectOptions {
        let mut options = sea_orm::ConnectOptions::new(&self.database_url);
        
        options
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .connect_timeout(self.connect_timeout)
            .idle_timeout(self.idle_timeout)
            .max_lifetime(self.max_lifetime)
            .sqlx_logging(self.sqlx_logging);

        if let Some(ref schema) = self.schema {
            options.set_schema_search_path(schema.clone());
        }

        options
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self::new("postgres://localhost/armature")
    }
}

/// Humantime serde module for duration serialization.
mod humantime_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

