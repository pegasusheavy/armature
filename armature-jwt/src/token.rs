// Token types and utilities

use serde::{Deserialize, Serialize};

/// A JWT token
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Token {
    /// The token string
    pub token: String,

    /// Token type (usually "Bearer")
    pub token_type: String,

    /// Expiration time in seconds
    pub expires_in: i64,
}

impl Token {
    /// Create a new token
    pub fn new(token: String, expires_in: i64) -> Self {
        Self {
            token,
            token_type: "Bearer".to_string(),
            expires_in,
        }
    }

    /// Create with custom token type
    pub fn with_type(token: String, token_type: String, expires_in: i64) -> Self {
        Self {
            token,
            token_type,
            expires_in,
        }
    }
}

/// A pair of access and refresh tokens
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenPair {
    /// Access token (short-lived)
    pub access_token: String,

    /// Refresh token (long-lived)
    pub refresh_token: String,

    /// Token type (usually "Bearer")
    pub token_type: String,

    /// Access token expiration in seconds
    pub expires_in: i64,

    /// Refresh token expiration in seconds
    pub refresh_expires_in: i64,
}

impl TokenPair {
    /// Create a new token pair
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: i64,
        refresh_expires_in: i64,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            refresh_expires_in,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Token::new("abc123".to_string(), 3600);
        assert_eq!(token.token, "abc123");
        assert_eq!(token.token_type, "Bearer");
        assert_eq!(token.expires_in, 3600);
    }

    #[test]
    fn test_token_pair() {
        let pair = TokenPair::new(
            "access123".to_string(),
            "refresh456".to_string(),
            3600,
            604800,
        );

        assert_eq!(pair.access_token, "access123");
        assert_eq!(pair.refresh_token, "refresh456");
        assert_eq!(pair.expires_in, 3600);
        assert_eq!(pair.refresh_expires_in, 604800);
    }
}
