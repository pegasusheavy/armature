//! Application configuration

use std::env;

/// Application configuration
#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    /// Used for the Playground's embedded link to the GraphQL endpoint.
    /// Does **not** move the endpoint itself: `#[controller("/graphql")]`
    /// in `main.rs` fixes the actual route at compile time (macro attributes
    /// can't read runtime config), so changing this without also editing
    /// that attribute leaves the Playground pointing at a route that isn't
    /// mounted there.
    pub graphql_path: String,
    /// Gates the `/playground` controller: when `false` it returns 404
    /// instead of serving GraphiQL. Off unless `GRAPHQL_PLAYGROUND=true`.
    pub graphql_playground: bool,
    /// Gates schema introspection: when `false` the schema is built with
    /// `disable_introspection()` and `GET /graphql/schema` returns 404, so the
    /// full SDL is not published to anonymous clients. Off unless
    /// `GRAPHQL_INTROSPECTION=true`.
    pub graphql_introspection: bool,
    /// Enforced via `SchemaBuilder::limit_depth` when the schema is built.
    pub graphql_depth_limit: usize,
    /// Enforced via `SchemaBuilder::limit_complexity` when the schema is built.
    pub graphql_complexity_limit: usize,
}

// There is intentionally no `jwt_secret` here: this template does not
// authenticate requests. If you add JWT auth, read `JWT_SECRET` as a
// *required* variable (no fallback default — a default would ship in this
// public template and let anyone forge tokens) and build the key with
// `armature_jwt::JwtConfig::new(secret)?`, which rejects secrets shorter than
// 32 bytes and known placeholders.

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
                .unwrap_or(false),
            graphql_introspection: env::var("GRAPHQL_INTROSPECTION")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            graphql_depth_limit: env::var("GRAPHQL_DEPTH_LIMIT")
                .ok()
                .and_then(|d| d.parse().ok())
                .unwrap_or(10),
            graphql_complexity_limit: env::var("GRAPHQL_COMPLEXITY_LIMIT")
                .ok()
                .and_then(|c| c.parse().ok())
                .unwrap_or(100),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::from_env()
    }
}
