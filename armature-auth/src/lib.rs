// Authentication and authorization for Armature

pub mod error;
pub mod guard;
pub mod oauth2;
pub mod password;
pub mod providers;
pub mod saml;
pub mod strategy;
pub mod user;

pub use error::{AuthError, Result};
pub use guard::{AuthGuard, Guard, RoleGuard};
pub use oauth2::{OAuth2Provider, OAuth2Token, OAuth2UserInfo};
pub use password::{PasswordHasher, PasswordVerifier};
pub use saml::{
    ContactInfo, IdpMetadata, SamlAssertion, SamlAuthRequest, SamlConfig, SamlProvider,
    SamlServiceProvider,
};
pub use strategy::{AuthStrategy, JwtStrategy, LocalStrategy};
pub use user::{AuthUser, UserContext};

// Re-export providers
pub use providers::{
    Auth0Provider, AwsCognitoProvider, GoogleProvider, MicrosoftEntraProvider, OktaProvider,
};

use armature_core::Provider;
use armature_jwt::JwtManager;
use std::sync::Arc;

/// Authentication service
#[derive(Clone)]
pub struct AuthService {
    jwt_manager: Option<Arc<JwtManager>>,
    password_hasher: PasswordHasher,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new() -> Self {
        Self {
            jwt_manager: None,
            password_hasher: PasswordHasher::default(),
        }
    }

    /// Create with JWT manager
    pub fn with_jwt(jwt_manager: JwtManager) -> Self {
        Self {
            jwt_manager: Some(Arc::new(jwt_manager)),
            password_hasher: PasswordHasher::default(),
        }
    }

    /// Set password hasher
    pub fn with_password_hasher(mut self, hasher: PasswordHasher) -> Self {
        self.password_hasher = hasher;
        self
    }

    /// Hash a password
    pub fn hash_password(&self, password: &str) -> Result<String> {
        self.password_hasher.hash(password)
    }

    /// Verify a password
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        self.password_hasher.verify(password, hash)
    }

    /// Get JWT manager
    pub fn jwt_manager(&self) -> Option<&JwtManager> {
        self.jwt_manager.as_deref()
    }

    /// Validate authentication
    pub fn validate<T: AuthUser>(&self, user: &T) -> Result<()> {
        if !user.is_active() {
            return Err(AuthError::InactiveUser);
        }

        Ok(())
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for AuthService {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let service = AuthService::new();

        let password = "test-password-123";
        let hash = service.hash_password(password).unwrap();

        assert!(service.verify_password(password, &hash).unwrap());
        assert!(!service.verify_password("wrong-password", &hash).unwrap());
    }
}
