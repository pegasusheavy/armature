// JWT claims structures

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Standard JWT claims (RFC 7519)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StandardClaims {
    /// Subject (user ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,

    /// Issuer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,

    /// Audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<Vec<String>>,

    /// Expiration time (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,

    /// Not before (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,

    /// Issued at (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,

    /// JWT ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
}

impl StandardClaims {
    /// Create new standard claims
    pub fn new() -> Self {
        Self {
            sub: None,
            iss: None,
            aud: None,
            exp: None,
            nbf: None,
            iat: Some(Utc::now().timestamp()),
            jti: None,
        }
    }

    /// Set subject
    pub fn with_subject(mut self, sub: String) -> Self {
        self.sub = Some(sub);
        self
    }

    /// Set issuer
    pub fn with_issuer(mut self, iss: String) -> Self {
        self.iss = Some(iss);
        self
    }

    /// Set audience
    pub fn with_audience(mut self, aud: Vec<String>) -> Self {
        self.aud = Some(aud);
        self
    }

    /// Set expiration (from now + duration in seconds)
    pub fn with_expiration(mut self, seconds: i64) -> Self {
        self.exp = Some(Utc::now().timestamp() + seconds);
        self
    }

    /// Set not before
    pub fn with_not_before(mut self, nbf: i64) -> Self {
        self.nbf = Some(nbf);
        self
    }

    /// Set JWT ID
    pub fn with_jti(mut self, jti: String) -> Self {
        self.jti = Some(jti);
        self
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.exp {
            exp < Utc::now().timestamp()
        } else {
            false
        }
    }
}

impl Default for StandardClaims {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom claims builder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims<T> {
    #[serde(flatten)]
    pub standard: StandardClaims,

    #[serde(flatten)]
    pub custom: T,
}

impl<T> Claims<T> {
    /// Create new claims with custom data
    pub fn new(custom: T) -> Self {
        Self {
            standard: StandardClaims::new(),
            custom,
        }
    }

    /// Create with standard claims
    pub fn with_standard(standard: StandardClaims, custom: T) -> Self {
        Self { standard, custom }
    }

    /// Set subject
    pub fn with_subject(mut self, sub: String) -> Self {
        self.standard.sub = Some(sub);
        self
    }

    /// Set expiration
    pub fn with_expiration(mut self, seconds: i64) -> Self {
        self.standard = self.standard.with_expiration(seconds);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_claims() {
        let claims = StandardClaims::new()
            .with_subject("user123".to_string())
            .with_issuer("my-app".to_string())
            .with_expiration(3600);

        assert_eq!(claims.sub, Some("user123".to_string()));
        assert_eq!(claims.iss, Some("my-app".to_string()));
        assert!(claims.exp.is_some());
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_custom_claims() {
        #[derive(Debug, Serialize, Deserialize)]
        struct UserClaims {
            email: String,
            role: String,
        }

        let claims = Claims::new(UserClaims {
            email: "user@example.com".to_string(),
            role: "admin".to_string(),
        })
        .with_subject("123".to_string())
        .with_expiration(3600);

        assert_eq!(claims.standard.sub, Some("123".to_string()));
        assert_eq!(claims.custom.email, "user@example.com");
    }

    #[test]
    fn test_is_expired() {
        let expired_claims = StandardClaims {
            exp: Some(Utc::now().timestamp() - 1000),
            ..Default::default()
        };

        assert!(expired_claims.is_expired());

        let valid_claims = StandardClaims {
            exp: Some(Utc::now().timestamp() + 1000),
            ..Default::default()
        };

        assert!(!valid_claims.is_expired());
    }

    #[test]
    fn test_standard_claims_default() {
        let claims = StandardClaims::default();
        assert!(claims.sub.is_none());
        assert!(claims.iss.is_none());
        assert!(claims.aud.is_none());
        assert!(claims.exp.is_none());
        assert!(claims.nbf.is_none());
        assert!(claims.iat.is_some()); // iat is set by default to current timestamp
        assert!(claims.jti.is_none());
    }

    #[test]
    fn test_with_audience() {
        let claims =
            StandardClaims::new().with_audience(vec!["api1".to_string(), "api2".to_string()]);
        assert_eq!(
            claims.aud,
            Some(vec!["api1".to_string(), "api2".to_string()])
        );
    }

    #[test]
    fn test_with_not_before() {
        let nbf = Utc::now().timestamp();
        let claims = StandardClaims::new().with_not_before(nbf);
        assert_eq!(claims.nbf, Some(nbf));
    }

    #[test]
    fn test_with_jti() {
        let claims = StandardClaims::new().with_jti("unique-id-123".to_string());
        assert_eq!(claims.jti, Some("unique-id-123".to_string()));
    }

    #[test]
    fn test_expiration_in_future() {
        let claims = StandardClaims::new().with_expiration(3600);
        assert!(claims.exp.is_some());
        let exp = claims.exp.unwrap();
        let now = Utc::now().timestamp();
        assert!(exp > now);
        assert!(exp <= now + 3600);
    }

    #[test]
    fn test_is_expired_no_expiration() {
        let claims = StandardClaims::default();
        assert!(!claims.is_expired()); // No expiration means not expired
    }

    #[test]
    fn test_is_expired_exactly_now() {
        let claims = StandardClaims {
            exp: Some(Utc::now().timestamp()),
            ..Default::default()
        };
        // Could be expired or not depending on timing - just check it doesn't panic
        let _ = claims.is_expired();
    }

    #[test]
    fn test_claims_with_custom_data() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct CustomData {
            user_id: u64,
            permissions: Vec<String>,
        }

        let custom = CustomData {
            user_id: 42,
            permissions: vec!["read".to_string(), "write".to_string()],
        };

        let claims = Claims::new(custom.clone())
            .with_subject("user42".to_string())
            .with_expiration(7200);

        assert_eq!(claims.custom, custom);
        assert_eq!(claims.standard.sub, Some("user42".to_string()));
    }

    #[test]
    fn test_claims_builder_chaining() {
        let claims = StandardClaims::new()
            .with_subject("user".to_string())
            .with_issuer("issuer".to_string())
            .with_audience(vec!["aud1".to_string()])
            .with_expiration(3600)
            .with_jti("jti-123".to_string());

        assert!(claims.sub.is_some());
        assert!(claims.iss.is_some());
        assert!(claims.aud.is_some());
        assert!(claims.exp.is_some());
        assert!(claims.jti.is_some());
    }

    #[test]
    fn test_claims_serialization() {
        let claims = StandardClaims::new()
            .with_subject("test".to_string())
            .with_expiration(3600);

        let json = serde_json::to_string(&claims).unwrap();
        assert!(json.contains("\"sub\":\"test\""));
    }

    #[test]
    fn test_claims_deserialization() {
        let json = r#"{"sub":"test","iss":"issuer","exp":1234567890}"#;
        let claims: StandardClaims = serde_json::from_str(json).unwrap();

        assert_eq!(claims.sub, Some("test".to_string()));
        assert_eq!(claims.iss, Some("issuer".to_string()));
        assert_eq!(claims.exp, Some(1234567890));
    }

    #[test]
    fn test_multiple_audiences() {
        let audiences = vec!["api1".to_string(), "api2".to_string(), "api3".to_string()];
        let claims = StandardClaims::new().with_audience(audiences.clone());
        assert_eq!(claims.aud, Some(audiences));
    }

    #[test]
    fn test_empty_audience() {
        let claims = StandardClaims::new().with_audience(vec![]);
        assert_eq!(claims.aud, Some(vec![]));
    }

    #[test]
    fn test_custom_claims_serialization() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Data {
            count: u32,
        }

        let claims = Claims::new(Data { count: 42 });
        let json = serde_json::to_string(&claims).unwrap();
        assert!(json.contains("\"count\":42"));
    }
}
