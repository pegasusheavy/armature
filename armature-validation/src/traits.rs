// Validation traits

use crate::ValidationError;
use async_trait::async_trait;

/// Trait for validatable types
pub trait Validate {
    /// Validate the value and return errors if any
    fn validate(&self) -> Result<(), Vec<ValidationError>>;
}

/// Trait for async validatable types
#[async_trait]
pub trait AsyncValidate {
    /// Async validation (e.g., database checks)
    async fn validate_async(&self) -> Result<(), Vec<ValidationError>>;
}

/// Trait for custom validators
pub trait Validator: Send + Sync {
    /// Validate a value
    fn validate(&self, value: &dyn std::any::Any, field: &str) -> Result<(), ValidationError>;

    /// Get validator name
    fn name(&self) -> &'static str;
}

/// Trait for async validators
#[async_trait]
pub trait AsyncValidator: Send + Sync {
    /// Async validate a value
    async fn validate_async(
        &self,
        value: &dyn std::any::Any,
        field: &str,
    ) -> Result<(), ValidationError>;

    /// Get validator name
    fn name(&self) -> &'static str;
}

/// Validation context for additional data
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Request data or additional context
    pub data: std::collections::HashMap<String, String>,
}

impl ValidationContext {
    /// Create a new validation context
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    /// Add context data
    pub fn with_data(mut self, key: String, value: String) -> Self {
        self.data.insert(key, value);
        self
    }

    /// Get context data
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self::new()
    }
}
