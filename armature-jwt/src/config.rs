// JWT configuration

use crate::Result;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Secret key for HS256/HS384/HS512 algorithms
    pub secret: Option<String>,

    /// Public key for RS256/RS384/RS512/ES256/ES384 algorithms (PEM format)
    pub public_key: Option<String>,

    /// Private key for RS256/RS384/RS512/ES256/ES384 algorithms (PEM format)
    pub private_key: Option<String>,

    /// Algorithm to use (default: HS256)
    pub algorithm: Algorithm,

    /// Token expiration time (default: 1 hour)
    pub expires_in: Duration,

    /// Refresh token expiration (default: 7 days)
    pub refresh_expires_in: Duration,

    /// Issuer (iss claim)
    pub issuer: Option<String>,

    /// Audience (aud claim)
    pub audience: Option<Vec<String>>,

    /// Validate expiration
    pub validate_exp: bool,

    /// Validate not before
    pub validate_nbf: bool,

    /// Leeway for time validation (seconds)
    pub leeway: u64,
}

impl JwtConfig {
    /// Create a new configuration with a secret
    pub fn new(secret: String) -> Self {
        Self {
            secret: Some(secret),
            public_key: None,
            private_key: None,
            algorithm: Algorithm::HS256,
            expires_in: Duration::from_secs(3600), // 1 hour
            refresh_expires_in: Duration::from_secs(604800), // 7 days
            issuer: None,
            audience: None,
            validate_exp: true,
            validate_nbf: false,
            leeway: 0,
        }
    }

    /// Create configuration with RSA keys
    pub fn with_rsa(private_key: String, public_key: String) -> Self {
        Self {
            secret: None,
            public_key: Some(public_key),
            private_key: Some(private_key),
            algorithm: Algorithm::RS256,
            expires_in: Duration::from_secs(3600),
            refresh_expires_in: Duration::from_secs(604800),
            issuer: None,
            audience: None,
            validate_exp: true,
            validate_nbf: false,
            leeway: 0,
        }
    }

    /// Set the algorithm
    pub fn with_algorithm(mut self, algorithm: Algorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// Set expiration time
    pub fn with_expiration(mut self, duration: Duration) -> Self {
        self.expires_in = duration;
        self
    }

    /// Set refresh token expiration
    pub fn with_refresh_expiration(mut self, duration: Duration) -> Self {
        self.refresh_expires_in = duration;
        self
    }

    /// Set issuer
    pub fn with_issuer(mut self, issuer: String) -> Self {
        self.issuer = Some(issuer);
        self
    }

    /// Set audience
    pub fn with_audience(mut self, audience: Vec<String>) -> Self {
        self.audience = Some(audience);
        self
    }

    /// Set leeway
    pub fn with_leeway(mut self, leeway: u64) -> Self {
        self.leeway = leeway;
        self
    }

    /// Get encoding key
    pub fn encoding_key(&self) -> Result<EncodingKey> {
        match self.algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                let secret = self.secret.as_ref().ok_or_else(|| {
                    crate::JwtError::ConfigError("Secret required for HMAC algorithms".to_string())
                })?;
                Ok(EncodingKey::from_secret(secret.as_bytes()))
            }
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
                let private_key = self.private_key.as_ref().ok_or_else(|| {
                    crate::JwtError::ConfigError(
                        "Private key required for RSA algorithms".to_string(),
                    )
                })?;
                EncodingKey::from_rsa_pem(private_key.as_bytes())
                    .map_err(|e| crate::JwtError::ConfigError(e.to_string()))
            }
            Algorithm::ES256 | Algorithm::ES384 => {
                let private_key = self.private_key.as_ref().ok_or_else(|| {
                    crate::JwtError::ConfigError(
                        "Private key required for ECDSA algorithms".to_string(),
                    )
                })?;
                EncodingKey::from_ec_pem(private_key.as_bytes())
                    .map_err(|e| crate::JwtError::ConfigError(e.to_string()))
            }
            _ => Err(crate::JwtError::ConfigError(
                "Unsupported algorithm".to_string(),
            )),
        }
    }

    /// Get decoding key
    pub fn decoding_key(&self) -> Result<DecodingKey> {
        match self.algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                let secret = self.secret.as_ref().ok_or_else(|| {
                    crate::JwtError::ConfigError("Secret required for HMAC algorithms".to_string())
                })?;
                Ok(DecodingKey::from_secret(secret.as_bytes()))
            }
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
                let public_key = self.public_key.as_ref().ok_or_else(|| {
                    crate::JwtError::ConfigError(
                        "Public key required for RSA algorithms".to_string(),
                    )
                })?;
                DecodingKey::from_rsa_pem(public_key.as_bytes())
                    .map_err(|e| crate::JwtError::ConfigError(e.to_string()))
            }
            Algorithm::ES256 | Algorithm::ES384 => {
                let public_key = self.public_key.as_ref().ok_or_else(|| {
                    crate::JwtError::ConfigError(
                        "Public key required for ECDSA algorithms".to_string(),
                    )
                })?;
                DecodingKey::from_ec_pem(public_key.as_bytes())
                    .map_err(|e| crate::JwtError::ConfigError(e.to_string()))
            }
            _ => Err(crate::JwtError::ConfigError(
                "Unsupported algorithm".to_string(),
            )),
        }
    }

    /// Get validation config
    pub fn validation(&self) -> Validation {
        let mut validation = Validation::new(self.algorithm);
        validation.validate_exp = self.validate_exp;
        validation.validate_nbf = self.validate_nbf;
        validation.leeway = self.leeway;

        if let Some(ref iss) = self.issuer {
            validation.set_issuer(&[iss]);
        }

        if let Some(ref aud) = self.audience {
            validation.set_audience(aud);
        }

        validation
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self::new("change-me-in-production".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = JwtConfig::default();
        assert!(config.secret.is_some());
        assert_eq!(config.algorithm, Algorithm::HS256);
        assert!(config.validate_exp);
    }

    #[test]
    fn test_config_builder() {
        let config = JwtConfig::new("secret".to_string())
            .with_algorithm(Algorithm::HS512)
            .with_expiration(Duration::from_secs(7200))
            .with_issuer("my-app".to_string())
            .with_leeway(60);

        assert_eq!(config.algorithm, Algorithm::HS512);
        assert_eq!(config.expires_in, Duration::from_secs(7200));
        assert_eq!(config.issuer, Some("my-app".to_string()));
        assert_eq!(config.leeway, 60);
    }

    #[test]
    fn test_encoding_key() {
        let config = JwtConfig::new("test-secret".to_string());
        let key = config.encoding_key();
        assert!(key.is_ok());
    }
}
