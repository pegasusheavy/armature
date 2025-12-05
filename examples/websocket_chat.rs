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
    fn new() -> Self {
        Self {
            manager: Arc::new(WebSocketManager::new()),
        }
    }

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
struct ChatController {
    chat_service: ChatService,
}

impl ChatController {
    async fn handle_connection(&self, room_name: String) -> Result<(), Error> {
        println!("New WebSocket connection to room: {}", room_name);

        let room = self.chat_service.get_room(&room_name).await;

        // In a real implementation, this would be called when a WebSocket upgrade happens
        // For now, we'll demonstrate the structure

        let welcome_msg = ChatMessage {
            user: "System".to_string(),
            message: format!("Welcome to {}!", room_name),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        room.broadcast_json(&welcome_msg).await?;

        Ok(())
    }

    async fn send_message(
        &self,
        room_name: String,
        message: ChatMessage,
    ) -> Result<Json<serde_json::Value>, Error> {
        self.chat_service
            .broadcast_to_room(&room_name, message)
            .await?;

        Ok(Json(serde_json::json!({
            "status": "sent"
        })))
    }

    async fn get_stats(&self, room_name: String) -> Result<Json<serde_json::Value>, Error> {
        let room = self.chat_service.get_room(&room_name).await;
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

    let app = create_chat_app();

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

    if let Err(e) = app.listen(3005).await {
        eprintln!("Server error: {}", e);
    }
}

fn create_chat_app() -> Application {
    let container = Container::new();
    let mut router = Router::new();

    // Register services
    let chat_service = ChatService::new();
    container.register(chat_service.clone());

    // Create controller
    let chat_controller = ChatController {
        chat_service: chat_service.clone(),
    };

    // Register routes
    let controller_clone = chat_controller.clone();
    router.add_route(Route {
        method: HttpMethod::POST,
        path: "/chat/:room/message".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = controller_clone.clone();
            Box::pin(async move {
                let room = req
                    .param("room")
                    .ok_or_else(|| Error::Validation("Missing room parameter".to_string()))?
                    .to_string();

                let message: ChatMessage = req.json()?;
                ctrl.send_message(room, message).await?.into_response()
            })
        }),
    });

    let controller_clone2 = chat_controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/chat/:room/stats".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = controller_clone2.clone();
            Box::pin(async move {
                let room = req
                    .param("room")
                    .ok_or_else(|| Error::Validation("Missing room parameter".to_string()))?
                    .to_string();

                ctrl.get_stats(room).await?.into_response()
            })
        }),
    });

    Application::new(container, router)
}
