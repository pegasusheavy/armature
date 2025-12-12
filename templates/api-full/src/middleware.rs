//! Application middleware

use crate::services::AuthService;
use std::sync::Arc;

/// Authentication middleware for protected routes
pub struct AuthMiddleware {
    auth_service: Arc<AuthService>,
    excluded_paths: Vec<String>,
}

impl AuthMiddleware {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self {
            auth_service,
            excluded_paths: vec![
                "/health".to_string(),
                "/health/live".to_string(),
                "/health/ready".to_string(),
                "/api/auth/login".to_string(),
                "/api/auth/register".to_string(),
                "/docs".to_string(),
            ],
        }
    }

    pub fn is_excluded(&self, path: &str) -> bool {
        self.excluded_paths.iter().any(|p| path.starts_with(p))
    }

    pub fn verify_token(&self, token: &str) -> Result<(), String> {
        self.auth_service.verify_token(token).map(|_| ())
    }
}

