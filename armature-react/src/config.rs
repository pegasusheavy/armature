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
