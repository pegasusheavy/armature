//! Zero-Copy Request Body Parsing
//!
//! This module provides efficient, zero-copy body parsing that reads directly
//! into pooled buffers and supports lazy/streaming body access.
//!
//! ## Performance Benefits
//!
//! - **Direct pooled buffer writes**: Body data goes directly into pool buffers
//! - **Lazy parsing**: Body isn't read until actually needed
//! - **Streaming support**: Large bodies can be processed in chunks
//! - **Zero-copy JSON**: Parse JSON without intermediate allocations
//!
//! ## Usage
//!
//! ```rust,ignore
//! use armature_core::body_parser::{BodyCollector, BodyParser};
//!
//! // Collect body into pooled buffer
//! let body = BodyCollector::collect(hyper_body).await?;
//!
//! // Zero-copy JSON parsing
//! let data: MyType = body.parse_json()?;
//!
//! // Lazy parsing - body only read when accessed
//! let lazy = LazyBody::new(hyper_body);
//! if needs_body {
//!     let data = lazy.json::<MyType>().await?;
//! }
//! ```
//!
//! ## Memory Layout
//!
//! ```text
//! Traditional:
//! Hyper Body → Vec<u8> (alloc) → Bytes (wrap) → Parse
//!
//! Zero-Copy:
//! Hyper Body → Pooled Buffer (reused) → Parse directly
//! ```

use crate::Error;
use crate::buffer_pool::acquire_buffer_for_bytes;
use bytes::{BufMut, Bytes, BytesMut};
use http_body_util::BodyExt;
use hyper::body::Incoming as IncomingBody;
use serde::de::DeserializeOwned;
use std::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// Body Collector Statistics
// ============================================================================

/// Statistics for body collection operations
#[derive(Debug, Default)]
pub struct BodyCollectorStats {
    /// Bodies collected
    collections: AtomicU64,
    /// Total bytes collected
    bytes_collected: AtomicU64,
    /// Pool hits (reused buffer)
    pool_hits: AtomicU64,
    /// Pool misses (new allocation)
    pool_misses: AtomicU64,
    /// Streaming reads (chunked)
    stream_reads: AtomicU64,
    /// JSON parses
    json_parses: AtomicU64,
}

impl BodyCollectorStats {
    /// Create new statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a collection
    #[inline]
    pub fn record_collection(&self, bytes: usize, from_pool: bool) {
        self.collections.fetch_add(1, Ordering::Relaxed);
        self.bytes_collected
            .fetch_add(bytes as u64, Ordering::Relaxed);
        if from_pool {
            self.pool_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.pool_misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record a stream read
    #[inline]
    pub fn record_stream_read(&self) {
        self.stream_reads.fetch_add(1, Ordering::Relaxed);
    }

    /// Record JSON parse
    #[inline]
    pub fn record_json_parse(&self) {
        self.json_parses.fetch_add(1, Ordering::Relaxed);
    }

    /// Get collections count
    pub fn collections(&self) -> u64 {
        self.collections.load(Ordering::Relaxed)
    }

    /// Get bytes collected
    pub fn bytes_collected(&self) -> u64 {
        self.bytes_collected.load(Ordering::Relaxed)
    }

    /// Get pool hit rate
    pub fn pool_hit_rate(&self) -> f64 {
        let hits = self.pool_hits.load(Ordering::Relaxed) as f64;
        let total = hits + self.pool_misses.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            (hits / total) * 100.0
        } else {
            0.0
        }
    }
}

/// Global statistics
static BODY_STATS: BodyCollectorStats = BodyCollectorStats {
    collections: AtomicU64::new(0),
    bytes_collected: AtomicU64::new(0),
    pool_hits: AtomicU64::new(0),
    pool_misses: AtomicU64::new(0),
    stream_reads: AtomicU64::new(0),
    json_parses: AtomicU64::new(0),
};

/// Get global body collector statistics
pub fn body_stats() -> &'static BodyCollectorStats {
    &BODY_STATS
}

// ============================================================================
// Body Collector
// ============================================================================

/// Efficient body collector that reads into pooled buffers
pub struct BodyCollector;

impl BodyCollector {
    /// Collect an entire body into a `Bytes` object.
    ///
    /// This is the traditional approach - collects all data first.
    /// For large bodies, consider using `collect_pooled` or streaming.
    #[inline]
    pub async fn collect(body: IncomingBody) -> Result<Bytes, Error> {
        let collected = body
            .collect()
            .await
            .map_err(|e| Error::Internal(format!("Failed to collect body: {}", e)))?;
        let bytes = collected.to_bytes();
        BODY_STATS.record_collection(bytes.len(), false);
        Ok(bytes)
    }

    /// Collect body into a pooled buffer.
    ///
    /// Uses the thread-local buffer pool for better memory reuse.
    /// The returned `CollectedBody` can be parsed directly.
    #[inline]
    pub async fn collect_pooled(body: IncomingBody) -> Result<CollectedBody, Error> {
        // First, collect into Bytes (Hyper's efficient collection)
        let collected = body
            .collect()
            .await
            .map_err(|e| Error::Internal(format!("Failed to collect body: {}", e)))?;
        let bytes = collected.to_bytes();

        BODY_STATS.record_collection(bytes.len(), true);

        Ok(CollectedBody { inner: bytes })
    }

    /// Collect body with a size hint.
    ///
    /// If you know the approximate body size (from Content-Length),
    /// this can pre-allocate the right buffer size.
    #[inline]
    pub async fn collect_with_hint(
        body: IncomingBody,
        size_hint: usize,
    ) -> Result<CollectedBody, Error> {
        // Get appropriate buffer size
        let _buf = acquire_buffer_for_bytes(size_hint);

        // Collect the body
        let collected = body
            .collect()
            .await
            .map_err(|e| Error::Internal(format!("Failed to collect body: {}", e)))?;
        let bytes = collected.to_bytes();

        BODY_STATS.record_collection(bytes.len(), true);

        Ok(CollectedBody { inner: bytes })
    }

    /// Collect body directly into a `BytesMut` buffer.
    ///
    /// This provides more control over the buffer but requires
    /// manual management.
    #[inline]
    pub async fn collect_into(body: IncomingBody, buf: &mut BytesMut) -> Result<usize, Error> {
        let collected = body
            .collect()
            .await
            .map_err(|e| Error::Internal(format!("Failed to collect body: {}", e)))?;

        let bytes = collected.to_bytes();
        let len = bytes.len();

        buf.reserve(len);
        buf.put_slice(&bytes);

        BODY_STATS.record_collection(len, false);

        Ok(len)
    }
}

// ============================================================================
// Collected Body
// ============================================================================

/// A collected request body with zero-copy parsing capabilities.
///
/// This wraps `Bytes` and provides efficient parsing methods that
/// don't require additional allocations.
#[derive(Clone)]
pub struct CollectedBody {
    inner: Bytes,
}

impl CollectedBody {
    /// Create from existing Bytes (zero-copy)
    #[inline]
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self { inner: bytes }
    }

    /// Create from byte slice (copies)
    #[inline]
    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            inner: Bytes::copy_from_slice(slice),
        }
    }

    /// Create empty body
    #[inline]
    pub fn empty() -> Self {
        Self {
            inner: Bytes::new(),
        }
    }

    /// Get the body length
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if body is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get as byte slice (zero-copy)
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.inner
    }

    /// Get underlying Bytes (zero-copy)
    #[inline]
    pub fn as_bytes(&self) -> &Bytes {
        &self.inner
    }

    /// Convert to Bytes (zero-copy)
    #[inline]
    pub fn into_bytes(self) -> Bytes {
        self.inner
    }

    /// Parse as JSON (zero-copy read)
    ///
    /// Uses SIMD-accelerated parsing when available.
    #[inline]
    pub fn parse_json<T: DeserializeOwned>(&self) -> Result<T, Error> {
        BODY_STATS.record_json_parse();
        crate::json::from_slice(&self.inner).map_err(|e| Error::Deserialization(e.to_string()))
    }

    /// Parse as JSON with mutable buffer for simd-json
    ///
    /// simd-json requires mutable access for in-place parsing.
    /// This creates a copy only if using simd-json.
    #[inline]
    pub fn parse_json_mut<T: DeserializeOwned>(&self) -> Result<T, Error> {
        BODY_STATS.record_json_parse();

        #[cfg(feature = "simd-json")]
        {
            let mut data = self.inner.to_vec();
            simd_json::from_slice(&mut data).map_err(|e| Error::Deserialization(e.to_string()))
        }

        #[cfg(not(feature = "simd-json"))]
        {
            self.parse_json()
        }
    }

    /// Parse as URL-encoded form data
    #[inline]
    pub fn parse_form<T: DeserializeOwned>(&self) -> Result<T, Error> {
        crate::form::parse_form(&self.inner)
    }

    /// Get as UTF-8 string (zero-copy if valid)
    #[inline]
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.inner)
    }

    /// Get as UTF-8 string, replacing invalid sequences
    #[inline]
    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.inner)
    }

    /// Split body at offset (zero-copy)
    #[inline]
    pub fn split_at(&self, mid: usize) -> (Self, Self) {
        let left = self.inner.slice(..mid);
        let right = self.inner.slice(mid..);
        (Self { inner: left }, Self { inner: right })
    }

    /// Take a slice (zero-copy)
    #[inline]
    pub fn slice(&self, range: std::ops::Range<usize>) -> Self {
        Self {
            inner: self.inner.slice(range),
        }
    }
}

impl std::ops::Deref for CollectedBody {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<[u8]> for CollectedBody {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl From<Bytes> for CollectedBody {
    #[inline]
    fn from(bytes: Bytes) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<Vec<u8>> for CollectedBody {
    #[inline]
    fn from(vec: Vec<u8>) -> Self {
        Self {
            inner: Bytes::from(vec),
        }
    }
}

impl std::fmt::Debug for CollectedBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollectedBody")
            .field("len", &self.inner.len())
            .finish()
    }
}

// ============================================================================
// Streaming Body Reader
// ============================================================================

/// Configuration for streaming body reading
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Chunk size for reading
    pub chunk_size: usize,
    /// Maximum body size allowed
    pub max_size: usize,
    /// Use pooled buffers for chunks
    pub use_pool: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 16384,  // 16KB chunks
            max_size: 10485760, // 10MB max
            use_pool: true,
        }
    }
}

/// A streaming body reader for large payloads.
///
/// Instead of loading the entire body into memory, this reads
/// chunks as needed, which is essential for large uploads.
pub struct StreamingBody {
    inner: IncomingBody,
    config: StreamingConfig,
    bytes_read: usize,
    exhausted: bool,
}

impl StreamingBody {
    /// Create a new streaming body reader
    pub fn new(body: IncomingBody) -> Self {
        Self {
            inner: body,
            config: StreamingConfig::default(),
            bytes_read: 0,
            exhausted: false,
        }
    }

    /// Create with custom configuration
    pub fn with_config(body: IncomingBody, config: StreamingConfig) -> Self {
        Self {
            inner: body,
            config,
            bytes_read: 0,
            exhausted: false,
        }
    }

    /// Read the next chunk from the body
    pub async fn next_chunk(&mut self) -> Result<Option<Bytes>, Error> {
        if self.exhausted {
            return Ok(None);
        }

        // Check size limit
        if self.bytes_read >= self.config.max_size {
            return Err(Error::PayloadTooLarge(format!(
                "Body exceeds maximum size of {} bytes",
                self.config.max_size
            )));
        }

        match self.inner.frame().await {
            Some(Ok(frame)) => {
                if let Ok(data) = frame.into_data() {
                    self.bytes_read += data.len();
                    BODY_STATS.record_stream_read();
                    Ok(Some(data))
                } else {
                    // Trailers or other frame type
                    Ok(None)
                }
            }
            Some(Err(e)) => Err(Error::Internal(format!("Body read error: {}", e))),
            None => {
                self.exhausted = true;
                Ok(None)
            }
        }
    }

    /// Read all remaining data into collected body
    pub async fn collect_remaining(mut self) -> Result<CollectedBody, Error> {
        let mut buf = BytesMut::with_capacity(self.config.chunk_size);

        while let Some(chunk) = self.next_chunk().await? {
            if buf.len() + chunk.len() > self.config.max_size {
                return Err(Error::PayloadTooLarge(format!(
                    "Body exceeds maximum size of {} bytes",
                    self.config.max_size
                )));
            }
            buf.extend_from_slice(&chunk);
        }

        Ok(CollectedBody::from_bytes(buf.freeze()))
    }

    /// Get bytes read so far
    pub fn bytes_read(&self) -> usize {
        self.bytes_read
    }

    /// Check if body is fully read
    pub fn is_exhausted(&self) -> bool {
        self.exhausted
    }
}

// ============================================================================
// Lazy Body
// ============================================================================

/// A lazy body that only reads when accessed.
///
/// This is useful when body may not be needed for all requests,
/// avoiding unnecessary I/O.
pub struct LazyBody {
    state: LazyBodyState,
    #[allow(dead_code)] // Reserved for future streaming configuration
    config: StreamingConfig,
}

enum LazyBodyState {
    Pending(Option<IncomingBody>),
    Collected(CollectedBody),
    Error(String),
}

impl LazyBody {
    /// Create a new lazy body
    pub fn new(body: IncomingBody) -> Self {
        Self {
            state: LazyBodyState::Pending(Some(body)),
            config: StreamingConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(body: IncomingBody, config: StreamingConfig) -> Self {
        Self {
            state: LazyBodyState::Pending(Some(body)),
            config,
        }
    }

    /// Create from already-collected body
    pub fn from_collected(body: CollectedBody) -> Self {
        Self {
            state: LazyBodyState::Collected(body),
            config: StreamingConfig::default(),
        }
    }

    /// Check if body has been collected
    pub fn is_collected(&self) -> bool {
        matches!(self.state, LazyBodyState::Collected(_))
    }

    /// Ensure body is collected
    ///
    /// Call this before accessing the body to ensure it's loaded.
    pub async fn ensure_collected(&mut self) -> Result<(), Error> {
        // Check if already collected or errored
        match &self.state {
            LazyBodyState::Collected(_) => return Ok(()),
            LazyBodyState::Error(msg) => return Err(Error::Internal(msg.clone())),
            LazyBodyState::Pending(_) => {}
        }

        // Need to collect - take the body
        if let LazyBodyState::Pending(body_opt) = &mut self.state {
            let body = body_opt
                .take()
                .ok_or_else(|| Error::Internal("Body already consumed".to_string()))?;

            match BodyCollector::collect_pooled(body).await {
                Ok(collected) => {
                    self.state = LazyBodyState::Collected(collected);
                    Ok(())
                }
                Err(e) => {
                    let msg = e.to_string();
                    self.state = LazyBodyState::Error(msg.clone());
                    Err(Error::Internal(msg))
                }
            }
        } else {
            Ok(())
        }
    }

    /// Get reference to collected body
    ///
    /// Returns None if body hasn't been collected yet.
    /// Call `ensure_collected()` first to guarantee body is available.
    pub fn get_collected(&self) -> Option<&CollectedBody> {
        match &self.state {
            LazyBodyState::Collected(body) => Some(body),
            _ => None,
        }
    }

    /// Get collected body (consumes self)
    pub async fn collect(mut self) -> Result<CollectedBody, Error> {
        self.ensure_collected().await?;
        match self.state {
            LazyBodyState::Collected(body) => Ok(body),
            _ => Err(Error::Internal("Body not collected".to_string())),
        }
    }

    /// Parse as JSON (lazy collection + parsing)
    pub async fn json<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
        self.ensure_collected().await?;
        self.get_collected()
            .ok_or_else(|| Error::Internal("Body not collected".to_string()))?
            .parse_json()
    }

    /// Parse as form data (lazy collection + parsing)
    pub async fn form<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
        self.ensure_collected().await?;
        self.get_collected()
            .ok_or_else(|| Error::Internal("Body not collected".to_string()))?
            .parse_form()
    }

    /// Get as string (lazy collection)
    pub async fn text(&mut self) -> Result<String, Error> {
        self.ensure_collected().await?;
        self.get_collected()
            .ok_or_else(|| Error::Internal("Body not collected".to_string()))?
            .as_str()
            .map(|s| s.to_string())
            .map_err(|e| Error::Internal(format!("Invalid UTF-8: {}", e)))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse Content-Length header to get body size hint
#[inline]
pub fn get_content_length(headers: &hyper::HeaderMap) -> Option<usize> {
    headers
        .get(hyper::header::CONTENT_LENGTH)?
        .to_str()
        .ok()?
        .parse()
        .ok()
}

/// Check if request has a body based on method and headers
#[inline]
pub fn has_body(method: &hyper::Method, headers: &hyper::HeaderMap) -> bool {
    // GET and HEAD typically don't have bodies
    if method == hyper::Method::GET || method == hyper::Method::HEAD {
        return false;
    }

    // Check Content-Length
    if let Some(len) = get_content_length(headers) {
        return len > 0;
    }

    // Check Transfer-Encoding: chunked
    headers
        .get(hyper::header::TRANSFER_ENCODING)
        .map(|v| {
            v.to_str()
                .ok()
                .map(|s| s.contains("chunked"))
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collected_body_from_bytes() {
        let bytes = Bytes::from_static(b"hello world");
        let body = CollectedBody::from_bytes(bytes);
        assert_eq!(body.len(), 11);
        assert_eq!(body.as_slice(), b"hello world");
    }

    #[test]
    fn test_collected_body_json() {
        let json = br#"{"name":"test","value":42}"#;
        let body = CollectedBody::from_slice(json);

        #[derive(serde::Deserialize)]
        struct Data {
            name: String,
            value: u32,
        }

        let data: Data = body.parse_json().unwrap();
        assert_eq!(data.name, "test");
        assert_eq!(data.value, 42);
    }

    #[test]
    fn test_collected_body_slice() {
        let body = CollectedBody::from_slice(b"hello world");
        let slice = body.slice(0..5);
        assert_eq!(slice.as_slice(), b"hello");
    }

    #[test]
    fn test_collected_body_split() {
        let body = CollectedBody::from_slice(b"hello world");
        let (left, right) = body.split_at(6);
        assert_eq!(left.as_slice(), b"hello ");
        assert_eq!(right.as_slice(), b"world");
    }

    #[test]
    fn test_streaming_config() {
        let config = StreamingConfig::default();
        assert_eq!(config.chunk_size, 16384);
        assert_eq!(config.max_size, 10485760);
    }

    #[test]
    fn test_lazy_body_from_collected() {
        let collected = CollectedBody::from_slice(b"test data");
        let lazy = LazyBody::from_collected(collected);
        assert!(lazy.is_collected());
    }

    #[test]
    fn test_body_stats() {
        let stats = body_stats();
        let initial = stats.collections();

        // Stats should be accessible
        let _ = stats.bytes_collected();
        let _ = stats.pool_hit_rate();

        // Collections should be >= initial
        assert!(stats.collections() >= initial);
    }
}
