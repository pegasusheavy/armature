//! Core library for the Armature HTTP framework.
//!
//! This module contains the foundational types, traits, and runtime components
//! for building web applications with Armature.
//!
//! ## Quick Start - HTTP Request Handling
//!
//! ```
//! use armature_core::HttpRequest;
//!
//! // Create an HTTP request using the constructor
//! let request = HttpRequest::new("GET".to_string(), "/api/users".to_string());
//!
//! assert_eq!(request.method, "GET");
//! assert_eq!(request.path, "/api/users");
//! assert!(request.headers.is_empty());
//! assert!(request.body.is_empty());
//!
//! // Create a POST request with query and path params
//! let mut post_request = HttpRequest::new("POST".to_string(), "/api/users/{id}".to_string());
//! post_request.path_params.insert("id".to_string(), "123".to_string());
//! post_request.query_params.insert("format".to_string(), "json".to_string());
//! post_request.body = b"{\"name\":\"John\"}".to_vec();
//!
//! assert_eq!(post_request.method, "POST");
//! assert_eq!(post_request.path_params.get("id").unwrap(), "123");
//! ```
//!
//! ## HTTP Response Builder
//!
//! ```
//! use armature_core::HttpResponse;
//!
//! // Create a successful response
//! let response = HttpResponse::ok()
//!     .with_header("Content-Type".to_string(), "application/json".to_string())
//!     .with_body(b"{\"status\":\"ok\"}".to_vec());
//!
//! assert_eq!(response.status, 200);
//! assert!(response.headers.contains_key("Content-Type"));
//!
//! // Create an error response
//! let not_found = HttpResponse::not_found()
//!     .with_body(b"Resource not found".to_vec());
//!
//! assert_eq!(not_found.status, 404);
//! ```

pub mod application;
pub mod container;
pub mod error;
pub mod form;
pub mod guard;
pub mod http;
pub mod interceptor;
pub mod middleware;
pub mod routing;
pub mod sse;
pub mod static_assets;
pub mod status;
pub mod tls;
pub mod traits;
pub mod websocket;

// Re-export commonly used types
pub use application::*;
pub use container::*;
pub use error::*;
pub use form::*;
pub use guard::*;
pub use http::*;
pub use interceptor::*;
pub use middleware::*;
pub use routing::{Route, Router}; // Explicit exports to avoid ambiguous HandlerFn
pub use sse::*;
pub use static_assets::*;
pub use status::*;
pub use tls::*;
pub use traits::*;
pub use websocket::*;
