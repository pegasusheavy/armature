//! Vue.js SSR (Server-Side Rendering) integration for Armature framework.
//!
//! This crate provides server-side rendering capabilities for Vue.js applications,
//! allowing you to render Vue components on the server using Node.js.
//!
//! ## Features
//!
//! - 🟢 **Vue SSR** - Full SSR support for Vue 3+
//! - 📦 **Static Assets** - Serve compiled Vue assets
//! - 🔄 **Component Rendering** - Server-side component rendering
//! - ⚡ **Performance** - Fast Node.js integration
//! - 🎯 **Route Control** - Exclude specific routes from SSR
//!
//! ## Quick Start - Configuration
//!
//! ```
//! use armature_vue::VueConfig;
//! use std::path::PathBuf;
//!
//! let config = VueConfig::new(PathBuf::from("dist"))
//!     .with_node_path("node".to_string())
//!     .with_server_entry("server.js".to_string())
//!     .with_static_dir(PathBuf::from("dist/client"))
//!     .with_cache(true);
//!
//! assert_eq!(config.build_dir, PathBuf::from("dist"));
//! assert_eq!(config.node_path, "node");
//! assert!(config.cache_enabled);
//! ```
//!
//! ## Cache Configuration
//!
//! ```
//! use armature_vue::VueConfig;
//! use std::path::PathBuf;
//!
//! // Enable caching with custom TTL
//! let cached = VueConfig::new(PathBuf::from("dist"))
//!     .with_cache(true)
//!     .with_cache_ttl(3600); // 1 hour
//!
//! assert!(cached.cache_enabled);
//! assert_eq!(cached.cache_ttl, 3600);
//!
//! // Disable caching
//! let uncached = VueConfig::new(PathBuf::from("dist"))
//!     .with_cache(false);
//!
//! assert!(!uncached.cache_enabled);
//! ```
//!
//! ## Production Configuration
//!
//! ```
//! use armature_vue::VueConfig;
//! use std::path::PathBuf;
//!
//! // Full production configuration
//! let prod_config = VueConfig::new(PathBuf::from("dist"))
//!     .with_node_path("node".to_string())
//!     .with_server_entry("entry-server.js".to_string())
//!     .with_static_dir(PathBuf::from("dist/client"))
//!     .with_cache(true)
//!     .with_cache_ttl(7200);
//!
//! assert_eq!(prod_config.server_entry, "entry-server.js");
//! assert_eq!(prod_config.cache_ttl, 7200);
//! assert!(prod_config.cache_enabled);
//! ```
//!
//! ## Static Directory Configuration
//!
//! ```
//! use armature_vue::VueConfig;
//! use std::path::PathBuf;
//!
//! // Configure static assets directory
//! let config = VueConfig::new(PathBuf::from("dist"))
//!     .with_node_path("node".to_string())
//!     .with_server_entry("server/index.js".to_string())
//!     .with_static_dir(PathBuf::from("dist/client/assets"));
//!
//! assert_eq!(config.static_dir, PathBuf::from("dist/client/assets"));
//! assert_eq!(config.server_entry, "server/index.js");
//! assert_eq!(config.build_dir, PathBuf::from("dist"));
//! ```
//!
//! ## Creating a Vue Service
//!
//! ```ignore
//! use armature_vue::{VueConfig, VueService};
//! use armature_core::HttpRequest;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = VueConfig::new(PathBuf::from("dist"))
//!     .with_node_path("node".to_string())
//!     .with_server_entry("server.js".to_string())
//!     .with_static_dir(PathBuf::from("dist/client"));
//!
//! let service = VueService::new(config);
//!
//! // Create a request
//! let request = HttpRequest::new("GET".to_string(), "/home".to_string());
//!
//! // Render the page
//! let response = service.render(&request).await?;
//! println!("Response status: {}", response.status);
//! # Ok(())
//! # }
//! ```

mod config;
mod renderer;
mod service;

pub use config::VueConfig;
pub use renderer::VueRenderer;
pub use service::VueService;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        // Ensure module compiles
    }
}
