use crate::error::{CsrfError, Result};

/// CSRF protection configuration
#[derive(Debug, Clone)]
pub struct CsrfConfig {
    /// Secret key for token signing (must be at least 32 bytes)
    pub secret: Vec<u8>,

    /// Token time-to-live in seconds
    pub token_ttl: i64,

    /// Cookie name for CSRF token
    pub cookie_name: String,

    /// Header name for CSRF token
    pub header_name: String,

    /// Form field name for CSRF token
    pub field_name: String,

    /// Cookie domain
    pub cookie_domain: Option<String>,

    /// Cookie path
    pub cookie_path: String,

    /// Cookie secure flag (HTTPS only)
    pub cookie_secure: bool,

    /// Cookie HttpOnly flag
    pub cookie_http_only: bool,

    /// Cookie SameSite policy
    pub cookie_same_site: SameSite,

    /// Safe HTTP methods (not checked for CSRF)
    pub safe_methods: Vec<String>,

    /// Paths to exclude from CSRF protection
    pub exclude_paths: Vec<String>,
}

/// Cookie SameSite attribute
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl SameSite {
    pub fn as_str(&self) -> &'static str {
        match self {
            SameSite::Strict => "Strict",
            SameSite::Lax => "Lax",
            SameSite::None => "None",
        }
    }
}

impl CsrfConfig {
    /// Create a new CSRF configuration
    pub fn new(secret: Vec<u8>) -> Result<Self> {
        if secret.len() < 32 {
            return Err(CsrfError::Internal(
                "Secret key must be at least 32 bytes".to_string(),
            ));
        }

        Ok(Self {
            secret,
            token_ttl: 3600, // 1 hour
            cookie_name: "csrf_token".to_string(),
            header_name: "X-CSRF-Token".to_string(),
            field_name: "csrf_token".to_string(),
            cookie_domain: None,
            cookie_path: "/".to_string(),
            cookie_secure: true,
            cookie_http_only: true,
            cookie_same_site: SameSite::Strict,
            safe_methods: vec![
                "GET".to_string(),
                "HEAD".to_string(),
                "OPTIONS".to_string(),
            ],
            exclude_paths: Vec::new(),
        })
    }

    /// Generate a secret key
    pub fn generate_secret() -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..32).map(|_| rng.r#gen()).collect()
    }

    /// Set token TTL
    pub fn with_token_ttl(mut self, ttl_seconds: i64) -> Self {
        self.token_ttl = ttl_seconds;
        self
    }

    /// Set cookie name
    pub fn with_cookie_name(mut self, name: impl Into<String>) -> Self {
        self.cookie_name = name.into();
        self
    }

    /// Set header name
    pub fn with_header_name(mut self, name: impl Into<String>) -> Self {
        self.header_name = name.into();
        self
    }

    /// Set field name
    pub fn with_field_name(mut self, name: impl Into<String>) -> Self {
        self.field_name = name.into();
        self
    }

    /// Set cookie domain
    pub fn with_cookie_domain(mut self, domain: impl Into<String>) -> Self {
        self.cookie_domain = Some(domain.into());
        self
    }

    /// Set cookie path
    pub fn with_cookie_path(mut self, path: impl Into<String>) -> Self {
        self.cookie_path = path.into();
        self
    }

    /// Set cookie secure flag
    pub fn with_cookie_secure(mut self, secure: bool) -> Self {
        self.cookie_secure = secure;
        self
    }

    /// Set cookie HttpOnly flag
    pub fn with_cookie_http_only(mut self, http_only: bool) -> Self {
        self.cookie_http_only = http_only;
        self
    }

    /// Set cookie SameSite policy
    pub fn with_cookie_same_site(mut self, same_site: SameSite) -> Self {
        self.cookie_same_site = same_site;
        self
    }

    /// Add safe methods
    pub fn with_safe_methods(mut self, methods: Vec<String>) -> Self {
        self.safe_methods = methods;
        self
    }

    /// Add excluded paths
    pub fn with_exclude_paths(mut self, paths: Vec<String>) -> Self {
        self.exclude_paths = paths;
        self
    }
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self::new(Self::generate_secret()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let secret = CsrfConfig::generate_secret();
        assert_eq!(secret.len(), 32);

        let config = CsrfConfig::new(secret).unwrap();
        assert_eq!(config.token_ttl, 3600);
        assert_eq!(config.cookie_name, "csrf_token");
    }

    #[test]
    fn test_config_builder() {
        let config = CsrfConfig::default()
            .with_token_ttl(7200)
            .with_cookie_name("_csrf")
            .with_cookie_secure(false);

        assert_eq!(config.token_ttl, 7200);
        assert_eq!(config.cookie_name, "_csrf");
        assert!(!config.cookie_secure);
    }

    #[test]
    fn test_invalid_secret_length() {
        let short_secret = vec![1, 2, 3];
        assert!(CsrfConfig::new(short_secret).is_err());
    }

    #[test]
    fn test_same_site_enum() {
        assert_eq!(SameSite::Strict.as_str(), "Strict");
        assert_eq!(SameSite::Lax.as_str(), "Lax");
        assert_eq!(SameSite::None.as_str(), "None");
    }
}

