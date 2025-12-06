use crate::{sanitizer::XssSanitizer, validator::XssValidator};
use armature_core::{Error as ArmatureError, HttpRequest, HttpResponse};
use serde_json::Value;
use std::sync::Arc;

/// XSS protection configuration
#[derive(Debug, Clone)]
pub struct XssConfig {
    /// Enable automatic sanitization
    pub auto_sanitize: bool,

    /// Enable validation (reject requests with XSS)
    pub enable_validation: bool,

    /// Paths to exclude from XSS protection
    pub exclude_paths: Vec<String>,

    /// Content types to check
    pub check_content_types: Vec<String>,
}

impl XssConfig {
    pub fn new() -> Self {
        Self {
            auto_sanitize: false,
            enable_validation: true,
            exclude_paths: Vec::new(),
            check_content_types: vec![
                "application/json".to_string(),
                "application/x-www-form-urlencoded".to_string(),
                "text/plain".to_string(),
            ],
        }
    }

    pub fn with_auto_sanitize(mut self, enable: bool) -> Self {
        self.auto_sanitize = enable;
        self
    }

    pub fn with_validation(mut self, enable: bool) -> Self {
        self.enable_validation = enable;
        self
    }

    pub fn with_exclude_paths(mut self, paths: Vec<String>) -> Self {
        self.exclude_paths = paths;
        self
    }
}

impl Default for XssConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// XSS protection middleware
#[derive(Clone)]
pub struct XssMiddleware {
    config: Arc<XssConfig>,
    sanitizer: XssSanitizer,
}

impl XssMiddleware {
    /// Create new XSS middleware
    pub fn new(config: XssConfig) -> Self {
        Self {
            config: Arc::new(config),
            sanitizer: XssSanitizer::new(),
        }
    }

    /// Create with custom sanitizer
    pub fn with_sanitizer(config: XssConfig, sanitizer: XssSanitizer) -> Self {
        Self {
            config: Arc::new(config),
            sanitizer,
        }
    }

    /// Check if request needs XSS protection
    pub fn needs_protection(&self, request: &HttpRequest) -> bool {
        let path = &request.path;
        for excluded in &self.config.exclude_paths {
            if path.starts_with(excluded) {
                return false;
            }
        }
        true
    }

    /// Validate request for XSS
    pub fn validate_request(&self, request: &HttpRequest) -> Result<(), ArmatureError> {
        if !self.config.enable_validation || !self.needs_protection(request) {
            return Ok(());
        }

        let body = &request.body;
        let body_str = String::from_utf8_lossy(body);

        // Check if content contains XSS
        if let Some(attack_type) = XssValidator::detect_attack_type(&body_str) {
            return Err(ArmatureError::BadRequest(format!(
                "XSS attack detected: {}",
                attack_type
            )));
        }

        Ok(())
    }

    /// Sanitize request body
    pub fn sanitize_request(&self, request: &mut HttpRequest) -> Result<(), ArmatureError> {
        if !self.config.auto_sanitize || !self.needs_protection(request) {
            return Ok(());
        }

        let body = request.body.clone();

        // Try to parse as JSON
        if let Ok(mut json) = serde_json::from_slice::<Value>(&body) {
            self.sanitize_json_value(&mut json)?;
            let sanitized = serde_json::to_vec(&json)
                .map_err(|e| ArmatureError::Internal(e.to_string()))?;
            request.body = sanitized;
            return Ok(());
        }

        // Otherwise sanitize as plain text
        let body_str = String::from_utf8_lossy(&body);
        let sanitized = self
            .sanitizer
            .sanitize(&body_str)
            .map_err(|e| ArmatureError::Internal(e.to_string()))?;
        request.body = sanitized.into_bytes();

        Ok(())
    }

    /// Recursively sanitize JSON values
    fn sanitize_json_value(&self, value: &mut Value) -> Result<(), ArmatureError> {
        match value {
            Value::String(s) => {
                *s = self
                    .sanitizer
                    .sanitize(s)
                    .map_err(|e| ArmatureError::Internal(e.to_string()))?;
            }
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    self.sanitize_json_value(item)?;
                }
            }
            Value::Object(obj) => {
                for (_key, val) in obj.iter_mut() {
                    self.sanitize_json_value(val)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Add XSS protection headers to response
    pub fn add_protection_headers(&self, mut response: HttpResponse) -> HttpResponse {
        // X-XSS-Protection header (legacy, but still useful)
        response.headers.insert(
            "X-XSS-Protection".to_string(),
            "1; mode=block".to_string(),
        );

        // X-Content-Type-Options (prevent MIME sniffing)
        response.headers.insert(
            "X-Content-Type-Options".to_string(),
            "nosniff".to_string(),
        );

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use armature_core::HttpMethod;

    #[test]
    fn test_validation_detects_xss() {
        let middleware = XssMiddleware::new(XssConfig::default());
        let mut req = HttpRequest::new(HttpMethod::Post, "/api/submit");
        req = req.with_body(b"<script>alert('XSS')</script>".to_vec());

        assert!(middleware.validate_request(&req).is_err());
    }

    #[test]
    fn test_validation_allows_safe_content() {
        let middleware = XssMiddleware::new(XssConfig::default());
        let mut req = HttpRequest::new(HttpMethod::Post, "/api/submit");
        req = req.with_body(b"<p>Hello world</p>".to_vec());

        assert!(middleware.validate_request(&req).is_ok());
    }

    #[test]
    fn test_exclude_paths() {
        let config = XssConfig::default().with_exclude_paths(vec!["/api/webhook".to_string()]);
        let middleware = XssMiddleware::new(config);

        let protected_req = HttpRequest::new(HttpMethod::Post, "/api/submit");
        assert!(middleware.needs_protection(&protected_req));

        let excluded_req = HttpRequest::new(HttpMethod::Post, "/api/webhook/receive");
        assert!(!middleware.needs_protection(&excluded_req));
    }

    #[test]
    fn test_add_protection_headers() {
        let middleware = XssMiddleware::new(XssConfig::default());
        let response = HttpResponse::ok();
        let protected_response = middleware.add_protection_headers(response);

        assert_eq!(
            protected_response.headers.get("X-XSS-Protection"),
            Some(&"1; mode=block".to_string())
        );
        assert_eq!(
            protected_response.headers.get("X-Content-Type-Options"),
            Some(&"nosniff".to_string())
        );
    }
}

