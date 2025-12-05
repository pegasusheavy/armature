// JWT authentication and authorization for Armature

pub mod claims;
pub mod config;
pub mod error;
pub mod service;
pub mod token;

pub use claims::{Claims, StandardClaims};
pub use config::JwtConfig;
pub use error::{JwtError, Result};
pub use service::JwtService;
pub use token::{Token, TokenPair};

use armature_core::Provider;

// Re-export jsonwebtoken types
pub use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};

/// JWT service for token management
#[derive(Clone)]
pub struct JwtManager {
    service: JwtService,
}

impl JwtManager {
    /// Create a new JWT manager
    pub fn new(config: JwtConfig) -> Result<Self> {
        let service = JwtService::new(config)?;
        Ok(Self { service })
    }

    /// Sign a token with claims
    pub fn sign<T: serde::Serialize>(&self, claims: &T) -> Result<String> {
        self.service.sign(claims)
    }

    /// Verify and decode a token
    pub fn verify<T: serde::de::DeserializeOwned>(&self, token: &str) -> Result<T> {
        self.service.verify(token)
    }

    /// Generate a token pair (access + refresh)
    pub fn generate_token_pair<T: serde::Serialize + Clone>(
        &self,
        claims: &T,
    ) -> Result<TokenPair> {
        self.service.generate_token_pair(claims)
    }

    /// Refresh an access token using a refresh token
    pub fn refresh_token<T: serde::de::DeserializeOwned + serde::Serialize + Clone>(
        &self,
        refresh_token: &str,
    ) -> Result<TokenPair> {
        self.service.refresh_token::<T>(refresh_token)
    }

    /// Decode a token without verification (useful for inspecting expired tokens)
    pub fn decode_unverified<T: serde::de::DeserializeOwned>(&self, token: &str) -> Result<T> {
        self.service.decode_unverified(token)
    }

    /// Get the configuration
    pub fn config(&self) -> &JwtConfig {
        self.service.config()
    }
}

impl Provider for JwtManager {}

impl Default for JwtManager {
    fn default() -> Self {
        Self::new(JwtConfig::default()).expect("Failed to create default JwtManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestClaims {
        sub: String,
        name: String,
    }

    #[test]
    fn test_sign_and_verify() {
        let config = JwtConfig::new("test-secret".to_string());
        let manager = JwtManager::new(config).unwrap();

        let claims = TestClaims {
            sub: "123".to_string(),
            name: "John Doe".to_string(),
        };

        let token = manager.sign(&claims).unwrap();
        let decoded: TestClaims = manager.verify(&token).unwrap();

        assert_eq!(decoded, claims);
    }

    #[test]
    fn test_invalid_token() {
        let config = JwtConfig::new("test-secret".to_string());
        let manager = JwtManager::new(config).unwrap();

        let result: Result<TestClaims> = manager.verify("invalid.token.here");
        assert!(result.is_err());
    }
}
