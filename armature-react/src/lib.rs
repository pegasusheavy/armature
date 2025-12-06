//! React SSR (Server-Side Rendering) integration for Armature framework.
//!
//! This crate provides server-side rendering capabilities for React applications,
//! allowing you to render React components on the server using Node.js.
//!
//! ## Features
//!
//! - ⚛️ **React SSR** - Full SSR support for React 18+
//! - 📦 **Static Assets** - Serve compiled React assets
//! - 🔄 **Component Rendering** - Server-side component rendering
//! - ⚡ **Performance** - Fast Node.js integration
//! - 🎯 **Route Control** - Exclude specific routes from SSR
//!
//! ## Quick Start - Configuration
//!
//! ```
//! use armature_react::ReactConfig;
//! use std::path::PathBuf;
//!
//! let config = ReactConfig::new(PathBuf::from("dist"))
//!     .with_node_path("node".to_string())
//!     .with_server_entry("server.js".to_string())
//!     .with_static_dir(PathBuf::from("dist/public"))
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
//! use armature_react::ReactConfig;
//! use std::path::PathBuf;
//!
//! // Enable caching with custom TTL
//! let cached = ReactConfig::new(PathBuf::from("dist"))
//!     .with_cache(true)
//!     .with_cache_ttl(7200); // 2 hours
//!
//! assert!(cached.cache_enabled);
//! assert_eq!(cached.cache_ttl, 7200);
//!
//! // Disable caching (default)
//! let uncached = ReactConfig::new(PathBuf::from("dist"))
//!     .with_cache(false);
//!
//! assert!(!uncached.cache_enabled);
//! ```
//!
//! ## Production vs Development Configuration
//!
//! ```
//! use armature_react::ReactConfig;
//! use std::path::PathBuf;
//!
//! // Production configuration (optimized, with caching)
//! let prod_config = ReactConfig::new(PathBuf::from("dist"))
//!     .with_node_path("node".to_string())
//!     .with_server_entry("server.js".to_string())
//!     .with_cache(true)
//!     .with_cache_ttl(3600);
//!
//! assert!(prod_config.cache_enabled);
//! assert_eq!(prod_config.cache_ttl, 3600);
//!
//! // Development configuration (no caching)
//! let dev_config = ReactConfig::new(PathBuf::from("build"))
//!     .with_node_path("node".to_string())
//!     .with_server_entry("dev-server.js".to_string())
//!     .with_cache(false);
//!
//! assert!(!dev_config.cache_enabled);
//! assert_eq!(dev_config.server_entry, "dev-server.js");
//! ```
//!
//! ## Static Assets Configuration
//!
//! ```
//! use armature_react::ReactConfig;
//! use std::path::PathBuf;
//!
//! // Configure static assets with compression
//! let config = ReactConfig::new(PathBuf::from("dist"))
//!     .with_node_path("node".to_string())
//!     .with_server_entry("server/index.js".to_string())
//!     .with_static_dir(PathBuf::from("dist/public"))
//!     .with_compression(true);
//!
//! assert_eq!(config.static_dir, PathBuf::from("dist/public"));
//! assert_eq!(config.server_entry, "server/index.js");
//! assert!(config.compression);
//! ```
//!
//! ## Creating a React Service
//!
//! ```ignore
//! use armature_react::{ReactConfig, ReactService};
//! use armature_core::HttpRequest;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ReactConfig::new(PathBuf::from("dist"))
//!     .with_node_path("node".to_string())
//!     .with_server_entry("server.js".to_string())
//!     .with_static_dir(PathBuf::from("dist/public"));
//!
//! let service = ReactService::new(config);
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

pub use config::ReactConfig;
pub use renderer::ReactRenderer;
pub use service::ReactService;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        // Ensure module compiles
    }
}
