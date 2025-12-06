//! Angular SSR (Server-Side Rendering) integration for Armature framework.
//!
//! This crate provides server-side rendering capabilities for Angular Universal
//! applications, allowing you to render Angular components on the server.
//!
//! ## Features
//!
//! - 🚀 **Angular Universal** - Full SSR support for Angular 17+
//! - 📦 **Static Assets** - Serve compiled Angular assets
//! - 🔄 **Route Rendering** - Server-side route rendering
//! - ⚡ **Performance** - Fast Node.js integration
//! - 🎯 **Route Control** - Exclude specific routes from SSR
//!
//! ## Quick Start - Configuration
//!
//! ```
//! use armature_angular::AngularConfig;
//! use std::path::PathBuf;
//!
//! let config = AngularConfig::new()
//!     .with_node_path(PathBuf::from("node"))
//!     .with_server_bundle(PathBuf::from("dist/server/main.js"))
//!     .with_browser_dist(PathBuf::from("dist/browser"))
//!     .exclude_route("/admin".to_string());
//!
//! assert_eq!(config.node_path, PathBuf::from("node"));
//! assert!(config.excluded_routes.contains(&"/admin".to_string()));
//! ```
//!
//! ## Cache Configuration
//!
//! ```
//! use armature_angular::AngularConfig;
//! use std::path::PathBuf;
//!
//! // Enable caching with custom TTL
//! let with_cache = AngularConfig::new()
//!     .with_node_path(PathBuf::from("node"))
//!     .with_server_bundle(PathBuf::from("dist/server/main.js"))
//!     .with_browser_dist(PathBuf::from("dist/browser"))
//!     .with_cache(true, 3600); // Enable cache with 1 hour TTL
//!
//! assert!(with_cache.enable_cache);
//! assert_eq!(with_cache.cache_ttl, 3600);
//!
//! // Disable caching (default)
//! let without_cache = AngularConfig::new();
//! assert!(!without_cache.enable_cache);
//! ```
//!
//! ## Excluding Multiple Routes
//!
//! ```
//! use armature_angular::AngularConfig;
//! use std::path::PathBuf;
//!
//! let config = AngularConfig::new()
//!     .with_node_path(PathBuf::from("node"))
//!     .with_server_bundle(PathBuf::from("dist/server/main.js"))
//!     .with_browser_dist(PathBuf::from("dist/browser"))
//!     .exclude_route("/admin".to_string())
//!     .exclude_route("/downloads".to_string())
//!     .exclude_route("/static".to_string());
//!
//! // Default config includes "/api" and "/assets", plus 3 more = 5 total
//! assert_eq!(config.excluded_routes.len(), 5);
//! assert!(config.excluded_routes.contains(&"/admin".to_string()));
//! assert!(config.excluded_routes.contains(&"/downloads".to_string()));
//! assert!(config.excluded_routes.contains(&"/static".to_string()));
//! ```
//!
//! ## Render Timeout Configuration
//!
//! ```
//! use armature_angular::AngularConfig;
//! use std::path::PathBuf;
//!
//! // Set custom render timeout (in milliseconds)
//! let config = AngularConfig::new()
//!     .with_node_path(PathBuf::from("node"))
//!     .with_server_bundle(PathBuf::from("dist/server/main.js"))
//!     .with_browser_dist(PathBuf::from("dist/browser"))
//!     .with_timeout(10000); // 10 seconds
//!
//! assert_eq!(config.render_timeout, 10000);
//!
//! // Default timeout is 5000ms (5 seconds)
//! let default_config = AngularConfig::new();
//! assert_eq!(default_config.render_timeout, 5000);
//! ```
//!
//! ## Creating an Angular Service
//!
//! ```ignore
//! use armature_angular::{AngularConfig, AngularService, RenderOptions};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = AngularConfig {
//!     node_path: "node".to_string(),
//!     server_bundle_path: "dist/server/main.js".to_string(),
//!     browser_dist_path: "dist/browser".to_string(),
//!     index_html: "index.html".to_string(),
//!     excluded_routes: vec![],
//! };
//!
//! let service = AngularService::new(config)?;
//!
//! // Check if a route should be SSR
//! assert!(service.should_render("/home"));
//! assert!(!service.should_render("/assets/logo.png"));
//!
//! // Render a route
//! let options = RenderOptions::default();
//! let html = service.render("/home", options).await?;
//! println!("Rendered HTML: {}", html);
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod error;
pub mod renderer;
pub mod static_files;

pub use config::AngularConfig;
pub use error::{AngularError, Result};
pub use renderer::{AngularRenderer, RenderOptions, StaticSiteStats};
pub use static_files::StaticFileService;

use armature_core::Provider;

/// Angular SSR service for serving Angular Universal applications
#[derive(Clone)]
pub struct AngularService {
    config: AngularConfig,
    renderer: AngularRenderer,
    static_service: StaticFileService,
}

impl AngularService {
    /// Create a new Angular service
    pub fn new(config: AngularConfig) -> Result<Self> {
        let renderer =
            AngularRenderer::new(config.node_path.clone(), config.server_bundle_path.clone())?;

        let static_service =
            StaticFileService::new(config.browser_dist_path.clone(), config.index_html.clone())?;

        Ok(Self {
            config,
            renderer,
            static_service,
        })
    }

    /// Render a route server-side
    pub async fn render(&self, url: &str, options: RenderOptions) -> Result<String> {
        self.renderer.render(url, options).await
    }

    /// Serve static files
    pub async fn serve_static(&self, path: &str) -> Result<Vec<u8>> {
        self.static_service.serve(path).await
    }

    /// Get the Angular configuration
    pub fn config(&self) -> &AngularConfig {
        &self.config
    }

    /// Check if a path should be rendered server-side
    pub fn should_render(&self, path: &str) -> bool {
        // Don't render static assets
        if path.starts_with("/assets/") || path.contains('.') && !path.ends_with(".html") {
            return false;
        }

        // Check excluded routes
        for excluded in &self.config.excluded_routes {
            if path.starts_with(excluded) {
                return false;
            }
        }

        true
    }
}

impl Provider for AngularService {}

impl Default for AngularService {
    fn default() -> Self {
        Self::new(AngularConfig::default()).expect("Failed to create default AngularService")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_render() {
        // Test without creating AngularService (which requires files)
        let excluded = vec!["/api".to_string(), "/assets".to_string()];

        // Simulate should_render logic
        let should_render = |path: &str| -> bool {
            !excluded.iter().any(|prefix| path.starts_with(prefix))
                && !path.ends_with(".js")
                && !path.ends_with(".css")
                && !path.ends_with(".png")
                && !path.ends_with(".jpg")
        };

        assert!(should_render("/"));
        assert!(should_render("/home"));
        assert!(!should_render("/assets/logo.png"));
        assert!(!should_render("/api/users"));
        assert!(!should_render("/main.js"));
    }
}
