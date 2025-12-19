import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-websocket-sse-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class WebsocketSseGuideComponent {
  page: DocPage = {
    title: 'WebSockets & Server-Sent Events',
    subtitle: 'Build real-time applications with WebSockets for bidirectional communication and Server-Sent Events (SSE) for efficient server-to-client streaming.',
    icon: '⚡',
    badge: 'Real-time',
    features: [
      {
        icon: '🔌',
        title: 'WebSockets',
        description: 'Full-duplex bidirectional communication'
      },
      {
        icon: '📡',
        title: 'Server-Sent Events',
        description: 'Efficient one-way server-to-client streaming'
      },
      {
        icon: '📢',
        title: 'Broadcasting',
        description: 'Send messages to all connected clients'
      },
      {
        icon: '🏠',
        title: 'Rooms & Channels',
        description: 'Organize connections into logical groups'
      }
    ],
    sections: [
      {
        id: 'when-to-use',
        title: 'When to Use What',
        content: `<p>Choose the right technology for your use case:</p>
        <ul>
          <li><strong>WebSockets</strong> — Chat apps, multiplayer games, collaborative editing, anything requiring client-to-server messages</li>
          <li><strong>SSE</strong> — Live feeds, notifications, stock tickers, progress updates, anything where clients only receive data</li>
        </ul>
        <p>SSE is simpler and works over HTTP/2, but WebSockets provide true bidirectional communication.</p>`
      },
      {
        id: 'websocket-setup',
        title: 'WebSocket Setup',
        content: `<p>Create a WebSocket endpoint for real-time bidirectional communication:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'websocket_handler.rs',
            code: `use armature::prelude::*;
use tokio::sync::broadcast;

#[derive(Clone, Serialize, Deserialize)]
struct ChatMessage {
    username: String,
    content: String,
    timestamp: u64,
}

#[controller("/ws")]
#[derive(Clone)]
struct WebSocketController {
    broadcast_tx: broadcast::Sender<ChatMessage>,
}

impl WebSocketController {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { broadcast_tx: tx }
    }

    #[websocket("/chat")]
    async fn chat(&self, ws: WebSocket) -> Result<(), Error> {
        let tx = self.broadcast_tx.clone();
        let mut rx = self.broadcast_tx.subscribe();

        // Split WebSocket into sender and receiver
        let (mut ws_tx, mut ws_rx) = ws.split();

        // Handle incoming messages
        let tx_clone = tx.clone();
        let recv_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_rx.next().await {
                if let Ok(text) = msg.to_text() {
                    let chat_msg: ChatMessage = serde_json::from_str(text)?;
                    tx_clone.send(chat_msg)?;
                }
            }
            Ok::<_, Error>(())
        });

        // Send messages to this client
        let send_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                let json = serde_json::to_string(&msg)?;
                ws_tx.send(Message::Text(json)).await?;
            }
            Ok::<_, Error>(())
        });

        // Wait for either task to complete
        tokio::select! {
            _ = recv_task => {},
            _ = send_task => {},
        }

        Ok(())
    }
}`
          }
        ]
      },
      {
        id: 'client-usage',
        title: 'Client-Side Usage',
        content: `<p>Connect to WebSocket from JavaScript:</p>`,
        codeBlocks: [
          {
            language: 'javascript',
            filename: 'client.js',
            code: `// Connect to WebSocket
const ws = new WebSocket('ws://localhost:3000/ws/chat');

ws.onopen = () => {
  console.log('Connected!');

  // Send a message
  ws.send(JSON.stringify({
    username: 'Alice',
    content: 'Hello, everyone!',
    timestamp: Date.now()
  }));
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log(\`\${message.username}: \${message.content}\`);
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('Disconnected');
};`
          }
        ]
      },
      {
        id: 'sse-setup',
        title: 'Server-Sent Events Setup',
        content: `<p>Create SSE endpoints for efficient server-to-client streaming:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'sse_handler.rs',
            code: `use armature::prelude::*;
use tokio_stream::StreamExt;

#[derive(Clone, Serialize)]
struct StockPrice {
    symbol: String,
    price: f64,
    change: f64,
}

#[controller("/events")]
#[derive(Default, Clone)]
struct SseController;

impl SseController {
    #[get("/stocks")]
    async fn stock_prices(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        // Create an SSE stream
        let stream = async_stream::stream! {
            let symbols = vec!["AAPL", "GOOGL", "MSFT"];
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            loop {
                interval.tick().await;

                for symbol in &symbols {
                    let price = StockPrice {
                        symbol: symbol.to_string(),
                        price: rand::random::<f64>() * 1000.0,
                        change: rand::random::<f64>() * 10.0 - 5.0,
                    };

                    yield ServerSentEvent::new()
                        .event("price_update")
                        .data(serde_json::to_string(&price).unwrap());
                }
            }
        };

        Ok(HttpResponse::sse(stream))
    }

    #[get("/notifications")]
    async fn notifications(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        let stream = async_stream::stream! {
            // Initial connection event
            yield ServerSentEvent::new()
                .event("connected")
                .data("Connection established");

            // Heartbeat to keep connection alive
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;
                yield ServerSentEvent::new()
                    .event("heartbeat")
                    .data(chrono::Utc::now().to_rfc3339());
            }
        };

        Ok(HttpResponse::sse(stream))
    }
}`
          }
        ]
      },
      {
        id: 'sse-client',
        title: 'SSE Client Usage',
        content: `<p>Consume SSE streams in JavaScript:</p>`,
        codeBlocks: [
          {
            language: 'javascript',
            filename: 'sse_client.js',
            code: `// Connect to SSE stream
const eventSource = new EventSource('/events/stocks');

// Listen for specific event types
eventSource.addEventListener('price_update', (event) => {
  const price = JSON.parse(event.data);
  console.log(\`\${price.symbol}: $\${price.price.toFixed(2)} (\${price.change > 0 ? '+' : ''}\${price.change.toFixed(2)}%)\`);
});

eventSource.addEventListener('connected', (event) => {
  console.log('Connected:', event.data);
});

eventSource.addEventListener('heartbeat', (event) => {
  console.log('Heartbeat:', event.data);
});

// Handle errors
eventSource.onerror = (error) => {
  console.error('SSE Error:', error);
  eventSource.close();
};`
          }
        ]
      },
      {
        id: 'broadcasting',
        title: 'Broadcasting to All Clients',
        content: `<p>Use broadcast channels to send messages to all connected clients:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use tokio::sync::broadcast;

struct BroadcastService {
    sender: broadcast::Sender<String>,
}

impl BroadcastService {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }

    pub fn broadcast(&self, message: String) -> Result<usize, Error> {
        Ok(self.sender.send(message)?)
    }
}

// Use in controller
#[controller("/broadcast")]
struct BroadcastController {
    service: BroadcastService,
}

impl BroadcastController {
    #[post("/send")]
    async fn send_to_all(&self, req: HttpRequest) -> Result<Json<usize>, Error> {
        let message: String = req.json().await?;
        let receivers = self.service.broadcast(message)?;
        Ok(Json(receivers))
    }
}`
          }
        ]
      },
      {
        id: 'rooms',
        title: 'Rooms & Channels',
        content: `<p>Organize connections into logical groups for targeted messaging:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use std::collections::HashMap;
use tokio::sync::{broadcast, RwLock};

struct RoomManager {
    rooms: RwLock<HashMap<String, broadcast::Sender<String>>>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
        }
    }

    pub async fn join(&self, room: &str) -> broadcast::Receiver<String> {
        let mut rooms = self.rooms.write().await;

        let sender = rooms.entry(room.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                tx
            });

        sender.subscribe()
    }

    pub async fn send_to_room(&self, room: &str, message: String) -> Result<(), Error> {
        let rooms = self.rooms.read().await;

        if let Some(sender) = rooms.get(room) {
            sender.send(message)?;
        }

        Ok(())
    }
}`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Heartbeats</strong> — Send periodic pings to detect dead connections</li>
          <li><strong>Reconnection</strong> — Implement client-side auto-reconnect with exponential backoff</li>
          <li><strong>Message queuing</strong> — Buffer messages for clients during brief disconnections</li>
          <li><strong>Rate limiting</strong> — Prevent abuse by limiting message frequency</li>
          <li><strong>Authentication</strong> — Validate tokens on WebSocket upgrade</li>
          <li><strong>Graceful shutdown</strong> — Close connections cleanly when server stops</li>
        </ul>`
      }
    ],
    relatedDocs: [
      {
        id: 'graphql-guide',
        title: 'GraphQL Subscriptions',
        description: 'Real-time data with GraphQL subscriptions'
      },
      {
        id: 'webhooks',
        title: 'Webhook Integration',
        description: 'Send and receive webhook events'
      },
      {
        id: 'graceful-shutdown',
        title: 'Graceful Shutdown',
        description: 'Clean connection handling on shutdown'
      }
    ],
    seeAlso: [
      { title: 'Security Guide', id: 'security-guide' },
      { title: 'Rate Limiting', id: 'rate-limiting' },
      { title: 'Health Checks', id: 'health-check' }
    ]
  };
}

