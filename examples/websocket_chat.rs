// WebSocket chat room example

use armature::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    user: String,
    message: String,
    timestamp: u64,
}

/// Chat service managing WebSocket rooms
#[injectable]
#[derive(Default, Clone)]
struct ChatService {
    manager: Arc<WebSocketManager>,
}

impl ChatService {
    async fn get_room(&self, room_name: &str) -> Arc<WebSocketRoom> {
        self.manager.get_or_create_room(room_name).await
    }

    async fn broadcast_to_room(&self, room_name: &str, message: ChatMessage) -> Result<(), Error> {
        let room = self.get_room(room_name).await;
        room.broadcast_json(&message).await
    }
}

/// Chat controller
#[controller("/chat")]
#[derive(Default, Clone)]
struct ChatController;

impl ChatController {
    #[post("/:room/message")]
    async fn send_message(req: HttpRequest) -> Result<Json<serde_json::Value>, Error> {
        let room_name = req
            .param("room")
            .ok_or_else(|| Error::Validation("Missing room parameter".to_string()))?;

        let message: ChatMessage = req.json()?;

        let service = ChatService::default();
        service.broadcast_to_room(&room_name, message).await?;

        Ok(Json(serde_json::json!({
            "status": "sent"
        })))
    }

    #[get("/:room/stats")]
    async fn get_stats(req: HttpRequest) -> Result<Json<serde_json::Value>, Error> {
        let room_name = req
            .param("room")
            .ok_or_else(|| Error::Validation("Missing room parameter".to_string()))?;

        let service = ChatService::default();
        let room = service.get_room(&room_name).await;
        let count = room.connection_count().await;

        Ok(Json(serde_json::json!({
            "room": room_name,
            "connections": count
        })))
    }
}

#[module(
    providers: [ChatService],
    controllers: [ChatController]
)]
#[derive(Default)]
struct AppModule;

#[tokio::main]
async fn main() {
    println!("🔌 Armature WebSocket Chat Example");
    println!("===================================\n");

    println!("Available endpoints:");
    println!("  POST /chat/:room/message - Send a message to a room");
    println!("  GET  /chat/:room/stats   - Get room statistics");
    println!("\n💡 In a full implementation:");
    println!("  WS   /chat/:room         - WebSocket endpoint for real-time chat");
    println!("\nExample usage:");
    println!("  curl -X POST http://localhost:3005/chat/general/message \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!("    -d '{{\"user\":\"Alice\",\"message\":\"Hello!\",\"timestamp\":1234567890}}'");
    println!();

    let app = Application::create::<AppModule>().await;

    if let Err(e) = app.listen(3005).await {
        eprintln!("Server error: {}", e);
    }
}
