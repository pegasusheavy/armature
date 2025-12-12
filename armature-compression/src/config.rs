//! Configuration for compression middleware

use crate::CompressionAlgorithm;

/// Configuration for the compression middleware
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// The compression algorithm to use
    pub algorithm: CompressionAlgorithm,

    /// Compression level (algorithm-specific range)
    pub level: u32,

    /// Minimum response size in bytes to compress (default: 860 bytes)
    /// Responses smaller than this won't be compressed
    pub min_size: usize,

    /// Content types to compress (default: text/*, application/json, etc.)
    pub compressible_types: Vec<String>,

    /// Whether to compress responses that already have Content-Encoding
    pub compress_encoded: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Auto,
            level: 0,      // Will use algorithm's default
            min_size: 860, // Typical MTU threshold
            compressible_types: default_compressible_types(),
            compress_encoded: false,
        }
    }
}

impl CompressionConfig {
    /// Create a new configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder for configuration
    pub fn builder() -> CompressionConfigBuilder {
        CompressionConfigBuilder::new()
    }

    /// Get the effective compression level for the configured algorithm
    pub fn effective_level(&self) -> u32 {
        if self.level == 0 {
            self.algorithm.default_level()
        } else {
            self.level
                .clamp(self.algorithm.min_level(), self.algorithm.max_level())
        }
    }

    /// Check if a content type should be compressed
    pub fn should_compress_content_type(&self, content_type: &str) -> bool {
        let ct_lower = content_type.to_lowercase();
        let ct_base = ct_lower.split(';').next().unwrap_or(&ct_lower).trim();

        self.compressible_types.iter().any(|pattern| {
            if pattern.ends_with("/*") {
                // Wildcard pattern like "text/*"
                let prefix = &pattern[..pattern.len() - 1];
                ct_base.starts_with(prefix)
            } else {
                ct_base == pattern
            }
        })
    }

    /// Check if a response should be compressed based on size
    pub fn should_compress_size(&self, size: usize) -> bool {
        size >= self.min_size
    }
}

/// Builder for CompressionConfig
#[derive(Debug, Clone, Default)]
pub struct CompressionConfigBuilder {
    config: CompressionConfig,
}

impl CompressionConfigBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self {
        Self {
            config: CompressionConfig::default(),
        }
    }

    /// Set the compression algorithm
    pub fn algorithm(mut self, algorithm: CompressionAlgorithm) -> Self {
        self.config.algorithm = algorithm;
        self
    }

    /// Set the compression level
    pub fn level(mut self, level: u32) -> Self {
        self.config.level = level;
        self
    }

    /// Set the minimum response size to compress
    pub fn min_size(mut self, min_size: usize) -> Self {
        self.config.min_size = min_size;
        self
    }

    /// Set the compressible content types
    pub fn compressible_types(mut self, types: Vec<String>) -> Self {
        self.config.compressible_types = types;
        self
    }

    /// Add a compressible content type
    pub fn add_compressible_type(mut self, content_type: impl Into<String>) -> Self {
        self.config.compressible_types.push(content_type.into());
        self
    }

    /// Set whether to compress already-encoded responses
    pub fn compress_encoded(mut self, compress: bool) -> Self {
        self.config.compress_encoded = compress;
        self
    }

    /// Use gzip compression
    #[cfg(feature = "gzip")]
    pub fn gzip(mut self) -> Self {
        self.config.algorithm = CompressionAlgorithm::Gzip;
        self
    }

    /// Use brotli compression
    #[cfg(feature = "brotli")]
    pub fn brotli(mut self) -> Self {
        self.config.algorithm = CompressionAlgorithm::Brotli;
        self
    }

    /// Use zstd compression
    #[cfg(feature = "zstd")]
    pub fn zstd(mut self) -> Self {
        self.config.algorithm = CompressionAlgorithm::Zstd;
        self
    }

    /// Disable compression
    pub fn no_compression(mut self) -> Self {
        self.config.algorithm = CompressionAlgorithm::None;
        self
    }

    /// Build the configuration
    pub fn build(self) -> CompressionConfig {
        self.config
    }
}

/// Default content types that should be compressed
fn default_compressible_types() -> Vec<String> {
    vec![
        // Text types
        "text/*".to_string(),
        // JSON
        "application/json".to_string(),
        "application/ld+json".to_string(),
        // JavaScript
        "application/javascript".to_string(),
        "application/x-javascript".to_string(),
        // XML
        "application/xml".to_string(),
        "application/xhtml+xml".to_string(),
        "application/rss+xml".to_string(),
        "application/atom+xml".to_string(),
        // SVG
        "image/svg+xml".to_string(),
        // Fonts
        "font/ttf".to_string(),
        "font/otf".to_string(),
        "application/vnd.ms-fontobject".to_string(),
        // Other
        "application/wasm".to_string(),
        "application/manifest+json".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CompressionConfig::default();
        assert_eq!(config.algorithm, CompressionAlgorithm::Auto);
        assert_eq!(config.min_size, 860);
        assert!(!config.compress_encoded);
    }

    #[test]
    fn test_builder() {
        let config = CompressionConfig::builder().min_size(1024).level(6).build();

        assert_eq!(config.min_size, 1024);
        assert_eq!(config.level, 6);
    }

    #[test]
    fn test_should_compress_content_type() {
        let config = CompressionConfig::default();

        // Should compress
        assert!(config.should_compress_content_type("text/html"));
        assert!(config.should_compress_content_type("text/css"));
        assert!(config.should_compress_content_type("text/plain; charset=utf-8"));
        assert!(config.should_compress_content_type("application/json"));
        assert!(config.should_compress_content_type("application/javascript"));
        assert!(config.should_compress_content_type("image/svg+xml"));

        // Should not compress
        assert!(!config.should_compress_content_type("image/png"));
        assert!(!config.should_compress_content_type("image/jpeg"));
        assert!(!config.should_compress_content_type("video/mp4"));
        assert!(!config.should_compress_content_type("application/octet-stream"));
    }

    #[test]
    fn test_should_compress_size() {
        let config = CompressionConfig::builder().min_size(1024).build();

        assert!(!config.should_compress_size(100));
        assert!(!config.should_compress_size(1023));
        assert!(config.should_compress_size(1024));
        assert!(config.should_compress_size(10000));
    }

    #[cfg(feature = "gzip")]
    #[test]
    fn test_effective_level_gzip() {
        let config = CompressionConfig::builder().gzip().build();
        assert_eq!(config.effective_level(), 6); // Default for gzip

        let config = CompressionConfig::builder().gzip().level(9).build();
        assert_eq!(config.effective_level(), 9);

        // Test clamping
        let config = CompressionConfig::builder().gzip().level(100).build();
        assert_eq!(config.effective_level(), 9); // Max for gzip
    }

    #[cfg(feature = "brotli")]
    #[test]
    fn test_builder_brotli() {
        let config = CompressionConfig::builder().brotli().build();
        assert_eq!(config.algorithm, CompressionAlgorithm::Brotli);
    }
}
