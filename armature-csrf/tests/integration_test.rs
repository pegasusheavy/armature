//! Integration tests for armature-csrf

use armature_csrf::*;
use armature_core::{HttpRequest};

#[test]
fn test_csrf_config_creation() {
    let secret = CsrfConfig::generate_secret();
    assert_eq!(secret.len(), 32);

    let config = CsrfConfig::new(secret).unwrap();
    assert_eq!(config.token_ttl, 3600);
    assert_eq!(config.cookie_name, "csrf_token");
}

#[test]
fn test_csrf_config_builder() {
    let config = CsrfConfig::default()
        .with_token_ttl(7200)
        .with_cookie_name("_csrf")
        .with_header_name("X-CSRF-TOKEN")
        .with_cookie_secure(true)
        .with_cookie_same_site(SameSite::Strict);

    assert_eq!(config.token_ttl, 7200);
    assert_eq!(config.cookie_name, "_csrf");
    assert_eq!(config.header_name, "X-CSRF-TOKEN");
    assert!(config.cookie_secure);
    assert_eq!(config.cookie_same_site, SameSite::Strict);
}

#[test]
fn test_csrf_token_generation() {
    let token = CsrfToken::generate(3600);

    assert!(!token.value.is_empty());
    assert!(!token.is_expired());
}

#[test]
fn test_csrf_token_encode_decode() {
    let secret = CsrfConfig::generate_secret();
    let token = CsrfToken::generate(3600);

    // Encode token
    let encoded = token.encode(&secret).unwrap();
    assert!(!encoded.is_empty());

    // Decode token
    let decoded = CsrfToken::decode(&encoded, &secret).unwrap();
    assert_eq!(decoded.value, token.value);
}

#[test]
fn test_csrf_token_with_session() {
    let token = CsrfToken::generate_with_session(3600, "session123".to_string());

    assert_eq!(token.session_id, Some("session123".to_string()));
    assert!(!token.is_expired());
}

#[test]
fn test_csrf_middleware_creation() {
    let config = CsrfConfig::default();
    let middleware = CsrfMiddleware::new(config);

    let req = HttpRequest::new("GET".to_string(), "/test".to_string());
    assert!(!middleware.needs_protection(&req));

    let req = HttpRequest::new("POST".to_string(), "/test".to_string());
    assert!(middleware.needs_protection(&req));
}

#[test]
fn test_csrf_middleware_excluded_paths() {
    let config = CsrfConfig::default()
        .with_exclude_paths(vec!["/api/webhook".to_string()]);
    let middleware = CsrfMiddleware::new(config);

    let req = HttpRequest::new("POST".to_string(), "/api/webhook/github".to_string());
    assert!(!middleware.needs_protection(&req));

    let req = HttpRequest::new("POST".to_string(), "/api/submit".to_string());
    assert!(middleware.needs_protection(&req));
}

#[test]
fn test_csrf_same_site_enum() {
    assert_eq!(SameSite::Strict.as_str(), "Strict");
    assert_eq!(SameSite::Lax.as_str(), "Lax");
    assert_eq!(SameSite::None.as_str(), "None");
}

#[test]
fn test_csrf_token_validation() {
    let token = CsrfToken::generate(3600);
    assert!(token.validate().is_ok());

    // Create an expired token
    let mut expired_token = CsrfToken::generate(0);
    expired_token.expires_at = chrono::Utc::now() - chrono::Duration::seconds(1);
    assert!(expired_token.is_expired());
    assert!(expired_token.validate().is_err());
}

#[test]
fn test_csrf_invalid_signature() {
    let secret1 = CsrfConfig::generate_secret();
    let secret2 = CsrfConfig::generate_secret();

    let token = CsrfToken::generate(3600);
    let encoded = token.encode(&secret1).unwrap();

    // Try to decode with different secret
    let result = CsrfToken::decode(&encoded, &secret2);
    assert!(result.is_err());
}

