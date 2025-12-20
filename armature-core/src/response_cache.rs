//! HTTP Response Caching
//!
//! This module provides comprehensive HTTP response caching support including:
//!
//! - `Cache-Control` header parsing and generation
//! - In-memory response caching with TTL
//! - Cache key generation from requests
//! - Vary header support
//! - Cache invalidation
//!
//! # Examples
//!
//! ## Cache-Control Headers
//!
//! ```
//! use armature_core::response_cache::{CacheControl, CacheDirective};
//! use std::time::Duration;
//!
//! // Create Cache-Control header
//! let cache_control = CacheControl::new()
//!     .public()
//!     .max_age(Duration::from_secs(3600))
//!     .must_revalidate();
//!
//! assert_eq!(cache_control.to_header_value(), "public, max-age=3600, must-revalidate");
//! ```
//!
//! ## Response Caching
//!
//! ```ignore
//! use armature_core::response_cache::{ResponseCache, CacheControl};
//!
//! let cache = ResponseCache::new();
//!
//! // Cache a response
//! cache.store(&request, &response).await;
//!
//! // Retrieve cached response
//! if let Some(cached) = cache.get(&request).await {
//!     return Ok(cached);
//! }
//! ```

use crate::{HttpRequest, HttpResponse};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;

// ============================================================================
// Cache Directives
// ============================================================================

/// Individual cache directive from Cache-Control header.
#[derive(Debug, Clone, PartialEq)]
pub enum CacheDirective {
    /// Response may be cached by any cache
    Public,
    /// Response is for a single user and must not be stored by shared caches
    Private,
    /// Response must not be stored in any cache
    NoStore,
    /// Response can be stored but must be validated before use
    NoCache,
    /// Maximum time the response is fresh (in seconds)
    MaxAge(u64),
    /// Maximum time a shared cache may store the response (in seconds)
    SMaxAge(u64),
    /// Response must be revalidated after becoming stale
    MustRevalidate,
    /// Shared caches must revalidate after becoming stale
    ProxyRevalidate,
    /// Response must not be transformed (e.g., compressed)
    NoTransform,
    /// Response is immutable and won't change
    Immutable,
    /// Client will accept stale response up to N seconds
    MaxStale(Option<u64>),
    /// Client wants response fresh for at least N seconds
    MinFresh(u64),
    /// Client will only accept cached response
    OnlyIfCached,
    /// Custom/unknown directive
    Extension(String, Option<String>),
}

impl CacheDirective {
    /// Parse a single directive from a string.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();

        // Check for directives with values
        if let Some((key, value)) = s.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');

            return match key {
                "max-age" => value.parse().ok().map(CacheDirective::MaxAge),
                "s-maxage" => value.parse().ok().map(CacheDirective::SMaxAge),
                "max-stale" => Some(CacheDirective::MaxStale(value.parse().ok())),
                "min-fresh" => value.parse().ok().map(CacheDirective::MinFresh),
                _ => Some(CacheDirective::Extension(
                    key.to_string(),
                    Some(value.to_string()),
                )),
            };
        }

        // Simple directives
        match s.as_str() {
            "public" => Some(CacheDirective::Public),
            "private" => Some(CacheDirective::Private),
            "no-store" => Some(CacheDirective::NoStore),
            "no-cache" => Some(CacheDirective::NoCache),
            "must-revalidate" => Some(CacheDirective::MustRevalidate),
            "proxy-revalidate" => Some(CacheDirective::ProxyRevalidate),
            "no-transform" => Some(CacheDirective::NoTransform),
            "immutable" => Some(CacheDirective::Immutable),
            "max-stale" => Some(CacheDirective::MaxStale(None)),
            "only-if-cached" => Some(CacheDirective::OnlyIfCached),
            _ => Some(CacheDirective::Extension(s, None)),
        }
    }

    /// Convert directive to header value string.
    pub fn to_header_value(&self) -> String {
        match self {
            CacheDirective::Public => "public".to_string(),
            CacheDirective::Private => "private".to_string(),
            CacheDirective::NoStore => "no-store".to_string(),
            CacheDirective::NoCache => "no-cache".to_string(),
            CacheDirective::MaxAge(secs) => format!("max-age={}", secs),
            CacheDirective::SMaxAge(secs) => format!("s-maxage={}", secs),
            CacheDirective::MustRevalidate => "must-revalidate".to_string(),
            CacheDirective::ProxyRevalidate => "proxy-revalidate".to_string(),
            CacheDirective::NoTransform => "no-transform".to_string(),
            CacheDirective::Immutable => "immutable".to_string(),
            CacheDirective::MaxStale(Some(secs)) => format!("max-stale={}", secs),
            CacheDirective::MaxStale(None) => "max-stale".to_string(),
            CacheDirective::MinFresh(secs) => format!("min-fresh={}", secs),
            CacheDirective::OnlyIfCached => "only-if-cached".to_string(),
            CacheDirective::Extension(key, Some(value)) => format!("{}={}", key, value),
            CacheDirective::Extension(key, None) => key.clone(),
        }
    }
}

// ============================================================================
// Cache-Control Header
// ============================================================================

/// Parsed or constructed Cache-Control header.
///
/// # Examples
///
/// ## Parsing
///
/// ```
/// use armature_core::response_cache::CacheControl;
///
/// let cc = CacheControl::parse("public, max-age=3600, must-revalidate");
/// assert!(cc.is_public());
/// assert_eq!(cc.max_age(), Some(3600));
/// ```
///
/// ## Building
///
/// ```
/// use armature_core::response_cache::CacheControl;
/// use std::time::Duration;
///
/// let cc = CacheControl::new()
///     .private()
///     .max_age(Duration::from_secs(300))
///     .no_transform();
///
/// assert!(cc.is_private());
/// ```
#[derive(Debug, Clone, Default)]
pub struct CacheControl {
    /// All directives in this Cache-Control header
    pub directives: Vec<CacheDirective>,
}

impl CacheControl {
    /// Create a new empty Cache-Control.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a Cache-Control header value.
    pub fn parse(header: &str) -> Self {
        let directives: Vec<CacheDirective> = header
            .split(',')
            .filter_map(|s| CacheDirective::parse(s.trim()))
            .collect();

        Self { directives }
    }

    /// Convert to header value string.
    pub fn to_header_value(&self) -> String {
        self.directives
            .iter()
            .map(|d| d.to_header_value())
            .collect::<Vec<_>>()
            .join(", ")
    }

    // ==================== Builder Methods ====================

    /// Add the `public` directive.
    pub fn public(mut self) -> Self {
        self.directives.push(CacheDirective::Public);
        self
    }

    /// Add the `private` directive.
    pub fn private(mut self) -> Self {
        self.directives.push(CacheDirective::Private);
        self
    }

    /// Add the `no-store` directive.
    pub fn no_store(mut self) -> Self {
        self.directives.push(CacheDirective::NoStore);
        self
    }

    /// Add the `no-cache` directive.
    pub fn no_cache(mut self) -> Self {
        self.directives.push(CacheDirective::NoCache);
        self
    }

    /// Add the `max-age` directive.
    pub fn max_age(mut self, duration: Duration) -> Self {
        self.directives
            .push(CacheDirective::MaxAge(duration.as_secs()));
        self
    }

    /// Add the `s-maxage` directive for shared caches.
    pub fn s_maxage(mut self, duration: Duration) -> Self {
        self.directives
            .push(CacheDirective::SMaxAge(duration.as_secs()));
        self
    }

    /// Add the `must-revalidate` directive.
    pub fn must_revalidate(mut self) -> Self {
        self.directives.push(CacheDirective::MustRevalidate);
        self
    }

    /// Add the `proxy-revalidate` directive.
    pub fn proxy_revalidate(mut self) -> Self {
        self.directives.push(CacheDirective::ProxyRevalidate);
        self
    }

    /// Add the `no-transform` directive.
    pub fn no_transform(mut self) -> Self {
        self.directives.push(CacheDirective::NoTransform);
        self
    }

    /// Add the `immutable` directive.
    pub fn immutable(mut self) -> Self {
        self.directives.push(CacheDirective::Immutable);
        self
    }

    /// Add a custom directive.
    pub fn directive(mut self, directive: CacheDirective) -> Self {
        self.directives.push(directive);
        self
    }

    // ==================== Query Methods ====================

    /// Check if `public` directive is present.
    pub fn is_public(&self) -> bool {
        self.directives
            .iter()
            .any(|d| matches!(d, CacheDirective::Public))
    }

    /// Check if `private` directive is present.
    pub fn is_private(&self) -> bool {
        self.directives
            .iter()
            .any(|d| matches!(d, CacheDirective::Private))
    }

    /// Check if `no-store` directive is present.
    pub fn is_no_store(&self) -> bool {
        self.directives
            .iter()
            .any(|d| matches!(d, CacheDirective::NoStore))
    }

    /// Check if `no-cache` directive is present.
    pub fn is_no_cache(&self) -> bool {
        self.directives
            .iter()
            .any(|d| matches!(d, CacheDirective::NoCache))
    }

    /// Check if `must-revalidate` directive is present.
    pub fn is_must_revalidate(&self) -> bool {
        self.directives
            .iter()
            .any(|d| matches!(d, CacheDirective::MustRevalidate))
    }

    /// Check if `immutable` directive is present.
    pub fn is_immutable(&self) -> bool {
        self.directives
            .iter()
            .any(|d| matches!(d, CacheDirective::Immutable))
    }

    /// Get the `max-age` value in seconds.
    pub fn get_max_age(&self) -> Option<u64> {
        self.directives.iter().find_map(|d| match d {
            CacheDirective::MaxAge(secs) => Some(*secs),
            _ => None,
        })
    }

    /// Get the `s-maxage` value in seconds.
    pub fn get_s_maxage(&self) -> Option<u64> {
        self.directives.iter().find_map(|d| match d {
            CacheDirective::SMaxAge(secs) => Some(*secs),
            _ => None,
        })
    }

    /// Check if the response is cacheable.
    pub fn is_cacheable(&self) -> bool {
        // Not cacheable if no-store is present
        if self.is_no_store() {
            return false;
        }

        // Cacheable if public, private, or has max-age/s-maxage
        self.is_public() || self.is_private() || self.get_max_age().is_some() || self.get_s_maxage().is_some()
    }

    /// Get the freshness lifetime in seconds.
    ///
    /// Returns s-maxage if present (for shared caches), otherwise max-age.
    pub fn freshness_lifetime(&self) -> Option<u64> {
        self.get_s_maxage().or_else(|| self.get_max_age())
    }

    // ==================== Preset Configurations ====================

    /// Create a "no-store" Cache-Control (never cache).
    pub fn never() -> Self {
        Self::new().no_store().no_cache()
    }

    /// Create a public cache with the given max-age.
    pub fn public_max_age(duration: Duration) -> Self {
        Self::new().public().max_age(duration)
    }

    /// Create a private cache with the given max-age.
    pub fn private_max_age(duration: Duration) -> Self {
        Self::new().private().max_age(duration)
    }

    /// Create an immutable public cache (for versioned assets).
    pub fn immutable_asset(duration: Duration) -> Self {
        Self::new()
            .public()
            .max_age(duration)
            .immutable()
    }

    /// Create a must-revalidate cache.
    pub fn revalidate(duration: Duration) -> Self {
        Self::new()
            .public()
            .max_age(duration)
            .must_revalidate()
    }
}

impl fmt::Display for CacheControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_header_value())
    }
}

// ============================================================================
// Cache Key
// ============================================================================

/// Cache key for HTTP responses.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// HTTP method
    pub method: String,
    /// Request path
    pub path: String,
    /// Query string (sorted for consistency)
    pub query: String,
    /// Vary header values that affect caching
    pub vary_values: Vec<(String, String)>,
}

impl CacheKey {
    /// Generate a cache key from a request.
    pub fn from_request(request: &HttpRequest) -> Self {
        Self::from_request_with_vary(request, &[])
    }

    /// Generate a cache key from a request with Vary headers.
    pub fn from_request_with_vary(request: &HttpRequest, vary_headers: &[&str]) -> Self {
        // Sort query params for consistent keys
        let mut query_params: Vec<_> = request.query_params.iter().collect();
        query_params.sort_by(|a, b| a.0.cmp(b.0));
        let query = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        // Collect Vary header values
        let mut vary_values: Vec<(String, String)> = vary_headers
            .iter()
            .filter_map(|header| {
                request
                    .headers
                    .get(*header)
                    .or_else(|| request.headers.get(&header.to_lowercase()))
                    .map(|v| (header.to_lowercase(), v.clone()))
            })
            .collect();
        vary_values.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            method: request.method.to_uppercase(),
            path: request.path.clone(),
            query,
            vary_values,
        }
    }

    /// Convert to a string representation suitable for use as a cache key.
    pub fn to_string_key(&self) -> String {
        let vary_str = if self.vary_values.is_empty() {
            String::new()
        } else {
            format!(
                "|{}",
                self.vary_values
                    .iter()
                    .map(|(k, v)| format!("{}:{}", k, v))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };

        if self.query.is_empty() {
            format!("{}:{}{}", self.method, self.path, vary_str)
        } else {
            format!("{}:{}?{}{}", self.method, self.path, self.query, vary_str)
        }
    }
}

impl fmt::Display for CacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_key())
    }
}

// ============================================================================
// Cached Response
// ============================================================================

/// A cached HTTP response with metadata.
#[derive(Debug, Clone)]
pub struct CachedResponse {
    /// The cached response
    pub response: CachedResponseData,
    /// When the response was cached
    pub cached_at: Instant,
    /// When the response expires
    pub expires_at: Instant,
    /// ETag of the cached response
    pub etag: Option<String>,
    /// Last-Modified timestamp
    pub last_modified: Option<SystemTime>,
    /// Vary headers that affect this cache entry
    pub vary: Vec<String>,
}

/// The actual cached response data.
#[derive(Debug, Clone)]
pub struct CachedResponseData {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Vec<u8>,
}

impl CachedResponse {
    /// Create a new cached response.
    pub fn new(response: &HttpResponse, ttl: Duration) -> Self {
        let now = Instant::now();

        let etag = response.headers.get("ETag").cloned();
        let last_modified = response
            .headers
            .get("Last-Modified")
            .and_then(|s| httpdate::parse_http_date(s).ok());
        let vary = response
            .headers
            .get("Vary")
            .map(|v| v.split(',').map(|s| s.trim().to_lowercase()).collect())
            .unwrap_or_default();

        Self {
            response: CachedResponseData {
                status: response.status,
                headers: response.headers.clone(),
                body: response.body.clone(),
            },
            cached_at: now,
            expires_at: now + ttl,
            etag,
            last_modified,
            vary,
        }
    }

    /// Check if the cached response is still fresh.
    pub fn is_fresh(&self) -> bool {
        Instant::now() < self.expires_at
    }

    /// Check if the cached response is stale.
    pub fn is_stale(&self) -> bool {
        !self.is_fresh()
    }

    /// Get the age of the cached response.
    pub fn age(&self) -> Duration {
        self.cached_at.elapsed()
    }

    /// Get the remaining TTL.
    pub fn remaining_ttl(&self) -> Option<Duration> {
        let now = Instant::now();
        if now < self.expires_at {
            Some(self.expires_at - now)
        } else {
            None
        }
    }

    /// Convert to an HttpResponse.
    pub fn to_response(&self) -> HttpResponse {
        let mut response = HttpResponse::from_parts(
            self.response.status,
            self.response.headers.clone(),
            self.response.body.clone(),
        );

        // Add Age header
        response.headers.insert(
            "Age".to_string(),
            self.age().as_secs().to_string(),
        );

        // Add X-Cache header
        response
            .headers
            .insert("X-Cache".to_string(), "HIT".to_string());

        response
    }
}

// ============================================================================
// In-Memory Response Cache
// ============================================================================

/// In-memory HTTP response cache.
///
/// # Examples
///
/// ```
/// use armature_core::response_cache::ResponseCache;
/// use armature_core::HttpRequest;
/// use std::time::Duration;
///
/// # tokio_test::block_on(async {
/// let cache = ResponseCache::new();
///
/// // Configure cache
/// let cache = ResponseCache::with_config(ResponseCacheConfig {
///     max_entries: 1000,
///     default_ttl: Duration::from_secs(300),
///     max_body_size: 1024 * 1024,  // 1MB
/// });
/// # });
/// ```
#[derive(Debug)]
pub struct ResponseCache {
    /// Cache configuration
    config: ResponseCacheConfig,
    /// Cached responses
    entries: Arc<RwLock<HashMap<String, CachedResponse>>>,
}

/// Configuration for the response cache.
#[derive(Debug, Clone)]
pub struct ResponseCacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,
    /// Default TTL for cached responses
    pub default_ttl: Duration,
    /// Maximum body size to cache (in bytes)
    pub max_body_size: usize,
    /// Only cache responses with these status codes
    pub cacheable_status_codes: Vec<u16>,
    /// Only cache these HTTP methods
    pub cacheable_methods: Vec<String>,
}

impl Default for ResponseCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl: Duration::from_secs(300), // 5 minutes
            max_body_size: 1024 * 1024,            // 1MB
            cacheable_status_codes: vec![200, 203, 204, 206, 300, 301, 404, 405, 410, 414, 501],
            cacheable_methods: vec!["GET".to_string(), "HEAD".to_string()],
        }
    }
}

impl ResponseCacheConfig {
    /// Create a new configuration with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of entries.
    pub fn max_entries(mut self, count: usize) -> Self {
        self.max_entries = count;
        self
    }

    /// Set the default TTL.
    pub fn default_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// Set the maximum body size to cache.
    pub fn max_body_size(mut self, size: usize) -> Self {
        self.max_body_size = size;
        self
    }
}

impl ResponseCache {
    /// Create a new response cache with default configuration.
    pub fn new() -> Self {
        Self::with_config(ResponseCacheConfig::default())
    }

    /// Create a new response cache with custom configuration.
    pub fn with_config(config: ResponseCacheConfig) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a cached response for a request.
    pub async fn get(&self, request: &HttpRequest) -> Option<HttpResponse> {
        self.get_with_vary(request, &[]).await
    }

    /// Get a cached response with Vary header support.
    pub async fn get_with_vary(
        &self,
        request: &HttpRequest,
        vary_headers: &[&str],
    ) -> Option<HttpResponse> {
        let key = CacheKey::from_request_with_vary(request, vary_headers);
        let key_str = key.to_string_key();

        let entries = self.entries.read().await;
        if let Some(cached) = entries.get(&key_str) {
            if cached.is_fresh() {
                return Some(cached.to_response());
            }
        }
        None
    }

    /// Store a response in the cache.
    pub async fn store(&self, request: &HttpRequest, response: &HttpResponse) {
        self.store_with_ttl(request, response, self.config.default_ttl)
            .await
    }

    /// Store a response with a specific TTL.
    pub async fn store_with_ttl(
        &self,
        request: &HttpRequest,
        response: &HttpResponse,
        ttl: Duration,
    ) {
        // Check if cacheable
        if !self.is_cacheable(request, response) {
            return;
        }

        // Get Vary headers from response
        let vary_headers: Vec<&str> = response
            .headers
            .get("Vary")
            .map(|v| v.split(',').map(|s| s.trim()).collect())
            .unwrap_or_default();

        let key = CacheKey::from_request_with_vary(request, &vary_headers);
        let key_str = key.to_string_key();
        let cached = CachedResponse::new(response, ttl);

        let mut entries = self.entries.write().await;

        // Evict if at capacity
        if entries.len() >= self.config.max_entries {
            self.evict_oldest(&mut entries);
        }

        entries.insert(key_str, cached);
    }

    /// Check if a request/response pair is cacheable.
    fn is_cacheable(&self, request: &HttpRequest, response: &HttpResponse) -> bool {
        // Check method
        if !self
            .config
            .cacheable_methods
            .contains(&request.method.to_uppercase())
        {
            return false;
        }

        // Check status code
        if !self
            .config
            .cacheable_status_codes
            .contains(&response.status)
        {
            return false;
        }

        // Check body size
        if response.body.len() > self.config.max_body_size {
            return false;
        }

        // Check Cache-Control
        if let Some(cc_header) = response.headers.get("Cache-Control") {
            let cc = CacheControl::parse(cc_header);
            if cc.is_no_store() {
                return false;
            }
        }

        true
    }

    /// Evict the oldest entry from the cache.
    fn evict_oldest(&self, entries: &mut HashMap<String, CachedResponse>) {
        // Find the oldest entry
        if let Some((oldest_key, _)) = entries
            .iter()
            .min_by_key(|(_, v)| v.cached_at)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            entries.remove(&oldest_key);
        }
    }

    /// Remove a specific entry from the cache.
    pub async fn invalidate(&self, request: &HttpRequest) {
        let key = CacheKey::from_request(request);
        let key_str = key.to_string_key();

        let mut entries = self.entries.write().await;
        entries.remove(&key_str);
    }

    /// Remove all entries matching a path prefix.
    pub async fn invalidate_prefix(&self, path_prefix: &str) {
        let mut entries = self.entries.write().await;
        entries.retain(|key, _| !key.contains(&format!(":{}", path_prefix)));
    }

    /// Clear all cached responses.
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }

    /// Remove all stale entries.
    pub async fn purge_stale(&self) {
        let mut entries = self.entries.write().await;
        entries.retain(|_, v| v.is_fresh());
    }

    /// Get cache statistics.
    pub async fn stats(&self) -> CacheStats {
        let entries = self.entries.read().await;
        let fresh_count = entries.values().filter(|e| e.is_fresh()).count();
        let stale_count = entries.len() - fresh_count;
        let total_size: usize = entries.values().map(|e| e.response.body.len()).sum();

        CacheStats {
            total_entries: entries.len(),
            fresh_entries: fresh_count,
            stale_entries: stale_count,
            total_size_bytes: total_size,
            max_entries: self.config.max_entries,
        }
    }
}

impl Default for ResponseCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of entries
    pub total_entries: usize,
    /// Number of fresh entries
    pub fresh_entries: usize,
    /// Number of stale entries
    pub stale_entries: usize,
    /// Total size of cached bodies in bytes
    pub total_size_bytes: usize,
    /// Maximum entries allowed
    pub max_entries: usize,
}

// ============================================================================
// Request/Response Extensions
// ============================================================================

/// Extension methods for HttpRequest related to caching.
impl HttpRequest {
    /// Get the Cache-Control header from the request.
    pub fn cache_control(&self) -> Option<CacheControl> {
        self.headers
            .get("Cache-Control")
            .or_else(|| self.headers.get("cache-control"))
            .map(|h| CacheControl::parse(h))
    }

    /// Check if the request allows cached responses.
    pub fn allows_cached(&self) -> bool {
        if let Some(cc) = self.cache_control() {
            // Check for no-cache or no-store
            !cc.is_no_cache() && !cc.is_no_store()
        } else {
            true
        }
    }

    /// Get the max-stale tolerance from the request.
    pub fn max_stale(&self) -> Option<u64> {
        self.cache_control().and_then(|cc| {
            cc.directives.iter().find_map(|d| match d {
                CacheDirective::MaxStale(secs) => Some(secs.unwrap_or(u64::MAX)),
                _ => None,
            })
        })
    }

    /// Generate a cache key for this request.
    pub fn cache_key(&self) -> CacheKey {
        CacheKey::from_request(self)
    }

    /// Generate a cache key with Vary headers.
    pub fn cache_key_with_vary(&self, vary_headers: &[&str]) -> CacheKey {
        CacheKey::from_request_with_vary(self, vary_headers)
    }
}

/// Extension methods for HttpResponse related to caching.
impl HttpResponse {
    /// Set the Cache-Control header.
    pub fn with_cache_control(mut self, cache_control: CacheControl) -> Self {
        self.headers
            .insert("Cache-Control".to_string(), cache_control.to_header_value());
        self
    }

    /// Set a "no-store" Cache-Control (never cache).
    pub fn no_cache(self) -> Self {
        self.with_cache_control(CacheControl::never())
    }

    /// Set a public cache with max-age.
    pub fn cache_public(self, max_age: Duration) -> Self {
        self.with_cache_control(CacheControl::public_max_age(max_age))
    }

    /// Set a private cache with max-age.
    pub fn cache_private(self, max_age: Duration) -> Self {
        self.with_cache_control(CacheControl::private_max_age(max_age))
    }

    /// Set cache for immutable assets.
    pub fn cache_immutable(self, max_age: Duration) -> Self {
        self.with_cache_control(CacheControl::immutable_asset(max_age))
    }

    /// Add Vary header.
    pub fn with_vary(mut self, headers: &[&str]) -> Self {
        let vary = headers.join(", ");
        self.headers.insert("Vary".to_string(), vary);
        self
    }

    /// Get the Cache-Control header from the response.
    pub fn get_cache_control(&self) -> Option<CacheControl> {
        self.headers
            .get("Cache-Control")
            .map(|h| CacheControl::parse(h))
    }

    /// Check if the response is cacheable based on Cache-Control.
    pub fn is_cacheable(&self) -> bool {
        if let Some(cc) = self.get_cache_control() {
            cc.is_cacheable()
        } else {
            // Default: only cache 200 OK without explicit Cache-Control
            self.status == 200
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_directive_parse() {
        assert_eq!(CacheDirective::parse("public"), Some(CacheDirective::Public));
        assert_eq!(CacheDirective::parse("private"), Some(CacheDirective::Private));
        assert_eq!(CacheDirective::parse("no-store"), Some(CacheDirective::NoStore));
        assert_eq!(CacheDirective::parse("max-age=3600"), Some(CacheDirective::MaxAge(3600)));
    }

    #[test]
    fn test_cache_control_parse() {
        let cc = CacheControl::parse("public, max-age=3600, must-revalidate");
        assert!(cc.is_public());
        assert_eq!(cc.get_max_age(), Some(3600));
        assert!(cc.is_must_revalidate());
    }

    #[test]
    fn test_cache_control_builder() {
        let cc = CacheControl::new()
            .public()
            .max_age(Duration::from_secs(3600))
            .must_revalidate();

        assert_eq!(cc.to_header_value(), "public, max-age=3600, must-revalidate");
    }

    #[test]
    fn test_cache_control_presets() {
        let never = CacheControl::never();
        assert!(never.is_no_store());
        assert!(never.is_no_cache());

        let public = CacheControl::public_max_age(Duration::from_secs(3600));
        assert!(public.is_public());
        assert_eq!(public.get_max_age(), Some(3600));

        let immutable = CacheControl::immutable_asset(Duration::from_secs(31536000));
        assert!(immutable.is_immutable());
    }

    #[test]
    fn test_cache_control_is_cacheable() {
        assert!(CacheControl::public_max_age(Duration::from_secs(3600)).is_cacheable());
        assert!(CacheControl::private_max_age(Duration::from_secs(3600)).is_cacheable());
        assert!(!CacheControl::never().is_cacheable());
    }

    #[test]
    fn test_cache_key_from_request() {
        let mut request = HttpRequest::new("GET".to_string(), "/api/users".to_string());
        request.query_params.insert("page".to_string(), "1".to_string());
        request.query_params.insert("limit".to_string(), "10".to_string());

        let key = CacheKey::from_request(&request);
        assert_eq!(key.method, "GET");
        assert_eq!(key.path, "/api/users");
        // Query params should be sorted
        assert!(key.query.contains("limit=10"));
        assert!(key.query.contains("page=1"));
    }

    #[test]
    fn test_cache_key_with_vary() {
        let mut request = HttpRequest::new("GET".to_string(), "/api/users".to_string());
        request.headers.insert("Accept".to_string(), "application/json".to_string());

        let key = CacheKey::from_request_with_vary(&request, &["Accept"]);
        assert_eq!(key.vary_values.len(), 1);
        assert_eq!(key.vary_values[0], ("accept".to_string(), "application/json".to_string()));
    }

    #[test]
    fn test_cached_response() {
        let mut response = HttpResponse::ok();
        response.body = b"Hello, World!".to_vec();
        response.headers.insert("ETag".to_string(), "\"abc123\"".to_string());

        let cached = CachedResponse::new(&response, Duration::from_secs(300));
        assert!(cached.is_fresh());
        assert_eq!(cached.etag, Some("\"abc123\"".to_string()));
    }

    #[tokio::test]
    async fn test_response_cache_store_and_get() {
        let cache = ResponseCache::new();
        let request = HttpRequest::new("GET".to_string(), "/api/users".to_string());
        let mut response = HttpResponse::ok();
        response.body = b"cached content".to_vec();

        cache.store(&request, &response).await;

        let cached = cache.get(&request).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().body, b"cached content");
    }

    #[tokio::test]
    async fn test_response_cache_invalidate() {
        let cache = ResponseCache::new();
        let request = HttpRequest::new("GET".to_string(), "/api/users".to_string());
        let response = HttpResponse::ok();

        cache.store(&request, &response).await;
        assert!(cache.get(&request).await.is_some());

        cache.invalidate(&request).await;
        assert!(cache.get(&request).await.is_none());
    }

    #[tokio::test]
    async fn test_response_cache_respects_no_store() {
        let cache = ResponseCache::new();
        let request = HttpRequest::new("GET".to_string(), "/api/users".to_string());
        let response = HttpResponse::ok().no_cache();

        cache.store(&request, &response).await;

        // Should not be cached due to no-store
        assert!(cache.get(&request).await.is_none());
    }

    #[test]
    fn test_response_cache_control_methods() {
        let response = HttpResponse::ok()
            .cache_public(Duration::from_secs(3600));

        let cc = response.get_cache_control().unwrap();
        assert!(cc.is_public());
        assert_eq!(cc.get_max_age(), Some(3600));
    }

    #[test]
    fn test_response_with_vary() {
        let response = HttpResponse::ok()
            .with_vary(&["Accept", "Accept-Encoding"]);

        assert_eq!(response.headers.get("Vary"), Some(&"Accept, Accept-Encoding".to_string()));
    }

    #[test]
    fn test_request_allows_cached() {
        let request = HttpRequest::new("GET".to_string(), "/api/users".to_string());
        assert!(request.allows_cached());

        let mut request_no_cache = HttpRequest::new("GET".to_string(), "/api/users".to_string());
        request_no_cache.headers.insert("Cache-Control".to_string(), "no-cache".to_string());
        assert!(!request_no_cache.allows_cached());
    }
}

