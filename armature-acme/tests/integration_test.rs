//! Integration tests for armature-acme

use armature_acme::*;

#[test]
fn test_acme_config_lets_encrypt_production() {
    let config = AcmeConfig::lets_encrypt_production(
        vec!["admin@example.com".to_string()],
        vec!["example.com".to_string()],
    );

    assert!(config.directory_url.contains("acme-v02.api.letsencrypt.org"));
    assert_eq!(config.email_contacts.len(), 1);
    assert_eq!(config.domains.len(), 1);
}

#[test]
fn test_acme_config_lets_encrypt_staging() {
    let config = AcmeConfig::lets_encrypt_staging(
        vec!["admin@example.com".to_string()],
        vec!["example.com".to_string()],
    );

    assert!(config.directory_url.contains("acme-staging-v02.api.letsencrypt.org"));
}

#[test]
fn test_acme_config_builder() {
    let config = AcmeConfig::lets_encrypt_production(
        vec!["admin@example.com".to_string()],
        vec!["example.com".to_string()],
    )
    .with_challenge_type(ChallengeType::Dns01)
    .with_cert_dir("/etc/certs")
    .with_renew_before_days(7);

    assert_eq!(config.challenge_type, ChallengeType::Dns01);
    assert_eq!(config.cert_dir, "/etc/certs");
    assert_eq!(config.renew_before_days, 7);
}

#[test]
fn test_acme_challenge_types() {
    assert_eq!(format!("{:?}", ChallengeType::Http01), "Http01");
    assert_eq!(format!("{:?}", ChallengeType::Dns01), "Dns01");
    assert_eq!(format!("{:?}", ChallengeType::TlsAlpn01), "TlsAlpn01");
}

#[test]
fn test_acme_error_display() {
    let err = AcmeError::InvalidUrl("bad url".to_string());
    let display = format!("{}", err);
    assert!(display.contains("bad url"));
}

#[test]
fn test_acme_config_zerossl() {
    let config = AcmeConfig::zerossl(
        vec!["admin@example.com".to_string()],
        vec!["example.com".to_string()],
        "eab_kid".to_string(),
        "eab_hmac_key".to_string(),
    );

    assert!(config.directory_url.contains("zerossl.com"));
    assert_eq!(config.eab_kid, Some("eab_kid".to_string()));
}

#[test]
fn test_acme_config_buypass() {
    let config = AcmeConfig::buypass(
        vec!["admin@example.com".to_string()],
        vec!["example.com".to_string()],
    );

    assert!(config.directory_url.contains("buypass.com"));
}

#[test]
fn test_acme_config_google_trust_services() {
    let config = AcmeConfig::google_trust_services(
        vec!["admin@example.com".to_string()],
        vec!["example.com".to_string()],
        "eab_kid".to_string(),
        "eab_hmac_key".to_string(),
    );

    assert!(config.directory_url.contains("dv.acme-v02.api.pki.goog"));
}


