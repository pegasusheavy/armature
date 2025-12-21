//! Batched Socket Operations
//!
//! This module provides utilities for reducing syscall overhead by batching
//! socket read/write operations.
//!
//! # Key Techniques
//!
//! - **Scatter-Gather I/O**: `readv`/`writev` for multiple buffers in one call
//! - **Receive Coalescing**: Accumulate data before processing
//! - **Send Batching**: Combine multiple writes into single syscall
//! - **Cork/Uncork**: TCP_CORK for optimal packet sizes
//!
//! # Performance Impact
//!
//! Each syscall has ~100-200ns overhead. Batching can reduce this by 5-10x
//! for workloads with many small operations.

use bytes::{Bytes, BytesMut};
use std::io::{self, IoSlice, IoSliceMut};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(unix)]
use std::os::unix::io::RawFd;

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for batched socket operations.
#[derive(Debug, Clone)]
pub struct BatchSocketConfig {
    /// Maximum bytes to accumulate before sending
    pub max_write_buffer: usize,
    /// Maximum number of buffers for writev
    pub max_iovec: usize,
    /// Minimum bytes before triggering send
    pub min_batch_size: usize,
    /// Use TCP_CORK for packet coalescing
    pub use_cork: bool,
    /// Use TCP_NODELAY after uncorking
    pub nodelay_after_uncork: bool,
    /// Maximum time to hold data before flush (microseconds)
    pub max_delay_us: u64,
}

impl Default for BatchSocketConfig {
    fn default() -> Self {
        Self {
            max_write_buffer: 64 * 1024, // 64KB
            max_iovec: 16,               // IOV_MAX is typically 1024, but 16 is practical
            min_batch_size: 1024,        // 1KB minimum before batching
            use_cork: true,
            nodelay_after_uncork: true,
            max_delay_us: 1000, // 1ms max delay
        }
    }
}

impl BatchSocketConfig {
    /// Create new configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Configuration for maximum throughput.
    pub fn throughput() -> Self {
        Self {
            max_write_buffer: 256 * 1024,
            max_iovec: 32,
            min_batch_size: 4096,
            use_cork: true,
            nodelay_after_uncork: false,
            max_delay_us: 5000, // 5ms acceptable delay
        }
    }

    /// Configuration for low latency.
    pub fn low_latency() -> Self {
        Self {
            max_write_buffer: 16 * 1024,
            max_iovec: 8,
            min_batch_size: 0, // No batching delay
            use_cork: false,
            nodelay_after_uncork: true,
            max_delay_us: 0,
        }
    }

    /// Builder: set max write buffer.
    pub fn max_write_buffer(mut self, size: usize) -> Self {
        self.max_write_buffer = size;
        self
    }

    /// Builder: set max iovec count.
    pub fn max_iovec(mut self, count: usize) -> Self {
        self.max_iovec = count;
        self
    }

    /// Builder: set minimum batch size.
    pub fn min_batch_size(mut self, size: usize) -> Self {
        self.min_batch_size = size;
        self
    }

    /// Builder: enable/disable TCP_CORK.
    pub fn use_cork(mut self, enabled: bool) -> Self {
        self.use_cork = enabled;
        self
    }

    /// Builder: set max delay.
    pub fn max_delay_us(mut self, us: u64) -> Self {
        self.max_delay_us = us;
        self
    }
}

// ============================================================================
// Scatter-Gather Writer
// ============================================================================

/// A writer that batches multiple buffers for a single writev call.
#[derive(Debug)]
pub struct ScatterWriter {
    /// Pending buffers to write
    buffers: Vec<Bytes>,
    /// Total bytes pending
    pending_bytes: usize,
    /// Configuration
    config: BatchSocketConfig,
    /// Statistics
    stats: WriterStats,
}

#[derive(Debug, Default)]
struct WriterStats {
    writes_batched: u64,
    bytes_written: u64,
    syscalls_saved: u64,
}

impl ScatterWriter {
    /// Create new scatter writer.
    pub fn new(config: BatchSocketConfig) -> Self {
        Self {
            buffers: Vec::with_capacity(config.max_iovec),
            pending_bytes: 0,
            config,
            stats: WriterStats::default(),
        }
    }

    /// Add a buffer to the batch.
    ///
    /// Returns true if the batch should be flushed.
    #[inline]
    pub fn push(&mut self, data: Bytes) -> bool {
        self.pending_bytes += data.len();
        self.buffers.push(data);
        self.stats.writes_batched += 1;

        self.should_flush()
    }

    /// Add multiple buffers.
    pub fn push_many(&mut self, data: impl IntoIterator<Item = Bytes>) -> bool {
        for buf in data {
            self.pending_bytes += buf.len();
            self.buffers.push(buf);
            self.stats.writes_batched += 1;
        }
        self.should_flush()
    }

    /// Check if batch should be flushed.
    #[inline]
    pub fn should_flush(&self) -> bool {
        self.pending_bytes >= self.config.max_write_buffer
            || self.buffers.len() >= self.config.max_iovec
    }

    /// Get pending byte count.
    #[inline]
    pub fn pending_bytes(&self) -> usize {
        self.pending_bytes
    }

    /// Get pending buffer count.
    #[inline]
    pub fn pending_count(&self) -> usize {
        self.buffers.len()
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    /// Prepare IoSlices for writev.
    pub fn as_io_slices(&self) -> Vec<IoSlice<'_>> {
        self.buffers.iter().map(|b| IoSlice::new(b)).collect()
    }

    /// Flush to a writer using writev.
    #[cfg(unix)]
    pub fn flush_to_fd(&mut self, fd: RawFd) -> io::Result<usize> {
        if self.buffers.is_empty() {
            return Ok(0);
        }

        let slices = self.as_io_slices();
        let written = writev(fd, &slices)?;

        self.stats.bytes_written += written as u64;
        self.stats.syscalls_saved += self.buffers.len().saturating_sub(1) as u64;
        SOCKET_STATS.record_writev(self.buffers.len(), written);

        // Remove fully written buffers
        self.consume(written);

        Ok(written)
    }

    /// Flush to an async writer.
    pub async fn flush_async<W: tokio::io::AsyncWriteExt + Unpin>(
        &mut self,
        writer: &mut W,
    ) -> io::Result<usize> {
        if self.buffers.is_empty() {
            return Ok(0);
        }

        // Collect into contiguous for async (tokio doesn't have vectored async write)
        let total: usize = self.buffers.iter().map(|b| b.len()).sum();
        let mut combined = BytesMut::with_capacity(total);
        for buf in &self.buffers {
            combined.extend_from_slice(buf);
        }

        writer.write_all(&combined).await?;

        self.stats.bytes_written += total as u64;
        self.stats.syscalls_saved += self.buffers.len().saturating_sub(1) as u64;
        SOCKET_STATS.record_write_batch(self.buffers.len(), total);

        self.buffers.clear();
        self.pending_bytes = 0;

        Ok(total)
    }

    /// Consume written bytes, removing fully-sent buffers.
    fn consume(&mut self, mut bytes: usize) {
        while bytes > 0 && !self.buffers.is_empty() {
            let buf_len = self.buffers[0].len();
            if bytes >= buf_len {
                bytes -= buf_len;
                self.pending_bytes -= buf_len;
                self.buffers.remove(0);
            } else {
                // Partial write - slice the buffer
                self.buffers[0] = self.buffers[0].slice(bytes..);
                self.pending_bytes -= bytes;
                break;
            }
        }
    }

    /// Get statistics.
    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.stats.writes_batched,
            self.stats.bytes_written,
            self.stats.syscalls_saved,
        )
    }

    /// Clear all pending data.
    pub fn clear(&mut self) {
        self.buffers.clear();
        self.pending_bytes = 0;
    }
}

// ============================================================================
// Gather Reader
// ============================================================================

/// A reader that uses readv for scatter-gather reads.
#[derive(Debug)]
pub struct GatherReader {
    /// Pre-allocated buffers for reading
    buffers: Vec<BytesMut>,
    /// Buffer size
    buffer_size: usize,
    /// Number of buffers
    buffer_count: usize,
    /// Statistics
    reads: u64,
    bytes_read: u64,
}

impl GatherReader {
    /// Create new gather reader.
    pub fn new(buffer_count: usize, buffer_size: usize) -> Self {
        let buffers = (0..buffer_count)
            .map(|_| BytesMut::with_capacity(buffer_size))
            .collect();

        Self {
            buffers,
            buffer_size,
            buffer_count,
            reads: 0,
            bytes_read: 0,
        }
    }

    /// Prepare buffers for readv.
    pub fn as_io_slices_mut(&mut self) -> Vec<IoSliceMut<'_>> {
        // Reset buffers to full capacity
        for buf in &mut self.buffers {
            buf.clear();
            buf.resize(self.buffer_size, 0);
        }

        self.buffers
            .iter_mut()
            .map(|b| IoSliceMut::new(b.as_mut()))
            .collect()
    }

    /// Read from fd using readv.
    #[cfg(unix)]
    pub fn read_from_fd(&mut self, fd: RawFd) -> io::Result<Vec<Bytes>> {
        let mut slices = self.as_io_slices_mut();
        let n = readv(fd, &mut slices)?;

        if n == 0 {
            return Ok(Vec::new());
        }

        self.reads += 1;
        self.bytes_read += n as u64;
        SOCKET_STATS.record_readv(self.buffer_count, n);

        // Collect filled buffers
        let mut result = Vec::new();
        let mut remaining = n;

        for buf in &mut self.buffers {
            if remaining == 0 {
                break;
            }
            let take = remaining.min(buf.len());
            buf.truncate(take);
            result.push(buf.split().freeze());
            remaining -= take;
        }

        Ok(result)
    }

    /// Read into a single buffer (for async).
    pub async fn read_async<R: tokio::io::AsyncReadExt + Unpin>(
        &mut self,
        reader: &mut R,
    ) -> io::Result<Bytes> {
        let mut buf = BytesMut::with_capacity(self.buffer_size * self.buffer_count);
        buf.resize(self.buffer_size * self.buffer_count, 0);

        let n = reader.read(&mut buf).await?;
        buf.truncate(n);

        self.reads += 1;
        self.bytes_read += n as u64;
        SOCKET_STATS.record_read_batch(1, n);

        Ok(buf.freeze())
    }

    /// Get statistics.
    pub fn stats(&self) -> (u64, u64) {
        (self.reads, self.bytes_read)
    }
}

// ============================================================================
// TCP Cork Manager
// ============================================================================

/// Manages TCP_CORK for optimal packet coalescing.
///
/// Cork delays sending until uncorked or buffer full, creating
/// optimal-sized packets for throughput.
#[derive(Debug)]
pub struct CorkManager {
    /// File descriptor
    #[cfg(unix)]
    fd: RawFd,
    /// Currently corked
    corked: bool,
    /// Use NODELAY after uncork
    nodelay_after: bool,
}

impl CorkManager {
    /// Create new cork manager.
    #[cfg(unix)]
    pub fn new(fd: RawFd, nodelay_after: bool) -> Self {
        Self {
            fd,
            corked: false,
            nodelay_after,
        }
    }

    #[cfg(not(unix))]
    pub fn new(_fd: i32, _nodelay_after: bool) -> Self {
        Self {
            corked: false,
            nodelay_after: false,
        }
    }

    /// Cork the socket (delay sending).
    #[cfg(unix)]
    pub fn cork(&mut self) -> io::Result<()> {
        if !self.corked {
            set_tcp_cork(self.fd, true)?;
            self.corked = true;
            SOCKET_STATS.record_cork();
        }
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn cork(&mut self) -> io::Result<()> {
        self.corked = true;
        Ok(())
    }

    /// Uncork the socket (flush pending data).
    #[cfg(unix)]
    pub fn uncork(&mut self) -> io::Result<()> {
        if self.corked {
            set_tcp_cork(self.fd, false)?;
            self.corked = false;
            SOCKET_STATS.record_uncork();

            if self.nodelay_after {
                set_tcp_nodelay(self.fd, true)?;
            }
        }
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn uncork(&mut self) -> io::Result<()> {
        self.corked = false;
        Ok(())
    }

    /// Check if currently corked.
    #[inline]
    pub fn is_corked(&self) -> bool {
        self.corked
    }

    /// Cork, execute function, then uncork.
    pub fn with_cork<F, R>(&mut self, f: F) -> io::Result<R>
    where
        F: FnOnce() -> R,
    {
        self.cork()?;
        let result = f();
        self.uncork()?;
        Ok(result)
    }
}

impl Drop for CorkManager {
    fn drop(&mut self) {
        // Ensure uncorked on drop
        let _ = self.uncork();
    }
}

// ============================================================================
// Batched Send Queue
// ============================================================================

/// A queue that batches sends for efficiency.
#[derive(Debug)]
pub struct SendQueue {
    /// Pending data
    queue: Vec<Bytes>,
    /// Total pending bytes
    pending_bytes: usize,
    /// Configuration
    config: BatchSocketConfig,
}

impl SendQueue {
    /// Create new send queue.
    pub fn new(config: BatchSocketConfig) -> Self {
        Self {
            queue: Vec::with_capacity(32),
            pending_bytes: 0,
            config,
        }
    }

    /// Enqueue data for sending.
    #[inline]
    pub fn enqueue(&mut self, data: Bytes) {
        self.pending_bytes += data.len();
        self.queue.push(data);
    }

    /// Enqueue from BytesMut.
    #[inline]
    pub fn enqueue_mut(&mut self, data: BytesMut) {
        self.enqueue(data.freeze());
    }

    /// Check if should flush.
    #[inline]
    pub fn should_flush(&self) -> bool {
        self.pending_bytes >= self.config.min_batch_size
            || self.queue.len() >= self.config.max_iovec
    }

    /// Get pending bytes.
    #[inline]
    pub fn pending_bytes(&self) -> usize {
        self.pending_bytes
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Drain queue for writing.
    pub fn drain(&mut self) -> Vec<Bytes> {
        self.pending_bytes = 0;
        std::mem::take(&mut self.queue)
    }

    /// Write all to fd using writev.
    #[cfg(unix)]
    pub fn flush_to_fd(&mut self, fd: RawFd) -> io::Result<usize> {
        if self.queue.is_empty() {
            return Ok(0);
        }

        let slices: Vec<IoSlice<'_>> = self.queue.iter().map(|b| IoSlice::new(b)).collect();
        let written = writev(fd, &slices)?;

        SOCKET_STATS.record_writev(self.queue.len(), written);

        // Handle partial writes
        let mut consumed = 0;
        let mut remaining = written;

        while remaining > 0 && consumed < self.queue.len() {
            let buf_len = self.queue[consumed].len();
            if remaining >= buf_len {
                remaining -= buf_len;
                consumed += 1;
            } else {
                self.queue[consumed] = self.queue[consumed].slice(remaining..);
                remaining = 0;
            }
        }

        self.queue.drain(..consumed);
        self.pending_bytes = self.queue.iter().map(|b| b.len()).sum();

        Ok(written)
    }

    /// Clear queue.
    pub fn clear(&mut self) {
        self.queue.clear();
        self.pending_bytes = 0;
    }
}

// ============================================================================
// Receive Accumulator
// ============================================================================

/// Accumulates received data for batch processing.
#[derive(Debug)]
pub struct ReceiveAccumulator {
    /// Accumulated data
    buffer: BytesMut,
    /// Minimum bytes before processing
    min_process: usize,
    /// Maximum buffer size
    max_buffer: usize,
}

impl ReceiveAccumulator {
    /// Create new accumulator.
    pub fn new(min_process: usize, max_buffer: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(max_buffer),
            min_process,
            max_buffer,
        }
    }

    /// Add received data.
    #[inline]
    pub fn push(&mut self, data: &[u8]) {
        if self.buffer.len() + data.len() <= self.max_buffer {
            self.buffer.extend_from_slice(data);
        }
    }

    /// Add Bytes.
    #[inline]
    pub fn push_bytes(&mut self, data: Bytes) {
        self.push(&data);
    }

    /// Check if ready for processing.
    #[inline]
    pub fn ready(&self) -> bool {
        self.buffer.len() >= self.min_process
    }

    /// Get accumulated length.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Take accumulated data.
    pub fn take(&mut self) -> Bytes {
        self.buffer.split().freeze()
    }

    /// Peek at accumulated data.
    pub fn peek(&self) -> &[u8] {
        &self.buffer
    }

    /// Clear buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

// ============================================================================
// Platform-Specific Helpers
// ============================================================================

/// Vectored write (writev).
#[cfg(unix)]
pub fn writev(fd: RawFd, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    let ret = unsafe {
        libc::writev(
            fd,
            bufs.as_ptr() as *const libc::iovec,
            bufs.len() as libc::c_int,
        )
    };

    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret as usize)
    }
}

/// Vectored read (readv).
#[cfg(unix)]
pub fn readv(fd: RawFd, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
    let ret = unsafe {
        libc::readv(
            fd,
            bufs.as_mut_ptr() as *mut libc::iovec,
            bufs.len() as libc::c_int,
        )
    };

    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret as usize)
    }
}

/// Set TCP_CORK option.
#[cfg(unix)]
pub fn set_tcp_cork(fd: RawFd, cork: bool) -> io::Result<()> {
    let val: libc::c_int = if cork { 1 } else { 0 };
    let ret = unsafe {
        libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_CORK,
            &val as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        )
    };

    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Set TCP_NODELAY option.
#[cfg(unix)]
pub fn set_tcp_nodelay(fd: RawFd, nodelay: bool) -> io::Result<()> {
    let val: libc::c_int = if nodelay { 1 } else { 0 };
    let ret = unsafe {
        libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_NODELAY,
            &val as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        )
    };

    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Send with MSG_MORE flag (like TCP_CORK per-send).
#[cfg(unix)]
pub fn send_more(fd: RawFd, data: &[u8]) -> io::Result<usize> {
    let ret = unsafe {
        libc::send(
            fd,
            data.as_ptr() as *const libc::c_void,
            data.len(),
            libc::MSG_MORE,
        )
    };

    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        SOCKET_STATS.record_send_more();
        Ok(ret as usize)
    }
}

/// Final send (without MSG_MORE).
#[cfg(unix)]
pub fn send_final(fd: RawFd, data: &[u8]) -> io::Result<usize> {
    let ret = unsafe { libc::send(fd, data.as_ptr() as *const libc::c_void, data.len(), 0) };

    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(ret as usize)
    }
}

// ============================================================================
// Statistics
// ============================================================================

/// Global socket batching statistics.
#[derive(Debug, Default)]
pub struct SocketBatchStats {
    /// writev calls
    writev_calls: AtomicU64,
    /// Buffers sent via writev
    writev_buffers: AtomicU64,
    /// Bytes sent via writev
    writev_bytes: AtomicU64,
    /// readv calls
    readv_calls: AtomicU64,
    /// Buffers read via readv
    readv_buffers: AtomicU64,
    /// Bytes read via readv
    readv_bytes: AtomicU64,
    /// Batched write operations
    write_batches: AtomicU64,
    /// Read batches
    read_batches: AtomicU64,
    /// Cork operations
    cork_ops: AtomicU64,
    /// Uncork operations
    uncork_ops: AtomicU64,
    /// MSG_MORE sends
    send_more_ops: AtomicU64,
}

impl SocketBatchStats {
    fn record_writev(&self, buffers: usize, bytes: usize) {
        self.writev_calls.fetch_add(1, Ordering::Relaxed);
        self.writev_buffers
            .fetch_add(buffers as u64, Ordering::Relaxed);
        self.writev_bytes.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    fn record_readv(&self, buffers: usize, bytes: usize) {
        self.readv_calls.fetch_add(1, Ordering::Relaxed);
        self.readv_buffers
            .fetch_add(buffers as u64, Ordering::Relaxed);
        self.readv_bytes.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    fn record_write_batch(&self, count: usize, _bytes: usize) {
        self.write_batches
            .fetch_add(count as u64, Ordering::Relaxed);
    }

    fn record_read_batch(&self, count: usize, _bytes: usize) {
        self.read_batches.fetch_add(count as u64, Ordering::Relaxed);
    }

    fn record_cork(&self) {
        self.cork_ops.fetch_add(1, Ordering::Relaxed);
    }

    fn record_uncork(&self) {
        self.uncork_ops.fetch_add(1, Ordering::Relaxed);
    }

    fn record_send_more(&self) {
        self.send_more_ops.fetch_add(1, Ordering::Relaxed);
    }

    /// Get writev calls.
    pub fn writev_calls(&self) -> u64 {
        self.writev_calls.load(Ordering::Relaxed)
    }

    /// Get writev buffers.
    pub fn writev_buffers(&self) -> u64 {
        self.writev_buffers.load(Ordering::Relaxed)
    }

    /// Get writev bytes.
    pub fn writev_bytes(&self) -> u64 {
        self.writev_bytes.load(Ordering::Relaxed)
    }

    /// Get readv calls.
    pub fn readv_calls(&self) -> u64 {
        self.readv_calls.load(Ordering::Relaxed)
    }

    /// Get readv buffers.
    pub fn readv_buffers(&self) -> u64 {
        self.readv_buffers.load(Ordering::Relaxed)
    }

    /// Get readv bytes.
    pub fn readv_bytes(&self) -> u64 {
        self.readv_bytes.load(Ordering::Relaxed)
    }

    /// Get cork operations.
    pub fn cork_ops(&self) -> u64 {
        self.cork_ops.load(Ordering::Relaxed)
    }

    /// Get uncork operations.
    pub fn uncork_ops(&self) -> u64 {
        self.uncork_ops.load(Ordering::Relaxed)
    }

    /// Get MSG_MORE sends.
    pub fn send_more_ops(&self) -> u64 {
        self.send_more_ops.load(Ordering::Relaxed)
    }

    /// Average buffers per writev.
    pub fn avg_buffers_per_writev(&self) -> f64 {
        let calls = self.writev_calls() as f64;
        if calls > 0.0 {
            self.writev_buffers() as f64 / calls
        } else {
            0.0
        }
    }

    /// Estimated syscalls saved.
    pub fn syscalls_saved(&self) -> u64 {
        self.writev_buffers().saturating_sub(self.writev_calls())
            + self.readv_buffers().saturating_sub(self.readv_calls())
    }
}

/// Global statistics.
static SOCKET_STATS: SocketBatchStats = SocketBatchStats {
    writev_calls: AtomicU64::new(0),
    writev_buffers: AtomicU64::new(0),
    writev_bytes: AtomicU64::new(0),
    readv_calls: AtomicU64::new(0),
    readv_buffers: AtomicU64::new(0),
    readv_bytes: AtomicU64::new(0),
    write_batches: AtomicU64::new(0),
    read_batches: AtomicU64::new(0),
    cork_ops: AtomicU64::new(0),
    uncork_ops: AtomicU64::new(0),
    send_more_ops: AtomicU64::new(0),
};

/// Get global socket batch statistics.
pub fn socket_batch_stats() -> &'static SocketBatchStats {
    &SOCKET_STATS
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_socket_config_default() {
        let config = BatchSocketConfig::default();
        assert_eq!(config.max_write_buffer, 64 * 1024);
        assert_eq!(config.max_iovec, 16);
        assert!(config.use_cork);
    }

    #[test]
    fn test_batch_socket_config_throughput() {
        let config = BatchSocketConfig::throughput();
        assert_eq!(config.max_write_buffer, 256 * 1024);
        assert!(config.use_cork);
    }

    #[test]
    fn test_batch_socket_config_low_latency() {
        let config = BatchSocketConfig::low_latency();
        assert_eq!(config.min_batch_size, 0);
        assert!(!config.use_cork);
    }

    #[test]
    fn test_scatter_writer_basic() {
        let config = BatchSocketConfig::default();
        let mut writer = ScatterWriter::new(config);

        assert!(writer.is_empty());

        writer.push(Bytes::from_static(b"hello"));
        assert!(!writer.is_empty());
        assert_eq!(writer.pending_bytes(), 5);
        assert_eq!(writer.pending_count(), 1);
    }

    #[test]
    fn test_scatter_writer_should_flush() {
        let config = BatchSocketConfig::default().max_iovec(2);
        let mut writer = ScatterWriter::new(config);

        writer.push(Bytes::from_static(b"hello"));
        assert!(!writer.should_flush());

        writer.push(Bytes::from_static(b"world"));
        assert!(writer.should_flush()); // Reached max_iovec
    }

    #[test]
    fn test_scatter_writer_io_slices() {
        let config = BatchSocketConfig::default();
        let mut writer = ScatterWriter::new(config);

        writer.push(Bytes::from_static(b"hello"));
        writer.push(Bytes::from_static(b" "));
        writer.push(Bytes::from_static(b"world"));

        let slices = writer.as_io_slices();
        assert_eq!(slices.len(), 3);
    }

    #[test]
    fn test_gather_reader_basic() {
        let reader = GatherReader::new(4, 1024);
        assert_eq!(reader.buffer_count, 4);
        assert_eq!(reader.buffer_size, 1024);
    }

    #[test]
    fn test_cork_manager() {
        // Just test the state tracking (no actual socket)
        let manager = CorkManager::new(0, true);
        assert!(!manager.is_corked());
    }

    #[test]
    fn test_send_queue() {
        let config = BatchSocketConfig::default().min_batch_size(10);
        let mut queue = SendQueue::new(config);

        assert!(queue.is_empty());

        queue.enqueue(Bytes::from_static(b"hello"));
        assert!(!queue.should_flush()); // Only 5 bytes, need 10

        queue.enqueue(Bytes::from_static(b"world!"));
        assert!(queue.should_flush()); // 11 bytes >= 10
    }

    #[test]
    fn test_receive_accumulator() {
        let mut acc = ReceiveAccumulator::new(10, 1024);

        assert!(acc.is_empty());
        assert!(!acc.ready());

        acc.push(b"hello");
        assert_eq!(acc.len(), 5);
        assert!(!acc.ready());

        acc.push(b"world!");
        assert!(acc.ready()); // 11 bytes >= 10

        let data = acc.take();
        assert_eq!(&data[..], b"helloworld!");
        assert!(acc.is_empty());
    }

    #[test]
    fn test_socket_batch_stats() {
        let stats = socket_batch_stats();
        let _ = stats.writev_calls();
        let _ = stats.readv_calls();
        let _ = stats.avg_buffers_per_writev();
        let _ = stats.syscalls_saved();
    }
}
