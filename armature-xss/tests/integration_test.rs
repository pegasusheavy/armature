//! Integration tests for armature-xss

use armature_xss::*;
use armature_core::{HttpRequest};

#[test]
fn test_xss_sanitizer_creation() {
    let sanitizer = XssSanitizer::new();
    assert!(format!("{:?}", sanitizer).contains("XssSanitizer"));
}

#[test]
fn test_xss_sanitizer_removes_script() {
    let sanitizer = XssSanitizer::new();
    let dirty = r#"<p>Hello</p><script>alert('XSS')</script>"#;
    let clean = sanitizer.sanitize(dirty).unwrap();
    
    assert!(clean.contains("<p>"));
    assert!(!clean.contains("<script>"));
    assert!(clean.contains("Hello"));
}

#[test]
fn test_xss_sanitizer_strict_mode() {
    let sanitizer = XssSanitizer::strict();
    let html = r#"<div><p><strong>Bold</strong></p></div>"#;
    let clean = sanitizer.sanitize(html).unwrap();
    
    // Strict mode doesn't allow <div>
    assert!(!clean.contains("<div>"));
    assert!(clean.contains("<strong>"));
}

#[test]
fn test_xss_sanitizer_permissive_mode() {
    let sanitizer = XssSanitizer::permissive();
    let html = r#"<div><img src="image.jpg" alt="Test"/></div>"#;
    let clean = sanitizer.sanitize(html).unwrap();
    
    assert!(clean.contains("<div>"));
    assert!(clean.contains("<img"));
}

#[test]
fn test_xss_encoder_html() {
    let text = r#"<script>alert("XSS")</script>"#;
    let encoded = XssEncoder::encode_html(text);
    
    assert!(!encoded.contains('<'));
    assert!(!encoded.contains('>'));
    assert!(encoded.contains("&lt;"));
    assert!(encoded.contains("&gt;"));
}

#[test]
fn test_xss_encoder_javascript() {
    let text = r#"'; alert('XSS'); //'"#;
    let encoded = XssEncoder::encode_javascript(text);
    
    assert!(encoded.contains("\\'"));
    assert!(!encoded.contains("';"));
}

#[test]
fn test_xss_encoder_url() {
    let text = "hello world&test=value";
    let encoded = XssEncoder::encode_url(text);
    
    assert!(encoded.contains("%20")); // space
    assert!(encoded.contains("%26")); // &
}

#[test]
fn test_xss_encoder_decode() {
    let original = r#"<div class="test">Hello & "goodbye"</div>"#;
    let encoded = XssEncoder::encode_html(original);
    let decoded = XssEncoder::decode_html(&encoded);
    
    assert_eq!(original, decoded);
}

#[test]
fn test_xss_validator_detects_script() {
    let xss = r#"<script>alert('XSS')</script>"#;
    assert!(XssValidator::contains_xss(xss));
    assert!(XssValidator::validate(xss).is_err());
}

#[test]
fn test_xss_validator_detects_onerror() {
    let xss = r#"<img src="x" onerror="alert('XSS')">"#;
    assert!(XssValidator::contains_xss(xss));
}

#[test]
fn test_xss_validator_detects_javascript_protocol() {
    let xss = r#"<a href="javascript:alert('XSS')">Click</a>"#;
    assert!(XssValidator::contains_xss(xss));
}

#[test]
fn test_xss_validator_safe_content() {
    let safe = r#"<p>Hello <strong>world</strong>!</p>"#;
    assert!(!XssValidator::contains_xss(safe));
    assert!(XssValidator::validate(safe).is_ok());
}

#[test]
fn test_xss_validator_attack_types() {
    assert_eq!(
        XssValidator::detect_attack_type(r#"<script>alert('XSS')</script>"#),
        Some("Script injection")
    );
    
    assert_eq!(
        XssValidator::detect_attack_type(r#"<img onerror="alert()">"#),
        Some("Event handler injection (onerror)")
    );
    
    assert_eq!(
        XssValidator::detect_attack_type(r#"<a href="javascript:void(0)">Link</a>"#),
        Some("JavaScript protocol")
    );
}

#[test]
fn test_xss_middleware_creation() {
    let config = XssConfig::default();
    let middleware = XssMiddleware::new(config);
    
    let req = HttpRequest::new("POST".to_string(), "/api/submit".to_string());
    assert!(middleware.needs_protection(&req));
}

#[test]
fn test_xss_middleware_excluded_paths() {
    let config = XssConfig::default()
        .with_exclude_paths(vec!["/api/webhook".to_string()]);
    let middleware = XssMiddleware::new(config);
    
    let req = HttpRequest::new("POST".to_string(), "/api/webhook/receive".to_string());
    assert!(!middleware.needs_protection(&req));
}

#[test]
fn test_xss_config_builder() {
    let config = XssConfig::new()
        .with_auto_sanitize(true)
        .with_validation(true)
        .with_exclude_paths(vec!["/public".to_string()]);
    
    assert!(config.auto_sanitize);
    assert!(config.enable_validation);
    assert_eq!(config.exclude_paths.len(), 1);
}

