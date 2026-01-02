//! Read Buffer Sizing Module
//!
//! This module provides adaptive read buffer sizing based on typical payload
//! patterns. By tuning buffer sizes to match actual traffic, we reduce memory
//! waste and improve throughput.
//!
//! ## How It Works
//!
//! 1. Track payload sizes as they arrive
//! 2. Compute running statistics (mean, percentiles)
//! 3. Adjust buffer sizes to match typical payloads
//! 4. Use separate strategies for different content types
//!
//! ## Buffer Size Strategy
//!
//! | Payload Size | Strategy |
//! |-------------|----------|
//! | < 1KB | Use tiny buffer (256B), expand as needed |
//! | 1KB - 8KB | Use small buffer (4KB) |
//! | 8KB - 32KB | Use medium buffer (16KB) |
//! | 32KB - 128KB | Use large buffer (64KB) |
//! | > 128KB | Streaming mode, chunked reading |
//!
//! ## Performance Impact
//!
//! - **Oversized buffers**: Waste memory, reduce cache efficiency
//! - **Undersized buffers**: Require reallocation/copy
//! - **Right-sized buffers**: Optimal memory use, fewer copies
//!
//! ## Example
//!
//! ```rust,ignore
//! use armature_core::read_buffer::{ReadBufferConfig, PayloadTracker};
//!
//! let mut tracker = PayloadTracker::new();
//!
//! // Record payload sizes as they arrive
//! tracker.record(1024);
//! tracker.record(2048);
//! tracker.record(512);
//!
//! // Get recommended buffer size
//! let size = tracker.recommended_buffer_size();
//! ```

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

// ============================================================================
// Buffer Size Constants
// ============================================================================

/// Tiny read buffer (256 bytes) - headers, small JSON
pub const TINY_BUFFER: usize = 256;

/// Small read buffer (4 KB) - typical JSON/form bodies
pub const SMALL_BUFFER: usize = 4096;

/// Medium read buffer (16 KB) - larger JSON, small files
pub const MEDIUM_BUFFER: usize = 16384;

/// Large read buffer (64 KB) - file uploads, bulk data
pub const LARGE_BUFFER: usize = 65536;

/// Huge read buffer (256 KB) - streaming, very large payloads
pub const HUGE_BUFFER: usize = 262144;

/// Default initial buffer size
pub const DEFAULT_INITIAL_BUFFER: usize = SMALL_BUFFER;

/// Minimum buffer size
pub const MIN_BUFFER: usize = 256;

/// Maximum buffer size before streaming
pub const MAX_BUFFER: usize = 1024 * 1024; // 1 MB

// ============================================================================
// Read Buffer Configuration
// ============================================================================

/// Configuration for read buffer sizing.
#[derive(Debug, Clone)]
pub struct ReadBufferConfig {
    /// Initial buffer size for new connections
    pub initial_size: usize,
    /// Minimum buffer size
    pub min_size: usize,
    /// Maximum buffer size before streaming
    pub max_size: usize,
    /// Growth factor when buffer needs expansion (1.5 = 50% growth)
    pub growth_factor: f32,
    /// Shrink threshold: shrink if usage < capacity * threshold
    pub shrink_threshold: f32,
    /// Enable adaptive sizing based on traffic patterns
    pub adaptive: bool,
    /// How often to recalculate optimal sizes (requests)
    pub recalculate_interval: usize,
    /// Use content-type hints for sizing
    pub content_type_hints: bool,
}

impl Default for ReadBufferConfig {
    fn default() -> Self {
        Self {
            initial_size: DEFAULT_INITIAL_BUFFER,
            min_size: MIN_BUFFER,
            max_size: MAX_BUFFER,
            growth_factor: 1.5,
            shrink_threshold: 0.25,
            adaptive: true,
            recalculate_interval: 1000,
            content_type_hints: true,
        }
    }
}

impl ReadBufferConfig {
    /// Create configuration optimized for high throughput.
    pub fn high_throughput() -> Self {
        Self {
            initial_size: MEDIUM_BUFFER,
            min_size: SMALL_BUFFER,
            max_size: MAX_BUFFER,
            growth_factor: 2.0,
            shrink_threshold: 0.1,
            adaptive: true,
            recalculate_interval: 500,
            content_type_hints: true,
        }
    }

    /// Create configuration optimized for low memory usage.
    pub fn low_memory() -> Self {
        Self {
            initial_size: TINY_BUFFER,
            min_size: MIN_BUFFER,
            max_size: LARGE_BUFFER,
            growth_factor: 1.25,
            shrink_threshold: 0.5,
            adaptive: true,
            recalculate_interval: 100,
            content_type_hints: true,
        }
    }

    /// Create configuration for API servers (typically small JSON).
    pub fn api_server() -> Self {
        Self {
            initial_size: SMALL_BUFFER,
            min_size: TINY_BUFFER,
            max_size: MEDIUM_BUFFER,
            growth_factor: 1.5,
            shrink_threshold: 0.25,
            adaptive: true,
            recalculate_interval: 1000,
            content_type_hints: true,
        }
    }

    /// Create configuration for file upload servers.
    pub fn file_upload() -> Self {
        Self {
            initial_size: LARGE_BUFFER,
            min_size: MEDIUM_BUFFER,
            max_size: MAX_BUFFER,
            growth_factor: 2.0,
            shrink_threshold: 0.1,
            adaptive: false, // Fixed large buffers
            recalculate_interval: 0,
            content_type_hints: false,
        }
    }

    /// Calculate next buffer size for growth.
    #[inline]
    pub fn grow_size(&self, current: usize) -> usize {
        let grown = (current as f32 * self.growth_factor) as usize;
        grown.min(self.max_size).max(self.min_size)
    }

    /// Calculate buffer size for shrinking.
    #[inline]
    pub fn shrink_size(&self, current: usize, used: usize) -> usize {
        let usage_ratio = used as f32 / current as f32;
        if usage_ratio < self.shrink_threshold && current > self.min_size {
            // Shrink to next power of 2 that fits the data
            let target = used.next_power_of_two();
            target.max(self.min_size)
        } else {
            current
        }
    }

    /// Get recommended buffer size for a content type.
    #[inline]
    pub fn size_for_content_type(&self, content_type: &str) -> usize {
        if !self.content_type_hints {
            return self.initial_size;
        }

        if content_type.contains("application/json") {
            SMALL_BUFFER // Most JSON is < 4KB
        } else if content_type.contains("text/html") {
            MEDIUM_BUFFER // HTML can be larger
        } else if content_type.contains("multipart/form-data")
            || content_type.contains("application/octet-stream")
        {
            LARGE_BUFFER // File uploads and binary data
        } else if content_type.contains("text/plain") {
            SMALL_BUFFER
        } else if content_type.contains("application/x-www-form-urlencoded") {
            TINY_BUFFER // Form data is usually small
        } else {
            self.initial_size
        }
    }
}

// ============================================================================
// Payload Statistics Tracker
// ============================================================================

/// Tracks payload sizes for adaptive buffer sizing.
#[derive(Debug)]
pub struct PayloadTracker {
    /// Total payloads recorded
    count: AtomicU64,
    /// Sum of all payload sizes
    total_bytes: AtomicU64,
    /// Sum of squares (for variance calculation)
    sum_squares: AtomicU64,
    /// Minimum payload size seen
    min_size: AtomicUsize,
    /// Maximum payload size seen
    max_size: AtomicUsize,
    /// Histogram buckets (powers of 2)
    histogram: [AtomicU64; 24], // 2^0 to 2^23 (1B to 8MB)
    /// Last recalculation time
    last_recalc: AtomicU64, // epoch millis
    /// Cached recommended size
    cached_size: AtomicUsize,
}

impl Default for PayloadTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl PayloadTracker {
    /// Create a new payload tracker.
    pub fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
            sum_squares: AtomicU64::new(0),
            min_size: AtomicUsize::new(usize::MAX),
            max_size: AtomicUsize::new(0),
            histogram: std::array::from_fn(|_| AtomicU64::new(0)),
            last_recalc: AtomicU64::new(0),
            cached_size: AtomicUsize::new(DEFAULT_INITIAL_BUFFER),
        }
    }

    /// Record a payload size.
    #[inline]
    pub fn record(&self, size: usize) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.total_bytes.fetch_add(size as u64, Ordering::Relaxed);

        // Update sum of squares (capped to prevent overflow)
        let sq = (size as u64).saturating_mul(size as u64);
        self.sum_squares
            .fetch_add(sq.min(u64::MAX / 2), Ordering::Relaxed);

        // Update min/max
        self.min_size.fetch_min(size, Ordering::Relaxed);
        self.max_size.fetch_max(size, Ordering::Relaxed);

        // Update histogram
        let bucket = if size == 0 {
            0
        } else {
            (64 - (size as u64).leading_zeros()).min(23) as usize
        };
        self.histogram[bucket].fetch_add(1, Ordering::Relaxed);
    }

    /// Get the total number of payloads recorded.
    #[inline]
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get the total bytes recorded.
    #[inline]
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes.load(Ordering::Relaxed)
    }

    /// Get the average payload size.
    #[inline]
    pub fn average_size(&self) -> f64 {
        let count = self.count();
        if count == 0 {
            return DEFAULT_INITIAL_BUFFER as f64;
        }
        self.total_bytes() as f64 / count as f64
    }

    /// Get the minimum payload size.
    #[inline]
    pub fn min_size(&self) -> usize {
        let min = self.min_size.load(Ordering::Relaxed);
        if min == usize::MAX {
            0
        } else {
            min
        }
    }

    /// Get the maximum payload size.
    #[inline]
    pub fn max_size(&self) -> usize {
        self.max_size.load(Ordering::Relaxed)
    }

    /// Calculate the standard deviation of payload sizes.
    pub fn std_deviation(&self) -> f64 {
        let count = self.count();
        if count < 2 {
            return 0.0;
        }

        let mean = self.average_size();
        let sum_sq = self.sum_squares.load(Ordering::Relaxed) as f64;
        let variance = (sum_sq / count as f64) - (mean * mean);
        variance.max(0.0).sqrt()
    }

    /// Get the percentile payload size (approximate via histogram).
    pub fn percentile(&self, p: f64) -> usize {
        let count = self.count();
        if count == 0 {
            return DEFAULT_INITIAL_BUFFER;
        }

        let target = (count as f64 * p / 100.0) as u64;
        let mut cumulative = 0u64;

        for (bucket, counter) in self.histogram.iter().enumerate() {
            cumulative += counter.load(Ordering::Relaxed);
            if cumulative >= target {
                return 1usize << bucket;
            }
        }

        self.max_size()
    }

    /// Get the recommended buffer size based on traffic patterns.
    ///
    /// This uses P90 (90th percentile) to size buffers so that
    /// 90% of payloads fit without reallocation.
    pub fn recommended_buffer_size(&self) -> usize {
        let count = self.count();
        if count < 10 {
            // Not enough data, use default
            return DEFAULT_INITIAL_BUFFER;
        }

        // Use P90 as the target size
        let p90 = self.percentile(90.0);

        // Round up to nearest standard buffer size
        if p90 <= TINY_BUFFER {
            TINY_BUFFER
        } else if p90 <= SMALL_BUFFER {
            SMALL_BUFFER
        } else if p90 <= MEDIUM_BUFFER {
            MEDIUM_BUFFER
        } else if p90 <= LARGE_BUFFER {
            LARGE_BUFFER
        } else if p90 <= HUGE_BUFFER {
            HUGE_BUFFER
        } else {
            // Cap at MAX_BUFFER, use streaming for larger
            MAX_BUFFER
        }
    }

    /// Update cached recommended size (call periodically).
    pub fn update_cached_size(&self) {
        let size = self.recommended_buffer_size();
        self.cached_size.store(size, Ordering::Relaxed);
        self.last_recalc.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            Ordering::Relaxed,
        );
    }

    /// Get cached recommended size (faster than recalculating).
    #[inline]
    pub fn cached_recommended_size(&self) -> usize {
        self.cached_size.load(Ordering::Relaxed)
    }

    /// Check if recalculation is needed.
    pub fn needs_recalc(&self, interval: Duration) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let last = self.last_recalc.load(Ordering::Relaxed);
        now.saturating_sub(last) > interval.as_millis() as u64
    }

    /// Get histogram data for analysis.
    pub fn histogram(&self) -> Vec<(usize, u64)> {
        self.histogram
            .iter()
            .enumerate()
            .map(|(i, c)| (1usize << i, c.load(Ordering::Relaxed)))
            .filter(|(_, c)| *c > 0)
            .collect()
    }

    /// Reset all statistics.
    pub fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
        self.total_bytes.store(0, Ordering::Relaxed);
        self.sum_squares.store(0, Ordering::Relaxed);
        self.min_size.store(usize::MAX, Ordering::Relaxed);
        self.max_size.store(0, Ordering::Relaxed);
        for counter in &self.histogram {
            counter.store(0, Ordering::Relaxed);
        }
        self.cached_size
            .store(DEFAULT_INITIAL_BUFFER, Ordering::Relaxed);
    }
}

// ============================================================================
// Content-Type Based Sizing
// ============================================================================

/// Content type categories for buffer sizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentCategory {
    /// JSON/XML/text API responses
    Api,
    /// HTML pages
    Html,
    /// Form data (urlencoded)
    Form,
    /// Multipart file uploads
    Multipart,
    /// Binary data
    Binary,
    /// Streaming content
    Streaming,
    /// Unknown/other
    Unknown,
}

impl ContentCategory {
    /// Detect content category from Content-Type header.
    pub fn from_content_type(content_type: &str) -> Self {
        let ct = content_type.to_lowercase();

        if ct.contains("application/json")
            || ct.contains("application/xml")
            || ct.contains("text/xml")
        {
            Self::Api
        } else if ct.contains("text/html") {
            Self::Html
        } else if ct.contains("application/x-www-form-urlencoded") {
            Self::Form
        } else if ct.contains("multipart/form-data") {
            Self::Multipart
        } else if ct.contains("application/octet-stream")
            || ct.contains("image/")
            || ct.contains("video/")
            || ct.contains("audio/")
        {
            Self::Binary
        } else if ct.contains("text/event-stream") || ct.contains("application/grpc") {
            Self::Streaming
        } else {
            Self::Unknown
        }
    }

    /// Get recommended initial buffer size for this content category.
    #[inline]
    pub const fn recommended_buffer_size(self) -> usize {
        match self {
            Self::Api => SMALL_BUFFER,        // 4KB - most JSON is small
            Self::Html => MEDIUM_BUFFER,      // 16KB - HTML varies
            Self::Form => TINY_BUFFER,        // 256B - form data is small
            Self::Multipart => LARGE_BUFFER,  // 64KB - file uploads
            Self::Binary => LARGE_BUFFER,     // 64KB - binary blobs
            Self::Streaming => MEDIUM_BUFFER, // 16KB - stream chunks
            Self::Unknown => SMALL_BUFFER,    // 4KB - safe default
        }
    }

    /// Get maximum buffer size before switching to streaming.
    #[inline]
    pub const fn streaming_threshold(self) -> usize {
        match self {
            Self::Api => LARGE_BUFFER,        // 64KB - large JSON is rare
            Self::Html => HUGE_BUFFER,        // 256KB - very long HTML
            Self::Form => MEDIUM_BUFFER,      // 16KB - forms shouldn't be huge
            Self::Multipart => MAX_BUFFER,    // 1MB - files can be large
            Self::Binary => MAX_BUFFER,       // 1MB - binary uploads
            Self::Streaming => MEDIUM_BUFFER, // 16KB - already streaming
            Self::Unknown => LARGE_BUFFER,    // 64KB - safe default
        }
    }
}

// ============================================================================
// Adaptive Buffer Sizer
// ============================================================================

/// Adaptive buffer sizer that learns from traffic patterns.
#[derive(Debug)]
pub struct AdaptiveBufferSizer {
    /// Configuration
    config: ReadBufferConfig,
    /// Payload tracker for requests
    request_tracker: PayloadTracker,
    /// Payload tracker for responses
    response_tracker: PayloadTracker,
    /// Per-content-type trackers
    api_tracker: PayloadTracker,
    html_tracker: PayloadTracker,
    multipart_tracker: PayloadTracker,
    /// Request count since last recalculation
    requests_since_recalc: AtomicUsize,
    /// Created timestamp
    created: Instant,
}

impl Default for AdaptiveBufferSizer {
    fn default() -> Self {
        Self::new(ReadBufferConfig::default())
    }
}

impl AdaptiveBufferSizer {
    /// Create a new adaptive buffer sizer.
    pub fn new(config: ReadBufferConfig) -> Self {
        Self {
            config,
            request_tracker: PayloadTracker::new(),
            response_tracker: PayloadTracker::new(),
            api_tracker: PayloadTracker::new(),
            html_tracker: PayloadTracker::new(),
            multipart_tracker: PayloadTracker::new(),
            requests_since_recalc: AtomicUsize::new(0),
            created: Instant::now(),
        }
    }

    /// Record a request payload.
    #[inline]
    pub fn record_request(&self, size: usize, content_type: Option<&str>) {
        self.request_tracker.record(size);

        if let Some(ct) = content_type {
            match ContentCategory::from_content_type(ct) {
                ContentCategory::Api => self.api_tracker.record(size),
                ContentCategory::Html => self.html_tracker.record(size),
                ContentCategory::Multipart => self.multipart_tracker.record(size),
                _ => {}
            }
        }

        self.maybe_recalculate();
    }

    /// Record a response payload.
    #[inline]
    pub fn record_response(&self, size: usize, content_type: Option<&str>) {
        self.response_tracker.record(size);

        if let Some(ct) = content_type {
            match ContentCategory::from_content_type(ct) {
                ContentCategory::Api => self.api_tracker.record(size),
                ContentCategory::Html => self.html_tracker.record(size),
                _ => {}
            }
        }
    }

    /// Check if recalculation is needed and perform it.
    fn maybe_recalculate(&self) {
        if !self.config.adaptive {
            return;
        }

        let count = self.requests_since_recalc.fetch_add(1, Ordering::Relaxed);
        if count >= self.config.recalculate_interval {
            self.requests_since_recalc.store(0, Ordering::Relaxed);
            self.request_tracker.update_cached_size();
            self.response_tracker.update_cached_size();
            self.api_tracker.update_cached_size();
            self.html_tracker.update_cached_size();
            self.multipart_tracker.update_cached_size();
        }
    }

    /// Get recommended read buffer size for requests.
    #[inline]
    pub fn request_buffer_size(&self) -> usize {
        if self.config.adaptive {
            self.request_tracker.cached_recommended_size()
        } else {
            self.config.initial_size
        }
    }

    /// Get recommended buffer size for a specific content type.
    #[inline]
    pub fn buffer_size_for_content_type(&self, content_type: &str) -> usize {
        if !self.config.adaptive {
            return self.config.size_for_content_type(content_type);
        }

        match ContentCategory::from_content_type(content_type) {
            ContentCategory::Api => self.api_tracker.cached_recommended_size(),
            ContentCategory::Html => self.html_tracker.cached_recommended_size(),
            ContentCategory::Multipart => self.multipart_tracker.cached_recommended_size(),
            category => category.recommended_buffer_size(),
        }
    }

    /// Get configuration.
    #[inline]
    pub fn config(&self) -> &ReadBufferConfig {
        &self.config
    }

    /// Get request tracker statistics.
    #[inline]
    pub fn request_stats(&self) -> &PayloadTracker {
        &self.request_tracker
    }

    /// Get response tracker statistics.
    #[inline]
    pub fn response_stats(&self) -> &PayloadTracker {
        &self.response_tracker
    }

    /// Get uptime.
    #[inline]
    pub fn uptime(&self) -> Duration {
        self.created.elapsed()
    }

    /// Reset all statistics.
    pub fn reset(&self) {
        self.request_tracker.reset();
        self.response_tracker.reset();
        self.api_tracker.reset();
        self.html_tracker.reset();
        self.multipart_tracker.reset();
        self.requests_since_recalc.store(0, Ordering::Relaxed);
    }
}

// ============================================================================
// Global Buffer Sizer
// ============================================================================

/// Global buffer sizing statistics.
#[derive(Debug, Default)]
pub struct BufferSizingStats {
    /// Total buffer allocations
    allocations: AtomicU64,
    /// Total buffer reallocations (growth)
    reallocations: AtomicU64,
    /// Bytes wasted due to oversized buffers
    bytes_wasted: AtomicU64,
    /// Perfect size hits (no reallocation needed)
    perfect_fits: AtomicU64,
}

impl BufferSizingStats {
    /// Create new stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an allocation.
    #[inline]
    pub fn record_allocation(&self, allocated: usize, used: usize) {
        self.allocations.fetch_add(1, Ordering::Relaxed);

        if used == allocated || (used as f64 / allocated as f64) > 0.75 {
            self.perfect_fits.fetch_add(1, Ordering::Relaxed);
        } else {
            let wasted = allocated.saturating_sub(used);
            self.bytes_wasted
                .fetch_add(wasted as u64, Ordering::Relaxed);
        }
    }

    /// Record a reallocation.
    #[inline]
    pub fn record_reallocation(&self) {
        self.reallocations.fetch_add(1, Ordering::Relaxed);
    }

    /// Get total allocations.
    pub fn allocations(&self) -> u64 {
        self.allocations.load(Ordering::Relaxed)
    }

    /// Get total reallocations.
    pub fn reallocations(&self) -> u64 {
        self.reallocations.load(Ordering::Relaxed)
    }

    /// Get bytes wasted.
    pub fn bytes_wasted(&self) -> u64 {
        self.bytes_wasted.load(Ordering::Relaxed)
    }

    /// Get perfect fit count.
    pub fn perfect_fits(&self) -> u64 {
        self.perfect_fits.load(Ordering::Relaxed)
    }

    /// Get perfect fit ratio (higher is better).
    pub fn perfect_fit_ratio(&self) -> f64 {
        let total = self.allocations();
        let perfect = self.perfect_fits();
        if total > 0 {
            (perfect as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get reallocation ratio (lower is better).
    pub fn reallocation_ratio(&self) -> f64 {
        let total = self.allocations();
        let reallocs = self.reallocations();
        if total > 0 {
            (reallocs as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Global buffer sizing statistics.
static BUFFER_SIZING_STATS: BufferSizingStats = BufferSizingStats {
    allocations: AtomicU64::new(0),
    reallocations: AtomicU64::new(0),
    bytes_wasted: AtomicU64::new(0),
    perfect_fits: AtomicU64::new(0),
};

/// Get global buffer sizing statistics.
pub fn buffer_sizing_stats() -> &'static BufferSizingStats {
    &BUFFER_SIZING_STATS
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_buffer_config_default() {
        let config = ReadBufferConfig::default();
        assert_eq!(config.initial_size, SMALL_BUFFER);
        assert!(config.adaptive);
    }

    #[test]
    fn test_read_buffer_config_presets() {
        let high = ReadBufferConfig::high_throughput();
        assert_eq!(high.initial_size, MEDIUM_BUFFER);

        let low = ReadBufferConfig::low_memory();
        assert_eq!(low.initial_size, TINY_BUFFER);

        let api = ReadBufferConfig::api_server();
        assert_eq!(api.initial_size, SMALL_BUFFER);

        let upload = ReadBufferConfig::file_upload();
        assert_eq!(upload.initial_size, LARGE_BUFFER);
        assert!(!upload.adaptive);
    }

    #[test]
    fn test_grow_size() {
        let config = ReadBufferConfig::default();
        assert_eq!(config.grow_size(4096), 6144); // 4096 * 1.5
        assert_eq!(config.grow_size(MAX_BUFFER), MAX_BUFFER); // Capped
    }

    #[test]
    fn test_shrink_size() {
        let config = ReadBufferConfig::default();
        // Should shrink when usage < 25% of capacity
        let shrunk = config.shrink_size(16384, 1000);
        assert!(shrunk < 16384);
        assert!(shrunk >= MIN_BUFFER);

        // Should not shrink when usage is high
        let not_shrunk = config.shrink_size(16384, 10000);
        assert_eq!(not_shrunk, 16384);
    }

    #[test]
    fn test_size_for_content_type() {
        let config = ReadBufferConfig::default();
        assert_eq!(
            config.size_for_content_type("application/json"),
            SMALL_BUFFER
        );
        assert_eq!(
            config.size_for_content_type("multipart/form-data"),
            LARGE_BUFFER
        );
        assert_eq!(config.size_for_content_type("text/html"), MEDIUM_BUFFER);
    }

    #[test]
    fn test_payload_tracker_basic() {
        let tracker = PayloadTracker::new();

        tracker.record(100);
        tracker.record(200);
        tracker.record(300);

        assert_eq!(tracker.count(), 3);
        assert_eq!(tracker.total_bytes(), 600);
        assert_eq!(tracker.min_size(), 100);
        assert_eq!(tracker.max_size(), 300);
        assert!((tracker.average_size() - 200.0).abs() < 0.1);
    }

    #[test]
    fn test_payload_tracker_percentile() {
        let tracker = PayloadTracker::new();

        // Record various sizes
        for _ in 0..100 {
            tracker.record(100); // Small
        }
        for _ in 0..100 {
            tracker.record(1000); // Medium
        }
        for _ in 0..10 {
            tracker.record(100000); // Large (10%)
        }

        let p50 = tracker.percentile(50.0);
        let p90 = tracker.percentile(90.0);
        let p99 = tracker.percentile(99.0);

        assert!(p50 <= p90);
        assert!(p90 <= p99);
    }

    #[test]
    fn test_payload_tracker_recommended_size() {
        let tracker = PayloadTracker::new();

        // Simulate typical API traffic (mostly small JSON)
        for i in 0..1000 {
            // Pseudo-random sizes between 500 and 1500
            let size = 500 + (i * 17 % 1000);
            tracker.record(size);
        }

        let recommended = tracker.recommended_buffer_size();
        // Should be SMALL or MEDIUM for typical API traffic
        assert!((TINY_BUFFER..=LARGE_BUFFER).contains(&recommended));
    }

    #[test]
    fn test_content_category_detection() {
        assert_eq!(
            ContentCategory::from_content_type("application/json"),
            ContentCategory::Api
        );
        assert_eq!(
            ContentCategory::from_content_type("text/html; charset=utf-8"),
            ContentCategory::Html
        );
        assert_eq!(
            ContentCategory::from_content_type("multipart/form-data; boundary=---"),
            ContentCategory::Multipart
        );
        assert_eq!(
            ContentCategory::from_content_type("image/png"),
            ContentCategory::Binary
        );
    }

    #[test]
    fn test_content_category_buffer_sizes() {
        assert_eq!(ContentCategory::Api.recommended_buffer_size(), SMALL_BUFFER);
        assert_eq!(
            ContentCategory::Html.recommended_buffer_size(),
            MEDIUM_BUFFER
        );
        assert_eq!(
            ContentCategory::Multipart.recommended_buffer_size(),
            LARGE_BUFFER
        );
    }

    #[test]
    fn test_adaptive_buffer_sizer() {
        let sizer = AdaptiveBufferSizer::new(ReadBufferConfig::default());

        // Record some requests
        sizer.record_request(1024, Some("application/json"));
        sizer.record_request(2048, Some("application/json"));
        sizer.record_request(512, Some("text/html"));

        let request_size = sizer.request_buffer_size();
        assert!(request_size >= MIN_BUFFER);
    }

    #[test]
    fn test_buffer_sizing_stats() {
        let stats = buffer_sizing_stats();
        stats.record_allocation(4096, 3000);
        stats.record_allocation(4096, 4000); // Perfect fit

        assert!(stats.allocations() >= 2);
    }

    #[test]
    fn test_histogram() {
        let tracker = PayloadTracker::new();

        tracker.record(100); // bucket ~7 (2^7 = 128)
        tracker.record(1000); // bucket ~10 (2^10 = 1024)
        tracker.record(10000); // bucket ~14 (2^14 = 16384)

        let hist = tracker.histogram();
        assert!(!hist.is_empty());
    }
}
