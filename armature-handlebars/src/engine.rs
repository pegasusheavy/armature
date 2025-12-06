//! Handlebars template engine wrapper

use crate::{config::HandlebarsConfig, error::HandlebarsError, helpers, Result};
use handlebars::Handlebars;
use serde::Serialize;
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Handlebars template engine
#[derive(Clone)]
pub struct HandlebarsEngine {
    handlebars: Arc<RwLock<Handlebars<'static>>>,
    config: HandlebarsConfig,
}

impl HandlebarsEngine {
    /// Create a new Handlebars engine with configuration
    pub fn new(config: HandlebarsConfig) -> Result<Self> {
        let mut handlebars = Handlebars::new();
        
        // Configure strict mode
        handlebars.set_strict_mode(config.strict_mode);
        
        // Configure HTML escaping
        if !config.escape_html {
            handlebars.register_escape_fn(handlebars::no_escape);
        }
        
        // Note: Custom delimiters are not supported in handlebars 5.x
        // Delimiters are always {{ }}
        
        // Register built-in helpers
        helpers::register_builtin_helpers(&mut handlebars);
        
        let engine = Self {
            handlebars: Arc::new(RwLock::new(handlebars)),
            config,
        };
        
        // Load templates from directory
        engine.load_templates()?;
        
        Ok(engine)
    }
    
    /// Load all templates from the configured directory
    fn load_templates(&self) -> Result<()> {
        if !self.config.template_dir.exists() {
            return Err(HandlebarsError::ConfigError(format!(
                "Template directory not found: {:?}",
                self.config.template_dir
            )));
        }
        
        // Walk the template directory and register templates
        self.load_templates_from_dir(&self.config.template_dir)?;
        
        // Load partials if specified
        if let Some(ref partials_dir) = self.config.partials_dir {
            if partials_dir.exists() {
                self.load_templates_from_dir(partials_dir)?;
            }
        }
        
        Ok(())
    }
    
    /// Load templates from a directory recursively
    fn load_templates_from_dir(&self, dir: &Path) -> Result<()> {
        use std::fs;
        
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                self.load_templates_from_dir(&path)?;
            } else if let Some(ext) = path.extension() {
                if ext == self.config.template_extension.trim_start_matches('.') {
                    let template_name = path
                        .strip_prefix(&self.config.template_dir)
                        .unwrap_or(&path)
                        .with_extension("")
                        .to_string_lossy()
                        .replace('\\', "/");
                    
                    let template_content = fs::read_to_string(&path)?;
                    
                    let mut handlebars = self.handlebars.write().unwrap();
                    handlebars.register_template_string(&template_name, template_content)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Render a template with data
    pub fn render<T: Serialize>(&self, template: &str, data: &T) -> Result<String> {
        // In dev mode, reload templates on each render
        if self.config.dev_mode {
            self.reload_templates()?;
        }
        
        let handlebars = self.handlebars.read().unwrap();
        handlebars
            .render(template, data)
            .map_err(HandlebarsError::from)
    }
    
    /// Render a template string (not from file)
    pub fn render_template<T: Serialize>(&self, template_str: &str, data: &T) -> Result<String> {
        let handlebars = self.handlebars.read().unwrap();
        handlebars
            .render_template(template_str, data)
            .map_err(HandlebarsError::from)
    }
    
    /// Register a template from string
    pub fn register_template(&self, name: &str, template: &str) -> Result<()> {
        let mut handlebars = self.handlebars.write().unwrap();
        handlebars
            .register_template_string(name, template)
            .map_err(HandlebarsError::from)
    }
    
    /// Register a partial
    pub fn register_partial(&self, name: &str, template: &str) -> Result<()> {
        let mut handlebars = self.handlebars.write().unwrap();
        handlebars
            .register_partial(name, template)
            .map_err(HandlebarsError::from)
    }
    
    /// Register a custom helper
    pub fn register_helper<F>(&self, name: &str, helper: F) -> Result<()>
    where
        F: handlebars::HelperDef + Send + Sync + 'static,
    {
        let mut handlebars = self.handlebars.write().unwrap();
        handlebars.register_helper(name, Box::new(helper));
        Ok(())
    }
    
    /// Unregister a template
    pub fn unregister_template(&self, name: &str) {
        let mut handlebars = self.handlebars.write().unwrap();
        handlebars.unregister_template(name);
    }
    
    /// Check if a template exists
    pub fn has_template(&self, name: &str) -> bool {
        let handlebars = self.handlebars.read().unwrap();
        handlebars.has_template(name)
    }
    
    /// Get list of registered template names
    pub fn get_templates(&self) -> Vec<String> {
        let handlebars = self.handlebars.read().unwrap();
        handlebars.get_templates().keys().cloned().collect()
    }
    
    /// Reload all templates from disk
    pub fn reload_templates(&self) -> Result<()> {
        let mut handlebars = self.handlebars.write().unwrap();
        handlebars.clear_templates();
        drop(handlebars);
        self.load_templates()
    }
    
    /// Get configuration
    pub fn config(&self) -> &HandlebarsConfig {
        &self.config
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
        
        // Create a test template
        fs::write(
            templates_dir.join("test.hbs"),
            "<h1>Hello {{name}}!</h1>"
        ).unwrap();
        
        fs::write(
            templates_dir.join("list.hbs"),
            "{{#each items}}<li>{{this}}</li>{{/each}}"
        ).unwrap();
        
        temp_dir
    }

    #[test]
    fn test_engine_creation() {
        let temp_dir = create_test_templates();
        let config = HandlebarsConfig::new(temp_dir.path().join("templates"));
        
        let engine = HandlebarsEngine::new(config);
        assert!(engine.is_ok());
    }
    
    #[test]
    fn test_render_template() {
        let temp_dir = create_test_templates();
        let config = HandlebarsConfig::new(temp_dir.path().join("templates"));
        let engine = HandlebarsEngine::new(config).unwrap();
        
        let data = json!({"name": "World"});
        let result = engine.render("test", &data).unwrap();
        assert_eq!(result, "<h1>Hello World!</h1>");
    }
    
    #[test]
    fn test_render_template_string() {
        let temp_dir = create_test_templates();
        let config = HandlebarsConfig::new(temp_dir.path().join("templates"));
        let engine = HandlebarsEngine::new(config).unwrap();
        
        let data = json!({"count": 42});
        let result = engine.render_template("Count: {{count}}", &data).unwrap();
        assert_eq!(result, "Count: 42");
    }
    
    #[test]
    fn test_register_template() {
        let temp_dir = create_test_templates();
        let config = HandlebarsConfig::new(temp_dir.path().join("templates"));
        let engine = HandlebarsEngine::new(config).unwrap();
        
        engine.register_template("custom", "<p>{{message}}</p>").unwrap();
        
        let data = json!({"message": "Hello"});
        let result = engine.render("custom", &data).unwrap();
        assert_eq!(result, "<p>Hello</p>");
    }
    
    #[test]
    fn test_has_template() {
        let temp_dir = create_test_templates();
        let config = HandlebarsConfig::new(temp_dir.path().join("templates"));
        let engine = HandlebarsEngine::new(config).unwrap();
        
        assert!(engine.has_template("test"));
        assert!(engine.has_template("list"));
        assert!(!engine.has_template("nonexistent"));
    }
    
    #[test]
    fn test_get_templates() {
        let temp_dir = create_test_templates();
        let config = HandlebarsConfig::new(temp_dir.path().join("templates"));
        let engine = HandlebarsEngine::new(config).unwrap();
        
        let templates = engine.get_templates();
        assert!(templates.contains(&"test".to_string()));
        assert!(templates.contains(&"list".to_string()));
    }
    
    #[test]
    fn test_strict_mode() {
        let temp_dir = create_test_templates();
        let config = HandlebarsConfig::new(temp_dir.path().join("templates"))
            .with_strict_mode(true);
        let engine = HandlebarsEngine::new(config).unwrap();
        
        engine.register_template("strict", "{{missing}}").unwrap();
        
        let data = json!({});
        let result = engine.render("strict", &data);
        assert!(result.is_err());
    }
}

