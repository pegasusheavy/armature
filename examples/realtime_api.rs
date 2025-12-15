//! Real-time Communication API Example
//!
//! This example demonstrates real-time communication patterns with Armature:
//! - WebSocket connections for bidirectional communication
//! - Server-Sent Events (SSE) for server-to-client streaming
//! - Broadcasting messages to multiple clients
//! - Room-based messaging
//!
//! Run with: `cargo run --example realtime_api`

use armature::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

// =============================================================================
// Message Types
// =============================================================================

/// Chat message for WebSocket communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: u64,
    pub username: String,
    pub content: String,
    pub timestamp: String,
    pub room: Option<String>,
}

/// Server event for SSE streaming.
#[derive(Debug, Clone, Serialize)]
pub struct ServerEvent {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: String,
}

/// User status update.
#[derive(Debug, Clone, Serialize)]
pub struct UserStatus {
    pub user_id: String,
    pub status: String,
    pub last_seen: String,
}

// =============================================================================
// Broadcasting Service
// =============================================================================

/// Service for broadcasting messages to connected clients.
#[derive(Clone)]
pub struct BroadcastService {
    sender: broadcast::Sender<ChatMessage>,
    message_id: Arc<AtomicU64>,
}

impl BroadcastService {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            message_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Subscribe to messages.
    pub fn subscribe(&self) -> broadcast::Receiver<ChatMessage> {
        self.sender.subscribe()
    }

    /// Broadcast a message to all subscribers.
    pub fn broadcast(&self, username: String, content: String, room: Option<String>) -> ChatMessage {
        let id = self.message_id.fetch_add(1, Ordering::SeqCst);
        let message = ChatMessage {
            id,
            username,
            content,
            timestamp: chrono::Utc::now().to_rfc3339(),
            room,
        };

        // Send to all subscribers (ignore errors if no subscribers)
        let _ = self.sender.send(message.clone());
        message
    }

    /// Get the number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

// =============================================================================
// Controllers
// =============================================================================

/// REST API endpoints for chat.
#[controller("/api/chat")]
struct ChatController;

#[controller_impl]
impl ChatController {
    /// POST /api/chat/messages - Send a message (broadcasts to all clients)
    #[post("/messages")]
    async fn send_message(
        &self,
        #[inject] broadcast: Arc<BroadcastService>,
        req: HttpRequest,
    ) -> Result<HttpResponse, Error> {
        #[derive(Deserialize)]
        struct SendMessage {
            username: String,
            content: String,
            room: Option<String>,
        }

        let body: SendMessage = req
            .json()
            .map_err(|e| Error::bad_request(format!("Invalid JSON: {}", e)))?;

        if body.content.trim().is_empty() {
            return Err(Error::validation("Message content cannot be empty"));
        }

        let message = broadcast.broadcast(body.username, body.content, body.room);
        HttpResponse::json(&message)
    }

    /// GET /api/chat/stats - Get chat statistics
    #[get("/stats")]
    async fn get_stats(
        &self,
        #[inject] broadcast: Arc<BroadcastService>,
    ) -> Result<HttpResponse, Error> {
        HttpResponse::json(&serde_json::json!({
            "active_connections": broadcast.subscriber_count(),
            "status": "online",
        }))
    }
}

/// Server-Sent Events endpoint.
#[controller("/api/events")]
struct EventsController;

#[controller_impl]
impl EventsController {
    /// GET /api/events/stream - SSE stream of server events
    #[get("/stream")]
    async fn event_stream(&self) -> Result<HttpResponse, Error> {
        // This would normally return an SSE stream
        // For demonstration, we'll return a simple message
        HttpResponse::json(&serde_json::json!({
            "message": "SSE streaming would be implemented here",
            "hint": "Use SseStream and SseBroadcaster from armature_core",
        }))
    }

    /// GET /api/events/heartbeat - SSE heartbeat stream
    #[get("/heartbeat")]
    async fn heartbeat(&self) -> Result<HttpResponse, Error> {
        HttpResponse::json(&serde_json::json!({
            "message": "Heartbeat endpoint for connection keep-alive",
            "interval_seconds": 30,
        }))
    }
}

/// WebSocket endpoint.
#[controller("/api/ws")]
struct WebSocketController;

#[controller_impl]
impl WebSocketController {
    /// GET /api/ws/info - WebSocket connection info
    #[get("/info")]
    async fn ws_info(&self) -> Result<HttpResponse, Error> {
        HttpResponse::json(&serde_json::json!({
            "websocket_url": "ws://localhost:3000/ws/chat",
            "supported_protocols": ["chat.v1"],
            "message_format": {
                "type": "string",
                "schema": {
                    "type": "object",
                    "properties": {
                        "action": {"type": "string", "enum": ["join", "leave", "message"]},
                        "room": {"type": "string"},
                        "content": {"type": "string"},
                    }
                }
            }
        }))
    }
}

// =============================================================================
// Demo: Periodic Event Generator
// =============================================================================

/// Spawns a background task that broadcasts periodic events.
async fn spawn_event_generator(broadcast: Arc<BroadcastService>) {
    let mut interval = interval(Duration::from_secs(30));
    let events = vec![
        ("System", "Server health check completed"),
        ("Bot", "Did you know? Armature supports WebSocket, SSE, and REST!"),
        ("System", "Connected clients: checking..."),
    ];
    let mut event_idx = 0;

    loop {
        interval.tick().await;

        let (username, content) = events[event_idx % events.len()];
        let content = if content.contains("checking") {
            format!("Connected clients: {}", broadcast.subscriber_count())
        } else {
            content.to_string()
        };

        broadcast.broadcast(
            username.to_string(),
            content,
            Some("announcements".to_string()),
        );

        event_idx += 1;
    }
}

// =============================================================================
// Module
// =============================================================================

#[module]
struct RealtimeModule;

#[module_impl]
impl RealtimeModule {
    fn providers(&self) -> Vec<ProviderRegistration> {
        vec![]
    }

    fn controllers(&self) -> Vec<ControllerRegistration> {
        vec![]
    }

    fn imports(&self) -> Vec<Box<dyn Module>> {
        vec![]
    }

    fn exports(&self) -> Vec<std::any::TypeId> {
        vec![]
    }
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() {
    // Initialize logging
    let _guard = LogConfig::new()
        .level(LogLevel::Info)
        .format(LogFormat::Pretty)
        .init();

    info!("Starting Real-time API example");

    // Create broadcast service
    let broadcast = Arc::new(BroadcastService::new(100));

    // Spawn background event generator
    let broadcast_clone = broadcast.clone();
    tokio::spawn(async move {
        spawn_event_generator(broadcast_clone).await;
    });

    // Create the DI container
    let container = Container::new();
    container.register(broadcast);

    // Build the application
    let app = Application::builder()
        .container(container)
        .build()
        .expect("Failed to build application");

    // Start the server
    info!("Server running at http://127.0.0.1:3000");
    info!("");
    info!("Available endpoints:");
    info!("");
    info!("  Chat API:");
    info!("    POST /api/chat/messages - Send a chat message");
    info!("    GET  /api/chat/stats    - Get chat statistics");
    info!("");
    info!("  Events API:");
    info!("    GET  /api/events/stream    - SSE event stream");
    info!("    GET  /api/events/heartbeat - Heartbeat stream");
    info!("");
    info!("  WebSocket Info:");
    info!("    GET  /api/ws/info - WebSocket connection details");
    info!("");
    info!("Test sending a message:");
    info!(r#"  curl -X POST http://localhost:3000/api/chat/messages \"#);
    info!(r#"    -H "Content-Type: application/json" \"#);
    info!(r#"    -d '{{"username":"Alice","content":"Hello!","room":"general"}}'"#);

    app.listen("127.0.0.1:3000").await.expect("Server failed");
}

