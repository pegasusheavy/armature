//! Configuration for Handlebars template engine

use std::path::PathBuf;

/// Configuration for Handlebars template engine
#[derive(Debug, Clone)]
pub struct HandlebarsConfig {
    /// Directory containing template files
    pub template_dir: PathBuf,
    
    /// Template file extension (default: ".hbs")
    pub template_extension: String,
    
    /// Enable development mode (disable template caching)
    pub dev_mode: bool,
    
    /// Enable strict mode (error on missing variables)
    pub strict_mode: bool,
    
    /// Directory for partials (if different from template_dir)
    pub partials_dir: Option<PathBuf>,
    
    /// Enable HTML escaping (default: true)
    pub escape_html: bool,
    
    /// Custom delimiters (default: {{ }})
    pub start_delimiter: Option<String>,
    pub end_delimiter: Option<String>,
}

impl HandlebarsConfig {
    /// Create a new configuration with template directory
    pub fn new(template_dir: impl Into<PathBuf>) -> Self {
        Self {
            template_dir: template_dir.into(),
            template_extension: ".hbs".to_string(),
            dev_mode: false,
            strict_mode: false,
            partials_dir: None,
            escape_html: true,
            start_delimiter: None,
            end_delimiter: None,
        }
    }
    
    /// Set template file extension
    pub fn with_extension(mut self, ext: impl Into<String>) -> Self {
        self.template_extension = ext.into();
        self
    }
    
    /// Enable development mode (no template caching)
    pub fn with_dev_mode(mut self, enable: bool) -> Self {
        self.dev_mode = enable;
        self
    }
    
    /// Enable strict mode (error on missing variables)
    pub fn with_strict_mode(mut self, enable: bool) -> Self {
        self.strict_mode = enable;
        self
    }
    
    /// Set partials directory
    pub fn with_partials_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.partials_dir = Some(dir.into());
        self
    }
    
    /// Enable/disable HTML escaping
    pub fn with_escape_html(mut self, enable: bool) -> Self {
        self.escape_html = enable;
        self
    }
    
    /// Set custom delimiters
    pub fn with_delimiters(
        mut self,
        start: impl Into<String>,
        end: impl Into<String>,
    ) -> Self {
        self.start_delimiter = Some(start.into());
        self.end_delimiter = Some(end.into());
        self
    }
}

impl Default for HandlebarsConfig {
    fn default() -> Self {
        Self::new("templates")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = HandlebarsConfig::new("views")
            .with_extension(".html")
            .with_dev_mode(true)
            .with_strict_mode(true);
        
        assert_eq!(config.template_dir, PathBuf::from("views"));
        assert_eq!(config.template_extension, ".html");
        assert!(config.dev_mode);
        assert!(config.strict_mode);
    }
    
    #[test]
    fn test_default_config() {
        let config = HandlebarsConfig::default();
        
        assert_eq!(config.template_dir, PathBuf::from("templates"));
        assert_eq!(config.template_extension, ".hbs");
        assert!(!config.dev_mode);
        assert!(!config.strict_mode);
        assert!(config.escape_html);
    }
    
    #[test]
    fn test_custom_delimiters() {
        let config = HandlebarsConfig::new("templates")
            .with_delimiters("[[", "]]");
        
        assert_eq!(config.start_delimiter, Some("[[".to_string()));
        assert_eq!(config.end_delimiter, Some("]]".to_string()));
    }
}

