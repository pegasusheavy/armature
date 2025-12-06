use crate::error::{Result, XssError};
use once_cell::sync::Lazy;
use regex::Regex;

/// XSS pattern validator
pub struct XssValidator;

// Common XSS attack patterns
static SCRIPT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)<script[^>]*>.*?</script>|<script[^>]*/>").unwrap()
});

static ONERROR_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)onerror\s*=").unwrap()
});

static ONCLICK_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)on\w+\s*=").unwrap()
});

static JAVASCRIPT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)javascript:").unwrap()
});

static DATA_URL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)data:text/html").unwrap()
});

static VBSCRIPT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)vbscript:").unwrap()
});

static IFRAME_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)<iframe[^>]*>").unwrap()
});

static OBJECT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)<object[^>]*>").unwrap()
});

static EMBED_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)<embed[^>]*>").unwrap()
});

impl XssValidator {
    /// Check if text contains potential XSS attacks
    pub fn contains_xss(text: &str) -> bool {
        SCRIPT_PATTERN.is_match(text)
            || ONERROR_PATTERN.is_match(text)
            || ONCLICK_PATTERN.is_match(text)
            || JAVASCRIPT_PATTERN.is_match(text)
            || DATA_URL_PATTERN.is_match(text)
            || VBSCRIPT_PATTERN.is_match(text)
            || IFRAME_PATTERN.is_match(text)
            || OBJECT_PATTERN.is_match(text)
            || EMBED_PATTERN.is_match(text)
    }

    /// Validate text and return error if XSS detected
    pub fn validate(text: &str) -> Result<()> {
        if Self::contains_xss(text) {
            return Err(XssError::ValidationFailed(
                "Potentially malicious content detected".to_string(),
            ));
        }
        Ok(())
    }

    /// Detect specific XSS attack type
    pub fn detect_attack_type(text: &str) -> Option<&'static str> {
        if SCRIPT_PATTERN.is_match(text) {
            return Some("Script injection");
        }
        if ONERROR_PATTERN.is_match(text) {
            return Some("Event handler injection (onerror)");
        }
        if ONCLICK_PATTERN.is_match(text) {
            return Some("Event handler injection");
        }
        if JAVASCRIPT_PATTERN.is_match(text) {
            return Some("JavaScript protocol");
        }
        if DATA_URL_PATTERN.is_match(text) {
            return Some("Data URL injection");
        }
        if VBSCRIPT_PATTERN.is_match(text) {
            return Some("VBScript injection");
        }
        if IFRAME_PATTERN.is_match(text) {
            return Some("Iframe injection");
        }
        if OBJECT_PATTERN.is_match(text) {
            return Some("Object tag injection");
        }
        if EMBED_PATTERN.is_match(text) {
            return Some("Embed tag injection");
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_injection() {
        let xss = r#"<script>alert('XSS')</script>"#;
        assert!(XssValidator::contains_xss(xss));
        assert!(XssValidator::validate(xss).is_err());
    }

    #[test]
    fn test_onerror_injection() {
        let xss = r#"<img src="x" onerror="alert('XSS')">"#;
        assert!(XssValidator::contains_xss(xss));
    }

    #[test]
    fn test_onclick_injection() {
        let xss = "<a href=\"#\" onclick=\"alert('XSS')\">Click</a>";
        assert!(XssValidator::contains_xss(xss));
    }

    #[test]
    fn test_javascript_protocol() {
        let xss = r#"<a href="javascript:alert('XSS')">Click</a>"#;
        assert!(XssValidator::contains_xss(xss));
    }

    #[test]
    fn test_data_url() {
        let xss = r#"<a href="data:text/html,<script>alert('XSS')</script>">Click</a>"#;
        assert!(XssValidator::contains_xss(xss));
    }

    #[test]
    fn test_iframe_injection() {
        let xss = r#"<iframe src="javascript:alert('XSS')"></iframe>"#;
        assert!(XssValidator::contains_xss(xss));
    }

    #[test]
    fn test_safe_content() {
        let safe = r#"<p>Hello <strong>world</strong>!</p>"#;
        assert!(!XssValidator::contains_xss(safe));
        assert!(XssValidator::validate(safe).is_ok());
    }

    #[test]
    fn test_detect_attack_type() {
        assert_eq!(
            XssValidator::detect_attack_type(r#"<script>alert('XSS')</script>"#),
            Some("Script injection")
        );
        assert_eq!(
            XssValidator::detect_attack_type(r#"<img onerror="alert()">"#),
            Some("Event handler injection (onerror)")
        );
        assert_eq!(
            XssValidator::detect_attack_type(r#"<a href="javascript:void(0)">Link</a>"#),
            Some("JavaScript protocol")
        );
    }
}

