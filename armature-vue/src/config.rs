// Vue.js SSR Configuration

use std::path::PathBuf;

/// Configuration for Vue.js Server-Side Rendering
#[derive(Debug, Clone)]
pub struct VueConfig {
    /// Path to the Vue build directory (contains server.js or server-bundle.json)
    pub build_dir: PathBuf,

    /// Path to the static assets directory
    pub static_dir: PathBuf,

    /// Server entry point (e.g., "server-bundle.js" or "server/index.js")
    pub server_entry: String,

    /// Enable caching of rendered pages
    pub cache_enabled: bool,

    /// Cache TTL in seconds
    pub cache_ttl: u64,

    /// Node.js executable path
    pub node_path: String,

    /// Enable compression
    pub compression: bool,

    /// Template HTML file path
    pub template_path: PathBuf,

    /// Client manifest path (for Vue SSR)
    pub client_manifest: Option<PathBuf>,
}

impl VueConfig {
    /// Create a new Vue configuration
    pub fn new(build_dir: PathBuf) -> Self {
        Self {
            static_dir: build_dir.join("dist/client"),
            template_path: build_dir.join("index.html"),
            build_dir: build_dir.clone(),
            server_entry: "server-bundle.js".to_string(),
            cache_enabled: false,
            cache_ttl: 300, // 5 minutes
            node_path: "node".to_string(),
            compression: true,
            client_manifest: None,
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

    /// Set template path
    pub fn with_template(mut self, path: PathBuf) -> Self {
        self.template_path = path;
        self
    }

    /// Set client manifest path
    pub fn with_client_manifest(mut self, path: PathBuf) -> Self {
        self.client_manifest = Some(path);
        self
    }
}

impl Default for VueConfig {
    fn default() -> Self {
        Self::new(PathBuf::from("dist"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = VueConfig::default();
        assert_eq!(config.node_path, "node");
        assert!(!config.cache_enabled);
        assert!(config.compression);
    }

    #[test]
    fn test_config_builder() {
        let config = VueConfig::new(PathBuf::from("build"))
            .with_node_path("/usr/bin/node".to_string())
            .with_cache(true)
            .with_cache_ttl(600)
            .with_compression(false);

        assert_eq!(config.node_path, "/usr/bin/node");
        assert!(config.cache_enabled);
        assert_eq!(config.cache_ttl, 600);
        assert!(!config.compression);
    }

    #[test]
    fn test_config_with_template() {
        let config = VueConfig::new(PathBuf::from("dist"))
            .with_template(PathBuf::from("templates/app.html"));

        assert_eq!(config.template_path, PathBuf::from("templates/app.html"));
    }

    #[test]
    fn test_config_with_client_manifest() {
        let config = VueConfig::new(PathBuf::from("dist"))
            .with_client_manifest(PathBuf::from("manifest.json"));

        assert_eq!(config.client_manifest, Some(PathBuf::from("manifest.json")));
    }

    #[test]
    fn test_config_clone() {
        let config1 = VueConfig::new(PathBuf::from("dist")).with_cache(true);
        let config2 = config1.clone();

        assert_eq!(config1.build_dir, config2.build_dir);
        assert_eq!(config1.cache_enabled, config2.cache_enabled);
    }

    #[test]
    fn test_config_cache_ttl_values() {
        let config1 = VueConfig::new(PathBuf::from("dist")).with_cache_ttl(0);
        let config2 = VueConfig::new(PathBuf::from("dist")).with_cache_ttl(3600);

        assert_eq!(config1.cache_ttl, 0);
        assert_eq!(config2.cache_ttl, 3600);
    }

    #[test]
    fn test_config_server_entry_path() {
        let config = VueConfig::new(PathBuf::from("dist"))
            .with_server_entry("ssr/server.js".to_string());

        assert_eq!(config.server_entry, "ssr/server.js");
    }

    #[test]
    fn test_config_all_options() {
        let config = VueConfig::new(PathBuf::from("dist"))
            .with_node_path("/custom/node".to_string())
            .with_server_entry("bundle.js".to_string())
            .with_cache(true)
            .with_cache_ttl(1200)
            .with_compression(true)
            .with_template(PathBuf::from("index.html"))
            .with_client_manifest(PathBuf::from("client.json"));

        assert_eq!(config.node_path, "/custom/node");
        assert!(config.cache_enabled);
        assert_eq!(config.cache_ttl, 1200);
        assert!(config.compression);
        assert_eq!(config.template_path, PathBuf::from("index.html"));
        assert!(config.client_manifest.is_some());
    }

    #[test]
    fn test_config_default_values() {
        let config = VueConfig::new(PathBuf::from("dist"));

        assert_eq!(config.node_path, "node");
        assert_eq!(config.cache_ttl, 300);
        assert!(!config.cache_enabled);
        assert!(config.compression);
    }
}
