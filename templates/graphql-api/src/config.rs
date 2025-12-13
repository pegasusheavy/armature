//! Application configuration

use std::env;

/// Application configuration
#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub graphql_path: String,
    pub graphql_playground: bool,
    pub graphql_depth_limit: usize,
    pub graphql_complexity_limit: usize,
    pub jwt_secret: String,
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            graphql_path: env::var("GRAPHQL_PATH").unwrap_or_else(|_| "/graphql".to_string()),
            graphql_playground: env::var("GRAPHQL_PLAYGROUND")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            graphql_depth_limit: env::var("GRAPHQL_DEPTH_LIMIT")
                .ok()
                .and_then(|d| d.parse().ok())
                .unwrap_or(10),
            graphql_complexity_limit: env::var("GRAPHQL_COMPLEXITY_LIMIT")
                .ok()
                .and_then(|c| c.parse().ok())
                .unwrap_or(100),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "default-secret-change-me".to_string()),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

