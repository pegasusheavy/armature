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

    #[test]
    fn test_config_clone() {
        let config1 = AngularConfig::new()
            .with_node_path(PathBuf::from("/custom/node"))
            .with_cache(true, 300);
        
        let config2 = config1.clone();
        assert_eq!(config1.node_path, config2.node_path);
        assert_eq!(config1.enable_cache, config2.enable_cache);
        assert_eq!(config1.cache_ttl, config2.cache_ttl);
    }

    #[test]
    fn test_exclude_multiple_routes() {
        let config = AngularConfig::new()
            .exclude_route("/admin".to_string())
            .exclude_route("/private".to_string())
            .exclude_route("/internal".to_string());
        
        assert_eq!(config.excluded_routes.len(), 5); // 2 default + 3 added
        assert!(config.excluded_routes.contains(&"/admin".to_string()));
        assert!(config.excluded_routes.contains(&"/private".to_string()));
        assert!(config.excluded_routes.contains(&"/internal".to_string()));
    }

    #[test]
    fn test_default_excluded_routes() {
        let config = AngularConfig::default();
        assert!(config.excluded_routes.contains(&"/api".to_string()));
        assert!(config.excluded_routes.contains(&"/assets".to_string()));
    }

    #[test]
    fn test_cache_disabled_by_default() {
        let config = AngularConfig::new();
        assert!(!config.enable_cache);
        assert_eq!(config.cache_ttl, 300);
    }

    #[test]
    fn test_cache_enable() {
        let config = AngularConfig::new().with_cache(true, 1200);
        assert!(config.enable_cache);
        assert_eq!(config.cache_ttl, 1200);
    }

    #[test]
    fn test_cache_disable() {
        let config = AngularConfig::new()
            .with_cache(true, 600)
            .with_cache(false, 600);
        assert!(!config.enable_cache);
    }

    #[test]
    fn test_custom_node_path() {
        let custom_path = PathBuf::from("/opt/nodejs/bin/node");
        let config = AngularConfig::new()
            .with_node_path(custom_path.clone());
        assert_eq!(config.node_path, custom_path);
    }

    #[test]
    fn test_cache_ttl_variations() {
        let config1 = AngularConfig::new().with_cache(true, 0);
        let config2 = AngularConfig::new().with_cache(true, 3600);
        let config3 = AngularConfig::new().with_cache(true, 86400);
        
        assert_eq!(config1.cache_ttl, 0);
        assert_eq!(config2.cache_ttl, 3600);
        assert_eq!(config3.cache_ttl, 86400);
    }

    #[test]
    fn test_route_exclusion_idempotent() {
        let config = AngularConfig::new()
            .exclude_route("/admin".to_string())
            .exclude_route("/admin".to_string());
        
        // Should still work (HashSet handles duplicates)
        assert!(config.excluded_routes.contains(&"/admin".to_string()));
    }

    #[test]
    fn test_empty_route_exclusion() {
        let config = AngularConfig::new().exclude_route("".to_string());
        assert!(config.excluded_routes.contains(&"".to_string()));
    }

    #[test]
    fn test_config_builder_chaining() {
        let config = AngularConfig::new()
            .with_node_path(PathBuf::from("/usr/local/bin/node"))
            .exclude_route("/test1".to_string())
            .exclude_route("/test2".to_string())
            .with_cache(true, 900);
        
        assert_eq!(config.node_path, PathBuf::from("/usr/local/bin/node"));
        assert!(config.excluded_routes.contains(&"/test1".to_string()));
        assert!(config.excluded_routes.contains(&"/test2".to_string()));
        assert!(config.enable_cache);
        assert_eq!(config.cache_ttl, 900);
    }
}
