//! Svelte SSR (Server-Side Rendering) integration for Armature framework.
//!
//! This crate provides server-side rendering capabilities for Svelte applications,
//! allowing you to render Svelte components on the server using Node.js.
//!
//! ## Features
//!
//! - 🧡 **Svelte SSR** - Full SSR support for Svelte 4+
//! - 📦 **Static Assets** - Serve compiled Svelte assets
//! - 🔄 **Component Rendering** - Server-side component rendering
//! - ⚡ **Performance** - Fast Node.js integration
//! - 🎯 **Route Control** - Exclude specific routes from SSR
//!
//! ## Quick Start - Configuration
//!
//! ```
//! use armature_svelte::SvelteConfig;
//! use std::path::PathBuf;
//!
//! let config = SvelteConfig::new(PathBuf::from("dist"))
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
//! ## Creating a Svelte Service
//!
//! ```ignore
//! use armature_svelte::{SvelteConfig, SvelteService};
//! use armature_core::HttpRequest;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = SvelteConfig::new(PathBuf::from("dist"))
//!     .with_node_path("node".to_string())
//!     .with_server_entry("server.js".to_string())
//!     .with_static_dir(PathBuf::from("dist/public"));
//!
//! let service = SvelteService::new(config);
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

pub use config::SvelteConfig;
pub use renderer::SvelteRenderer;
pub use service::SvelteService;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        // Ensure module compiles
    }
}
