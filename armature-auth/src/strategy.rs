// Authentication strategies

use crate::{AuthError, AuthUser, Result};
use armature_jwt::JwtManager;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Authentication strategy trait
#[async_trait]
pub trait AuthStrategy<T: AuthUser>: Send + Sync {
    /// Authenticate and return user
    async fn authenticate(&self, credentials: &dyn std::any::Any) -> Result<T>;
}

/// Local authentication strategy (username/password)
pub struct LocalStrategy<T: AuthUser> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: AuthUser> LocalStrategy<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: AuthUser> Default for LocalStrategy<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Local credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalCredentials {
    pub username: String,
    pub password: String,
}

/// JWT authentication strategy
pub struct JwtStrategy<T: AuthUser> {
    _jwt_manager: JwtManager,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: AuthUser> JwtStrategy<T> {
    pub fn new(jwt_manager: JwtManager) -> Self {
        Self {
            _jwt_manager: jwt_manager,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Extract token from Authorization header
    pub fn extract_token<'a>(&self, header: &'a str) -> Result<&'a str> {
        header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AuthError::InvalidToken("Invalid Bearer token format".to_string()))
    }
}

/// JWT credentials
#[derive(Debug, Clone)]
pub struct JwtCredentials {
    pub token: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_credentials() {
        let creds = LocalCredentials {
            username: "user@example.com".to_string(),
            password: "password123".to_string(),
        };

        assert_eq!(creds.username, "user@example.com");
        assert_eq!(creds.password, "password123");
    }

    #[test]
    fn test_jwt_token_extraction() {
        use crate::UserContext;
        use armature_jwt::JwtConfig;

        let config = JwtConfig::new("test-secret".to_string());
        let jwt_manager = JwtManager::new(config).unwrap();
        let strategy = JwtStrategy::<UserContext>::new(jwt_manager);

        let valid_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...";
        let token = strategy.extract_token(valid_header);
        assert!(token.is_ok());

        let invalid_header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...";
        let token = strategy.extract_token(invalid_header);
        assert!(token.is_err());
    }
}
