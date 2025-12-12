//! Integration tests for armature-jwt

use armature_jwt::*;
use jsonwebtoken::Algorithm;
use std::time::Duration;

#[test]
fn test_jwt_manager_creation() {
    let config = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string());
    let manager = JwtManager::new(config).unwrap();
    // JwtManager created successfully
    let _ = manager;
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
}

#[test]
fn test_jwt_config_with_algorithm() {
    let config = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string())
        .with_algorithm(Algorithm::HS256);
    assert_eq!(config.algorithm, Algorithm::HS256);

    let config = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string())
        .with_algorithm(Algorithm::HS384);
    assert_eq!(config.algorithm, Algorithm::HS384);

    let config = JwtConfig::new("test_secret_key_32_bytes_long!!!".to_string())
        .with_algorithm(Algorithm::HS512);
    assert_eq!(config.algorithm, Algorithm::HS512);
}

#[test]
fn test_jwt_config_default_values() {
    let config = JwtConfig::new("test_secret".to_string());

    // Default algorithm should be HS256
    assert_eq!(config.algorithm, Algorithm::HS256);
}
