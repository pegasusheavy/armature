// Server-Sent Events (SSE) support for Armature

use crate::Error;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// Server-Sent Event
#[derive(Debug, Clone)]
pub struct ServerSentEvent {
    /// Event ID (optional)
    pub id: Option<String>,
    /// Event type (optional)
    pub event: Option<String>,
    /// Event data
    pub data: String,
    /// Retry interval in milliseconds (optional)
    pub retry: Option<u64>,
}

impl ServerSentEvent {
    /// Create a new SSE with just data
    pub fn new(data: String) -> Self {
        Self {
            id: None,
            event: None,
            data,
            retry: None,
        }
    }

    /// Create a new SSE with data and event type
    pub fn with_event(event: String, data: String) -> Self {
        Self {
            id: None,
            event: Some(event),
            data,
            retry: None,
        }
    }

    /// Create a new SSE with all fields
    pub fn full(id: String, event: String, data: String, retry: u64) -> Self {
        Self {
            id: Some(id),
            event: Some(event),
            data,
            retry: Some(retry),
        }
    }

    /// Convert to SSE format string
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        let mut output = String::new();

        if let Some(ref id) = self.id {
            output.push_str(&format!("id: {}\n", id));
        }

        if let Some(ref event) = self.event {
            output.push_str(&format!("event: {}\n", event));
        }

        // Handle multi-line data
        for line in self.data.lines() {
            output.push_str(&format!("data: {}\n", line));
        }

        if let Some(retry) = self.retry {
            output.push_str(&format!("retry: {}\n", retry));
        }

        output.push('\n');
        output
    }
}

/// SSE stream builder
pub struct SseStream {
    tx: mpsc::Sender<Result<String, Error>>,
}

impl SseStream {
    /// Create a new SSE stream
    pub fn new() -> (Self, ReceiverStream<Result<String, Error>>) {
        let (tx, rx) = mpsc::channel(100);
        let stream = ReceiverStream::new(rx);
        (Self { tx }, stream)
    }

    /// Send an event
    pub async fn send(&self, event: ServerSentEvent) -> Result<(), Error> {
        self.tx
            .send(Ok(event.to_string()))
            .await
            .map_err(|e| Error::Internal(format!("Failed to send SSE: {}", e)))
    }

    /// Send a simple message
    pub async fn send_message(&self, data: String) -> Result<(), Error> {
        self.send(ServerSentEvent::new(data)).await
    }

    /// Send a typed event
    pub async fn send_event(&self, event: String, data: String) -> Result<(), Error> {
        self.send(ServerSentEvent::with_event(event, data)).await
    }

    /// Send JSON data
    pub async fn send_json<T: serde::Serialize>(&self, data: &T) -> Result<(), Error> {
        let json = serde_json::to_string(data).map_err(|e| Error::Serialization(e.to_string()))?;
        self.send_message(json).await
    }

    /// Send a keep-alive comment
    pub async fn send_keep_alive(&self) -> Result<(), Error> {
        self.tx
            .send(Ok(": keep-alive\n\n".to_string()))
            .await
            .map_err(|e| Error::Internal(format!("Failed to send keep-alive: {}", e)))
    }
}

impl Default for SseStream {
    fn default() -> Self {
        Self::new().0
    }
}

/// SSE broadcaster for multiple clients
pub struct SseBroadcaster {
    clients: tokio::sync::RwLock<Vec<mpsc::Sender<Result<String, Error>>>>,
}

impl SseBroadcaster {
    pub fn new() -> Self {
        Self {
            clients: tokio::sync::RwLock::new(Vec::new()),
        }
    }

    /// Register a new client
    pub async fn register(&self) -> ReceiverStream<Result<String, Error>> {
        let (tx, rx) = mpsc::channel(100);
        let mut clients = self.clients.write().await;
        clients.push(tx);
        ReceiverStream::new(rx)
    }

    /// Broadcast an event to all clients
    pub async fn broadcast(&self, event: ServerSentEvent) -> Result<(), Error> {
        let data_str = event.to_string();
        let mut clients = self.clients.write().await;

        // Remove disconnected clients
        clients.retain(|tx| !tx.is_closed());

        // Send to all clients
        for tx in clients.iter() {
            let _ = tx.send(Ok(data_str.clone())).await;
        }

        Ok(())
    }

    /// Broadcast a simple message
    pub async fn broadcast_message(&self, data: String) -> Result<(), Error> {
        self.broadcast(ServerSentEvent::new(data)).await
    }

    /// Broadcast JSON data
    pub async fn broadcast_json<T: serde::Serialize>(&self, data: &T) -> Result<(), Error> {
        let json = serde_json::to_string(data).map_err(|e| Error::Serialization(e.to_string()))?;
        self.broadcast_message(json).await
    }

    /// Get number of connected clients
    pub async fn client_count(&self) -> usize {
        let clients = self.clients.read().await;
        clients.len()
    }

    /// Start a keep-alive task
    pub fn start_keep_alive(
        self: std::sync::Arc<Self>,
        interval: Duration,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                let comment_str = ": keep-alive\n\n".to_string();
                let mut clients = self.clients.write().await;
                clients.retain(|tx| !tx.is_closed());
                for tx in clients.iter() {
                    let _ = tx.send(Ok(comment_str.clone())).await;
                }
            }
        })
    }
}

impl Default for SseBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}
