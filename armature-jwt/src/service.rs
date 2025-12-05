// JWT service implementation

use crate::{JwtConfig, JwtError, Result, StandardClaims, TokenPair};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use serde::{Serialize, de::DeserializeOwned};

/// JWT service for token operations
#[derive(Clone)]
pub struct JwtService {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtService {
    /// Create a new JWT service
    pub fn new(config: JwtConfig) -> Result<Self> {
        let encoding_key = config.encoding_key()?;
        let decoding_key = config.decoding_key()?;
        let validation = config.validation();

        Ok(Self {
            config,
            encoding_key,
            decoding_key,
            validation,
        })
    }

    /// Sign a token with claims
    pub fn sign<T: Serialize>(&self, claims: &T) -> Result<String> {
        let header = Header::new(self.config.algorithm);
        encode(&header, claims, &self.encoding_key).map_err(JwtError::from)
    }

    /// Verify and decode a token
    pub fn verify<T: DeserializeOwned>(&self, token: &str) -> Result<T> {
        let token_data: TokenData<T> = decode(token, &self.decoding_key, &self.validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
                jsonwebtoken::errors::ErrorKind::InvalidSignature => JwtError::InvalidSignature,
                _ => JwtError::EncodingError(e),
            })?;

        Ok(token_data.claims)
    }

    /// Decode without verification (useful for inspecting tokens)
    pub fn decode_unverified<T: DeserializeOwned>(&self, token: &str) -> Result<T> {
        let mut validation = Validation::new(self.config.algorithm);
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;

        let token_data: TokenData<T> =
            decode(token, &self.decoding_key, &validation).map_err(JwtError::from)?;

        Ok(token_data.claims)
    }

    /// Generate a token pair (access + refresh)
    pub fn generate_token_pair<T: Serialize + Clone>(&self, claims: &T) -> Result<TokenPair> {
        // Generate access token
        let access_token = self.sign(claims)?;

        // Generate refresh token (same claims, but longer expiration)
        let refresh_token = self.sign(claims)?;

        Ok(TokenPair::new(
            access_token,
            refresh_token,
            self.config.expires_in.as_secs() as i64,
            self.config.refresh_expires_in.as_secs() as i64,
        ))
    }

    /// Refresh an access token
    pub fn refresh_token<T: DeserializeOwned + Serialize + Clone>(
        &self,
        refresh_token: &str,
    ) -> Result<TokenPair> {
        // Verify the refresh token
        let claims: T = self.verify(refresh_token)?;

        // Generate new token pair
        self.generate_token_pair(&claims)
    }

    /// Sign with standard claims
    pub fn sign_standard(
        &self,
        sub: String,
        additional_claims: Option<serde_json::Value>,
    ) -> Result<String> {
        let mut claims = StandardClaims::new()
            .with_subject(sub)
            .with_expiration(self.config.expires_in.as_secs() as i64);

        if let Some(iss) = &self.config.issuer {
            claims = claims.with_issuer(iss.clone());
        }

        if let Some(aud) = &self.config.audience {
            claims = claims.with_audience(aud.clone());
        }

        if let Some(additional) = additional_claims {
            // Merge additional claims
            let mut combined = serde_json::to_value(&claims)
                .map_err(|e| JwtError::SerializationError(e.to_string()))?;

            if let (Some(obj), serde_json::Value::Object(add_obj)) =
                (combined.as_object_mut(), additional)
            {
                obj.extend(add_obj);
            }

            let token = self.sign(&combined)?;
            return Ok(token);
        }

        self.sign(&claims)
    }

    /// Get the configuration
    pub fn config(&self) -> &JwtConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestClaims {
        sub: String,
        name: String,
        exp: i64,
    }

    #[test]
    fn test_sign_and_verify() {
        let config = JwtConfig::new("test-secret".to_string());
        let service = JwtService::new(config).unwrap();

        let exp = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();

        let claims = TestClaims {
            sub: "123".to_string(),
            name: "Test User".to_string(),
            exp,
        };

        let token = service.sign(&claims).unwrap();
        let decoded: TestClaims = service.verify(&token).unwrap();

        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.name, claims.name);
    }

    #[test]
    fn test_invalid_signature() {
        let config1 = JwtConfig::new("secret1".to_string());
        let service1 = JwtService::new(config1).unwrap();

        let config2 = JwtConfig::new("secret2".to_string());
        let service2 = JwtService::new(config2).unwrap();

        let exp = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();

        let claims = TestClaims {
            sub: "123".to_string(),
            name: "Test".to_string(),
            exp,
        };

        let token = service1.sign(&claims).unwrap();
        let result: Result<TestClaims> = service2.verify(&token);

        assert!(matches!(result, Err(JwtError::InvalidSignature)));
    }

    #[test]
    fn test_token_pair_generation() {
        let config = JwtConfig::new("test-secret".to_string());
        let service = JwtService::new(config).unwrap();

        let exp = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();

        let claims = TestClaims {
            sub: "123".to_string(),
            name: "Test".to_string(),
            exp,
        };

        let pair = service.generate_token_pair(&claims).unwrap();

        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert_eq!(pair.token_type, "Bearer");
    }

    #[test]
    fn test_decode_unverified() {
        let config = JwtConfig::new("test-secret".to_string());
        let service = JwtService::new(config).unwrap();

        let exp = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();

        let claims = TestClaims {
            sub: "123".to_string(),
            name: "Test".to_string(),
            exp,
        };

        let token = service.sign(&claims).unwrap();
        let decoded: TestClaims = service.decode_unverified(&token).unwrap();

        assert_eq!(decoded, claims);
    }
}
