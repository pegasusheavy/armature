use crate::error::{CsrfError, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// CSRF token with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfToken {
    /// Random token value
    pub value: String,

    /// Token creation timestamp
    pub created_at: DateTime<Utc>,

    /// Token expiration timestamp
    pub expires_at: DateTime<Utc>,

    /// Session identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl CsrfToken {
    /// Generate a new CSRF token
    pub fn generate(ttl_seconds: i64) -> Self {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.r#gen();
        let value = URL_SAFE_NO_PAD.encode(random_bytes);

        let created_at = Utc::now();
        let expires_at = created_at + Duration::seconds(ttl_seconds);

        Self {
            value,
            created_at,
            expires_at,
            session_id: None,
        }
    }

    /// Generate a token with session binding
    pub fn generate_with_session(ttl_seconds: i64, session_id: String) -> Self {
        let mut token = Self::generate(ttl_seconds);
        token.session_id = Some(session_id);
        token
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Validate token
    pub fn validate(&self) -> Result<()> {
        if self.is_expired() {
            return Err(CsrfError::TokenExpired);
        }
        Ok(())
    }

    /// Encode token to signed string
    pub fn encode(&self, secret: &[u8]) -> Result<String> {
        let json = serde_json::to_string(self)?;
        let signature = Self::sign(&json, secret);
        let encoded = format!("{}.{}", json, signature);
        Ok(URL_SAFE_NO_PAD.encode(encoded))
    }

    /// Decode and verify signed token
    pub fn decode(encoded: &str, secret: &[u8]) -> Result<Self> {
        let decoded = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|e| CsrfError::ValidationFailed(e.to_string()))?;

        let decoded_str = String::from_utf8(decoded)
            .map_err(|e| CsrfError::ValidationFailed(e.to_string()))?;

        let parts: Vec<&str> = decoded_str.split('.').collect();
        if parts.len() != 2 {
            return Err(CsrfError::InvalidToken);
        }

        let json = parts[0];
        let signature = parts[1];

        // Verify signature
        let expected_signature = Self::sign(json, secret);
        if signature != expected_signature {
            return Err(CsrfError::InvalidToken);
        }

        // Deserialize token
        let token: CsrfToken = serde_json::from_str(json)?;
        token.validate()?;

        Ok(token)
    }

    /// Sign data with HMAC-SHA256
    fn sign(data: &str, secret: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(secret)
            .expect("HMAC can take key of any size");
        mac.update(data.as_bytes());
        let result = mac.finalize();
        URL_SAFE_NO_PAD.encode(result.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let token = CsrfToken::generate(3600);
        assert!(!token.value.is_empty());
        assert!(!token.is_expired());
    }

    #[test]
    fn test_token_with_session() {
        let token = CsrfToken::generate_with_session(3600, "session123".to_string());
        assert_eq!(token.session_id, Some("session123".to_string()));
    }

    #[test]
    fn test_token_expiration() {
        let mut token = CsrfToken::generate(0);
        token.expires_at = Utc::now() - Duration::seconds(1);
        assert!(token.is_expired());
        assert!(token.validate().is_err());
    }

    #[test]
    fn test_token_encode_decode() {
        let secret = b"test_secret_key_32_bytes_long!!!";
        let token = CsrfToken::generate(3600);

        let encoded = token.encode(secret).unwrap();
        let decoded = CsrfToken::decode(&encoded, secret).unwrap();

        assert_eq!(token.value, decoded.value);
    }

    #[test]
    fn test_invalid_signature() {
        let secret = b"test_secret_key_32_bytes_long!!!";
        let token = CsrfToken::generate(3600);
        let encoded = token.encode(secret).unwrap();

        let wrong_secret = b"wrong_secret_key_32_bytes_long!!";
        assert!(CsrfToken::decode(&encoded, wrong_secret).is_err());
    }
}

