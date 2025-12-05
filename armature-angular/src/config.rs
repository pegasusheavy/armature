// Configuration for Angular SSR

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for Angular SSR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AngularConfig {
    /// Path to Node.js executable
    pub node_path: PathBuf,

    /// Path to the Angular server bundle (main.js from dist/server)
    pub server_bundle_path: PathBuf,

    /// Path to the browser distribution folder (dist/browser)
    pub browser_dist_path: PathBuf,

    /// Path to index.html
    pub index_html: PathBuf,

    /// Routes to exclude from SSR (e.g., /api/*)
    pub excluded_routes: Vec<String>,

    /// Enable caching of rendered pages
    pub enable_cache: bool,

    /// Cache TTL in seconds
    pub cache_ttl: u64,

    /// Timeout for SSR rendering in milliseconds
    pub render_timeout: u64,
}

impl Default for AngularConfig {
    fn default() -> Self {
        Self {
            node_path: PathBuf::from("node"),
            server_bundle_path: PathBuf::from("dist/server/main.js"),
            browser_dist_path: PathBuf::from("dist/browser"),
            index_html: PathBuf::from("dist/browser/index.html"),
            excluded_routes: vec!["/api".to_string(), "/assets".to_string()],
            enable_cache: false,
            cache_ttl: 300,       // 5 minutes
            render_timeout: 5000, // 5 seconds
        }
    }
}

impl AngularConfig {
    /// Create a new configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the Node.js path
    pub fn with_node_path(mut self, path: PathBuf) -> Self {
        self.node_path = path;
        self
    }

    /// Set the server bundle path
    pub fn with_server_bundle(mut self, path: PathBuf) -> Self {
        self.server_bundle_path = path;
        self
    }

    /// Set the browser distribution path
    pub fn with_browser_dist(mut self, path: PathBuf) -> Self {
        self.browser_dist_path = path;
        self
    }

    /// Add an excluded route
    pub fn exclude_route(mut self, route: String) -> Self {
        self.excluded_routes.push(route);
        self
    }

    /// Enable caching
    pub fn with_cache(mut self, enabled: bool, ttl: u64) -> Self {
        self.enable_cache = enabled;
        self.cache_ttl = ttl;
        self
    }

    /// Set render timeout
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.render_timeout = timeout;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AngularConfig::default();
        assert_eq!(config.node_path, PathBuf::from("node"));
        assert_eq!(config.excluded_routes.len(), 2);
        assert!(!config.enable_cache);
    }

    #[test]
    fn test_config_builder() {
        let config = AngularConfig::new()
            .with_node_path(PathBuf::from("/usr/bin/node"))
            .exclude_route("/admin".to_string())
            .with_cache(true, 600);

        assert_eq!(config.node_path, PathBuf::from("/usr/bin/node"));
        assert_eq!(config.excluded_routes.len(), 3);
        assert!(config.enable_cache);
        assert_eq!(config.cache_ttl, 600);
    }
}
