// Validation errors

use std::fmt;

/// Validation error for a single field
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Field name that failed validation
    pub field: String,

    /// Error message
    pub message: String,

    /// Validation constraint that failed
    pub constraint: String,

    /// Value that failed validation (optional)
    pub value: Option<String>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        let field = field.into();
        Self {
            constraint: "custom".to_string(),
            message: message.into(),
            field,
            value: None,
        }
    }

    /// Set the constraint name
    pub fn with_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraint = constraint.into();
        self
    }

    /// Set the invalid value
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Collection of validation errors
#[derive(Debug, Clone)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationError>,
}

impl ValidationErrors {
    /// Create a new validation errors collection
    pub fn new(errors: Vec<ValidationError>) -> Self {
        Self { errors }
    }

    /// Check if there are any errors
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the number of errors
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Add an error
    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Get errors for a specific field
    pub fn get_field_errors(&self, field: &str) -> Vec<&ValidationError> {
        self.errors.iter().filter(|e| e.field == field).collect()
    }

    /// Convert to JSON representation
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "errors": self.errors.iter().map(|e| {
                serde_json::json!({
                    "field": e.field,
                    "message": e.message,
                    "constraint": e.constraint,
                    "value": e.value,
                })
            }).collect::<Vec<_>>()
        })
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, "{}", error)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

impl From<Vec<ValidationError>> for ValidationErrors {
    fn from(errors: Vec<ValidationError>) -> Self {
        Self::new(errors)
    }
}
