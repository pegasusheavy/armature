//! # Armature Core
//!
//! Core library for the Armature HTTP framework - a modern, type-safe web framework for Rust
//! inspired by Angular and NestJS.
//!
//! This crate provides the foundational types, traits, and runtime components
//! for building web applications with Armature.
//!
//! ## Features
//!
//! - **HTTP Handling**: Request/Response types with fluent builders
//! - **Routing**: Path parameters, query strings, and constraints
//! - **Dependency Injection**: Type-safe DI container
//! - **Middleware**: Composable request/response processing
//! - **Guards**: Authentication and authorization
//! - **Resilience**: Circuit breakers, retries, bulkheads, and timeouts
//! - **Logging**: Structured logging with tracing integration
//! - **Health Checks**: Readiness and liveness probes
//! - **WebSocket & SSE**: Real-time communication support
//!
//! ## Quick Start
//!
//! ### HTTP Request Handling
//!
//! ```
//! use armature_core::HttpRequest;
//!
//! // Create an HTTP request
//! let request = HttpRequest::new("GET".to_string(), "/api/users".to_string());
//!
//! assert_eq!(request.method, "GET");
//! assert_eq!(request.path, "/api/users");
//!
//! // Access path and query parameters
//! let mut post = HttpRequest::new("POST".to_string(), "/api/users/123".to_string());
//! post.path_params.insert("id".to_string(), "123".to_string());
//! post.query_params.insert("format".to_string(), "json".to_string());
//! post.body = b"{\"name\":\"John\"}".to_vec();
//!
//! assert_eq!(post.param("id"), Some(&"123".to_string()));
//! assert_eq!(post.query("format"), Some(&"json".to_string()));
//! ```
//!
//! ### HTTP Response Builder
//!
//! ```
//! use armature_core::HttpResponse;
//! use serde_json::json;
//!
//! // JSON response (shorthand)
//! let response = HttpResponse::json(&json!({"message": "Hello"})).unwrap();
//! assert_eq!(response.status, 200);
//!
//! // HTML response
//! let html = HttpResponse::html("<h1>Welcome</h1>");
//! assert_eq!(html.status, 200);
//!
//! // Redirect
//! let redirect = HttpResponse::redirect("https://example.com");
//! assert_eq!(redirect.status, 302);
//!
//! // With fluent builder
//! let custom = HttpResponse::ok()
//!     .content_type("application/xml")
//!     .cache_control("max-age=3600")
//!     .with_body(b"<xml/>".to_vec());
//! ```
//!
//! ### Dependency Injection
//!
//! ```
//! use armature_core::Container;
//!
//! #[derive(Clone, Default)]
//! struct Config { debug: bool }
//!
//! #[derive(Clone)]
//! struct UserService { config: std::sync::Arc<Config> }
//!
//! let container = Container::new();
//!
//! // Register services
//! container.register(Config { debug: true });
//!
//! // Resolve services
//! let config = container.require::<Config>();
//! assert!(config.debug);
//!
//! // Get or use default
//! let config2 = container.get_or_default::<Config>();
//! ```
//!
//! ### Error Handling
//!
//! ```
//! use armature_core::Error;
//!
//! // Create errors with convenience methods
//! let err = Error::not_found("User not found");
//! assert_eq!(err.status_code(), 404);
//! assert!(err.is_client_error());
//!
//! let err = Error::validation("Email is required");
//! assert_eq!(err.status_code(), 400);
//!
//! // Get help suggestions
//! let err = Error::unauthorized("Invalid token");
//! if let Some(help) = err.help() {
//!     println!("Help: {}", help);
//! }
//! ```
//!
//! ## Module Overview
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`application`] | Application bootstrap and lifecycle |
//! | [`container`] | Dependency injection container |
//! | [`routing`] | Request routing and handlers |
//! | [`middleware`] | Middleware chain processing |
//! | [`guard`] | Route guards for authorization |
//! | [`resilience`] | Circuit breaker, retry, bulkhead patterns |
//! | [`health`] | Health check endpoints |
//! | [`logging`] | Structured logging |
//! | [`websocket`] | WebSocket support |
//! | [`sse`] | Server-Sent Events |

pub mod application;
pub mod arena;
pub mod body;
pub mod body_limits;
pub mod container;
pub mod error;
pub mod extensions;
pub mod extractors;
pub mod form;
pub mod guard;
pub mod handler;
pub mod health;
pub mod json;
pub mod simd_parser;
pub mod hmr;
pub mod http;
pub mod interceptor;
pub mod lifecycle;
pub mod logging;
pub mod middleware;
pub mod module;
pub mod pipeline;
pub mod pagination;
pub mod resilience;
pub mod route_constraint;
pub mod route_group;
pub mod route_registry;
pub mod routing;
pub mod shutdown;
pub mod sse;
pub mod static_assets;
pub mod status;
pub mod timeout;
pub mod tls;
pub mod traits;
pub mod websocket;

// Re-export commonly used types
pub use application::*;
pub use body_limits::*;
pub use container::*;
pub use error::*;
pub use extensions::Extensions;
pub use extractors::{
    Body, ContentType, Form, FromRequest, FromRequestNamed, Header, Headers, Method, Path,
    PathParams, Query, RawBody, State,
};
pub use form::*;
pub use guard::*;
pub use handler::{BoxedHandler, Handler, IntoHandler, OptimizedHandlerFn};
pub use health::*;
pub use hmr::*;
pub use http::*;
pub use interceptor::*;
pub use lifecycle::*;
pub use logging::*;
pub use middleware::*;
pub use module::*;
pub use pagination::*;
pub use resilience::{
    BackoffStrategy, Bulkhead, BulkheadConfig, BulkheadError, BulkheadStats, CircuitBreaker,
    CircuitBreakerConfig, CircuitBreakerError, CircuitBreakerStats, CircuitState, Fallback,
    FallbackBuilder, FallbackChain, Retry, RetryConfig, RetryError, Timeout as ResilienceTimeout,
    TimeoutConfig, TimeoutError, fallback_default, fallback_value,
};
pub use route_constraint::*;
pub use route_group::*;
pub use route_registry::{OptimizedRouteHandler, RouteEntry, RouteHandlerFn};
pub use routing::{OptimizedHandler, Route, Router}; // Explicit exports to avoid ambiguous HandlerFn
pub use shutdown::*;
pub use sse::*;
pub use static_assets::*;
pub use status::*;
pub use timeout::*;
pub use tls::*;
pub use traits::*;
pub use websocket::*;

// Re-export inventory for route registration macros
pub use inventory;
