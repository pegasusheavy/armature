//! Utility macros for the Armature framework
//!
//! This crate provides convenient macros to make working with Armature easier
//! and reduce boilerplate code.
//!
//! ## Response Macros
//!
//! Quick HTTP response creation:
//! - `json!` - Create JSON responses
//! - `html!` - Create HTML responses
//! - `text!` - Create plain text responses
//!
//! ## Validation Macros
//!
//! Input validation helpers:
//! - `validate!` - Validate fields with custom rules
//! - `validate_required!` - Check required fields
//! - `validate_email!` - Validate email format
//!
//! ## Test Macros
//!
//! Testing utilities:
//! - `test_request!` - Create test HTTP requests
//! - `assert_json!` - Assert JSON response equality
//! - `assert_status!` - Assert HTTP status codes
//!
//! ## Model Macros
//!
//! Database model helpers:
//! - `#[derive(Model)]` - Common model traits
//! - `#[derive(ApiModel)]` - API serialization
//!
//! ## Error Handling
//!
//! Quick error creation:
//! - `bail!` - Return early with an error
//! - `ensure!` - Conditional error return

use proc_macro::TokenStream;

mod response;
mod validation;
mod test_helpers;
mod model;
mod error_helpers;

// ============================================================================
// Response Macros
// ============================================================================

/// Create a JSON response with automatic serialization
///
/// # Examples
///
/// ```ignore
/// // Simple JSON response
/// json!(200, { "message": "Success", "id": 123 })
///
/// // With status helper
/// json!(ok, { "user": user_data })
///
/// // Just data (defaults to 200 OK)
/// json!({ "items": vec })
/// ```
#[proc_macro]
pub fn json(input: TokenStream) -> TokenStream {
    response::json_impl(input)
}

/// Create an HTML response
///
/// # Examples
///
/// ```ignore
/// html!(200, "<h1>Hello</h1>")
/// html!(ok, "<p>Content</p>")
/// html!("<html>...</html>")
/// ```
#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    response::html_impl(input)
}

/// Create a plain text response
///
/// # Examples
///
/// ```ignore
/// text!(200, "Plain text content")
/// text!(ok, "Success")
/// text!("Hello, world!")
/// ```
#[proc_macro]
pub fn text(input: TokenStream) -> TokenStream {
    response::text_impl(input)
}

/// Create a redirect response
///
/// # Examples
///
/// ```ignore
/// redirect!("/home")
/// redirect!(301, "/new-location")
/// redirect!(temporary, "/temp")
/// ```
#[proc_macro]
pub fn redirect(input: TokenStream) -> TokenStream {
    response::redirect_impl(input)
}

// ============================================================================
// Validation Macros
// ============================================================================

/// Validate a field with a custom validation function
///
/// # Examples
///
/// ```ignore
/// validate!(email, is_valid_email, "Invalid email format");
/// validate!(age >= 18, "Must be 18 or older");
/// validate!(password.len() >= 8, "Password too short");
/// ```
#[proc_macro]
pub fn validate(input: TokenStream) -> TokenStream {
    validation::validate_impl(input)
}

/// Validate that required fields are present
///
/// # Examples
///
/// ```ignore
/// validate_required!(name, email, password);
/// ```
#[proc_macro]
pub fn validate_required(input: TokenStream) -> TokenStream {
    validation::validate_required_impl(input)
}

/// Validate email format
///
/// # Examples
///
/// ```ignore
/// validate_email!(user_email);
/// ```
#[proc_macro]
pub fn validate_email(input: TokenStream) -> TokenStream {
    validation::validate_email_impl(input)
}

// ============================================================================
// Test Helper Macros
// ============================================================================

/// Create a test HTTP request
///
/// # Examples
///
/// ```ignore
/// let req = test_request!(GET "/users");
/// let req = test_request!(POST "/users", json!({ "name": "Alice" }));
/// let req = test_request!(GET "/users/123", headers: { "Authorization": "Bearer token" });
/// ```
#[proc_macro]
pub fn test_request(input: TokenStream) -> TokenStream {
    test_helpers::test_request_impl(input)
}

/// Assert JSON response equality
///
/// # Examples
///
/// ```ignore
/// assert_json!(response, { "id": 1, "name": "Alice" });
/// ```
#[proc_macro]
pub fn assert_json(input: TokenStream) -> TokenStream {
    test_helpers::assert_json_impl(input)
}

/// Assert HTTP status code
///
/// # Examples
///
/// ```ignore
/// assert_status!(response, 200);
/// assert_status!(response, ok);
/// ```
#[proc_macro]
pub fn assert_status(input: TokenStream) -> TokenStream {
    test_helpers::assert_status_impl(input)
}

// ============================================================================
// Error Handling Macros
// ============================================================================

/// Return early with an error
///
/// # Examples
///
/// ```ignore
/// bail!("User not found");
/// bail!(NotFound, "User {} not found", id);
/// ```
#[proc_macro]
pub fn bail(input: TokenStream) -> TokenStream {
    error_helpers::bail_impl(input)
}

/// Ensure a condition is true, otherwise return an error
///
/// # Examples
///
/// ```ignore
/// ensure!(user.is_active(), "User account is inactive");
/// ensure!(age >= 18, BadRequest, "Must be 18 or older");
/// ```
#[proc_macro]
pub fn ensure(input: TokenStream) -> TokenStream {
    error_helpers::ensure_impl(input)
}

// ============================================================================
// Model Derive Macros
// ============================================================================

/// Derive common model traits
///
/// Automatically implements: Debug, Clone, Serialize, Deserialize
///
/// # Examples
///
/// ```ignore
/// #[derive(Model)]
/// pub struct User {
///     pub id: i64,
///     pub name: String,
///     pub email: String,
/// }
/// ```
#[proc_macro_derive(Model, attributes(model))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    model::derive_model_impl(input)
}

/// Derive API model traits with field visibility control
///
/// # Examples
///
/// ```ignore
/// #[derive(ApiModel)]
/// pub struct User {
///     pub id: i64,
///     pub name: String,
///     #[api(skip)]
///     pub password: String,
/// }
/// ```
#[proc_macro_derive(ApiModel, attributes(api))]
pub fn derive_api_model(input: TokenStream) -> TokenStream {
    model::derive_api_model_impl(input)
}

/// Derive resource model with CRUD operations
///
/// # Examples
///
/// ```ignore
/// #[derive(Resource)]
/// #[resource(table = "users")]
/// pub struct User {
///     #[resource(primary_key)]
///     pub id: i64,
///     pub name: String,
/// }
/// ```
#[proc_macro_derive(Resource, attributes(resource))]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    model::derive_resource_impl(input)
}

