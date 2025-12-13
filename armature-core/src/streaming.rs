//! Streaming HTTP Responses
//!
//! This module provides support for streaming HTTP responses, enabling efficient
//! delivery of large data sets, real-time data, and chunked transfers.
//!
//! # Features
//!
//! - Chunked transfer encoding
//! - Async stream-based response bodies
//! - JSON array streaming (NDJSON)
//! - Text/line streaming
//! - Binary data streaming
//! - Progress callbacks
//!
//! # Examples
//!
//! ## Basic Streaming
//!
//! ```ignore
//! use armature_core::streaming::{StreamingResponse, ByteStream};
//!
//! async fn stream_data() -> StreamingResponse {
//!     let (stream, sender) = ByteStream::new();
//!
//!     tokio::spawn(async move {
//!         for i in 0..100 {
//!             sender.send(format!("chunk {}\n", i).into_bytes()).await;
//!         }
//!     });
//!
//!     StreamingResponse::new(stream)
//!         .content_type("text/plain")
//! }
//! ```
//!
//! ## JSON Streaming (NDJSON)
//!
//! ```ignore
//! use armature_core::streaming::{StreamingResponse, JsonStream};
//!
//! async fn stream_json() -> StreamingResponse {
//!     let (stream, sender) = JsonStream::new();
//!
//!     tokio::spawn(async move {
//!         for user in load_users() {
//!             sender.send_json(&user).await;
//!         }
//!     });
//!
//!     StreamingResponse::ndjson(stream)
//! }
//! ```

use crate::{Error, HttpResponse};
use bytes::Bytes;
use futures_util::Stream;
use serde::Serialize;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::mpsc;

// ============================================================================
// Streaming Body Types
// ============================================================================

/// A chunk of streaming data.
#[derive(Debug, Clone)]
pub enum StreamChunk {
    /// Raw bytes
    Bytes(Bytes),
    /// End of stream
    End,
    /// Error occurred
    Error(String),
}

impl From<Vec<u8>> for StreamChunk {
    fn from(v: Vec<u8>) -> Self {
        StreamChunk::Bytes(Bytes::from(v))
    }
}

impl From<Bytes> for StreamChunk {
    fn from(b: Bytes) -> Self {
        StreamChunk::Bytes(b)
    }
}

impl From<String> for StreamChunk {
    fn from(s: String) -> Self {
        StreamChunk::Bytes(Bytes::from(s))
    }
}

impl From<&str> for StreamChunk {
    fn from(s: &str) -> Self {
        StreamChunk::Bytes(Bytes::from(s.to_owned()))
    }
}

// ============================================================================
// Byte Stream
// ============================================================================

/// A stream of raw bytes for streaming responses.
///
/// # Example
///
/// ```
/// use armature_core::streaming::ByteStream;
///
/// # tokio_test::block_on(async {
/// let (stream, sender) = ByteStream::new();
///
/// // Send data in background
/// tokio::spawn(async move {
///     sender.send(b"Hello, ".to_vec()).await.ok();
///     sender.send(b"World!".to_vec()).await.ok();
///     sender.close().await;
/// });
/// # });
/// ```
pub struct ByteStream {
    receiver: mpsc::Receiver<StreamChunk>,
}

/// Sender half of a byte stream.
pub struct ByteStreamSender {
    sender: mpsc::Sender<StreamChunk>,
    bytes_sent: Arc<AtomicU64>,
}

impl ByteStream {
    /// Create a new byte stream with default buffer size (64).
    pub fn new() -> (Self, ByteStreamSender) {
        Self::with_buffer_size(64)
    }

    /// Create a new byte stream with custom buffer size.
    pub fn with_buffer_size(size: usize) -> (Self, ByteStreamSender) {
        let (sender, receiver) = mpsc::channel(size);
        let bytes_sent = Arc::new(AtomicU64::new(0));
        (
            Self { receiver },
            ByteStreamSender {
                sender,
                bytes_sent,
            },
        )
    }
}

impl Default for ByteStream {
    fn default() -> Self {
        let (stream, _) = Self::new();
        stream
    }
}

impl Stream for ByteStream {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.receiver).poll_recv(cx) {
            Poll::Ready(Some(chunk)) => match chunk {
                StreamChunk::Bytes(bytes) => Poll::Ready(Some(Ok(bytes))),
                StreamChunk::End => Poll::Ready(None),
                StreamChunk::Error(e) => Poll::Ready(Some(Err(Error::Internal(e)))),
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl ByteStreamSender {
    /// Send bytes to the stream.
    pub async fn send(&self, data: impl Into<Vec<u8>>) -> Result<(), Error> {
        let bytes = data.into();
        let len = bytes.len() as u64;
        self.sender
            .send(StreamChunk::Bytes(Bytes::from(bytes)))
            .await
            .map_err(|e| Error::Internal(format!("Failed to send to stream: {}", e)))?;
        self.bytes_sent.fetch_add(len, Ordering::Relaxed);
        Ok(())
    }

    /// Send bytes from a Bytes object.
    pub async fn send_bytes(&self, bytes: Bytes) -> Result<(), Error> {
        let len = bytes.len() as u64;
        self.sender
            .send(StreamChunk::Bytes(bytes))
            .await
            .map_err(|e| Error::Internal(format!("Failed to send to stream: {}", e)))?;
        self.bytes_sent.fetch_add(len, Ordering::Relaxed);
        Ok(())
    }

    /// Send a string to the stream.
    pub async fn send_str(&self, s: &str) -> Result<(), Error> {
        self.send(s.as_bytes().to_vec()).await
    }

    /// Signal an error to the stream.
    pub async fn send_error(&self, error: impl Into<String>) -> Result<(), Error> {
        self.sender
            .send(StreamChunk::Error(error.into()))
            .await
            .map_err(|e| Error::Internal(format!("Failed to send error: {}", e)))
    }

    /// Close the stream.
    pub async fn close(&self) {
        let _ = self.sender.send(StreamChunk::End).await;
    }

    /// Get the total bytes sent so far.
    pub fn bytes_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed)
    }

    /// Check if the receiver has been dropped.
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }
}

// ============================================================================
// JSON Stream (NDJSON)
// ============================================================================

/// A stream for sending JSON objects as newline-delimited JSON (NDJSON).
///
/// Each JSON object is serialized and followed by a newline character.
/// This format is compatible with tools like `jq` and is easy to parse.
///
/// # Example
///
/// ```
/// use armature_core::streaming::JsonStream;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct User { id: u64, name: String }
///
/// # tokio_test::block_on(async {
/// let (stream, sender) = JsonStream::new();
///
/// tokio::spawn(async move {
///     sender.send_json(&User { id: 1, name: "Alice".into() }).await.ok();
///     sender.send_json(&User { id: 2, name: "Bob".into() }).await.ok();
///     sender.close().await;
/// });
/// # });
/// ```
pub struct JsonStream {
    inner: ByteStream,
}

/// Sender half of a JSON stream.
pub struct JsonStreamSender {
    inner: ByteStreamSender,
    items_sent: Arc<AtomicU64>,
}

impl JsonStream {
    /// Create a new JSON stream.
    pub fn new() -> (Self, JsonStreamSender) {
        Self::with_buffer_size(64)
    }

    /// Create a new JSON stream with custom buffer size.
    pub fn with_buffer_size(size: usize) -> (Self, JsonStreamSender) {
        let (stream, sender) = ByteStream::with_buffer_size(size);
        let items_sent = Arc::new(AtomicU64::new(0));
        (
            Self { inner: stream },
            JsonStreamSender {
                inner: sender,
                items_sent,
            },
        )
    }

    /// Get the inner byte stream.
    pub fn into_inner(self) -> ByteStream {
        self.inner
    }
}

impl Default for JsonStream {
    fn default() -> Self {
        let (stream, _) = Self::new();
        stream
    }
}

impl Stream for JsonStream {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

impl JsonStreamSender {
    /// Send a JSON-serializable value.
    pub async fn send_json<T: Serialize>(&self, value: &T) -> Result<(), Error> {
        let json =
            serde_json::to_string(value).map_err(|e| Error::Serialization(e.to_string()))?;
        self.inner.send(format!("{}\n", json)).await?;
        self.items_sent.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Send a raw JSON string (must be valid JSON).
    pub async fn send_raw(&self, json: &str) -> Result<(), Error> {
        self.inner.send(format!("{}\n", json.trim())).await?;
        self.items_sent.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Signal an error as a JSON object.
    pub async fn send_error(&self, error: impl Into<String>) -> Result<(), Error> {
        let error_json = serde_json::json!({
            "error": error.into()
        });
        self.send_json(&error_json).await
    }

    /// Close the stream.
    pub async fn close(&self) {
        self.inner.close().await;
    }

    /// Get the total items sent so far.
    pub fn items_sent(&self) -> u64 {
        self.items_sent.load(Ordering::Relaxed)
    }

    /// Check if the receiver has been dropped.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

// ============================================================================
// Text/Line Stream
// ============================================================================

/// A stream for sending text lines.
///
/// Each message is followed by a newline character.
pub struct TextStream {
    inner: ByteStream,
}

/// Sender half of a text stream.
pub struct TextStreamSender {
    inner: ByteStreamSender,
    lines_sent: Arc<AtomicU64>,
}

impl TextStream {
    /// Create a new text stream.
    pub fn new() -> (Self, TextStreamSender) {
        Self::with_buffer_size(64)
    }

    /// Create a new text stream with custom buffer size.
    pub fn with_buffer_size(size: usize) -> (Self, TextStreamSender) {
        let (stream, sender) = ByteStream::with_buffer_size(size);
        let lines_sent = Arc::new(AtomicU64::new(0));
        (
            Self { inner: stream },
            TextStreamSender {
                inner: sender,
                lines_sent,
            },
        )
    }

    /// Get the inner byte stream.
    pub fn into_inner(self) -> ByteStream {
        self.inner
    }
}

impl Default for TextStream {
    fn default() -> Self {
        let (stream, _) = Self::new();
        stream
    }
}

impl Stream for TextStream {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

impl TextStreamSender {
    /// Send a line of text (newline is added automatically).
    pub async fn send_line(&self, line: &str) -> Result<(), Error> {
        self.inner.send(format!("{}\n", line)).await?;
        self.lines_sent.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Send raw text (no newline added).
    pub async fn send(&self, text: &str) -> Result<(), Error> {
        self.inner.send(text.as_bytes().to_vec()).await
    }

    /// Close the stream.
    pub async fn close(&self) {
        self.inner.close().await;
    }

    /// Get the total lines sent so far.
    pub fn lines_sent(&self) -> u64 {
        self.lines_sent.load(Ordering::Relaxed)
    }

    /// Check if the receiver has been dropped.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

// ============================================================================
// Streaming Response
// ============================================================================

/// A streaming HTTP response.
///
/// Unlike `HttpResponse` which buffers the entire body, `StreamingResponse`
/// sends data as it becomes available using chunked transfer encoding.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```ignore
/// use armature_core::streaming::{StreamingResponse, ByteStream};
///
/// let (stream, sender) = ByteStream::new();
///
/// // Spawn task to produce data
/// tokio::spawn(async move {
///     for i in 0..10 {
///         sender.send(format!("Line {}\n", i)).await.ok();
///         tokio::time::sleep(Duration::from_millis(100)).await;
///     }
///     sender.close().await;
/// });
///
/// StreamingResponse::new(stream)
///     .status(200)
///     .content_type("text/plain")
/// ```
pub struct StreamingResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// The stream body
    body: StreamBody,
}

/// The body of a streaming response.
pub enum StreamBody {
    /// A byte stream
    Bytes(ByteStream),
    /// A JSON stream
    Json(JsonStream),
    /// A text stream
    Text(TextStream),
    /// An empty body
    Empty,
}

impl StreamingResponse {
    /// Create a new streaming response from a byte stream.
    pub fn new(stream: ByteStream) -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body: StreamBody::Bytes(stream),
        }
    }

    /// Create a new NDJSON streaming response.
    pub fn ndjson(stream: JsonStream) -> Self {
        let mut response = Self {
            status: 200,
            headers: HashMap::new(),
            body: StreamBody::Json(stream),
        };
        response
            .headers
            .insert("Content-Type".to_string(), "application/x-ndjson".to_string());
        response
    }

    /// Create a new text streaming response.
    pub fn text(stream: TextStream) -> Self {
        let mut response = Self {
            status: 200,
            headers: HashMap::new(),
            body: StreamBody::Text(stream),
        };
        response.headers.insert(
            "Content-Type".to_string(),
            "text/plain; charset=utf-8".to_string(),
        );
        response
    }

    /// Create an empty streaming response.
    pub fn empty() -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body: StreamBody::Empty,
        }
    }

    /// Set the HTTP status code.
    pub fn status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    /// Set the Content-Type header.
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.headers
            .insert("Content-Type".to_string(), content_type.into());
        self
    }

    /// Add a header.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set Cache-Control to no-cache (recommended for streams).
    pub fn no_cache(mut self) -> Self {
        self.headers.insert(
            "Cache-Control".to_string(),
            "no-cache, no-store, must-revalidate".to_string(),
        );
        self
    }

    /// Enable CORS for the response.
    pub fn cors(mut self, origin: impl Into<String>) -> Self {
        self.headers
            .insert("Access-Control-Allow-Origin".to_string(), origin.into());
        self
    }

    /// Set X-Content-Type-Options to nosniff.
    pub fn nosniff(mut self) -> Self {
        self.headers
            .insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        self
    }

    /// Get the stream body, consuming the response.
    pub fn into_body(self) -> StreamBody {
        self.body
    }

    /// Check if this is an empty response.
    pub fn is_empty(&self) -> bool {
        matches!(self.body, StreamBody::Empty)
    }
}

impl Default for StreamingResponse {
    fn default() -> Self {
        Self::empty()
    }
}

// ============================================================================
// Stream Iterators
// ============================================================================

/// Stream items from an async iterator.
///
/// # Example
///
/// ```ignore
/// use armature_core::streaming::stream_iter;
///
/// let items = vec![1, 2, 3, 4, 5];
/// let (stream, _) = stream_iter(items.into_iter(), |i| format!("{}\n", i));
/// ```
pub fn stream_iter<I, T, F>(iter: I, transform: F) -> (ByteStream, tokio::task::JoinHandle<()>)
where
    I: Iterator<Item = T> + Send + 'static,
    T: Send + 'static,
    F: Fn(T) -> Vec<u8> + Send + 'static,
{
    let (stream, sender) = ByteStream::new();
    let items: Vec<T> = iter.collect(); // Collect to avoid iterator lifetime issues
    let handle = tokio::spawn(async move {
        for item in items {
            if sender.send(transform(item)).await.is_err() {
                break;
            }
        }
        sender.close().await;
    });
    (stream, handle)
}

/// Stream items from an async iterator with delays.
pub fn stream_iter_with_delay<I, T, F>(
    iter: I,
    transform: F,
    delay: Duration,
) -> (ByteStream, tokio::task::JoinHandle<()>)
where
    I: Iterator<Item = T> + Send + 'static,
    T: Send + 'static,
    F: Fn(T) -> Vec<u8> + Send + 'static,
{
    let (stream, sender) = ByteStream::new();
    let items: Vec<T> = iter.collect(); // Collect to avoid iterator lifetime issues
    let handle = tokio::spawn(async move {
        for item in items {
            if sender.send(transform(item)).await.is_err() {
                break;
            }
            tokio::time::sleep(delay).await;
        }
        sender.close().await;
    });
    (stream, handle)
}

/// Stream JSON items from an iterator.
pub fn stream_json_iter<I, T>(iter: I) -> (JsonStream, tokio::task::JoinHandle<()>)
where
    I: Iterator<Item = T> + Send + 'static,
    T: Serialize + Send + Sync + 'static,
{
    let (stream, sender) = JsonStream::new();
    let items: Vec<T> = iter.collect(); // Collect to avoid iterator lifetime issues
    let handle = tokio::spawn(async move {
        for item in items {
            if sender.send_json(&item).await.is_err() {
                break;
            }
        }
        sender.close().await;
    });
    (stream, handle)
}

// ============================================================================
// Stream from Reader
// ============================================================================

/// Stream data from an async reader (e.g., file, network).
///
/// # Example
///
/// ```ignore
/// use tokio::fs::File;
/// use armature_core::streaming::stream_reader;
///
/// let file = File::open("large_file.bin").await?;
/// let (stream, _) = stream_reader(file, 8192);  // 8KB chunks
/// ```
pub fn stream_reader<R>(reader: R, chunk_size: usize) -> (ByteStream, tokio::task::JoinHandle<()>)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    use tokio::io::AsyncReadExt;

    let (stream, sender) = ByteStream::new();
    let handle = tokio::spawn(async move {
        let mut reader = reader;
        let mut buffer = vec![0u8; chunk_size];

        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if sender.send(buffer[..n].to_vec()).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let _ = sender.send_error(e.to_string()).await;
                    break;
                }
            }
        }
        sender.close().await;
    });
    (stream, handle)
}

// ============================================================================
// Progress Tracking
// ============================================================================

/// A wrapper that tracks progress of a stream.
pub struct ProgressStream {
    inner: ByteStream,
    bytes_received: Arc<AtomicU64>,
    callback: Option<Box<dyn Fn(u64) + Send + Sync>>,
}

impl ProgressStream {
    /// Create a new progress tracking stream.
    pub fn new(inner: ByteStream) -> Self {
        Self {
            inner,
            bytes_received: Arc::new(AtomicU64::new(0)),
            callback: None,
        }
    }

    /// Set a callback to be called on each chunk received.
    pub fn on_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(u64) + Send + Sync + 'static,
    {
        self.callback = Some(Box::new(callback));
        self
    }

    /// Get the total bytes received so far.
    pub fn bytes_received(&self) -> u64 {
        self.bytes_received.load(Ordering::Relaxed)
    }
}

impl Stream for ProgressStream {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                let len = bytes.len() as u64;
                let total = self.bytes_received.fetch_add(len, Ordering::Relaxed) + len;
                if let Some(ref callback) = self.callback {
                    callback(total);
                }
                Poll::Ready(Some(Ok(bytes)))
            }
            other => other,
        }
    }
}

// ============================================================================
// Conversion to HttpResponse
// ============================================================================

impl StreamingResponse {
    /// Collect the entire stream into an HttpResponse.
    ///
    /// This buffers the entire response body, defeating the purpose of streaming.
    /// Only use when you need to convert to a buffered response.
    pub async fn into_buffered(mut self) -> Result<HttpResponse, Error> {
        use futures_util::StreamExt;

        let mut body = Vec::new();

        match &mut self.body {
            StreamBody::Bytes(stream) => {
                while let Some(chunk) = stream.next().await {
                    body.extend_from_slice(&chunk?);
                }
            }
            StreamBody::Json(stream) => {
                while let Some(chunk) = stream.next().await {
                    body.extend_from_slice(&chunk?);
                }
            }
            StreamBody::Text(stream) => {
                while let Some(chunk) = stream.next().await {
                    body.extend_from_slice(&chunk?);
                }
            }
            StreamBody::Empty => {}
        }

        let mut response = HttpResponse::new(self.status);
        response.headers = self.headers;
        response.body = body;
        Ok(response)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn test_byte_stream() {
        let (mut stream, sender) = ByteStream::new();

        tokio::spawn(async move {
            sender.send(b"hello".to_vec()).await.unwrap();
            sender.send(b" world".to_vec()).await.unwrap();
            sender.close().await;
        });

        let mut result = Vec::new();
        while let Some(chunk) = stream.next().await {
            result.extend_from_slice(&chunk.unwrap());
        }

        assert_eq!(result, b"hello world");
    }

    #[tokio::test]
    async fn test_json_stream() {
        let (mut stream, sender) = JsonStream::new();

        #[derive(Serialize)]
        struct Item {
            id: u64,
        }

        tokio::spawn(async move {
            sender.send_json(&Item { id: 1 }).await.unwrap();
            sender.send_json(&Item { id: 2 }).await.unwrap();
            sender.close().await;
        });

        let mut result = Vec::new();
        while let Some(chunk) = stream.next().await {
            result.extend_from_slice(&chunk.unwrap());
        }

        let result_str = String::from_utf8(result).unwrap();
        assert!(result_str.contains("{\"id\":1}"));
        assert!(result_str.contains("{\"id\":2}"));
    }

    #[tokio::test]
    async fn test_text_stream() {
        let (mut stream, sender) = TextStream::new();

        tokio::spawn(async move {
            sender.send_line("line 1").await.unwrap();
            sender.send_line("line 2").await.unwrap();
            sender.close().await;
        });

        let mut result = Vec::new();
        while let Some(chunk) = stream.next().await {
            result.extend_from_slice(&chunk.unwrap());
        }

        let result_str = String::from_utf8(result).unwrap();
        assert_eq!(result_str, "line 1\nline 2\n");
    }

    #[tokio::test]
    async fn test_streaming_response() {
        let (stream, sender) = ByteStream::new();

        tokio::spawn(async move {
            sender.send(b"test data".to_vec()).await.unwrap();
            sender.close().await;
        });

        let response = StreamingResponse::new(stream)
            .status(200)
            .content_type("text/plain")
            .no_cache();

        assert_eq!(response.status, 200);
        assert_eq!(
            response.headers.get("Content-Type"),
            Some(&"text/plain".to_string())
        );
    }

    #[tokio::test]
    async fn test_stream_iter() {
        let items = vec![1, 2, 3];
        let (mut stream, _) = stream_iter(items.into_iter(), |i| format!("{}", i).into_bytes());

        let mut result = Vec::new();
        while let Some(chunk) = stream.next().await {
            result.extend_from_slice(&chunk.unwrap());
        }

        assert_eq!(String::from_utf8(result).unwrap(), "123");
    }

    #[tokio::test]
    async fn test_bytes_sent_tracking() {
        let (stream, sender) = ByteStream::new();

        sender.send(b"hello".to_vec()).await.unwrap();
        assert_eq!(sender.bytes_sent(), 5);

        sender.send(b" world".to_vec()).await.unwrap();
        assert_eq!(sender.bytes_sent(), 11);

        // Keep stream alive until we're done
        drop(stream);
    }

    #[tokio::test]
    async fn test_json_items_sent_tracking() {
        let (stream, sender) = JsonStream::new();

        #[derive(Serialize)]
        struct Item {
            id: u64,
        }

        sender.send_json(&Item { id: 1 }).await.unwrap();
        assert_eq!(sender.items_sent(), 1);

        sender.send_json(&Item { id: 2 }).await.unwrap();
        assert_eq!(sender.items_sent(), 2);

        // Keep stream alive until we're done
        drop(stream);
    }

    #[tokio::test]
    async fn test_streaming_response_into_buffered() {
        let (stream, sender) = ByteStream::new();

        tokio::spawn(async move {
            sender.send(b"buffered".to_vec()).await.unwrap();
            sender.close().await;
        });

        let response = StreamingResponse::new(stream)
            .status(200)
            .content_type("text/plain");

        let buffered = response.into_buffered().await.unwrap();
        assert_eq!(buffered.status, 200);
        assert_eq!(buffered.body, b"buffered");
    }

    #[test]
    fn test_stream_chunk_from() {
        let from_vec: StreamChunk = vec![1, 2, 3].into();
        assert!(matches!(from_vec, StreamChunk::Bytes(_)));

        let from_string: StreamChunk = "hello".to_string().into();
        assert!(matches!(from_string, StreamChunk::Bytes(_)));

        let from_str: StreamChunk = "world".into();
        assert!(matches!(from_str, StreamChunk::Bytes(_)));
    }
}

