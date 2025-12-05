//! Validation framework for Armature
//!
//! Provides comprehensive data validation with built-in validators,
//! custom validation rules, and automatic request validation.
//!
//! # Examples
//!
//! ## Basic Validation
//!
//! ```
//! use armature_validation::{Validate, ValidationError, NotEmpty, MinLength, IsEmail};
//!
//! struct UserInput {
//!     name: String,
//!     email: String,
//! }
//!
//! impl Validate for UserInput {
//!     fn validate(&self) -> Result<(), Vec<ValidationError>> {
//!         let mut errors = Vec::new();
//!
//!         if let Err(e) = NotEmpty::validate(&self.name, "name") {
//!             errors.push(e);
//!         }
//!         if let Err(e) = MinLength(3).validate(&self.name, "name") {
//!             errors.push(e);
//!         }
//!         if let Err(e) = IsEmail::validate(&self.email, "email") {
//!             errors.push(e);
//!         }
//!
//!         if errors.is_empty() {
//!             Ok(())
//!         } else {
//!             Err(errors)
//!         }
//!     }
//! }
//!
//! let input = UserInput {
//!     name: "John".to_string(),
//!     email: "john@example.com".to_string(),
//! };
//! assert!(input.validate().is_ok());
//! ```
//!
//! ## Validation Rules Builder
//!
//! ```
//! use armature_validation::{ValidationRules, NotEmpty, MinLength};
//!
//! // Create rules for a field
//! let rules = ValidationRules::for_field("username")
//!     .add(|value, field| NotEmpty::validate(value, field))
//!     .add(|value, field| MinLength(3).validate(value, field));
//!
//! // Validate a value
//! let result = rules.validate("john");
//! assert!(result.is_ok());
//! ```
//!
//! ## Number Validation
//!
//! ```
//! use armature_validation::{Min, Max, InRange, IsPositive};
//!
//! // Min/Max validation
//! assert!(Min(18).validate(25, "age").is_ok());
//! assert!(Max(100).validate(50, "age").is_ok());
//!
//! // Range validation
//! let range = InRange { min: 1, max: 10 };
//! assert!(range.validate(5, "score").is_ok());
//!
//! // Positive number
//! assert!(IsPositive::validate_i32(42, "count").is_ok());
//! ```

mod errors;
mod pipe;
mod rules;
mod traits;
mod validators;

pub use errors::*;
pub use pipe::*;
pub use rules::*;
pub use traits::*;
pub use validators::*;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        // Ensure module compiles
    }
}
