//! Testing utilities for Armature framework.
//!
//! This crate provides comprehensive testing tools for Armature applications.
//!
//! ## Features
//!
//! - 🧪 **TestApp** - Integration test builder
//! - 📡 **TestClient** - HTTP test client
//! - 🎭 **MockService** - Service mocking
//! - 👁️ **Spy** - Method call tracking
//! - ✅ **Assertions** - Fluent test assertions
//!
//! ## Quick Start
//!
//! ```no_run
//! use armature_testing::*;
//! use armature_core::{HttpRequest, HttpResponse, Error};
//!
//! # tokio_test::block_on(async {
//! // Create a test app
//! let app = TestAppBuilder::new()
//!     .with_route("/hello", |_req| async {
//!         Ok(HttpResponse::ok().with_body(b"Hello!".to_vec()))
//!     })
//!     .build();
//!
//! // Make test requests
//! let client = app.client();
//! let response = client.get("/hello").await;
//! assert_eq!(response.status(), Some(200));
//! assert_eq!(response.body_string(), Some("Hello!".to_string()));
//! # });
//! ```
//!
//! ## Testing Controllers
//!
//! ```no_run
//! use armature_testing::*;
//! use armature_core::{HttpRequest, HttpResponse, Error};
//!
//! # tokio_test::block_on(async {
//! let app = TestAppBuilder::new()
//!     .with_route("/api/users", |_req| async {
//!         Ok(HttpResponse::ok().with_json(&serde_json::json!({
//!             "users": ["Alice", "Bob"]
//!         }))?)
//!     })
//!     .build();
//!
//! let client = app.client();
//! let response = client.get("/api/users").await;
//! let json: serde_json::Value = response.body_json().unwrap();
//! assert_eq!(json["users"][0], "Alice");
//! # });
//! ```
//!
//! ## Mock Services
//!
//! ```
//! use armature_testing::MockService;
//!
//! // Create a mock service
//! let mock = MockService::<String>::new();
//! mock.record_call("get_user");
//!
//! // Verify calls
//! assert_eq!(mock.call_count(), 1);
//! assert!(mock.was_called("get_user"));
//! ```
//!
//! ## Assertions
//!
//! ```
//! use armature_testing::*;
//! use armature_core::HttpResponse;
//!
//! let response = HttpResponse::ok()
//!     .with_header("Content-Type".to_string(), "application/json".to_string())
//!     .with_body(b"{\"status\":\"ok\"}".to_vec());
//!
//! let test_response = TestResponse::Success(response);
//!
//! // Assert status
//! assert_status(&test_response, 200);
//!
//! // Assert headers
//! assert_header(&test_response, "Content-Type", "application/json");
//! ```

mod assertions;
mod mock;
mod test_app;
mod test_client;
mod test_container;

pub use assertions::{assert_header, assert_json, assert_status};
pub use mock::{MockController, MockProvider, MockService};
pub use test_app::{TestApp, TestAppBuilder};
pub use test_client::{TestClient, TestResponse};
pub use test_container::TestContainer;

// Re-export common testing utilities
pub use tokio::test as tokio_test;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        // Ensure module compiles
    }
}
