//! # Armature CSRF Protection
//!
//! Cross-Site Request Forgery (CSRF) protection for Armature applications.
//!
//! ## Features
//!
//! - ✅ **Token-based Protection** - Synchronizer token pattern
//! - ✅ **Signed Tokens** - HMAC-SHA256 signed tokens
//! - ✅ **Configurable** - Cookie names, headers, expiration
//! - ✅ **Middleware Integration** - Easy integration with Armature
//! - ✅ **Session Binding** - Optional session-specific tokens
//! - ✅ **Path Exclusion** - Exclude specific paths from protection
//!
//! ## Quick Start
//!
//! ```rust
//! use armature_csrf::{CsrfConfig, CsrfMiddleware};
//!
//! // Create configuration with generated secret
//! let config = CsrfConfig::default();
//!
//! // Or with custom secret
//! let secret = CsrfConfig::generate_secret();
//! let config = CsrfConfig::new(secret).unwrap()
//!     .with_token_ttl(3600)
//!     .with_cookie_secure(true);
//!
//! // Create middleware
//! let csrf = CsrfMiddleware::new(config);
//! ```
//!
//! ## Usage with Armature
//!
//! ```ignore
//! use armature::prelude::*;
//! use armature_csrf::{CsrfConfig, CsrfMiddleware};
//!
//! #[controller("/api")]
//! struct ApiController {
//!     csrf: CsrfMiddleware,
//! }
//!
//! impl ApiController {
//!     fn new() -> Self {
//!         Self {
//!             csrf: CsrfMiddleware::new(CsrfConfig::default()),
//!         }
//!     }
//!
//!     #[post("/submit")]
//!     async fn submit(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
//!         // Validate CSRF token
//!         self.csrf.validate_request(&req)?;
//!
//!         // Process request
//!         Ok(HttpResponse::ok())
//!     }
//!
//!     #[get("/form")]
//!     async fn form(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
//!         // Generate token for form
//!         let token = self.csrf.generate_token().unwrap();
//!         let mut response = HttpResponse::ok();
//!
//!         // Add token as cookie
//!         response = self.csrf.add_token_cookie(response, &token).unwrap();
//!
//!         Ok(response)
//!     }
//! }
//! ```

pub mod config;
pub mod error;
pub mod middleware;
pub mod token;

pub use config::{CsrfConfig, SameSite};
pub use error::{CsrfError, Result};
pub use middleware::CsrfMiddleware;
pub use token::CsrfToken;

