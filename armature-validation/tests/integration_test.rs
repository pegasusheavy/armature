//! Integration tests for armature-validation

use armature_validation::*;

#[test]
fn test_is_email_validator() {
    assert!(IsEmail::validate("user@example.com", "email").is_ok());
    assert!(IsEmail::validate("test.user+tag@domain.co.uk", "email").is_ok());
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
fn test_min_length_validator() {
    assert!(MinLength::validate("hello", "text", 3).is_ok());
    assert!(MinLength::validate("hi", "text", 5).is_err());
}

#[test]
fn test_max_length_validator() {
    assert!(MaxLength::validate("hello", "text", 10).is_ok());
    assert!(MaxLength::validate("hello world", "text", 5).is_err());
}

#[test]
fn test_is_alpha_validator() {
    assert!(IsAlpha::validate("abcXYZ", "text").is_ok());
    assert!(IsAlpha::validate("abc123", "text").is_err());
    assert!(IsAlpha::validate("", "text").is_err());
}

#[test]
fn test_is_alphanumeric_validator() {
    assert!(IsAlphanumeric::validate("abc123", "text").is_ok());
    assert!(IsAlphanumeric::validate("abc-123", "text").is_err());
}

#[test]
fn test_is_numeric_validator() {
    assert!(IsNumeric::validate("12345", "text").is_ok());
    assert!(IsNumeric::validate("123.45", "text").is_err());
    assert!(IsNumeric::validate("abc", "text").is_err());
}

#[test]
fn test_in_range_validator() {
    assert!(InRange::validate(5, "number", 1, 10).is_ok());
    assert!(InRange::validate(0, "number", 1, 10).is_err());
    assert!(InRange::validate(11, "number", 1, 10).is_err());
}

#[test]
fn test_matches_pattern_validator() {
    assert!(MatchesPattern::validate("abc123", "code", r"^[a-z]+\d+$").is_ok());
    assert!(MatchesPattern::validate("123abc", "code", r"^[a-z]+\d+$").is_err());
}

#[test]
fn test_not_empty_validator() {
    assert!(NotEmpty::validate("hello", "text").is_ok());
    assert!(NotEmpty::validate("", "text").is_err());
    assert!(NotEmpty::validate("   ", "text").is_err());
}

#[test]
fn test_contains_validator() {
    assert!(Contains::validate("hello world", "text", "world").is_ok());
    assert!(Contains::validate("hello", "text", "world").is_err());
}

#[test]
fn test_starts_with_validator() {
    assert!(StartsWith::validate("hello world", "text", "hello").is_ok());
    assert!(StartsWith::validate("world hello", "text", "hello").is_err());
}

#[test]
fn test_ends_with_validator() {
    assert!(EndsWith::validate("hello world", "text", "world").is_ok());
    assert!(EndsWith::validate("world hello", "text", "world").is_err());
}

#[test]
fn test_matches_validator() {
    let regex = regex::Regex::new(r"^\d{3}-\d{3}-\d{4}$").unwrap();
    assert!(
        Matches(regex.clone())
            .validate("123-456-7890", "phone")
            .is_ok()
    );
    assert!(Matches(regex).validate("invalid", "phone").is_err());
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
