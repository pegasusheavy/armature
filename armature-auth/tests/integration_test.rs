//! Integration tests for armature-auth

use armature_auth::*;

#[test]
fn test_auth_service_creation() {
    let service = AuthService::new();
    assert!(service.jwt_manager().is_none());
}

#[test]
fn test_password_hashing_and_verification() {
    let service = AuthService::new();

    let password = "secure_password123";
    let hash = service.hash_password(password).unwrap();

    // Verify correct password
    assert!(service.verify_password(password, &hash).unwrap());

    // Verify wrong password
    assert!(!service.verify_password("wrong_password", &hash).unwrap());
}

#[test]
fn test_auth_error_display() {
    let err = AuthError::InvalidCredentials;
    let display = format!("{}", err);
    assert!(display.contains("Invalid credentials"));

    let err = AuthError::AuthenticationFailed("Bad token".to_string());
    let display = format!("{}", err);
    assert!(display.contains("Bad token"));
}

#[test]
fn test_auth_error_variants() {
    // Test various error types compile correctly
    let _ = AuthError::UserNotFound;
    let _ = AuthError::InactiveUser;
    let _ = AuthError::Unauthorized;
    let _ = AuthError::Forbidden("Insufficient permissions".to_string());
    let _ = AuthError::InvalidToken("Expired".to_string());
    let _ = AuthError::TokenExpired;
    let _ = AuthError::MissingRole("admin".to_string());
    let _ = AuthError::MissingPermission("write".to_string());
}

#[test]
fn test_google_config_creation() {
    use armature_auth::providers::GoogleConfig;

    let config = GoogleConfig::new(
        "client_id".to_string(),
        "client_secret".to_string(),
        "http://localhost:3000/callback".to_string(),
    );

    assert_eq!(config.client_id, "client_id");
    assert_eq!(config.client_secret, "client_secret");
    assert_eq!(config.redirect_url, "http://localhost:3000/callback");
    assert!(!config.scopes.is_empty()); // Default scopes
}

#[test]
fn test_google_config_with_custom_scopes() {
    use armature_auth::providers::GoogleConfig;

    let config = GoogleConfig::new(
        "id".to_string(),
        "secret".to_string(),
        "callback".to_string(),
    )
    .with_scopes(vec!["profile".to_string(), "email".to_string()]);

    assert_eq!(config.scopes.len(), 2);
}

#[cfg(feature = "saml")]
#[test]
fn test_saml_config_creation() {
    let config = SamlConfig::new(
        "https://myapp.com".to_string(),
        "https://myapp.com/callback".to_string(),
        IdpMetadata::Url("https://idp.example.com/metadata".to_string()),
    );

    assert_eq!(config.entity_id, "https://myapp.com");
    assert_eq!(config.acs_url, "https://myapp.com/callback");
}

#[cfg(feature = "saml")]
#[test]
fn test_idp_metadata_variants() {
    let url_meta = IdpMetadata::Url("https://idp.example.com/metadata".to_string());
    let xml_meta = IdpMetadata::Xml("<EntityDescriptor>...</EntityDescriptor>".to_string());

    if let IdpMetadata::Url(url) = url_meta {
        assert!(url.contains("idp.example.com"));
    }

    if let IdpMetadata::Xml(xml) = xml_meta {
        assert!(xml.contains("EntityDescriptor"));
    }
}
