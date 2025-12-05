// React SSR Configuration

use std::path::PathBuf;

/// Configuration for React Server-Side Rendering
#[derive(Debug, Clone)]
pub struct ReactConfig {
    /// Path to the React build directory (contains server.js)
    pub build_dir: PathBuf,

    /// Path to the static assets directory
    pub static_dir: PathBuf,

    /// Server entry point (e.g., "server/index.js")
    pub server_entry: String,

    /// Enable caching of rendered pages
    pub cache_enabled: bool,

    /// Cache TTL in seconds
    pub cache_ttl: u64,

    /// Node.js executable path
    pub node_path: String,

    /// Enable compression
    pub compression: bool,
}

impl ReactConfig {
    /// Create a new React configuration
    pub fn new(build_dir: PathBuf) -> Self {
        Self {
            static_dir: build_dir.join("static"),
            build_dir,
            server_entry: "server/index.js".to_string(),
            cache_enabled: false,
            cache_ttl: 300, // 5 minutes
            node_path: "node".to_string(),
            compression: true,
        }
    }

    /// Set the static directory
    pub fn with_static_dir(mut self, dir: PathBuf) -> Self {
        self.static_dir = dir;
        self
    }

    /// Set the server entry point
    pub fn with_server_entry(mut self, entry: String) -> Self {
        self.server_entry = entry;
        self
    }

    /// Enable or disable caching
    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    }

    /// Set cache TTL
    pub fn with_cache_ttl(mut self, ttl: u64) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Set Node.js path
    pub fn with_node_path(mut self, path: String) -> Self {
        self.node_path = path;
        self
    }

    /// Enable or disable compression
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.compression = enabled;
        self
    }
}

impl Default for ReactConfig {
    fn default() -> Self {
        Self::new(PathBuf::from("build"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = ReactConfig::new(PathBuf::from("dist"));
        assert_eq!(config.build_dir, PathBuf::from("dist"));
        assert_eq!(config.server_entry, "server/index.js");
        assert_eq!(config.cache_ttl, 300);
    }

    #[test]
    fn test_config_with_server_entry() {
        let config = ReactConfig::new(PathBuf::from("build"))
            .with_server_entry("server.js".to_string());

        assert_eq!(config.server_entry, "server.js");
    }

    #[test]
    fn test_config_with_static_dir() {
        let config = ReactConfig::new(PathBuf::from("build"))
            .with_static_dir(PathBuf::from("static"));

        assert_eq!(config.static_dir, PathBuf::from("static"));
    }

    #[test]
    fn test_config_default() {
        let config = ReactConfig::default();
        assert_eq!(config.build_dir, PathBuf::from("build"));
    }

    #[test]
    fn test_config_builder_pattern() {
        let config = ReactConfig::new(PathBuf::from("dist"))
            .with_server_entry("ssr/server.js".to_string())
            .with_static_dir(PathBuf::from("public"))
            .with_cache(true)
            .with_cache_ttl(600);

        assert_eq!(config.build_dir, PathBuf::from("dist"));
        assert_eq!(config.server_entry, "ssr/server.js");
        assert_eq!(config.static_dir, PathBuf::from("public"));
        assert!(config.cache_enabled);
        assert_eq!(config.cache_ttl, 600);
    }

    #[test]
    fn test_config_clone() {
        let config1 = ReactConfig::new(PathBuf::from("build"))
            .with_server_entry("server.js".to_string());
        let config2 = config1.clone();

        assert_eq!(config1.build_dir, config2.build_dir);
        assert_eq!(config1.server_entry, config2.server_entry);
    }

    #[test]
    fn test_config_debug() {
        let config = ReactConfig::new(PathBuf::from("build"));
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("build"));
    }

    #[test]
    fn test_config_with_nested_paths() {
        let config = ReactConfig::new(PathBuf::from("dist/production"))
            .with_server_entry("ssr/main/server.bundle.js".to_string())
            .with_static_dir(PathBuf::from("assets/static"));

        assert!(config.server_entry.contains("ssr"));
        assert!(config.static_dir.to_str().unwrap().contains("assets"));
    }

    #[test]
    fn test_config_cache_toggle() {
        let config = ReactConfig::new(PathBuf::from("build"))
            .with_cache(true)
            .with_cache_ttl(1200);

        assert!(config.cache_enabled);
        assert_eq!(config.cache_ttl, 1200);
    }

    #[test]
    fn test_config_compression_toggle() {
        let config1 = ReactConfig::new(PathBuf::from("build")).with_compression(true);
        let config2 = ReactConfig::new(PathBuf::from("build")).with_compression(false);

        assert!(config1.compression);
        assert!(!config2.compression);
    }

    #[test]
    fn test_config_node_path() {
        let config = ReactConfig::new(PathBuf::from("build"))
            .with_node_path("/custom/node".to_string());

        assert_eq!(config.node_path, "/custom/node");
    }
}
