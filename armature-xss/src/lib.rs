//! # Armature XSS Protection
//!
//! Cross-Site Scripting (XSS) protection for Armature applications.
//!
//! ## Features
//!
//! - ✅ **HTML Sanitization** - Remove dangerous HTML/JavaScript
//! - ✅ **Content Encoding** - HTML, JavaScript, URL, CSS encoding
//! - ✅ **Pattern Validation** - Detect XSS attack patterns
//! - ✅ **Middleware Integration** - Automatic request validation
//! - ✅ **Configurable** - Strict, default, or permissive modes
//! - ✅ **Protection Headers** - X-XSS-Protection, X-Content-Type-Options
//!
//! ## Quick Start
//!
//! ```rust
//! use armature_xss::{XssMiddleware, XssConfig, XssSanitizer, XssEncoder};
//!
//! // Create middleware
//! let xss = XssMiddleware::new(XssConfig::default());
//!
//! // Sanitize HTML
//! let sanitizer = XssSanitizer::new();
//! let clean = sanitizer.sanitize("<script>alert('XSS')</script>").unwrap();
//!
//! // Encode for HTML context
//! let encoded = XssEncoder::encode_html("<script>alert('XSS')</script>");
//! ```
//!
//! ## Usage with Armature
//!
//! ```ignore
//! use armature::prelude::*;
//! use armature_xss::{XssMiddleware, XssConfig};
//!
//! #[controller("/api")]
//! struct ApiController {
//!     xss: XssMiddleware,
//! }
//!
//! impl ApiController {
//!     fn new() -> Self {
//!         Self {
//!             xss: XssMiddleware::new(XssConfig::default()),
//!         }
//!     }
//!
//!     #[post("/submit")]
//!     async fn submit(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
//!         // Validate request for XSS
//!         self.xss.validate_request(&req)?;
//!
//!         // Process request
//!         let mut response = HttpResponse::ok();
//!
//!         // Add XSS protection headers
//!         response = self.xss.add_protection_headers(response);
//!
//!         Ok(response)
//!     }
//! }
//! ```
//!
//! ## Sanitization
//!
//! ```rust
//! use armature_xss::XssSanitizer;
//!
//! // Default sanitizer (balanced)
//! let sanitizer = XssSanitizer::new();
//! let clean = sanitizer.sanitize(r#"<p>Hello</p><script>alert('XSS')</script>"#).unwrap();
//! assert!(clean.contains("<p>"));
//! assert!(!clean.contains("<script>"));
//!
//! // Strict sanitizer (minimal HTML)
//! let strict = XssSanitizer::strict();
//!
//! // Permissive sanitizer (more HTML allowed)
//! let permissive = XssSanitizer::permissive();
//! ```
//!
//! ## Encoding
//!
//! ```rust
//! use armature_xss::XssEncoder;
//!
//! // HTML context
//! let html = XssEncoder::encode_html("<script>alert('XSS')</script>");
//!
//! // JavaScript context
//! let js = XssEncoder::encode_javascript("'; alert('XSS'); //");
//!
//! // URL context
//! let url = XssEncoder::encode_url("hello world&test=value");
//!
//! // CSS context
//! let css = XssEncoder::encode_css("expression(alert('XSS'))");
//! ```
//!
//! ## HTML Entity Encoding
//!
//! ```rust
//! use armature_xss::XssEncoder;
//!
//! // Encode dangerous characters for HTML
//! let input = "<script>alert('XSS')</script>";
//! let encoded = XssEncoder::encode_html(input);
//!
//! // Verify encoding
//! assert!(encoded.contains("&lt;"));
//! assert!(encoded.contains("&gt;"));
//! assert!(!encoded.contains("<script>"));
//! assert!(!encoded.contains("</script>"));
//! ```
//!
//! ## XSS Pattern Detection
//!
//! ```rust
//! use armature_xss::XssValidator;
//!
//! // Detect script tags
//! assert!(XssValidator::contains_xss("<script>alert('XSS')</script>"));
//!
//! // Detect event handlers
//! assert!(XssValidator::contains_xss("<img src=x onerror=alert('XSS')>"));
//!
//! // Detect javascript: URLs
//! assert!(XssValidator::contains_xss("<a href='javascript:alert(1)'>Click</a>"));
//!
//! // Safe content should pass
//! assert!(!XssValidator::contains_xss("<p>Hello World</p>"));
//! ```

pub mod encoder;
pub mod error;
pub mod middleware;
pub mod sanitizer;
pub mod validator;

pub use encoder::XssEncoder;
pub use error::{Result, XssError};
pub use middleware::{XssConfig, XssMiddleware};
pub use sanitizer::XssSanitizer;
pub use validator::XssValidator;

