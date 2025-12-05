// Built-in validators

use crate::ValidationError;
use once_cell::sync::Lazy;
use regex::Regex;

// Common regex patterns
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap()
});

static URL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap());

static UUID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap()
});

static ALPHA_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z]+$").unwrap());

static ALPHANUMERIC_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9]+$").unwrap());

static NUMERIC_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[0-9]+$").unwrap());

// String validators

/// Validates that a string is not empty
pub struct NotEmpty;

impl NotEmpty {
    pub fn validate(value: &str, field: &str) -> Result<(), ValidationError> {
        if value.trim().is_empty() {
            Err(
                ValidationError::new(field, format!("{} should not be empty", field))
                    .with_constraint("notEmpty"),
            )
        } else {
            Ok(())
        }
    }
}

/// Validates minimum string length
pub struct MinLength(pub usize);

impl MinLength {
    pub fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError> {
        if value.len() < self.0 {
            Err(ValidationError::new(
                field,
                format!("{} must be at least {} characters", field, self.0),
            )
            .with_constraint("minLength")
            .with_value(value.to_string()))
        } else {
            Ok(())
        }
    }
}

/// Validates maximum string length
pub struct MaxLength(pub usize);

impl MaxLength {
    pub fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError> {
        if value.len() > self.0 {
            Err(ValidationError::new(
                field,
                format!("{} must be at most {} characters", field, self.0),
            )
            .with_constraint("maxLength")
            .with_value(value.to_string()))
        } else {
            Ok(())
        }
    }
}

/// Validates email format
pub struct IsEmail;

impl IsEmail {
    pub fn validate(value: &str, field: &str) -> Result<(), ValidationError> {
        if EMAIL_REGEX.is_match(value) {
            Ok(())
        } else {
            Err(
                ValidationError::new(field, format!("{} must be a valid email", field))
                    .with_constraint("isEmail")
                    .with_value(value.to_string()),
            )
        }
    }
}

/// Validates URL format
pub struct IsUrl;

impl IsUrl {
    pub fn validate(value: &str, field: &str) -> Result<(), ValidationError> {
        if URL_REGEX.is_match(value) {
            Ok(())
        } else {
            Err(
                ValidationError::new(field, format!("{} must be a valid URL", field))
                    .with_constraint("isUrl")
                    .with_value(value.to_string()),
            )
        }
    }
}

/// Validates UUID format
pub struct IsUuid;

impl IsUuid {
    pub fn validate(value: &str, field: &str) -> Result<(), ValidationError> {
        if UUID_REGEX.is_match(value) {
            Ok(())
        } else {
            Err(
                ValidationError::new(field, format!("{} must be a valid UUID", field))
                    .with_constraint("isUuid")
                    .with_value(value.to_string()),
            )
        }
    }
}

/// Validates alphabetic characters only
pub struct IsAlpha;

impl IsAlpha {
    pub fn validate(value: &str, field: &str) -> Result<(), ValidationError> {
        if ALPHA_REGEX.is_match(value) {
            Ok(())
        } else {
            Err(
                ValidationError::new(field, format!("{} must contain only letters", field))
                    .with_constraint("isAlpha")
                    .with_value(value.to_string()),
            )
        }
    }
}

/// Validates alphanumeric characters only
pub struct IsAlphanumeric;

impl IsAlphanumeric {
    pub fn validate(value: &str, field: &str) -> Result<(), ValidationError> {
        if ALPHANUMERIC_REGEX.is_match(value) {
            Ok(())
        } else {
            Err(ValidationError::new(
                field,
                format!("{} must contain only letters and numbers", field),
            )
            .with_constraint("isAlphanumeric")
            .with_value(value.to_string()))
        }
    }
}

/// Validates numeric characters only
pub struct IsNumeric;

impl IsNumeric {
    pub fn validate(value: &str, field: &str) -> Result<(), ValidationError> {
        if NUMERIC_REGEX.is_match(value) {
            Ok(())
        } else {
            Err(
                ValidationError::new(field, format!("{} must contain only numbers", field))
                    .with_constraint("isNumeric")
                    .with_value(value.to_string()),
            )
        }
    }
}

// Number validators

/// Validates minimum value
pub struct Min<T>(pub T);

impl Min<i32> {
    pub fn validate(&self, value: i32, field: &str) -> Result<(), ValidationError> {
        if value < self.0 {
            Err(
                ValidationError::new(field, format!("{} must be at least {}", field, self.0))
                    .with_constraint("min")
                    .with_value(value.to_string()),
            )
        } else {
            Ok(())
        }
    }
}

impl Min<f64> {
    pub fn validate(&self, value: f64, field: &str) -> Result<(), ValidationError> {
        if value < self.0 {
            Err(
                ValidationError::new(field, format!("{} must be at least {}", field, self.0))
                    .with_constraint("min")
                    .with_value(value.to_string()),
            )
        } else {
            Ok(())
        }
    }
}

/// Validates maximum value
pub struct Max<T>(pub T);

impl Max<i32> {
    pub fn validate(&self, value: i32, field: &str) -> Result<(), ValidationError> {
        if value > self.0 {
            Err(
                ValidationError::new(field, format!("{} must be at most {}", field, self.0))
                    .with_constraint("max")
                    .with_value(value.to_string()),
            )
        } else {
            Ok(())
        }
    }
}

impl Max<f64> {
    pub fn validate(&self, value: f64, field: &str) -> Result<(), ValidationError> {
        if value > self.0 {
            Err(
                ValidationError::new(field, format!("{} must be at most {}", field, self.0))
                    .with_constraint("max")
                    .with_value(value.to_string()),
            )
        } else {
            Ok(())
        }
    }
}

/// Validates value is positive
pub struct IsPositive;

impl IsPositive {
    pub fn validate_i32(value: i32, field: &str) -> Result<(), ValidationError> {
        if value > 0 {
            Ok(())
        } else {
            Err(
                ValidationError::new(field, format!("{} must be a positive number", field))
                    .with_constraint("isPositive")
                    .with_value(value.to_string()),
            )
        }
    }

    pub fn validate_f64(value: f64, field: &str) -> Result<(), ValidationError> {
        if value > 0.0 {
            Ok(())
        } else {
            Err(
                ValidationError::new(field, format!("{} must be a positive number", field))
                    .with_constraint("isPositive")
                    .with_value(value.to_string()),
            )
        }
    }
}

/// Validates value is in range
pub struct InRange<T> {
    pub min: T,
    pub max: T,
}

impl InRange<i32> {
    pub fn validate(&self, value: i32, field: &str) -> Result<(), ValidationError> {
        if value >= self.min && value <= self.max {
            Ok(())
        } else {
            Err(ValidationError::new(
                field,
                format!("{} must be between {} and {}", field, self.min, self.max),
            )
            .with_constraint("inRange")
            .with_value(value.to_string()))
        }
    }
}

/// Custom regex validator
pub struct Matches(pub Regex);

impl Matches {
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self(Regex::new(pattern)?))
    }

    pub fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError> {
        if self.0.is_match(value) {
            Ok(())
        } else {
            Err(
                ValidationError::new(field, format!("{} does not match required pattern", field))
                    .with_constraint("matches")
                    .with_value(value.to_string()),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_empty() {
        assert!(NotEmpty::validate("test", "field").is_ok());
        assert!(NotEmpty::validate("", "field").is_err());
        assert!(NotEmpty::validate("   ", "field").is_err());
    }

    #[test]
    fn test_min_length() {
        let validator = MinLength(5);
        assert!(validator.validate("hello", "field").is_ok());
        assert!(validator.validate("hi", "field").is_err());
    }

    #[test]
    fn test_is_email() {
        assert!(IsEmail::validate("test@example.com", "email").is_ok());
        assert!(IsEmail::validate("invalid", "email").is_err());
    }

    #[test]
    fn test_min_value() {
        let validator = Min(10);
        assert!(validator.validate(15, "age").is_ok());
        assert!(validator.validate(5, "age").is_err());
    }
}
