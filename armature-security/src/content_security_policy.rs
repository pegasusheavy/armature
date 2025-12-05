//! Content Security Policy (CSP) configuration
//!
//! CSP helps prevent XSS attacks by declaring which dynamic resources are allowed to load.

use std::collections::HashMap;

/// Content Security Policy configuration
#[derive(Debug, Clone)]
pub struct CspConfig {
    /// CSP directives
    pub directives: HashMap<String, Vec<String>>,

    /// Report violations only (doesn't enforce)
    pub report_only: bool,
}

impl CspConfig {
    /// Create a new CSP configuration
    pub fn new() -> Self {
        Self {
            directives: HashMap::new(),
            report_only: false,
        }
    }

    /// Add a directive
    pub fn directive(mut self, name: impl Into<String>, values: Vec<String>) -> Self {
        self.directives.insert(name.into(), values);
        self
    }

    /// Set default-src directive
    pub fn default_src(self, sources: Vec<String>) -> Self {
        self.directive("default-src", sources)
    }

    /// Set script-src directive
    pub fn script_src(self, sources: Vec<String>) -> Self {
        self.directive("script-src", sources)
    }

    /// Set style-src directive
    pub fn style_src(self, sources: Vec<String>) -> Self {
        self.directive("style-src", sources)
    }

    /// Set img-src directive
    pub fn img_src(self, sources: Vec<String>) -> Self {
        self.directive("img-src", sources)
    }

    /// Set connect-src directive
    pub fn connect_src(self, sources: Vec<String>) -> Self {
        self.directive("connect-src", sources)
    }

    /// Set font-src directive
    pub fn font_src(self, sources: Vec<String>) -> Self {
        self.directive("font-src", sources)
    }

    /// Set object-src directive
    pub fn object_src(self, sources: Vec<String>) -> Self {
        self.directive("object-src", sources)
    }

    /// Set media-src directive
    pub fn media_src(self, sources: Vec<String>) -> Self {
        self.directive("media-src", sources)
    }

    /// Set frame-src directive
    pub fn frame_src(self, sources: Vec<String>) -> Self {
        self.directive("frame-src", sources)
    }

    /// Enable report-only mode
    pub fn report_only(mut self, enabled: bool) -> Self {
        self.report_only = enabled;
        self
    }

    /// Convert to header value
    pub fn to_header_value(&self) -> String {
        let mut parts = Vec::new();

        for (directive, values) in &self.directives {
            let value_str = values.join(" ");
            parts.push(format!("{} {}", directive, value_str));
        }

        parts.join("; ")
    }
}

impl Default for CspConfig {
    fn default() -> Self {
        Self::new()
            .default_src(vec!["'self'".to_string()])
            .script_src(vec!["'self'".to_string()])
            .style_src(vec!["'self'".to_string(), "'unsafe-inline'".to_string()])
            .img_src(vec!["'self'".to_string(), "data:".to_string(), "https:".to_string()])
            .font_src(vec!["'self'".to_string()])
            .connect_src(vec!["'self'".to_string()])
            .object_src(vec!["'none'".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csp_default() {
        let csp = CspConfig::default();
        let header = csp.to_header_value();

        assert!(header.contains("default-src 'self'"));
        assert!(header.contains("script-src 'self'"));
        assert!(header.contains("object-src 'none'"));
    }

    #[test]
    fn test_csp_custom() {
        let csp = CspConfig::new()
            .default_src(vec!["'self'".to_string()])
            .script_src(vec!["'self'".to_string(), "https://cdn.example.com".to_string()]);

        let header = csp.to_header_value();
        assert!(header.contains("script-src 'self' https://cdn.example.com"));
    }

    #[test]
    fn test_csp_report_only() {
        let csp = CspConfig::default().report_only(true);
        assert!(csp.report_only);
    }
}

