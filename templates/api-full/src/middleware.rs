//! Application middleware

use armature::prelude::*;
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

/// Implement the Middleware trait for AuthMiddleware
#[async_trait::async_trait]
impl armature::Middleware for AuthMiddleware {
    async fn handle(
        &self,
        req: HttpRequest,
        next: Box<
            dyn FnOnce(
                    HttpRequest,
                )
                    -> std::pin::Pin<
                        Box<
                            dyn std::future::Future<
                                    Output = Result<HttpResponse, Error>,
                                > + Send,
                        >,
                    > + Send,
        >,
    ) -> Result<HttpResponse, Error> {
        // Check if path is excluded from authentication
        if self.is_excluded(&req.path) {
            return next(req).await;
        }

        // Extract bearer token from Authorization header
        let token = req.headers
            .get("authorization")
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|s| s.to_string());

        match token {
            Some(t) => {
                // Verify token
                match self.verify_token(&t) {
                    Ok(()) => next(req).await,
                    Err(e) => Err(Error::Unauthorized(e)),
                }
            }
            None => Err(Error::Unauthorized("Missing authorization header".to_string())),
        }
    }
}

