//! Handlebars templating integration for Armature framework
//!
//! This crate provides server-side template rendering using Handlebars templates.
//!
//! ## Features
//!
//! - 🎨 Full Handlebars templating support
//! - 🔧 Built-in helpers (eq, ne, lt, gt, upper, lower, len, etc.)
//! - 🧩 Partial templates support
//! - 🔄 Development mode with hot-reload
//! - ⚙️ Configurable delimiters
//! - 🛡️ Strict mode for missing variables
//! - 🔒 HTML escaping (configurable)
//! - 📝 Custom helper registration
//!
//! ## Example
//!
//! ```no_run
//! use armature_handlebars::{HandlebarsConfig, HandlebarsService};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create configuration
//! let config = HandlebarsConfig::new("templates")
//!     .with_extension(".hbs")
//!     .with_dev_mode(false);
//!
//! // Create service
//! let service = HandlebarsService::new(config)?;
//!
//! // Render template
//! let data = json!({"name": "World", "count": 42});
//! let html = service.render("index", &data).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Built-in Helpers
//!
//! - **Comparison**: `eq`, `ne`, `lt`, `gt`, `lte`, `gte`
//! - **Logic**: `and`, `or`, `not`
//! - **String**: `upper`, `lower`, `capitalize`
//! - **Utility**: `len`, `json`, `default`
//!
//! ## Template Example
//!
//! ```handlebars
//! <h1>Hello {{name}}!</h1>
//!
//! {{#if (gt count 10)}}
//!   <p>Count is greater than 10!</p>
//! {{/if}}
//!
//! <ul>
//! {{#each items}}
//!   <li>{{upper this}}</li>
//! {{/each}}
//! </ul>
//! ```

pub mod config;
pub mod engine;
pub mod error;
pub mod helpers;

pub use config::HandlebarsConfig;
pub use engine::HandlebarsEngine;
pub use error::{HandlebarsError, Result};

use armature_core::{HttpRequest, HttpResponse, Provider};
use serde::Serialize;

/// Handlebars template service for Armature
#[derive(Clone)]
pub struct HandlebarsService {
    engine: HandlebarsEngine,
}

impl HandlebarsService {
    /// Create a new Handlebars service
    pub fn new(config: HandlebarsConfig) -> Result<Self> {
        let engine = HandlebarsEngine::new(config)?;
        Ok(Self { engine })
    }

    /// Render a template with data
    pub async fn render<T: Serialize>(&self, template: &str, data: &T) -> Result<String> {
        // Use tokio::task::spawn_blocking for CPU-intensive template rendering
        let engine = self.engine.clone();
        let template = template.to_string();
        let data_json = serde_json::to_value(data)?;

        tokio::task::spawn_blocking(move || engine.render(&template, &data_json))
            .await
            .map_err(|e| HandlebarsError::RenderError(e.to_string()))?
    }

    /// Render a template and return as HTTP response
    pub async fn render_response<T: Serialize>(
        &self,
        template: &str,
        data: &T,
    ) -> Result<HttpResponse> {
        let html = self.render(template, data).await?;

        Ok(HttpResponse::ok()
            .with_header("Content-Type".to_string(), "text/html; charset=utf-8".to_string())
            .with_body(html.into_bytes()))
    }

    /// Render a template string (not from file)
    pub async fn render_template<T: Serialize>(&self, template_str: &str, data: &T) -> Result<String> {
        let engine = self.engine.clone();
        let template_str = template_str.to_string();
        let data_json = serde_json::to_value(data)?;

        tokio::task::spawn_blocking(move || engine.render_template(&template_str, &data_json))
            .await
            .map_err(|e| HandlebarsError::RenderError(e.to_string()))?
    }

    /// Register a template from string
    pub fn register_template(&self, name: &str, template: &str) -> Result<()> {
        self.engine.register_template(name, template)
    }

    /// Register a partial
    pub fn register_partial(&self, name: &str, template: &str) -> Result<()> {
        self.engine.register_partial(name, template)
    }

    /// Register a custom helper
    pub fn register_helper<F>(&self, name: &str, helper: F) -> Result<()>
    where
        F: handlebars::HelperDef + Send + Sync + 'static,
    {
        self.engine.register_helper(name, helper)
    }

    /// Check if a template exists
    pub fn has_template(&self, name: &str) -> bool {
        self.engine.has_template(name)
    }

    /// Get list of registered template names
    pub fn get_templates(&self) -> Vec<String> {
        self.engine.get_templates()
    }

    /// Reload all templates from disk
    pub async fn reload_templates(&self) -> Result<()> {
        let engine = self.engine.clone();
        tokio::task::spawn_blocking(move || engine.reload_templates())
            .await
            .map_err(|e| HandlebarsError::RenderError(e.to_string()))?
    }

    /// Get configuration
    pub fn config(&self) -> &HandlebarsConfig {
        self.engine.config()
    }

    /// Get engine reference
    pub fn engine(&self) -> &HandlebarsEngine {
        &self.engine
    }
}

impl Provider for HandlebarsService {}

impl Default for HandlebarsService {
    fn default() -> Self {
        Self::new(HandlebarsConfig::default()).expect("Failed to create default HandlebarsService")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_templates() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let templates_dir = temp_dir.path().join("templates");
        fs::create_dir(&templates_dir).unwrap();

        fs::write(
            templates_dir.join("hello.hbs"),
            "<h1>Hello {{name}}!</h1>"
        ).unwrap();

        temp_dir
    }

    #[tokio::test]
    async fn test_service_render() {
        let temp_dir = create_test_templates();
        let config = HandlebarsConfig::new(temp_dir.path().join("templates"));
        let service = HandlebarsService::new(config).unwrap();

        let data = json!({"name": "Armature"});
        let result = service.render("hello", &data).await.unwrap();
        assert_eq!(result, "<h1>Hello Armature!</h1>");
    }

    #[tokio::test]
    async fn test_service_render_response() {
        let temp_dir = create_test_templates();
        let config = HandlebarsConfig::new(temp_dir.path().join("templates"));
        let service = HandlebarsService::new(config).unwrap();

        let data = json!({"name": "Armature"});
        let response = service.render_response("hello", &data).await.unwrap();

        assert_eq!(response.status, 200);
        assert_eq!(
            response.headers.get("Content-Type"),
            Some(&"text/html; charset=utf-8".to_string())
        );
        assert_eq!(
            String::from_utf8(response.body).unwrap(),
            "<h1>Hello Armature!</h1>"
        );
    }

    #[tokio::test]
    async fn test_service_render_template_string() {
        let temp_dir = create_test_templates();
        let config = HandlebarsConfig::new(temp_dir.path().join("templates"));
        let service = HandlebarsService::new(config).unwrap();

        let data = json!({"value": 123});
        let result = service.render_template("Value: {{value}}", &data).await.unwrap();
        assert_eq!(result, "Value: 123");
    }
}
