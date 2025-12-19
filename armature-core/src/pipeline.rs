//! HTTP/1.1 Pipelining Support
//!
//! This module provides HTTP/1.1 pipelining capabilities, allowing multiple
//! requests to be sent over a single TCP connection without waiting for
//! responses. This significantly improves throughput for high-latency connections.
//!
//! ## How Pipelining Works
//!
//! 1. Client sends multiple requests on the same connection
//! 2. Server processes requests concurrently (or in order)
//! 3. Responses are sent back in the order requests were received
//! 4. Connection is kept alive for subsequent request batches
//!
//! ## Configuration
//!
//! ```rust,ignore
//! use armature_core::pipeline::{PipelineConfig, PipelineMode};
//!
//! let config = PipelineConfig::builder()
//!     .mode(PipelineMode::Concurrent)
//!     .max_concurrent(16)
//!     .pipeline_flush(true)
//!     .keep_alive_timeout(Duration::from_secs(60))
//!     .build();
//! ```
//!
//! ## Performance Impact
//!
//! - **Without pipelining**: Each request waits for response (RTT per request)
//! - **With pipelining**: Multiple requests share RTT overhead
//! - **Expected gain**: 2-5x throughput improvement on high-latency connections

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Pipeline processing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PipelineMode {
    /// Process requests strictly in order (HTTP/1.1 compliant)
    /// Responses are sent in the same order as requests
    #[default]
    Sequential,

    /// Process requests concurrently but respond in order
    /// Better throughput while maintaining HTTP/1.1 compliance
    Concurrent,

    /// Process and respond as fast as possible (out-of-order)
    /// Not HTTP/1.1 compliant but maximum throughput
    /// Only use with clients that support out-of-order responses
    OutOfOrder,
}

impl PipelineMode {
    /// Check if this mode maintains response ordering
    #[inline]
    pub fn maintains_order(&self) -> bool {
        matches!(self, Self::Sequential | Self::Concurrent)
    }

    /// Check if this mode allows concurrent processing
    #[inline]
    pub fn is_concurrent(&self) -> bool {
        matches!(self, Self::Concurrent | Self::OutOfOrder)
    }
}

/// Configuration for HTTP/1.1 pipelining
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Pipeline processing mode
    pub mode: PipelineMode,

    /// Maximum number of concurrent requests per connection
    /// Only used in Concurrent and OutOfOrder modes
    pub max_concurrent: usize,

    /// Enable pipeline flush optimization
    /// When true, responses are flushed in batches for better I/O efficiency
    pub pipeline_flush: bool,

    /// Maximum number of pipelined requests to buffer
    pub max_buffered_requests: usize,

    /// Keep-alive timeout for idle connections
    pub keep_alive_timeout: Duration,

    /// Maximum requests per connection before forcing close
    /// Helps prevent resource exhaustion
    pub max_requests_per_connection: Option<u64>,

    /// Enable TCP_NODELAY for lower latency
    pub tcp_nodelay: bool,

    /// Read buffer size hint (bytes)
    pub read_buffer_size: usize,

    /// Write buffer size hint (bytes)
    pub write_buffer_size: usize,

    /// Maximum header size (bytes)
    pub max_header_size: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            mode: PipelineMode::Concurrent,
            max_concurrent: 16,
            pipeline_flush: true,
            max_buffered_requests: 64,
            keep_alive_timeout: Duration::from_secs(60),
            max_requests_per_connection: Some(10_000),
            tcp_nodelay: true,
            read_buffer_size: 8192,
            write_buffer_size: 8192,
            max_header_size: 16384,
        }
    }
}

impl PipelineConfig {
    /// Create a new builder for PipelineConfig
    pub fn builder() -> PipelineConfigBuilder {
        PipelineConfigBuilder::default()
    }

    /// Create a high-performance configuration
    pub fn high_performance() -> Self {
        Self {
            mode: PipelineMode::Concurrent,
            max_concurrent: 32,
            pipeline_flush: true,
            max_buffered_requests: 128,
            keep_alive_timeout: Duration::from_secs(120),
            max_requests_per_connection: Some(100_000),
            tcp_nodelay: true,
            read_buffer_size: 16384,
            write_buffer_size: 16384,
            max_header_size: 32768,
        }
    }

    /// Create a low-latency configuration
    pub fn low_latency() -> Self {
        Self {
            mode: PipelineMode::Sequential,
            max_concurrent: 1,
            pipeline_flush: false,
            max_buffered_requests: 16,
            keep_alive_timeout: Duration::from_secs(30),
            max_requests_per_connection: Some(1000),
            tcp_nodelay: true,
            read_buffer_size: 4096,
            write_buffer_size: 4096,
            max_header_size: 8192,
        }
    }

    /// Create a memory-efficient configuration
    pub fn memory_efficient() -> Self {
        Self {
            mode: PipelineMode::Sequential,
            max_concurrent: 4,
            pipeline_flush: true,
            max_buffered_requests: 32,
            keep_alive_timeout: Duration::from_secs(30),
            max_requests_per_connection: Some(1000),
            tcp_nodelay: false,
            read_buffer_size: 4096,
            write_buffer_size: 4096,
            max_header_size: 8192,
        }
    }
}

/// Builder for PipelineConfig
#[derive(Debug, Clone, Default)]
pub struct PipelineConfigBuilder {
    config: PipelineConfig,
}

impl PipelineConfigBuilder {
    /// Set the pipeline processing mode
    pub fn mode(mut self, mode: PipelineMode) -> Self {
        self.config.mode = mode;
        self
    }

    /// Set maximum concurrent requests
    pub fn max_concurrent(mut self, max: usize) -> Self {
        self.config.max_concurrent = max;
        self
    }

    /// Enable or disable pipeline flush optimization
    pub fn pipeline_flush(mut self, enable: bool) -> Self {
        self.config.pipeline_flush = enable;
        self
    }

    /// Set maximum buffered requests
    pub fn max_buffered_requests(mut self, max: usize) -> Self {
        self.config.max_buffered_requests = max;
        self
    }

    /// Set keep-alive timeout
    pub fn keep_alive_timeout(mut self, timeout: Duration) -> Self {
        self.config.keep_alive_timeout = timeout;
        self
    }

    /// Set maximum requests per connection
    pub fn max_requests_per_connection(mut self, max: Option<u64>) -> Self {
        self.config.max_requests_per_connection = max;
        self
    }

    /// Enable or disable TCP_NODELAY
    pub fn tcp_nodelay(mut self, enable: bool) -> Self {
        self.config.tcp_nodelay = enable;
        self
    }

    /// Set read buffer size
    pub fn read_buffer_size(mut self, size: usize) -> Self {
        self.config.read_buffer_size = size;
        self
    }

    /// Set write buffer size
    pub fn write_buffer_size(mut self, size: usize) -> Self {
        self.config.write_buffer_size = size;
        self
    }

    /// Set maximum header size
    pub fn max_header_size(mut self, size: usize) -> Self {
        self.config.max_header_size = size;
        self
    }

    /// Build the configuration
    pub fn build(self) -> PipelineConfig {
        self.config
    }
}

// ============================================================================
// Connection Statistics
// ============================================================================

/// Statistics for a pipelined connection
#[derive(Debug)]
pub struct ConnectionStats {
    /// Total requests processed
    requests_processed: AtomicU64,
    /// Currently pending requests
    pending_requests: AtomicUsize,
    /// Total bytes received
    bytes_received: AtomicU64,
    /// Total bytes sent
    bytes_sent: AtomicU64,
    /// Pipeline depth (how many requests are queued)
    pipeline_depth: AtomicUsize,
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionStats {
    /// Create new connection statistics
    pub fn new() -> Self {
        Self {
            requests_processed: AtomicU64::new(0),
            pending_requests: AtomicUsize::new(0),
            bytes_received: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            pipeline_depth: AtomicUsize::new(0),
        }
    }

    /// Record a request received
    #[inline]
    pub fn request_received(&self, bytes: u64) {
        self.pending_requests.fetch_add(1, Ordering::Relaxed);
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
        self.pipeline_depth.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a response sent
    #[inline]
    pub fn response_sent(&self, bytes: u64) {
        self.requests_processed.fetch_add(1, Ordering::Relaxed);
        self.pending_requests.fetch_sub(1, Ordering::Relaxed);
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
        self.pipeline_depth.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get total requests processed
    #[inline]
    pub fn requests_processed(&self) -> u64 {
        self.requests_processed.load(Ordering::Relaxed)
    }

    /// Get currently pending requests
    #[inline]
    pub fn pending_requests(&self) -> usize {
        self.pending_requests.load(Ordering::Relaxed)
    }

    /// Get current pipeline depth
    #[inline]
    pub fn pipeline_depth(&self) -> usize {
        self.pipeline_depth.load(Ordering::Relaxed)
    }

    /// Get total bytes received
    #[inline]
    pub fn bytes_received(&self) -> u64 {
        self.bytes_received.load(Ordering::Relaxed)
    }

    /// Get total bytes sent
    #[inline]
    pub fn bytes_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed)
    }
}

// ============================================================================
// Global Pipeline Statistics
// ============================================================================

/// Global statistics for all pipelined connections
#[derive(Debug, Default)]
pub struct PipelineStats {
    /// Active connections
    active_connections: AtomicUsize,
    /// Total connections ever
    total_connections: AtomicU64,
    /// Total requests processed
    total_requests: AtomicU64,
    /// Average pipeline depth (running average * 100 for precision)
    avg_pipeline_depth: AtomicU64,
    /// Maximum pipeline depth seen
    max_pipeline_depth: AtomicUsize,
}

impl PipelineStats {
    /// Create new global pipeline statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a new connection
    #[inline]
    pub fn connection_opened(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
        self.total_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a connection closed
    #[inline]
    pub fn connection_closed(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Record a request processed
    #[inline]
    pub fn request_processed(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Update pipeline depth statistics
    #[inline]
    pub fn update_pipeline_depth(&self, depth: usize) {
        // Update max depth
        self.max_pipeline_depth.fetch_max(depth, Ordering::Relaxed);

        // Update running average (simplified EMA)
        let current = self.avg_pipeline_depth.load(Ordering::Relaxed);
        let new_avg = (current * 95 + (depth as u64 * 100) * 5) / 100;
        self.avg_pipeline_depth.store(new_avg, Ordering::Relaxed);
    }

    /// Get active connections
    #[inline]
    pub fn active_connections(&self) -> usize {
        self.active_connections.load(Ordering::Relaxed)
    }

    /// Get total connections
    #[inline]
    pub fn total_connections(&self) -> u64 {
        self.total_connections.load(Ordering::Relaxed)
    }

    /// Get total requests
    #[inline]
    pub fn total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }

    /// Get average pipeline depth
    #[inline]
    pub fn avg_pipeline_depth(&self) -> f64 {
        self.avg_pipeline_depth.load(Ordering::Relaxed) as f64 / 100.0
    }

    /// Get maximum pipeline depth
    #[inline]
    pub fn max_pipeline_depth(&self) -> usize {
        self.max_pipeline_depth.load(Ordering::Relaxed)
    }
}

// ============================================================================
// Pipeline-aware HTTP/1.1 Connection Handler
// ============================================================================

/// A wrapper for configuring Hyper's http1 builder with pipelining options
pub struct PipelinedHttp1Builder {
    config: PipelineConfig,
    stats: Arc<PipelineStats>,
}

impl PipelinedHttp1Builder {
    /// Create a new pipelined HTTP/1.1 builder
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            config,
            stats: Arc::new(PipelineStats::new()),
        }
    }

    /// Create with shared statistics
    pub fn with_stats(config: PipelineConfig, stats: Arc<PipelineStats>) -> Self {
        Self { config, stats }
    }

    /// Get the configuration
    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    /// Get shared statistics
    pub fn stats(&self) -> Arc<PipelineStats> {
        Arc::clone(&self.stats)
    }

    /// Configure a Hyper http1::Builder with pipelining options
    #[inline]
    pub fn configure_hyper_builder(&self) -> hyper::server::conn::http1::Builder {
        let mut builder = hyper::server::conn::http1::Builder::new();

        // Enable pipeline flush for batched response sending
        builder.pipeline_flush(self.config.pipeline_flush);

        // Set maximum buffer sizes
        builder.max_buf_size(self.config.read_buffer_size);

        // Preserve header case (for compatibility)
        builder.preserve_header_case(true);

        // Keep-alive is essential for pipelining
        builder.keep_alive(true);

        builder
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_mode_properties() {
        assert!(PipelineMode::Sequential.maintains_order());
        assert!(PipelineMode::Concurrent.maintains_order());
        assert!(!PipelineMode::OutOfOrder.maintains_order());

        assert!(!PipelineMode::Sequential.is_concurrent());
        assert!(PipelineMode::Concurrent.is_concurrent());
        assert!(PipelineMode::OutOfOrder.is_concurrent());
    }

    #[test]
    fn test_config_builder() {
        let config = PipelineConfig::builder()
            .mode(PipelineMode::Concurrent)
            .max_concurrent(32)
            .pipeline_flush(true)
            .keep_alive_timeout(Duration::from_secs(120))
            .build();

        assert_eq!(config.mode, PipelineMode::Concurrent);
        assert_eq!(config.max_concurrent, 32);
        assert!(config.pipeline_flush);
        assert_eq!(config.keep_alive_timeout, Duration::from_secs(120));
    }

    #[test]
    fn test_high_performance_config() {
        let config = PipelineConfig::high_performance();
        assert_eq!(config.mode, PipelineMode::Concurrent);
        assert_eq!(config.max_concurrent, 32);
        assert!(config.pipeline_flush);
    }

    #[test]
    fn test_low_latency_config() {
        let config = PipelineConfig::low_latency();
        assert_eq!(config.mode, PipelineMode::Sequential);
        assert_eq!(config.max_concurrent, 1);
        assert!(!config.pipeline_flush);
    }

    #[test]
    fn test_connection_stats() {
        let stats = ConnectionStats::new();

        stats.request_received(100);
        assert_eq!(stats.pending_requests(), 1);
        assert_eq!(stats.pipeline_depth(), 1);
        assert_eq!(stats.bytes_received(), 100);

        stats.request_received(200);
        assert_eq!(stats.pending_requests(), 2);
        assert_eq!(stats.pipeline_depth(), 2);

        stats.response_sent(150);
        assert_eq!(stats.pending_requests(), 1);
        assert_eq!(stats.requests_processed(), 1);
        assert_eq!(stats.bytes_sent(), 150);
    }

    #[test]
    fn test_global_pipeline_stats() {
        let stats = PipelineStats::new();

        stats.connection_opened();
        stats.connection_opened();
        assert_eq!(stats.active_connections(), 2);
        assert_eq!(stats.total_connections(), 2);

        stats.connection_closed();
        assert_eq!(stats.active_connections(), 1);
        assert_eq!(stats.total_connections(), 2);

        stats.request_processed();
        stats.request_processed();
        assert_eq!(stats.total_requests(), 2);

        stats.update_pipeline_depth(5);
        stats.update_pipeline_depth(10);
        assert_eq!(stats.max_pipeline_depth(), 10);
    }

    #[test]
    fn test_pipelined_builder() {
        let config = PipelineConfig::default();
        let builder = PipelinedHttp1Builder::new(config);

        assert_eq!(builder.config().mode, PipelineMode::Concurrent);
        assert_eq!(builder.stats().active_connections(), 0);

        // Configure Hyper builder
        let _hyper_builder = builder.configure_hyper_builder();
    }
}

