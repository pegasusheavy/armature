// Validation rules builder

use crate::ValidationError;

type ValidatorFn = Box<dyn Fn(&str, &str) -> Result<(), ValidationError> + Send + Sync>;

/// Builder for creating validation rules
pub struct ValidationRules {
    validators: Vec<ValidatorFn>,
    field: String,
}

impl ValidationRules {
    /// Create new validation rules for a field
    pub fn for_field(field: impl Into<String>) -> Self {
        Self {
            validators: Vec::new(),
            field: field.into(),
        }
    }

    /// Add a custom validator function
    #[allow(clippy::should_implement_trait)]
    pub fn add<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str, &str) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }

    /// Validate a value against all rules
    pub fn validate(&self, value: &str) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for validator in &self.validators {
            if let Err(error) = validator(value, &self.field) {
                errors.push(error);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Validation rules builder for complex validation scenarios
pub struct ValidationBuilder {
    rules: Vec<ValidationRules>,
}

impl ValidationBuilder {
    /// Create a new validation builder
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add rules for a field
    pub fn field(mut self, rules: ValidationRules) -> Self {
        self.rules.push(rules);
        self
    }

    /// Validate all fields
    pub fn validate(
        &self,
        data: &std::collections::HashMap<String, String>,
    ) -> Result<(), Vec<ValidationError>> {
        let mut all_errors = Vec::new();

        for rule in &self.rules {
            if let Some(value) = data.get(&rule.field) {
                if let Err(mut errors) = rule.validate(value) {
                    all_errors.append(&mut errors);
                }
            }
        }

        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }
}

impl Default for ValidationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validators::*;

    #[test]
    fn test_validation_rules() {
        let rules = ValidationRules::for_field("email")
            .add(|value, field| NotEmpty::validate(value, field))
            .add(|value, field| IsEmail::validate(value, field));

        assert!(rules.validate("test@example.com").is_ok());
        assert!(rules.validate("invalid").is_err());
        assert!(rules.validate("").is_err());
    }

    #[test]
    fn test_validation_builder() {
        let mut data = std::collections::HashMap::new();
        data.insert("name".to_string(), "John".to_string());
        data.insert("email".to_string(), "john@example.com".to_string());

        let builder = ValidationBuilder::new()
            .field(
                ValidationRules::for_field("name")
                    .add(|value, field| NotEmpty::validate(value, field)),
            )
            .field(
                ValidationRules::for_field("email")
                    .add(|value, field| IsEmail::validate(value, field)),
            );

        assert!(builder.validate(&data).is_ok());
    }
}
