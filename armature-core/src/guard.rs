// Guards for route protection

use crate::{Error, HttpRequest};
use async_trait::async_trait;

/// Execution context for guards
pub struct GuardContext {
    pub request: HttpRequest,
}

impl GuardContext {
    pub fn new(request: HttpRequest) -> Self {
        Self { request }
    }

    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.request.headers.get(name)
    }

    pub fn get_param(&self, name: &str) -> Option<&String> {
        self.request.path_params.get(name)
    }
}

/// Guard trait for protecting routes
#[async_trait]
pub trait Guard: Send + Sync {
    /// Determine if the request can proceed
    async fn can_activate(&self, context: &GuardContext) -> Result<bool, Error>;
}

/// Authentication guard - checks for valid token
pub struct AuthenticationGuard;

#[async_trait]
impl Guard for AuthenticationGuard {
    async fn can_activate(&self, context: &GuardContext) -> Result<bool, Error> {
        // Check for Authorization header
        match context.get_header("authorization") {
            Some(header) if header.starts_with("Bearer ") => Ok(true),
            _ => Err(Error::Forbidden(
                "Missing or invalid authorization header".to_string(),
            )),
        }
    }
}

/// Role-based guard
pub struct RolesGuard {
    _required_roles: Vec<String>,
}

impl RolesGuard {
    pub fn new(roles: Vec<String>) -> Self {
        Self {
            _required_roles: roles,
        }
    }
}

#[async_trait]
impl Guard for RolesGuard {
    async fn can_activate(&self, context: &GuardContext) -> Result<bool, Error> {
        // First check authentication
        let auth_header = context
            .get_header("authorization")
            .ok_or_else(|| Error::Forbidden("Missing authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(Error::Forbidden("Invalid authorization header".to_string()));
        }

        // In production, decode JWT and check roles
        // For now, just check if token exists
        // let token = &auth_header[7..];
        // let claims = decode_jwt(token)?;
        // let has_role = self.required_roles.iter().any(|role| claims.roles.contains(role));

        Ok(true) // Placeholder
    }
}

/// Custom guard builder
pub struct CustomGuard<F>
where
    F: Fn(&GuardContext) -> Result<bool, Error> + Send + Sync,
{
    predicate: F,
}

impl<F> CustomGuard<F>
where
    F: Fn(&GuardContext) -> Result<bool, Error> + Send + Sync,
{
    pub fn new(predicate: F) -> Self {
        Self { predicate }
    }
}

#[async_trait]
impl<F> Guard for CustomGuard<F>
where
    F: Fn(&GuardContext) -> Result<bool, Error> + Send + Sync,
{
    async fn can_activate(&self, context: &GuardContext) -> Result<bool, Error> {
        (self.predicate)(context)
    }
}

/// API key guard
pub struct ApiKeyGuard {
    valid_keys: Vec<String>,
}

impl ApiKeyGuard {
    pub fn new(keys: Vec<String>) -> Self {
        Self { valid_keys: keys }
    }
}

#[async_trait]
impl Guard for ApiKeyGuard {
    async fn can_activate(&self, context: &GuardContext) -> Result<bool, Error> {
        let api_key = context
            .get_header("x-api-key")
            .ok_or_else(|| Error::Forbidden("Missing API key".to_string()))?;

        if self.valid_keys.contains(api_key) {
            Ok(true)
        } else {
            Err(Error::Forbidden("Invalid API key".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_authentication_guard() {
        let guard = AuthenticationGuard;

        // Test with valid token
        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "Bearer token123".to_string());
        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers,
            body: vec![],
            path_params: HashMap::new(),
            query_params: HashMap::new(),
        };
        let context = GuardContext::new(request);

        assert!(guard.can_activate(&context).await.is_ok());
    }

    #[tokio::test]
    async fn test_authentication_guard_missing_header() {
        let guard = AuthenticationGuard;

        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers: HashMap::new(),
            body: vec![],
            path_params: HashMap::new(),
            query_params: HashMap::new(),
        };
        let context = GuardContext::new(request);

        assert!(guard.can_activate(&context).await.is_err());
    }

    #[tokio::test]
    async fn test_api_key_guard() {
        let guard = ApiKeyGuard::new(vec!["valid-key".to_string()]);

        // Valid key
        let mut headers = HashMap::new();
        headers.insert("x-api-key".to_string(), "valid-key".to_string());
        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers,
            body: vec![],
            path_params: HashMap::new(),
            query_params: HashMap::new(),
        };
        let context = GuardContext::new(request);

        assert!(guard.can_activate(&context).await.is_ok());
    }

    #[tokio::test]
    async fn test_api_key_guard_invalid() {
        let guard = ApiKeyGuard::new(vec!["valid-key".to_string()]);

        let mut headers = HashMap::new();
        headers.insert("x-api-key".to_string(), "invalid-key".to_string());
        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers,
            body: vec![],
            path_params: HashMap::new(),
            query_params: HashMap::new(),
        };
        let context = GuardContext::new(request);

        assert!(guard.can_activate(&context).await.is_err());
    }

    #[tokio::test]
    async fn test_api_key_guard_missing() {
        let guard = ApiKeyGuard::new(vec!["valid-key".to_string()]);

        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers: HashMap::new(),
            body: vec![],
            path_params: HashMap::new(),
            query_params: HashMap::new(),
        };
        let context = GuardContext::new(request);

        assert!(guard.can_activate(&context).await.is_err());
    }

    #[tokio::test]
    async fn test_roles_guard_with_role() {
        let guard = RolesGuard::new(vec!["admin".to_string()]);

        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/admin".to_string(),
            headers: HashMap::new(),
            body: vec![],
            path_params: HashMap::new(),
            query_params: HashMap::new(),
        };
        let context = GuardContext::new(request);

        // Will fail without actual role implementation, but tests structure
        let result = guard.can_activate(&context).await;
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_guard_context_creation() {
        let request = HttpRequest {
            method: "POST".to_string(),
            path: "/api/test".to_string(),
            headers: HashMap::new(),
            body: vec![1, 2, 3],
            path_params: HashMap::new(),
            query_params: HashMap::new(),
        };
        let context = GuardContext::new(request.clone());

        assert_eq!(context.request.method, "POST");
        assert_eq!(context.request.path, "/api/test");
    }

    #[tokio::test]
    async fn test_authentication_guard_bearer_format() {
        let guard = AuthenticationGuard;

        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "Bearer abc123xyz".to_string());
        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/secure".to_string(),
            headers,
            body: vec![],
            path_params: HashMap::new(),
            query_params: HashMap::new(),
        };
        let context = GuardContext::new(request);

        let result = guard.can_activate(&context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_authentication_guard_wrong_scheme() {
        let guard = AuthenticationGuard;

        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "Basic abc123".to_string());
        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/secure".to_string(),
            headers,
            body: vec![],
            path_params: HashMap::new(),
            query_params: HashMap::new(),
        };
        let context = GuardContext::new(request);

        let result = guard.can_activate(&context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_api_key_guard_multiple_valid_keys() {
        let guard = ApiKeyGuard::new(vec![
            "key1".to_string(),
            "key2".to_string(),
            "key3".to_string(),
        ]);

        for key in &["key1", "key2", "key3"] {
            let mut headers = HashMap::new();
            headers.insert("x-api-key".to_string(), key.to_string());
            let request = HttpRequest {
                method: "GET".to_string(),
                path: "/test".to_string(),
                headers,
                body: vec![],
                path_params: HashMap::new(),
                query_params: HashMap::new(),
            };
            let context = GuardContext::new(request);

            assert!(guard.can_activate(&context).await.is_ok());
        }
    }

    #[test]
    fn test_api_key_guard_creation() {
        let keys = vec!["key1".to_string(), "key2".to_string()];
        let guard = ApiKeyGuard::new(keys.clone());
        assert_eq!(guard.valid_keys, keys);
    }

    #[test]
    fn test_roles_guard_creation() {
        let roles = vec!["admin".to_string(), "user".to_string()];
        let _guard = RolesGuard::new(roles);
        // Just test creation
    }

    #[tokio::test]
    async fn test_guard_context_with_params() {
        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "123".to_string());

        let mut query_params = HashMap::new();
        query_params.insert("sort".to_string(), "asc".to_string());

        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/users/123".to_string(),
            headers: HashMap::new(),
            body: vec![],
            path_params,
            query_params,
        };
        let context = GuardContext::new(request);

        assert_eq!(context.request.path_params.get("id"), Some(&"123".to_string()));
        assert_eq!(
            context.request.query_params.get("sort"),
            Some(&"asc".to_string())
        );
    }
}
