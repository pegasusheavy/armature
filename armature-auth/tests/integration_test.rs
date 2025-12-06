//! Integration tests for armature-auth

use armature_auth::*;

// Note: Many auth tests would require external services (OAuth providers, SAML IdP)
// These tests focus on configuration and structure

#[test]
fn test_oauth2_provider_config_google() {
    let config = OAuth2ProviderConfig::google(
        "client_id".to_string(),
        "client_secret".to_string(),
        "http://localhost:3000/callback".to_string(),
    );

    assert_eq!(config.name, "google");
    assert!(config.authorization_url.contains("accounts.google.com"));
    assert!(config.token_url.contains("oauth2.googleapis.com"));
}

#[test]
fn test_oauth2_provider_config_github() {
    let config = OAuth2ProviderConfig::github(
        "client_id".to_string(),
        "client_secret".to_string(),
        "http://localhost:3000/callback".to_string(),
    );

    assert_eq!(config.name, "github");
    assert!(config.authorization_url.contains("github.com"));
}

#[test]
fn test_oauth2_config_builder() {
    let config = OAuth2Config::new("default")
        .with_provider(OAuth2ProviderConfig::google(
            "id".to_string(),
            "secret".to_string(),
            "http://localhost/callback".to_string(),
        ))
        .with_pkce(true);

    assert_eq!(config.strategy, "default");
    assert!(config.use_pkce);
    assert_eq!(config.providers.len(), 1);
}

#[test]
fn test_saml_config_creation() {
    let config = SamlConfig::new(
        "https://idp.example.com/metadata".to_string(),
        "https://sp.example.com".to_string(),
    );

    assert_eq!(config.idp_metadata_url, "https://idp.example.com/metadata");
    assert_eq!(config.sp_entity_id, "https://sp.example.com");
}

#[test]
fn test_saml_config_builder() {
    let config = SamlConfig::new(
        "https://idp.example.com/metadata".to_string(),
        "https://sp.example.com".to_string(),
    )
    .with_cert_path("/path/to/cert.pem")
    .with_key_path("/path/to/key.pem")
    .with_assertion_consumer_service_url("https://sp.example.com/acs");

    assert_eq!(config.cert_path, Some("/path/to/cert.pem".to_string()));
    assert_eq!(config.key_path, Some("/path/to/key.pem".to_string()));
    assert_eq!(config.acs_url, Some("https://sp.example.com/acs".to_string()));
}

#[test]
fn test_auth_error_display() {
    let err = AuthError::InvalidCredentials("Bad password".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Bad password"));
}

#[test]
fn test_oauth2_provider_config_microsoft() {
    let config = OAuth2ProviderConfig::microsoft(
        "client_id".to_string(),
        "client_secret".to_string(),
        "http://localhost:3000/callback".to_string(),
    );

    assert_eq!(config.name, "microsoft");
    assert!(config.authorization_url.contains("login.microsoftonline.com"));
}

#[test]
fn test_oauth2_provider_config_custom() {
    let config = OAuth2ProviderConfig::custom(
        "custom_provider".to_string(),
        "client_id".to_string(),
        "client_secret".to_string(),
        "https://custom.com/oauth/authorize".to_string(),
        "https://custom.com/oauth/token".to_string(),
        "http://localhost:3000/callback".to_string(),
    );

    assert_eq!(config.name, "custom_provider");
    assert_eq!(config.authorization_url, "https://custom.com/oauth/authorize");
    assert_eq!(config.token_url, "https://custom.com/oauth/token");
}

#[test]
fn test_oauth2_scopes() {
    let config = OAuth2ProviderConfig::google(
        "id".to_string(),
        "secret".to_string(),
        "callback".to_string(),
    )
    .with_scopes(vec!["profile".to_string(), "email".to_string()]);

    assert_eq!(config.scopes.len(), 2);
}


