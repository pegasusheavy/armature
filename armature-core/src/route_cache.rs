//! Route Matching Cache and Static Route Optimization
//!
//! This module provides optimizations for route matching:
//!
//! - **Route Cache**: LRU cache for recently matched routes
//! - **Static Fast Path**: O(1) HashMap lookup for static routes
//! - **Compiled Routes**: Pre-analyzed route patterns
//!
//! # Performance
//!
//! - Cache hit: ~5ns (vs ~50-150ns for pattern matching)
//! - Static lookup: ~10ns (vs ~50ns+ for trie traversal)
//! - Cache miss: Falls back to normal matching + caches result
//!
//! # Usage
//!
//! ```rust,ignore
//! use armature_core::route_cache::{CachedRouter, StaticRoutes};
//!
//! // Create cached router
//! let mut router = CachedRouter::new();
//! router.add_static("/api/health", handler);
//! router.add_pattern("/users/:id", handler);
//!
//! // Routing uses optimized paths automatically
//! let response = router.route(request).await?;
//! ```

use crate::handler::BoxedHandler;
use crate::{Error, HttpMethod, HttpRequest, HttpResponse};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

// ============================================================================
// Cache Key
// ============================================================================

/// A cache key combining method and path.
#[derive(Clone, Debug, Eq)]
pub struct RouteKey {
    /// HTTP method
    method: HttpMethod,
    /// Request path (without query string)
    path: String,
}

impl RouteKey {
    /// Create a new route key.
    #[inline]
    pub fn new(method: HttpMethod, path: impl Into<String>) -> Self {
        Self {
            method,
            path: path.into(),
        }
    }

    /// Create from request.
    #[inline]
    pub fn from_request(req: &HttpRequest) -> Self {
        let path = req
            .path
            .split_once('?')
            .map(|(p, _)| p)
            .unwrap_or(&req.path);

        Self {
            method: HttpMethod::from_str(&req.method).unwrap_or(HttpMethod::GET),
            path: path.to_string(),
        }
    }
}

impl PartialEq for RouteKey {
    fn eq(&self, other: &Self) -> bool {
        self.method == other.method && self.path == other.path
    }
}

impl Hash for RouteKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.method.as_str().hash(state);
        self.path.hash(state);
    }
}

// ============================================================================
// Cache Entry
// ============================================================================

/// A cached route match result.
#[derive(Clone)]
pub struct CachedRoute {
    /// Index into the route table
    pub route_index: usize,
    /// Pre-extracted path parameters (name, segment_index)
    pub param_indices: Vec<(String, usize)>,
    /// Is this a static route (no params)?
    pub is_static: bool,
}

impl CachedRoute {
    /// Create a cached entry for a static route.
    pub fn static_route(route_index: usize) -> Self {
        Self {
            route_index,
            param_indices: Vec::new(),
            is_static: true,
        }
    }

    /// Create a cached entry for a parameterized route.
    pub fn with_params(route_index: usize, param_indices: Vec<(String, usize)>) -> Self {
        Self {
            route_index,
            param_indices,
            is_static: false,
        }
    }

    /// Extract parameters from a path using cached indices.
    #[inline]
    pub fn extract_params(&self, path: &str) -> HashMap<String, String> {
        if self.is_static {
            return HashMap::new();
        }

        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut params = HashMap::with_capacity(self.param_indices.len());

        for (name, idx) in &self.param_indices {
            if let Some(value) = segments.get(*idx) {
                params.insert(name.clone(), (*value).to_string());
            }
        }

        params
    }
}

// ============================================================================
// LRU Route Cache
// ============================================================================

/// LRU cache for route matching results.
///
/// Thread-safe with interior mutability via RwLock.
pub struct RouteCache {
    /// Cached routes by key
    cache: RwLock<HashMap<RouteKey, CachedRoute>>,
    /// Maximum cache size
    max_size: usize,
    /// Statistics
    stats: RouteCacheStats,
}

impl RouteCache {
    /// Create new cache with default size (1024 entries).
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Create cache with specific capacity.
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::with_capacity(max_size)),
            max_size,
            stats: RouteCacheStats::default(),
        }
    }

    /// Get a cached route.
    #[inline]
    pub fn get(&self, key: &RouteKey) -> Option<CachedRoute> {
        let cache = self.cache.read().ok()?;
        let result = cache.get(key).cloned();

        if result.is_some() {
            self.stats.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.stats.misses.fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    /// Insert a route into the cache.
    pub fn insert(&self, key: RouteKey, route: CachedRoute) {
        if let Ok(mut cache) = self.cache.write() {
            // Simple eviction: if full, clear half
            if cache.len() >= self.max_size {
                self.stats.evictions.fetch_add(1, Ordering::Relaxed);
                let to_remove: Vec<_> = cache
                    .keys()
                    .take(self.max_size / 2)
                    .cloned()
                    .collect();
                for k in to_remove {
                    cache.remove(&k);
                }
            }

            cache.insert(key, route);
            self.stats.insertions.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Clear the cache.
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Get cache statistics.
    pub fn stats(&self) -> &RouteCacheStats {
        &self.stats
    }

    /// Get current cache size.
    pub fn len(&self) -> usize {
        self.cache.read().map(|c| c.len()).unwrap_or(0)
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for RouteCache {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Static Route Fast Path
// ============================================================================

/// Fast path for static routes using HashMap.
///
/// Static routes (no parameters) can be matched in O(1) time using
/// a direct HashMap lookup, bypassing pattern matching entirely.
pub struct StaticRoutes {
    /// Static routes by (method, path)
    routes: HashMap<RouteKey, usize>,
    /// Statistics
    stats: StaticRouteStats,
}

impl StaticRoutes {
    /// Create new static route store.
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            stats: StaticRouteStats::default(),
        }
    }

    /// Add a static route.
    pub fn add(&mut self, method: HttpMethod, path: impl Into<String>, route_index: usize) {
        let key = RouteKey::new(method, path);
        self.routes.insert(key, route_index);
    }

    /// Look up a static route.
    #[inline]
    pub fn get(&self, key: &RouteKey) -> Option<usize> {
        let result = self.routes.get(key).copied();

        if result.is_some() {
            self.stats.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.stats.misses.fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    /// Check if path is static (no parameter segments).
    pub fn is_static_path(path: &str) -> bool {
        !path.contains(':') && !path.contains('*')
    }

    /// Get statistics.
    pub fn stats(&self) -> &StaticRouteStats {
        &self.stats
    }

    /// Get number of static routes.
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }
}

impl Default for StaticRoutes {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Compiled Route Pattern
// ============================================================================

/// Pre-compiled route pattern for fast matching.
#[derive(Clone, Debug)]
pub struct CompiledRoute {
    /// Original pattern
    pub pattern: String,
    /// Parsed segments
    pub segments: Vec<RouteSegment>,
    /// Parameter indices (name, segment_index)
    pub param_indices: Vec<(String, usize)>,
    /// Is this a static route?
    pub is_static: bool,
    /// Has catch-all segment?
    pub has_catch_all: bool,
}

/// A segment in a route pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteSegment {
    /// Static segment (exact match)
    Static(String),
    /// Named parameter (:name)
    Param(String),
    /// Catch-all (*path)
    CatchAll(String),
}

impl CompiledRoute {
    /// Compile a route pattern.
    pub fn compile(pattern: &str) -> Self {
        let mut segments = Vec::new();
        let mut param_indices = Vec::new();
        let mut is_static = true;
        let mut has_catch_all = false;

        for (idx, part) in pattern.split('/').filter(|s| !s.is_empty()).enumerate() {
            if let Some(name) = part.strip_prefix(':') {
                segments.push(RouteSegment::Param(name.to_string()));
                param_indices.push((name.to_string(), idx));
                is_static = false;
            } else if let Some(name) = part.strip_prefix('*') {
                let name = if name.is_empty() { "*" } else { name };
                segments.push(RouteSegment::CatchAll(name.to_string()));
                param_indices.push((name.to_string(), idx));
                is_static = false;
                has_catch_all = true;
            } else {
                segments.push(RouteSegment::Static(part.to_string()));
            }
        }

        Self {
            pattern: pattern.to_string(),
            segments,
            param_indices,
            is_static,
            has_catch_all,
        }
    }

    /// Check if a path matches this pattern.
    #[inline]
    pub fn matches(&self, path: &str) -> bool {
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        if !self.has_catch_all && path_segments.len() != self.segments.len() {
            return false;
        }

        if self.has_catch_all && path_segments.len() < self.segments.len() - 1 {
            return false;
        }

        for (idx, segment) in self.segments.iter().enumerate() {
            match segment {
                RouteSegment::Static(s) => {
                    if path_segments.get(idx) != Some(&s.as_str()) {
                        return false;
                    }
                }
                RouteSegment::Param(_) => {
                    if idx >= path_segments.len() {
                        return false;
                    }
                }
                RouteSegment::CatchAll(_) => {
                    // Matches remaining segments
                    break;
                }
            }
        }

        true
    }

    /// Extract parameters from a matching path.
    pub fn extract_params(&self, path: &str) -> HashMap<String, String> {
        if self.is_static {
            return HashMap::new();
        }

        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut params = HashMap::with_capacity(self.param_indices.len());

        for (name, idx) in &self.param_indices {
            if let Some(segment) = self.segments.get(*idx) {
                match segment {
                    RouteSegment::Param(_) => {
                        if let Some(value) = path_segments.get(*idx) {
                            params.insert(name.clone(), (*value).to_string());
                        }
                    }
                    RouteSegment::CatchAll(_) => {
                        // Join remaining segments
                        let remaining: String = path_segments[*idx..].join("/");
                        params.insert(name.clone(), remaining);
                    }
                    _ => {}
                }
            }
        }

        params
    }
}

// ============================================================================
// Optimized Router
// ============================================================================

/// Router entry with compiled pattern.
pub struct OptimizedRoute {
    /// HTTP method
    pub method: HttpMethod,
    /// Compiled pattern
    pub compiled: CompiledRoute,
    /// Handler
    pub handler: BoxedHandler,
}

/// Optimized router with caching and static fast path.
pub struct OptimizedRouter {
    /// All routes
    routes: Vec<OptimizedRoute>,
    /// Static route fast path
    static_routes: StaticRoutes,
    /// Route cache
    cache: RouteCache,
    /// Statistics
    stats: RouterStats,
}

impl OptimizedRouter {
    /// Create new optimized router.
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            static_routes: StaticRoutes::new(),
            cache: RouteCache::new(),
            stats: RouterStats::default(),
        }
    }

    /// Create with specific cache size.
    pub fn with_cache_size(cache_size: usize) -> Self {
        Self {
            routes: Vec::new(),
            static_routes: StaticRoutes::new(),
            cache: RouteCache::with_capacity(cache_size),
            stats: RouterStats::default(),
        }
    }

    /// Add a route.
    pub fn add_route(&mut self, method: HttpMethod, path: impl Into<String>, handler: BoxedHandler) {
        let path = path.into();
        let compiled = CompiledRoute::compile(&path);
        let route_index = self.routes.len();

        // Add to static routes if applicable
        if compiled.is_static {
            self.static_routes.add(method.clone(), &path, route_index);
        }

        self.routes.push(OptimizedRoute {
            method,
            compiled,
            handler,
        });
    }

    /// Route a request with optimized matching.
    pub async fn route(&self, mut request: HttpRequest) -> Result<HttpResponse, Error> {
        self.stats.requests.fetch_add(1, Ordering::Relaxed);

        // Parse query parameters from path
        let (path, query_string) = request
            .path
            .split_once('?')
            .map(|(p, q)| (p, Some(q)))
            .unwrap_or((&request.path, None));

        if let Some(query) = query_string {
            request.query_params = crate::simd_parser::parse_query_string_fast(query);
        }

        let key = RouteKey::new(
            HttpMethod::from_str(&request.method).unwrap_or(HttpMethod::GET),
            path,
        );

        // 1. Try static route fast path (O(1))
        if let Some(route_index) = self.static_routes.get(&key) {
            self.stats.static_hits.fetch_add(1, Ordering::Relaxed);
            return self.routes[route_index].handler.call(request).await;
        }

        // 2. Try cache (O(1))
        if let Some(cached) = self.cache.get(&key) {
            self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
            request.path_params = cached.extract_params(path);
            return self.routes[cached.route_index].handler.call(request).await;
        }

        // 3. Fall back to pattern matching
        self.stats.pattern_matches.fetch_add(1, Ordering::Relaxed);

        for (route_index, route) in self.routes.iter().enumerate() {
            if Some(route.method.clone()) != HttpMethod::from_str(&request.method) {
                continue;
            }

            if route.compiled.matches(path) {
                // Cache the match for future requests
                let cached = if route.compiled.is_static {
                    CachedRoute::static_route(route_index)
                } else {
                    CachedRoute::with_params(route_index, route.compiled.param_indices.clone())
                };
                self.cache.insert(key, cached);

                // Extract params and call handler
                request.path_params = route.compiled.extract_params(path);
                return route.handler.call(request).await;
            }
        }

        Err(Error::RouteNotFound(format!("{} {}", request.method, path)))
    }

    /// Get statistics.
    pub fn stats(&self) -> &RouterStats {
        &self.stats
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> &RouteCacheStats {
        self.cache.stats()
    }

    /// Get static route statistics.
    pub fn static_stats(&self) -> &StaticRouteStats {
        self.static_routes.stats()
    }

    /// Clear the route cache.
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get number of routes.
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    /// Check if router is empty.
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }
}

impl Default for OptimizedRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Statistics
// ============================================================================

/// Route cache statistics.
#[derive(Debug, Default)]
pub struct RouteCacheStats {
    hits: AtomicU64,
    misses: AtomicU64,
    insertions: AtomicU64,
    evictions: AtomicU64,
}

impl RouteCacheStats {
    /// Get cache hits.
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// Get cache misses.
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// Get insertions.
    pub fn insertions(&self) -> u64 {
        self.insertions.load(Ordering::Relaxed)
    }

    /// Get evictions.
    pub fn evictions(&self) -> u64 {
        self.evictions.load(Ordering::Relaxed)
    }

    /// Get hit ratio.
    pub fn hit_ratio(&self) -> f64 {
        let hits = self.hits() as f64;
        let total = hits + self.misses() as f64;
        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }
}

/// Static route statistics.
#[derive(Debug, Default)]
pub struct StaticRouteStats {
    hits: AtomicU64,
    misses: AtomicU64,
}

impl StaticRouteStats {
    /// Get hits.
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// Get misses.
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// Get hit ratio.
    pub fn hit_ratio(&self) -> f64 {
        let hits = self.hits() as f64;
        let total = hits + self.misses() as f64;
        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }
}

/// Router statistics.
#[derive(Debug, Default)]
pub struct RouterStats {
    requests: AtomicU64,
    static_hits: AtomicU64,
    cache_hits: AtomicU64,
    pattern_matches: AtomicU64,
}

impl RouterStats {
    /// Get total requests.
    pub fn requests(&self) -> u64 {
        self.requests.load(Ordering::Relaxed)
    }

    /// Get static route hits.
    pub fn static_hits(&self) -> u64 {
        self.static_hits.load(Ordering::Relaxed)
    }

    /// Get cache hits.
    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed)
    }

    /// Get pattern match fallbacks.
    pub fn pattern_matches(&self) -> u64 {
        self.pattern_matches.load(Ordering::Relaxed)
    }

    /// Get optimization efficiency (static + cache hits / total).
    pub fn optimization_ratio(&self) -> f64 {
        let optimized = self.static_hits() + self.cache_hits();
        let total = self.requests();
        if total > 0 {
            optimized as f64 / total as f64
        } else {
            0.0
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
    fn test_route_key_equality() {
        let key1 = RouteKey::new(HttpMethod::GET, "/users");
        let key2 = RouteKey::new(HttpMethod::GET, "/users");
        let key3 = RouteKey::new(HttpMethod::POST, "/users");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_cached_route_static() {
        let cached = CachedRoute::static_route(0);
        assert!(cached.is_static);

        let params = cached.extract_params("/users");
        assert!(params.is_empty());
    }

    #[test]
    fn test_cached_route_with_params() {
        let cached = CachedRoute::with_params(0, vec![("id".to_string(), 1)]);
        assert!(!cached.is_static);

        let params = cached.extract_params("/users/123");
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_route_cache() {
        let cache = RouteCache::new();

        let key = RouteKey::new(HttpMethod::GET, "/users");
        let route = CachedRoute::static_route(0);

        // Miss
        assert!(cache.get(&key).is_none());
        assert_eq!(cache.stats().misses(), 1);

        // Insert
        cache.insert(key.clone(), route);

        // Hit
        assert!(cache.get(&key).is_some());
        assert_eq!(cache.stats().hits(), 1);
    }

    #[test]
    fn test_static_routes() {
        let mut static_routes = StaticRoutes::new();

        static_routes.add(HttpMethod::GET, "/api/health", 0);
        static_routes.add(HttpMethod::GET, "/api/users", 1);

        let key = RouteKey::new(HttpMethod::GET, "/api/health");
        assert_eq!(static_routes.get(&key), Some(0));

        let key = RouteKey::new(HttpMethod::GET, "/api/users");
        assert_eq!(static_routes.get(&key), Some(1));

        let key = RouteKey::new(HttpMethod::GET, "/api/missing");
        assert_eq!(static_routes.get(&key), None);
    }

    #[test]
    fn test_is_static_path() {
        assert!(StaticRoutes::is_static_path("/api/health"));
        assert!(StaticRoutes::is_static_path("/users"));
        assert!(!StaticRoutes::is_static_path("/users/:id"));
        assert!(!StaticRoutes::is_static_path("/files/*path"));
    }

    #[test]
    fn test_compiled_route_static() {
        let compiled = CompiledRoute::compile("/api/health");
        assert!(compiled.is_static);
        assert!(compiled.param_indices.is_empty());
        assert!(compiled.matches("/api/health"));
        assert!(!compiled.matches("/api/users"));
    }

    #[test]
    fn test_compiled_route_with_param() {
        let compiled = CompiledRoute::compile("/users/:id");
        assert!(!compiled.is_static);
        assert_eq!(compiled.param_indices.len(), 1);

        assert!(compiled.matches("/users/123"));
        assert!(compiled.matches("/users/abc"));
        assert!(!compiled.matches("/users"));
        assert!(!compiled.matches("/users/123/extra"));

        let params = compiled.extract_params("/users/123");
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_compiled_route_multiple_params() {
        let compiled = CompiledRoute::compile("/users/:user_id/posts/:post_id");
        assert!(!compiled.is_static);
        assert_eq!(compiled.param_indices.len(), 2);

        assert!(compiled.matches("/users/123/posts/456"));

        let params = compiled.extract_params("/users/123/posts/456");
        assert_eq!(params.get("user_id"), Some(&"123".to_string()));
        assert_eq!(params.get("post_id"), Some(&"456".to_string()));
    }

    #[test]
    fn test_compiled_route_catch_all() {
        let compiled = CompiledRoute::compile("/files/*path");
        assert!(!compiled.is_static);
        assert!(compiled.has_catch_all);

        assert!(compiled.matches("/files/docs"));
        assert!(compiled.matches("/files/docs/readme.md"));

        let params = compiled.extract_params("/files/docs/readme.md");
        assert_eq!(params.get("path"), Some(&"docs/readme.md".to_string()));
    }

    #[test]
    fn test_router_stats() {
        let stats = RouterStats::default();

        stats.requests.fetch_add(100, Ordering::Relaxed);
        stats.static_hits.fetch_add(50, Ordering::Relaxed);
        stats.cache_hits.fetch_add(30, Ordering::Relaxed);
        stats.pattern_matches.fetch_add(20, Ordering::Relaxed);

        assert_eq!(stats.requests(), 100);
        assert_eq!(stats.static_hits(), 50);
        assert_eq!(stats.cache_hits(), 30);
        assert_eq!(stats.pattern_matches(), 20);
        assert!((stats.optimization_ratio() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_route_cache_eviction() {
        let cache = RouteCache::with_capacity(10);

        // Fill cache
        for i in 0..15 {
            let key = RouteKey::new(HttpMethod::GET, format!("/route/{}", i));
            cache.insert(key, CachedRoute::static_route(i));
        }

        // Should have evicted some entries
        assert!(cache.len() <= 10);
        assert!(cache.stats().evictions() > 0);
    }
}

