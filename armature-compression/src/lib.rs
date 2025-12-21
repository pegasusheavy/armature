//! HTTP Response Compression Middleware for Armature
//!
//! This crate provides middleware for compressing HTTP responses using various
//! compression algorithms including gzip, brotli, and zstd.
//!
//! # Features
//!
//! - `gzip` - Enable gzip compression (enabled by default)
//! - `brotli` - Enable brotli compression (enabled by default)
//! - `zstd` - Enable zstd compression
//! - `full` - Enable all compression algorithms
//!
//! # Example
//!
//! ```rust,no_run
//! use armature_compression::{CompressionMiddleware, CompressionConfig, CompressionAlgorithm};
//!
//! // Create middleware with default settings (auto-select best algorithm)
//! let middleware = CompressionMiddleware::new();
//!
//! // Or with specific configuration
//! let config = CompressionConfig::builder()
//!     .algorithm(CompressionAlgorithm::Brotli)
//!     .min_size(1024)  // Only compress responses > 1KB
//!     .level(6)        // Compression level (1-9 for most algorithms)
//!     .build();
//! let middleware = CompressionMiddleware::with_config(config);
//! ```
//!
//! # Compression Algorithm Selection
//!
//! When using `CompressionAlgorithm::Auto`, the middleware will select the best
//! compression algorithm based on the client's `Accept-Encoding` header:
//!
//! 1. **Brotli** (`br`) - Best compression ratio, preferred for text content
//! 2. **Zstd** (`zstd`) - Fast compression with good ratios
//! 3. **Gzip** (`gzip`) - Most widely supported, good fallback
//!
//! # Content Types
//!
//! By default, the middleware compresses the following content types:
//!
//! - `text/*` (HTML, CSS, JavaScript, plain text, etc.)
//! - `application/json`
//! - `application/javascript`
//! - `application/xml`
//! - `image/svg+xml`
//!
//! Binary content types like images, videos, and already-compressed files
//! are not compressed by default.

mod algorithm;
mod config;
mod error;
mod middleware;
pub mod streaming;

pub use algorithm::CompressionAlgorithm;
pub use config::{CompressionConfig, CompressionConfigBuilder};
pub use error::CompressionError;
pub use middleware::CompressionMiddleware;
pub use streaming::{
    AsyncStreamingCompressor, CompressionStats, StreamingCompressor, StreamingConfig,
};

/// Result type for compression operations
pub type Result<T> = std::result::Result<T, CompressionError>;
