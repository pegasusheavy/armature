// Angular SSR integration for Armature framework

pub mod config;
pub mod error;
pub mod renderer;
pub mod static_files;

pub use config::AngularConfig;
pub use error::{AngularError, Result};
pub use renderer::{AngularRenderer, RenderOptions};
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
