use crate::error::{Result, XssError};
use ammonia::Builder;

/// HTML sanitizer to prevent XSS attacks
#[derive(Debug, Clone)]
pub struct XssSanitizer {
    allowed_tags: Vec<String>,
    allowed_attributes: Vec<String>,
    strip_comments: bool,
}

impl XssSanitizer {
    /// Create a new sanitizer with default safe settings
    pub fn new() -> Self {
        Self {
            allowed_tags: vec![
                "a", "b", "br", "code", "div", "em", "h1", "h2", "h3", "h4", "h5", "h6", "i",
                "li", "ol", "p", "pre", "span", "strong", "ul",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            allowed_attributes: vec!["href", "title", "class", "id"]
                .into_iter()
                .map(String::from)
                .collect(),
            strip_comments: true,
        }
    }

    /// Create a strict sanitizer (very limited HTML)
    pub fn strict() -> Self {
        Self {
            allowed_tags: vec!["b", "br", "em", "i", "p", "strong"]
                .into_iter()
                .map(String::from)
                .collect(),
            allowed_attributes: Vec::new(),
            strip_comments: true,
        }
    }

    /// Create a permissive sanitizer (more HTML allowed)
    pub fn permissive() -> Self {
        Self {
            allowed_tags: vec![
                "a", "abbr", "b", "blockquote", "br", "cite", "code", "dd", "del", "div", "dl",
                "dt", "em", "h1", "h2", "h3", "h4", "h5", "h6", "hr", "i", "img", "ins", "li",
                "ol", "p", "pre", "q", "small", "span", "strong", "sub", "sup", "table", "tbody",
                "td", "th", "thead", "tr", "ul",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            allowed_attributes: vec!["alt", "class", "href", "id", "src", "title"]
                .into_iter()
                .map(String::from)
                .collect(),
            strip_comments: true,
        }
    }

    /// Set allowed HTML tags
    pub fn with_allowed_tags(mut self, tags: Vec<String>) -> Self {
        self.allowed_tags = tags;
        self
    }

    /// Set allowed attributes
    pub fn with_allowed_attributes(mut self, attributes: Vec<String>) -> Self {
        self.allowed_attributes = attributes;
        self
    }

    /// Set whether to strip HTML comments
    pub fn with_strip_comments(mut self, strip: bool) -> Self {
        self.strip_comments = strip;
        self
    }

    /// Sanitize HTML string
    pub fn sanitize(&self, html: &str) -> Result<String> {
        let mut builder = Builder::default();

        // Configure allowed tags
        builder.tags(self.allowed_tags.iter().map(|s| s.as_str()).collect());

        // Configure allowed attributes
        let attrs: std::collections::HashSet<&str> = self.allowed_attributes.iter().map(|s| s.as_str()).collect();
        builder.generic_attributes(attrs);

        // Configure comment stripping
        if self.strip_comments {
            builder.strip_comments(true);
        }

        let cleaned = builder
            .clean(html)
            .to_string();

        Ok(cleaned)
    }

    /// Sanitize and validate (returns error if content was modified)
    pub fn sanitize_strict(&self, html: &str) -> Result<String> {
        let sanitized = self.sanitize(html)?;

        // If content changed significantly, it might contain malicious code
        if sanitized.len() < html.len() * 2 / 3 {
            return Err(XssError::MaliciousContent(
                "Input contains suspicious HTML".to_string(),
            ));
        }

        Ok(sanitized)
    }
}

impl Default for XssSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_script_tag() {
        let sanitizer = XssSanitizer::new();
        let dirty = r#"<p>Hello</p><script>alert('XSS')</script>"#;
        let clean = sanitizer.sanitize(dirty).unwrap();

        assert!(!clean.contains("script"));
        assert!(clean.contains("Hello"));
    }

    #[test]
    fn test_sanitize_onclick_attribute() {
        let sanitizer = XssSanitizer::new();
        let dirty = "<a href=\"#\" onclick=\"alert('XSS')\">Click</a>";
        let clean = sanitizer.sanitize(dirty).unwrap();

        assert!(!clean.contains("onclick"));
        assert!(clean.contains("Click"));
    }

    #[test]
    fn test_strict_sanitizer() {
        let sanitizer = XssSanitizer::strict();
        let html = r#"<div><p><strong>Bold</strong></p></div>"#;
        let clean = sanitizer.sanitize(html).unwrap();

        assert!(!clean.contains("<div>"));
        assert!(clean.contains("<strong>"));
    }

    #[test]
    fn test_permissive_sanitizer() {
        let sanitizer = XssSanitizer::permissive();
        let html = r#"<div><img src="image.jpg" alt="Test"/></div>"#;
        let clean = sanitizer.sanitize(html).unwrap();

        assert!(clean.contains("<div>"));
        assert!(clean.contains("<img"));
    }

    #[test]
    fn test_sanitize_strict_rejects_suspicious() {
        let sanitizer = XssSanitizer::new();
        let suspicious = r#"<p>x</p><script>lots of malicious code here</script>"#;

        assert!(sanitizer.sanitize_strict(suspicious).is_err());
    }
}

