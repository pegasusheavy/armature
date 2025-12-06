//! Integration tests for armature-jwt

use armature_jwt::*;
use jsonwebtoken::Algorithm;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestClaims {
    sub: String,
    name: String,
    admin: bool,
}

#[test]
fn test_jwt_manager_creation() {
    let config = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string());
    let manager = JwtManager::new(config);
    
    assert!(format!("{:?}", manager).contains("JwtManager"));
}

#[test]
fn test_jwt_sign_and_verify() {
    let config = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string())
        .with_expiration(Duration::from_secs(3600));
    let manager = JwtManager::new(config);
    
    let claims = TestClaims {
        sub: "user123".to_string(),
        name: "John Doe".to_string(),
        admin: true,
    };
    
    // Sign token
    let token = manager.sign(&claims).unwrap();
    assert!(!token.is_empty());
    
    // Verify token
    let verified: Claims<TestClaims> = manager.verify(&token).unwrap();
    assert_eq!(verified.claims.sub, "user123");
    assert_eq!(verified.claims.name, "John Doe");
    assert!(verified.claims.admin);
}

#[test]
fn test_jwt_with_standard_claims() {
    let config = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string())
        .with_issuer("test_issuer".to_string())
        .with_audience(vec!["test_audience".to_string()]);
    let manager = JwtManager::new(config);
    
    let custom_claims = TestClaims {
        sub: "user123".to_string(),
        name: "John Doe".to_string(),
        admin: false,
    };
    
    let token = manager.sign(&custom_claims).unwrap();
    let verified: Claims<TestClaims> = manager.verify(&token).unwrap();
    
    // Check standard claims
    assert!(verified.standard.iss.is_some());
    assert_eq!(verified.standard.iss.unwrap(), "test_issuer");
}

#[test]
fn test_jwt_expired_token() {
    let config = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string())
        .with_expiration(Duration::from_secs(0)); // Expires immediately
    let manager = JwtManager::new(config);
    
    let claims = TestClaims {
        sub: "user123".to_string(),
        name: "John Doe".to_string(),
        admin: true,
    };
    
    let token = manager.sign(&claims).unwrap();
    
    // Wait a moment to ensure expiration
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    // Should fail verification
    let result: Result<Claims<TestClaims>> = manager.verify(&token);
    assert!(result.is_err());
}

#[test]
fn test_jwt_invalid_token() {
    let config = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string());
    let manager = JwtManager::new(config);
    
    let result: Result<Claims<TestClaims>> = manager.verify("invalid.token.here");
    assert!(result.is_err());
}

#[test]
fn test_jwt_algorithm_variants() {
    // HS256
    let config_hs256 = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string())
        .with_algorithm(Algorithm::HS256);
    let manager_hs256 = JwtManager::new(config_hs256);
    
    let claims = TestClaims {
        sub: "user123".to_string(),
        name: "Test User".to_string(),
        admin: false,
    };
    
    let token = manager_hs256.sign(&claims).unwrap();
    assert!(manager_hs256.verify::<Claims<TestClaims>>(&token).is_ok());
    
    // HS384
    let config_hs384 = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string())
        .with_algorithm(Algorithm::HS384);
    let manager_hs384 = JwtManager::new(config_hs384);
    let token = manager_hs384.sign(&claims).unwrap();
    assert!(manager_hs384.verify::<Claims<TestClaims>>(&token).is_ok());
    
    // HS512
    let config_hs512 = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string())
        .with_algorithm(Algorithm::HS512);
    let manager_hs512 = JwtManager::new(config_hs512);
    let token = manager_hs512.sign(&claims).unwrap();
    assert!(manager_hs512.verify::<Claims<TestClaims>>(&token).is_ok());
}

#[test]
fn test_jwt_config_builder() {
    let config = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string())
        .with_expiration(Duration::from_secs(7200))
        .with_issuer("my_app".to_string())
        .with_audience(vec!["api".to_string()])
        .with_leeway(60);
    
    assert_eq!(config.expires_in, Duration::from_secs(7200));
    assert_eq!(config.issuer, Some("my_app".to_string()));
    assert_eq!(config.audience, Some(vec!["api".to_string()]));
    assert_eq!(config.leeway, 60);
    assert!(config.validate_exp);
    assert!(!config.validate_nbf);
}
