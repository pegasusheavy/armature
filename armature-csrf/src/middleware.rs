use crate::{config::CsrfConfig, error::CsrfError, token::CsrfToken};
use armature_core::{Error as ArmatureError, HttpRequest, HttpResponse};
use std::sync::Arc;

/// CSRF protection middleware
#[derive(Clone)]
pub struct CsrfMiddleware {
    config: Arc<CsrfConfig>,
}

impl CsrfMiddleware {
    /// Create new CSRF middleware
    pub fn new(config: CsrfConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Check if request needs CSRF protection
    pub fn needs_protection(&self, request: &HttpRequest) -> bool {
        // Check if method is safe
        if self
            .config
            .safe_methods
            .contains(&request.method.to_string())
        {
            return false;
        }

        // Check if path is excluded
        let path = &request.path;
        for excluded in &self.config.exclude_paths {
            if path.starts_with(excluded) {
                return false;
            }
        }

        true
    }

    /// Generate CSRF token for response
    pub fn generate_token(&self) -> Result<CsrfToken, CsrfError> {
        Ok(CsrfToken::generate(self.config.token_ttl))
    }

    /// Add CSRF token to response as cookie
    pub fn add_token_cookie(
        &self,
        mut response: HttpResponse,
        token: &CsrfToken,
    ) -> Result<HttpResponse, CsrfError> {
        let encoded_token = token.encode(&self.config.secret)?;

        let mut cookie = format!(
            "{}={}; Path={}",
            self.config.cookie_name, encoded_token, self.config.cookie_path
        );

        if let Some(ref domain) = self.config.cookie_domain {
            cookie.push_str(&format!("; Domain={}", domain));
        }

        if self.config.cookie_secure {
            cookie.push_str("; Secure");
        }

        if self.config.cookie_http_only {
            cookie.push_str("; HttpOnly");
        }

        cookie.push_str(&format!("; SameSite={}", self.config.cookie_same_site.as_str()));

        response
            .headers
            .insert("Set-Cookie".to_string(), cookie);

        Ok(response)
    }

    /// Validate CSRF token from request
    pub fn validate_request(&self, request: &HttpRequest) -> Result<(), ArmatureError> {
        if !self.needs_protection(request) {
            return Ok(());
        }

        // Get token from header or form field
        let token_string = self
            .get_token_from_header(request)
            .or_else(|| self.get_token_from_body(request))
            .ok_or_else(|| {
                ArmatureError::Forbidden(format!(
                    "Missing CSRF token. Provide token in '{}' header or '{}' form field",
                    self.config.header_name, self.config.field_name
                ))
            })?;

        // Decode and validate token
        CsrfToken::decode(&token_string, &self.config.secret).map_err(|e| match e {
            CsrfError::TokenExpired => {
                ArmatureError::Forbidden("CSRF token expired".to_string())
            }
            CsrfError::InvalidToken => {
                ArmatureError::Forbidden("Invalid CSRF token".to_string())
            }
            _ => ArmatureError::Forbidden("CSRF validation failed".to_string()),
        })?;

        Ok(())
    }

    /// Get token from header
    fn get_token_from_header(&self, request: &HttpRequest) -> Option<String> {
        request
            .headers
            .get(&self.config.header_name)
            .or_else(|| request.headers.get(&self.config.header_name.to_lowercase()))
            .cloned()
    }

    /// Get token from request body
    fn get_token_from_body(&self, request: &HttpRequest) -> Option<String> {
        // Try to parse as JSON
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&request.body) {
            if let Some(token) = json.get(&self.config.field_name) {
                return token.as_str().map(|s| s.to_string());
            }
        }

        // Try to parse as form data
        if let Ok(form_data) = serde_urlencoded::from_bytes::<Vec<(String, String)>>(&request.body) {
            for (key, value) in form_data {
                if key == self.config.field_name {
                    return Some(value);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use armature_core::HttpMethod;

    #[test]
    fn test_safe_methods() {
        let config = CsrfConfig::default();
        let middleware = CsrfMiddleware::new(config);

        let get_req = HttpRequest::new(HttpMethod::Get, "/test");
        assert!(!middleware.needs_protection(&get_req));

        let post_req = HttpRequest::new(HttpMethod::Post, "/test");
        assert!(middleware.needs_protection(&post_req));
    }

    #[test]
    fn test_excluded_paths() {
        let config = CsrfConfig::default().with_exclude_paths(vec!["/api/public".to_string()]);
        let middleware = CsrfMiddleware::new(config);

        let excluded_req = HttpRequest::new(HttpMethod::Post, "/api/public/login");
        assert!(!middleware.needs_protection(&excluded_req));

        let protected_req = HttpRequest::new(HttpMethod::Post, "/api/private/action");
        assert!(middleware.needs_protection(&protected_req));
    }

    #[test]
    fn test_token_generation() {
        let middleware = CsrfMiddleware::new(CsrfConfig::default());
        let token = middleware.generate_token().unwrap();
        assert!(!token.value.is_empty());
    }

    #[test]
    fn test_add_token_cookie() {
        let middleware = CsrfMiddleware::new(CsrfConfig::default());
        let token = middleware.generate_token().unwrap();
        let response = HttpResponse::ok();

        let response_with_cookie = middleware.add_token_cookie(response, &token).unwrap();
        assert!(response_with_cookie.headers.contains_key("Set-Cookie"));
    }
}

