// Svelte SSR Configuration

use std::path::PathBuf;

/// Configuration for Svelte Server-Side Rendering
#[derive(Debug, Clone)]
pub struct SvelteConfig {
    /// Path to the Svelte build directory (contains server output)
    pub build_dir: PathBuf,

    /// Path to the static assets directory
    pub static_dir: PathBuf,

    /// Server entry point (e.g., "server/index.js" from SvelteKit build)
    pub server_entry: String,

    /// Enable caching of rendered pages
    pub cache_enabled: bool,

    /// Cache TTL in seconds
    pub cache_ttl: u64,

    /// Node.js executable path
    pub node_path: String,

    /// Enable compression
    pub compression: bool,

    /// Enable client-side hydration
    pub hydration: bool,

    /// Prerender pages at build time
    pub prerender: bool,
}

impl SvelteConfig {
    /// Create a new Svelte configuration
    pub fn new(build_dir: PathBuf) -> Self {
        Self {
            static_dir: build_dir.join("client"),
            build_dir: build_dir.clone(),
            server_entry: "server/index.js".to_string(),
            cache_enabled: false,
            cache_ttl: 300, // 5 minutes
            node_path: "node".to_string(),
            compression: true,
            hydration: true,
            prerender: false,
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

    /// Enable or disable client-side hydration
    pub fn with_hydration(mut self, enabled: bool) -> Self {
        self.hydration = enabled;
        self
    }

    /// Enable or disable prerendering
    pub fn with_prerender(mut self, enabled: bool) -> Self {
        self.prerender = enabled;
        self
    }
}

impl Default for SvelteConfig {
    fn default() -> Self {
        Self::new(PathBuf::from("build"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SvelteConfig::default();
        assert_eq!(config.node_path, "node");
        assert!(!config.cache_enabled);
        assert!(config.compression);
        assert!(config.hydration);
        assert!(!config.prerender);
    }

    #[test]
    fn test_config_builder() {
        let config = SvelteConfig::new(PathBuf::from("build"))
            .with_node_path("/usr/bin/node".to_string())
            .with_cache(true)
            .with_cache_ttl(600)
            .with_hydration(false)
            .with_prerender(true);

        assert_eq!(config.node_path, "/usr/bin/node");
        assert!(config.cache_enabled);
        assert_eq!(config.cache_ttl, 600);
        assert!(!config.hydration);
        assert!(config.prerender);
    }

    #[test]
    fn test_config_with_server_entry() {
        let config = SvelteConfig::new(PathBuf::from("build"))
            .with_server_entry("server/index.js".to_string());

        assert_eq!(config.server_entry, "server/index.js");
    }


    #[test]
    fn test_config_clone() {
        let config1 = SvelteConfig::new(PathBuf::from("build"))
            .with_prerender(true)
            .with_hydration(false);
        let config2 = config1.clone();

        assert_eq!(config1.prerender, config2.prerender);
        assert_eq!(config1.hydration, config2.hydration);
    }

    #[test]
    fn test_config_compression_toggle() {
        let config1 = SvelteConfig::new(PathBuf::from("build")).with_compression(true);
        let config2 = SvelteConfig::new(PathBuf::from("build")).with_compression(false);

        assert!(config1.compression);
        assert!(!config2.compression);
    }

    #[test]
    fn test_config_cache_ttl_range() {
        let config1 = SvelteConfig::new(PathBuf::from("build")).with_cache_ttl(0);
        let config2 = SvelteConfig::new(PathBuf::from("build")).with_cache_ttl(7200);

        assert_eq!(config1.cache_ttl, 0);
        assert_eq!(config2.cache_ttl, 7200);
    }

    #[test]
    fn test_config_hydration_and_prerender() {
        let config = SvelteConfig::new(PathBuf::from("build"))
            .with_hydration(true)
            .with_prerender(true);

        assert!(config.hydration);
        assert!(config.prerender);
    }

    #[test]
    fn test_config_all_features() {
        let config = SvelteConfig::new(PathBuf::from("dist"))
            .with_node_path("/opt/node".to_string())
            .with_server_entry("ssr/bundle.js".to_string())
            .with_cache(true)
            .with_cache_ttl(900)
            .with_compression(true)
            .with_hydration(true)
            .with_prerender(true);

        assert_eq!(config.node_path, "/opt/node");
        assert!(config.cache_enabled);
        assert_eq!(config.cache_ttl, 900);
        assert!(config.compression);
        assert!(config.hydration);
        assert!(config.prerender);
    }

    #[test]
    fn test_config_default_cache_ttl() {
        let config = SvelteConfig::new(PathBuf::from("build"));
        assert_eq!(config.cache_ttl, 300);
    }

    #[test]
    fn test_config_different_build_dirs() {
        let config1 = SvelteConfig::new(PathBuf::from("build"));
        let config2 = SvelteConfig::new(PathBuf::from("dist"));
        let config3 = SvelteConfig::new(PathBuf::from(".svelte-kit/output"));

        assert_eq!(config1.build_dir, PathBuf::from("build"));
        assert_eq!(config2.build_dir, PathBuf::from("dist"));
        assert_eq!(config3.build_dir, PathBuf::from(".svelte-kit/output"));
    }
}
