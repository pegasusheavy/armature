//! OpenAPI 3.0 specification generation and Swagger UI integration for Armature
//!
//! This crate provides tools for generating OpenAPI specifications and serving
//! interactive API documentation via Swagger UI.
//!
//! # Features
//!
//! - **Programmatic API**: Build OpenAPI specs with a fluent builder API
//! - **Swagger UI**: Serve interactive API documentation
//! - **JSON/YAML Export**: Export specifications in multiple formats
//! - **Type-Safe**: Strongly typed OpenAPI 3.0 specification
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use armature_openapi::*;
//!
//! let spec = OpenApiBuilder::new("My API", "1.0.0")
//!     .description("A wonderful API")
//!     .server("http://localhost:3000", None)
//!     .add_bearer_auth("bearer")
//!     .build();
//!
//! let config = SwaggerConfig::new("/api-docs", spec);
//! ```

pub mod builder;
pub mod spec;
pub mod swagger;

pub use builder::*;
pub use spec::*;
pub use swagger::*;
