//! Integration tests for armature-validation

use armature_validation::*;

#[test]
fn test_not_empty_validator() {
    assert!(NotEmpty::validate("hello", "text").is_ok());
    assert!(NotEmpty::validate("", "text").is_err());
}

#[test]
fn test_min_length_validator() {
    let validator = MinLength(3);
    assert!(validator.validate("hello", "text").is_ok());
    assert!(validator.validate("hi", "text").is_err());
}

#[test]
fn test_max_length_validator() {
    let validator = MaxLength(10);
    assert!(validator.validate("hello", "text").is_ok());
    assert!(validator.validate("hello world today", "text").is_err());
}

#[test]
fn test_is_email_validator() {
    assert!(IsEmail::validate("user@example.com", "email").is_ok());
    assert!(IsEmail::validate("test.user@domain.co.uk", "email").is_ok());
    assert!(IsEmail::validate("invalid-email", "email").is_err());
    assert!(IsEmail::validate("@example.com", "email").is_err());
}

#[test]
fn test_is_url_validator() {
    assert!(IsUrl::validate("https://example.com", "url").is_ok());
    assert!(IsUrl::validate("http://localhost:8080/path", "url").is_ok());
    assert!(IsUrl::validate("not-a-url", "url").is_err());
}

#[test]
fn test_is_uuid_validator() {
    assert!(IsUuid::validate("550e8400-e29b-41d4-a716-446655440000", "id").is_ok());
    assert!(IsUuid::validate("invalid-uuid", "id").is_err());
}

#[test]
fn test_is_alpha_validator() {
    assert!(IsAlpha::validate("abcXYZ", "text").is_ok());
    assert!(IsAlpha::validate("abc123", "text").is_err());
}

#[test]
fn test_is_alphanumeric_validator() {
    assert!(IsAlphanumeric::validate("abc123", "text").is_ok());
    assert!(IsAlphanumeric::validate("abc-123", "text").is_err());
}

#[test]
fn test_is_numeric_validator() {
    assert!(IsNumeric::validate("12345", "text").is_ok());
    assert!(IsNumeric::validate("abc", "text").is_err());
}

#[test]
fn test_matches_validator() {
    let regex = regex::Regex::new(r"^\d{3}-\d{3}-\d{4}$").unwrap();
    let validator = Matches(regex);
    assert!(validator.validate("123-456-7890", "phone").is_ok());
    assert!(validator.validate("invalid", "phone").is_err());
}

#[test]
fn test_validation_rules_builder() {
    let rules = ValidationRules::for_field("username");
    let rules = rules.add(|value: &str, field: &str| {
        if value.len() < 3 {
            Err(ValidationError::new(field, "must be at least 3 characters"))
        } else {
            Ok(())
        }
    });

    assert!(rules.validate("user123").is_ok());
    assert!(rules.validate("ab").is_err());
}

#[test]
fn test_validation_error_creation() {
    let error = ValidationError::new("email", "invalid email format");

    assert_eq!(error.field, "email");
    assert_eq!(error.message, "invalid email format");
}
