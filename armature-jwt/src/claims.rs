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
}
