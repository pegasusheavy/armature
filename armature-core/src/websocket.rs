// WebSocket support for Armature

use crate::Error;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message as WsMessage;

/// WebSocket message type
#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close,
}

impl From<WsMessage> for WebSocketMessage {
    fn from(msg: WsMessage) -> Self {
        match msg {
            WsMessage::Text(text) => WebSocketMessage::Text(text),
            WsMessage::Binary(data) => WebSocketMessage::Binary(data),
            WsMessage::Ping(data) => WebSocketMessage::Ping(data),
            WsMessage::Pong(data) => WebSocketMessage::Pong(data),
            WsMessage::Close(_) => WebSocketMessage::Close,
            _ => WebSocketMessage::Close,
        }
    }
}

impl From<WebSocketMessage> for WsMessage {
    fn from(msg: WebSocketMessage) -> Self {
        match msg {
            WebSocketMessage::Text(text) => WsMessage::Text(text),
            WebSocketMessage::Binary(data) => WsMessage::Binary(data),
            WebSocketMessage::Ping(data) => WsMessage::Ping(data),
            WebSocketMessage::Pong(data) => WsMessage::Pong(data),
            WebSocketMessage::Close => WsMessage::Close(None),
        }
    }
}

/// WebSocket connection handle
pub struct WebSocketConnection {
    id: String,
    tx: broadcast::Sender<WebSocketMessage>,
}

impl WebSocketConnection {
    pub fn new(id: String) -> (Self, broadcast::Receiver<WebSocketMessage>) {
        let (tx, rx) = broadcast::channel(100);
        (Self { id, tx }, rx)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub async fn send(&self, message: WebSocketMessage) -> Result<(), Error> {
        self.tx
            .send(message)
            .map_err(|e| Error::Internal(format!("Failed to send message: {}", e)))?;
        Ok(())
    }

    pub async fn send_text(&self, text: String) -> Result<(), Error> {
        self.send(WebSocketMessage::Text(text)).await
    }

    pub async fn send_json<T: serde::Serialize>(&self, data: &T) -> Result<(), Error> {
        let json = serde_json::to_string(data).map_err(|e| Error::Serialization(e.to_string()))?;
        self.send_text(json).await
    }
}

/// WebSocket room for broadcasting to multiple connections
pub struct WebSocketRoom {
    _name: String,
    connections: Arc<RwLock<HashMap<String, broadcast::Sender<WebSocketMessage>>>>,
}

impl WebSocketRoom {
    pub fn new(name: String) -> Self {
        Self {
            _name: name,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(&self, id: String, tx: broadcast::Sender<WebSocketMessage>) {
        let mut connections = self.connections.write().await;
        connections.insert(id, tx);
    }

    pub async fn remove_connection(&self, id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(id);
    }

    pub async fn broadcast(&self, message: WebSocketMessage) -> Result<(), Error> {
        let connections = self.connections.read().await;
        for tx in connections.values() {
            let _ = tx.send(message.clone());
        }
        Ok(())
    }

    pub async fn broadcast_text(&self, text: String) -> Result<(), Error> {
        self.broadcast(WebSocketMessage::Text(text)).await
    }

    pub async fn broadcast_json<T: serde::Serialize>(&self, data: &T) -> Result<(), Error> {
        let json = serde_json::to_string(data).map_err(|e| Error::Serialization(e.to_string()))?;
        self.broadcast_text(json).await
    }

    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }
}

/// WebSocket manager for handling multiple rooms
pub struct WebSocketManager {
    rooms: Arc<RwLock<HashMap<String, Arc<WebSocketRoom>>>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_or_create_room(&self, name: &str) -> Arc<WebSocketRoom> {
        let mut rooms = self.rooms.write().await;
        rooms
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(WebSocketRoom::new(name.to_string())))
            .clone()
    }

    pub async fn get_room(&self, name: &str) -> Option<Arc<WebSocketRoom>> {
        let rooms = self.rooms.read().await;
        rooms.get(name).cloned()
    }

    pub async fn remove_room(&self, name: &str) {
        let mut rooms = self.rooms.write().await;
        rooms.remove(name);
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle a WebSocket connection
pub async fn handle_websocket<S, F, Fut>(
    mut stream: WebSocketStream<S>,
    _connection_id: String,
    mut handler: F,
) -> Result<(), Error>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    F: FnMut(WebSocketMessage) -> Fut,
    Fut: std::future::Future<Output = Result<(), Error>>,
{
    // Handle incoming messages
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(msg) => {
                if msg.is_close() {
                    break;
                }
                let ws_msg: WebSocketMessage = msg.into();
                if let Err(e) = handler(ws_msg).await {
                    eprintln!("WebSocket handler error: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
