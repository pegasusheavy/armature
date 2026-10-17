//! Application configuration

use std::env;
use std::fmt;

/// Minimum `JWT_SECRET` length in bytes. HS256 needs a key at least as long
/// as its 256-bit output; `armature-jwt` enforces the same floor (and rejects
/// well-known placeholders), this check just fails earlier with a clearer
/// message.
pub const MIN_JWT_SECRET_LEN: usize = 32;

#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub database_url: Option<String>,
    pub redis_url: Option<String>,
    pub environment: Environment,
}

// Hand-written so the JWT secret (and any credentials embedded in the
// database/redis URLs) never reach a log line via `{:?}`.
impl fmt::Debug for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("jwt_secret", &"<redacted>")
            .field("jwt_expiry_hours", &self.jwt_expiry_hours)
            .field("database_url", &self.database_url.as_ref().map(|_| "<redacted>"))
            .field("redis_url", &self.redis_url.as_ref().map(|_| "<redacted>"))
            .field("environment", &self.environment)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    Development,
    Production,
    Test,
}

/// Why the configuration could not be loaded.
#[derive(Debug)]
pub struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}

impl AppConfig {
    /// Load configuration from the environment.
    ///
    /// `JWT_SECRET` is **required** and must be at least
    /// [`MIN_JWT_SECRET_LEN`] bytes. There is deliberately no fallback: a
    /// built-in default would be public (it ships in this template) and
    /// anyone could forge tokens for a deployment that forgot to set it.
    /// Generate one with `openssl rand -hex 32`.
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = match env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "production" | "prod" => Environment::Production,
            "test" => Environment::Test,
            _ => Environment::Development,
        };

        let jwt_secret = env::var("JWT_SECRET").map_err(|_| {
            ConfigError(
                "JWT_SECRET is not set; generate one with `openssl rand -hex 32`".to_string(),
            )
        })?;
        if jwt_secret.len() < MIN_JWT_SECRET_LEN {
            return Err(ConfigError(format!(
                "JWT_SECRET must be at least {MIN_JWT_SECRET_LEN} bytes (got {}); \
                 generate one with `openssl rand -hex 32`",
                jwt_secret.len()
            )));
        }

        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            jwt_secret,
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            database_url: env::var("DATABASE_URL").ok(),
            redis_url: env::var("REDIS_URL").ok(),
            environment,
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }
}
