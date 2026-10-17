# WebSocket and Server-Sent Events Guide

This guide explains how to use WebSockets and Server-Sent Events (SSE) in Armature for real-time communication.

## Overview

Armature provides built-in support for two real-time communication patterns:

1. **WebSockets** - Full-duplex, bidirectional communication
2. **Server-Sent Events (SSE)** - Server-to-client streaming

## WebSocket Support

WebSocket support lives in the standalone `armature-websocket` crate. It is a
genuine `tokio-tungstenite`-based server: it binds its own TCP listener,
performs the WebSocket upgrade handshake, and manages connections and rooms
directly. It runs *alongside* your Armature HTTP application rather than as
an upgrade path off an existing HTTP route — the two share state (e.g. a
`RoomManager`) so an HTTP endpoint and a real WebSocket client can interact
with the same rooms.

The runnable reference for everything in this section is
[`examples/websocket_chat.rs`](../examples/websocket_chat.rs), a chat server
that was built against and tested with a real WebSocket client. Run it with:

```bash
cargo run --example websocket_chat
```

### Security Defaults

The server is locked down until you open it up explicitly:

- **Loopback bind.** The default `bind_addr` is `127.0.0.1:9001`. Set `.bind_addr(..)` to
  expose it.
- **Origin allowlist (CSWSH defence).** Browsers always send `Origin` on an upgrade. By
  default every request carrying an `Origin` header is refused with `403` until you list
  the allowed origins with `.allowed_origins([..])` (exact `scheme://host[:port]`,
  case-insensitive). Requests without `Origin` come from non-browser clients and go on to
  the authorizer. `OriginPolicy::Any` disables the check (development only; it logs a
  warning).
- **Handshake authorization.** `WebSocketServerBuilder::build` fails unless you call
  either `.authorizer(..)` or the explicit `.allow_anonymous()`. The authorizer runs
  inside the upgrade handshake, before the connection exists, and returns the
  connection's `Principal`; a rejection becomes a plain `401`/`403` response.
- **Bounded resources.** `handshake_timeout` (10 s), `max_connections` (10,000),
  `max_message_size`/`max_frame_size` (64 KiB), `max_write_buffer_size` (4 MiB) and a
  per-connection `outbound_queue_capacity` (1,024 messages). A peer that stops reading is
  evicted with `WebSocketError::SlowConsumer` instead of buffering without limit.

### Core Components

#### Authorizer and Principal

A `HandshakeAuthorizer` sees the upgrade request (`HandshakeRequest`, an
`http::Request<()>`) and returns a `Principal` or a `HandshakeRejection`. Any
`Fn(&HandshakeRequest) -> Result<Principal, HandshakeRejection>` closure is one:

```rust
use armature_websocket::{HandshakeRejection, HandshakeRequest, Principal};

let authorizer = move |req: &HandshakeRequest| {
    let token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(HandshakeRejection::unauthorized)?;

    // Verify the token with your auth stack (e.g. armature_jwt::JwtManager::verify).
    let user_id = verify_token(token).map_err(|_| HandshakeRejection::unauthorized())?;

    Ok(Principal::new(user_id).with_attribute("role", "member"))
};
```

Browsers cannot set an `Authorization` header on a WebSocket upgrade; for browser clients
authenticate with a session cookie (and rely on the Origin allowlist), or a short-lived
signed token in the query string. `Principal::anonymous()` is what `allow_anonymous()`
attaches. `Principal`'s `Debug` output redacts attribute values.

#### Message

The wire-level message type. It carries a `message_type` and a `payload`, with
constructors and helpers for each kind:

```rust
use armature_websocket::Message;

let text = Message::text("Hello!");
let binary = Message::binary(vec![1, 2, 3]);
let json = Message::json(&my_value)?; // serializes to a Text message

// Inspecting a received message
if let Some(text) = message.as_text() {
    // ...
}
let bytes: &[u8] = message.as_bytes();
message.is_text();
message.is_binary();
message.is_ping();
message.is_pong();
message.is_close();

// Deserializing a JSON payload
let value: MyType = message.parse_json()?;
```

#### Connection

A handle to a single, live WebSocket connection. Connections are created
internally by the server as clients connect; you get one back from a
`RoomManager` by connection ID:

```rust
use armature_websocket::RoomManager;

if let Some(connection) = room_manager.get_connection(connection_id) {
    let who = connection.principal().subject(); // set by the authorizer
    connection.send_text("Private message")?;
    connection.send_json(&payload)?;
    connection.is_open();
    connection.close();
}
```

`Connection::send`, `send_text`, `send_binary`, and `send_json` are all
synchronous — they hand the message to a bounded internal queue that a
background writer task drains, so there is no `.await` on the send side. When
the queue is full the peer is a slow consumer: `send` returns
`WebSocketError::SlowConsumer` and the connection is evicted.

#### Room and RoomManager

`Room` tracks membership for a single named room; `RoomManager` owns every
room and connection and is the type you actually interact with:

```rust
use armature_websocket::RoomManager;
use std::sync::Arc;

let rooms = Arc::new(RoomManager::new());

// Membership
rooms.join_room(connection_id, "lobby")?;   // creates "lobby" if needed
rooms.leave_room(connection_id, "lobby")?;

// Broadcasting
let sent = rooms.broadcast_to_room("lobby", Message::text("hi"))?;           // -> usize delivered
let sent = rooms.broadcast_to_room_except("lobby", message, connection_id)?; // skip the sender
let sent = rooms.broadcast_all(message);                                     // every connection, every room

// Inspecting a room
if let Some(room) = rooms.get_room("lobby") {
    let member_count = room.len();
    let member_ids = room.members();
}

// Bookkeeping
rooms.room_ids();
rooms.connection_ids();
rooms.connection_count();
rooms.room_count();
```

`join_room`/`leave_room`/`broadcast_to_room`/`broadcast_to_room_except` are all
synchronous and return a `WebSocketResult<T>` (`Err(WebSocketError::RoomNotFound(_))`
or `Err(WebSocketError::ConnectionNotFound(_))` on bad IDs) — there is no `.await`.

#### WebSocketHandler

Your application logic lives in a `WebSocketHandler` implementation. Only
`on_message` is required; the rest have no-op or sensible default
implementations:

Every connection hook receives the `Principal` the authorizer returned:

```rust
use armature_websocket::{Message, Principal, WebSocketError, WebSocketHandler};
use async_trait::async_trait;

#[derive(Clone, Default)]
struct ChatHandler;

#[async_trait]
impl WebSocketHandler for ChatHandler {
    async fn on_connect(&self, connection_id: &str, principal: &Principal) {
        println!("{connection_id} connected as {:?}", principal.subject());
    }

    async fn on_message(&self, connection_id: &str, principal: &Principal, message: Message) {
        // Handle an incoming frame (see examples/websocket_chat.rs for a
        // full join/chat/leave protocol built on top of this). Authorize
        // per-message actions (e.g. joining a room) against `principal`.
    }

    async fn on_disconnect(&self, connection_id: &str, _principal: &Principal) {
        println!("{connection_id} disconnected");
    }

    async fn on_error(&self, connection_id: &str, error: &WebSocketError) {
        eprintln!("{connection_id} error: {error}");
    }
}
```

`armature_websocket::LoggingHandler` is a ready-made no-op handler that just
logs connect/message/disconnect events, useful for smoke-testing a server.

#### WebSocketServer

Ties a `WebSocketHandler` to a bound TCP listener and drives the accept loop:

```rust
use armature_websocket::{RoomManager, WebSocketServerBuilder};
use std::net::SocketAddr;
use std::sync::Arc;

// Expose the server explicitly (the default is 127.0.0.1:9001).
let bind_addr: SocketAddr = "0.0.0.0:3006".parse().unwrap();
let server = WebSocketServerBuilder::new()
    .bind_addr(bind_addr)
    .max_message_size(64 * 1024)
    // Browser pages from these origins only.
    .allowed_origins(["https://chat.example.com"])
    // Required: an authorizer, or the explicit `.allow_anonymous()`.
    .authorizer(authorizer)
    .build(ChatHandler::default())?; // returns WebSocketResult<WebSocketServer<_>>

// The server owns a RoomManager; clone the Arc out so other parts of your
// app (e.g. an HTTP controller) can join/broadcast into the same rooms.
let rooms: Arc<RoomManager> = Arc::clone(server.room_manager());

tokio::spawn(async move {
    if let Err(e) = server.run().await {
        eprintln!("WebSocket server error: {e}");
    }
});
```

Without the builder, `WebSocketServer::new(config, handler, authorizer)` takes a
`WebSocketServerConfig` and an authorizer (`AllowAnonymous` to opt out) and also returns
a `WebSocketResult`.

### Usage Example

The pattern below mirrors `examples/websocket_chat.rs`: a real
`WebSocketServer` running next to the Armature HTTP application, sharing one
`RoomManager` so an HTTP-posted message reaches connected WebSocket clients
and vice versa.

#### 1. Define the handler

```rust
use armature_websocket::{Message, Principal, RoomManager, WebSocketError, WebSocketHandler};
use async_trait::async_trait;
use std::sync::{Arc, OnceLock};

static CHAT_ROOMS: OnceLock<Arc<RoomManager>> = OnceLock::new();

fn chat_rooms() -> &'static Arc<RoomManager> {
    CHAT_ROOMS.get().expect("WebSocket server must be started first")
}

#[derive(Clone, Default)]
struct ChatHandler;

#[async_trait]
impl WebSocketHandler for ChatHandler {
    async fn on_message(&self, connection_id: &str, principal: &Principal, message: Message) {
        let Some(text) = message.as_text() else { return };
        // Parse `text` into your own protocol and call
        // chat_rooms().join_room(...) / .broadcast_to_room(...) as needed,
        // checking that `principal` may join the requested room.
    }

    async fn on_disconnect(&self, connection_id: &str, _principal: &Principal) {
        // RoomManager::unregister_connection (called by the server right
        // after this hook returns) already removes the connection from
        // every room it was a member of.
    }
}
```

#### 2. Start the WebSocket server and publish its RoomManager

```rust
use armature_websocket::{HandshakeRejection, HandshakeRequest, Principal, WebSocketServerBuilder};
use std::net::SocketAddr;

fn start_ws_server() -> armature_websocket::WebSocketResult<()> {
    let bind_addr = SocketAddr::from(([0, 0, 0, 0], 3006));
    let server = WebSocketServerBuilder::new()
        .bind_addr(bind_addr)
        .allowed_origins(["https://chat.example.com"])
        .authorizer(|req: &HandshakeRequest| {
            // See "Authorizer and Principal" above.
            authenticate(req).ok_or_else(HandshakeRejection::unauthorized)
        })
        .build(ChatHandler::default())?;

    CHAT_ROOMS
        .set(Arc::clone(server.room_manager()))
        .expect("start_ws_server must only be called once");

    tokio::spawn(async move {
        let _ = server.run().await;
    });
    Ok(())
}
```

#### 3. Create an injectable service and controller that share the same rooms

```rust
use armature::prelude::*;

#[injectable]
#[derive(Default, Clone)]
struct ChatService;

impl ChatService {
    async fn broadcast_to_room(&self, room: &str, text: &str) -> Result<usize, Error> {
        let payload = Message::text(text);
        match chat_rooms().broadcast_to_room(room, payload) {
            Ok(sent) => Ok(sent),
            Err(WebSocketError::RoomNotFound(_)) => Ok(0), // no clients yet, not an error
            Err(e) => Err(Error::Internal(e.to_string())),
        }
    }
}

#[controller("/chat")]
#[derive(Default, Clone)]
struct ChatController {
    chat_service: ChatService,
}

#[routes]
impl ChatController {
    #[post("/:room/message")]
    async fn send_message(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        let room = req.param("room").ok_or_else(|| Error::Validation("Missing room".into()))?;
        let body: serde_json::Value = req.json()?;
        let delivered = self.chat_service.broadcast_to_room(room, &body.to_string()).await?;
        HttpResponse::json(&serde_json::json!({ "delivered_to": delivered }))
    }
}
```

#### 4. Register in the module and start both servers

```rust
#[module(
    providers: [ChatService],
    controllers: [ChatController]
)]
#[derive(Default, Clone)]
struct AppModule;

#[tokio::main]
async fn main() {
    start_ws_server().expect("WebSocket server config");

    let app = Application::create::<AppModule>().await;
    app.listen(3005).await.unwrap();
}
```

See `examples/websocket_chat.rs` for the complete, working version of this
example, including a full JSON-based join/chat/leave protocol.

### Broadcasting Patterns

#### Room-based Broadcasting

```rust
// Send to everyone in a specific room
let sent = rooms.broadcast_to_room("room-1", Message::json(&event)?)?;
```

#### Targeted Messaging

```rust
// Send to one specific, already-connected client
if let Some(connection) = rooms.get_connection(connection_id) {
    connection.send_text("Private message")?;
}
```

#### Excluding the Sender

```rust
// Notify everyone in the room except the connection that triggered it
let sent = rooms.broadcast_to_room_except("room-1", Message::json(&event)?, connection_id)?;
```

#### Global Broadcasting

```rust
// Broadcast to every connection on the server, regardless of room
let sent = rooms.broadcast_all(Message::json(&event)?);
```

## Server-Sent Events (SSE)

### Core Components

#### ServerSentEvent

Represents an SSE event:

```rust
// Simple message
let event = ServerSentEvent::new("Hello".to_string());

// Typed event
let event = ServerSentEvent::with_event(
    "user_joined".to_string(),
    "Alice joined".to_string()
);

// Full event
let event = ServerSentEvent::full(
    "123".to_string(),        // ID
    "notification".to_string(), // Event type
    "You have mail".to_string(), // Data
    5000                        // Retry (ms)
);
```

#### SseStream

Single client SSE stream:

```rust
let (stream, receiver) = SseStream::new();

// Send events
stream.send_message("Hello".to_string()).await?;
stream.send_event("update".to_string(), "Data".to_string()).await?;
stream.send_json(&data).await?;

// Keep-alive
stream.send_keep_alive().await?;
```

#### SseBroadcaster

Broadcast SSE to multiple clients:

```rust
let broadcaster = Arc::new(SseBroadcaster::new());

// Register clients
let stream1 = broadcaster.register().await;
let stream2 = broadcaster.register().await;

// Broadcast to all
broadcaster.broadcast_message("Update".to_string()).await?;
broadcaster.broadcast_json(&data).await?;

// Get client count
let count = broadcaster.client_count().await;

// Auto keep-alive
broadcaster.clone().start_keep_alive(Duration::from_secs(30));
```

### Usage Example

#### 1. Create an SSE Service

```rust
#[injectable]
#[derive(Clone)]
struct StockTickerService {
    broadcaster: Arc<SseBroadcaster>,
}

impl Default for StockTickerService {
    fn default() -> Self {
        let broadcaster = Arc::new(SseBroadcaster::new());

        // Start broadcasting stock prices
        let broadcaster_clone = broadcaster.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            loop {
                interval.tick().await;

                let price = StockPrice {
                    symbol: "AAPL".to_string(),
                    price: 150.0,
                };

                let event = ServerSentEvent::with_event(
                    "price".to_string(),
                    serde_json::to_string(&price).unwrap(),
                );

                let _ = broadcaster_clone.broadcast(event).await;
            }
        });

        // Start keep-alive
        broadcaster.clone().start_keep_alive(Duration::from_secs(15));

        Self { broadcaster }
    }
}
```

#### 2. Create an SSE Controller

```rust
#[controller("/events")]
#[derive(Default, Clone)]
struct EventsController {
    stock_ticker: StockTickerService,
}

impl EventsController {
    async fn subscribe_stocks(&self) -> ReceiverStream<Result<String, Error>> {
        self.stock_ticker.broadcaster.register().await
    }
}
```

### Client-Side Usage

#### JavaScript

```javascript
// Connect to SSE endpoint
const source = new EventSource('http://localhost:3000/events/stocks');

// Listen for specific events
source.addEventListener('price', (event) => {
    const data = JSON.parse(event.data);
    console.log('Stock price:', data);
});

// Listen for all events
source.onmessage = (event) => {
    console.log('Message:', event.data);
};

// Handle errors
source.onerror = (error) => {
    console.error('SSE error:', error);
};

// Close connection
source.close();
```

#### curl

```bash
# Subscribe to SSE stream (-N for no buffering)
curl -N http://localhost:3000/events/stocks
```

## Comparison: WebSocket vs SSE

### Use WebSockets When:

✅ You need bidirectional communication
✅ Low latency is critical
✅ Binary data transfer is required
✅ Building chat applications
✅ Real-time collaborative editing
✅ Gaming or interactive applications

### Use SSE When:

✅ Server → Client communication only
✅ Text-based data is sufficient
✅ Simple implementation needed
✅ Automatic reconnection desired
✅ Event streaming (stocks, notifications)
✅ Progress updates
✅ Live feeds

## Best Practices

### WebSocket Best Practices

1. **Connection Management**

`RoomManager` already tracks every registered connection and its room
memberships for you (`register_connection`/`unregister_connection`, called by
`WebSocketServer` itself). Keep any *application-level* per-connection state
(like `examples/websocket_chat.rs`'s `Session`) in your own map keyed by
connection ID, and clean it up in `on_disconnect`:

```rust
// Application-level session state, separate from RoomManager's own bookkeeping
let mut sessions: HashMap<String, Session> = HashMap::new();

// Clean up in WebSocketHandler::on_disconnect
sessions.remove(connection_id);
```

2. **Error Handling**
```rust
// Connection::send_text is synchronous - no `.await`
if let Err(e) = connection.send_text(msg) {
    eprintln!("Failed to send: {}", e);
    // The connection is almost certainly closed/dead; no need to manually
    // remove it from RoomManager - on_disconnect + unregister_connection
    // handles that once the server's connection loop notices.
}
```

3. **Rate Limiting**
```rust
// Limit message rate
let mut last_message = Instant::now();
if last_message.elapsed() < Duration::from_millis(100) {
    return Err(Error::Validation("Too many messages".into()));
}
```

4. **Authentication and Authorization**

Authenticate in the handshake authorizer (see [Authorizer and Principal](#authorizer-and-principal)),
never in a message after the connection is open. Allowlist the browser origins that may
connect, and use `.allow_anonymous()` only for genuinely public feeds. Authorize
per-message actions against the `Principal` passed to `on_message`:

```rust
async fn on_message(&self, connection_id: &str, principal: &Principal, message: Message) {
    if let Ok(Join { room }) = message.parse_json::<Join>() {
        if !may_join(principal, &room) {
            return; // or send an error frame
        }
        let _ = chat_rooms().join_room(connection_id, &room);
    }
}
```

### SSE Best Practices

1. **Keep-Alive**
```rust
// Send periodic keep-alive
broadcaster.start_keep_alive(Duration::from_secs(30));
```

2. **Event IDs**
```rust
// Use IDs for resumability
let mut id = 0;
loop {
    id += 1;
    let event = ServerSentEvent {
        id: Some(id.to_string()),
        event: Some("update".into()),
        data: "...".into(),
        retry: None,
    };
}
```

3. **Retry Configuration**
```rust
// Set retry interval
let event = ServerSentEvent {
    retry: Some(5000), // 5 seconds
    ..Default::default()
};
```

4. **Clean Up Disconnected Clients**
```rust
// Remove closed connections periodically
let mut clients = broadcaster.clients.write().await;
clients.retain(|tx| !tx.is_closed());
```

## Performance Considerations

### WebSocket Performance

- **Memory:** ~2KB per connection
- **CPU:** Minimal when idle
- **Latency:** < 1ms typically
- **Throughput:** Depends on message size

### SSE Performance

- **Memory:** ~1KB per client
- **CPU:** Low overhead
- **Latency:** 1-2ms typically
- **Throughput:** Limited by HTTP/1.1

### Scaling

#### Horizontal Scaling

Use Redis pub/sub for multi-server setups:

```rust
// Subscribe to Redis channel
let mut pubsub = redis_client.get_async_connection().await?;
pubsub.subscribe("chat:room1").await?;

// Forward to WebSocket clients in the matching local room. RoomManager's
// broadcast methods are synchronous, so no `.await` here.
let mut stream = pubsub.on_message();
while let Some(msg) = stream.next().await {
    let payload: String = msg.get_payload()?;
    rooms.broadcast_to_room("room1", Message::text(payload))?;
}
```

#### Connection Limits

```rust
// Limit connections per room. Room::len() is synchronous.
const MAX_CONNECTIONS: usize = 1000;

if let Some(room) = rooms.get_room(room_id) {
    if room.len() >= MAX_CONNECTIONS {
        return Err(Error::Internal("Room full".into()));
    }
}
```

## Testing

### Unit Testing

`Connection`s are created internally by `WebSocketServer` as clients connect
(there is no public constructor), so exercising room/broadcast logic in a
unit test means spinning up a real server on a loopback port and connecting
with a WebSocket client, exactly like `armature-websocket`'s own test suite
does:

```rust
use armature_websocket::{Message, WebSocketServerBuilder};
use tokio::net::TcpListener;

#[tokio::test]
async fn test_websocket_broadcast() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener); // free the port for the server

    let server = WebSocketServerBuilder::new()
        .bind_addr(addr)
        .allow_anonymous() // test clients send no Origin and no credentials
        .build(ChatHandler::default())
        .unwrap();
    let rooms = Arc::clone(server.room_manager());

    tokio::spawn(async move {
        let _ = server.run().await;
    });

    let url = format!("ws://{addr}");
    let (mut ws1, _) = tokio_tungstenite::connect_async(&url).await.unwrap();
    let (mut ws2, _) = tokio_tungstenite::connect_async(&url).await.unwrap();

    // Wait for both connections to register, then join them to a room
    // (in a real handler this happens in response to a client message).
    tokio::time::sleep(Duration::from_millis(50)).await;
    for id in rooms.connection_ids() {
        rooms.join_room(&id, "test").unwrap();
    }

    assert_eq!(rooms.get_room("test").unwrap().len(), 2);

    let sent = rooms.broadcast_to_room("test", Message::text("Hello")).unwrap();
    assert_eq!(sent, 2);
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_sse_stream() {
    let broadcaster = Arc::new(SseBroadcaster::new());
    let mut stream = broadcaster.register().await;

    // Broadcast message
    broadcaster.broadcast_message("Test".to_string()).await.unwrap();

    // Receive message
    let msg = stream.next().await.unwrap().unwrap();
    assert!(msg.contains("Test"));
}
```

## Troubleshooting

### WebSocket Issues

**Connection fails:**
- `403 Origin not allowed`: add the page's origin to `.allowed_origins([..])`
- `401`/`403` from the authorizer: check the credentials the client sends
- Server unreachable from other hosts: the default bind is `127.0.0.1`; set `.bind_addr(..)`
- Check HTTP → WebSocket upgrade, protocol headers, firewall/proxy settings

**Messages not received:**
- Verify connection is open
- Check error logs
- Test with simple echo server

### SSE Issues

**Stream disconnects:**
- Implement keep-alive
- Check server timeout settings
- Verify HTTP/1.1 support

**Messages delayed:**
- Disable response buffering
- Use `Content-Type: text/event-stream`
- Check network conditions

## Future Enhancements

Planned features:

- [ ] `#[websocket]` decorator for automatic upgrade
- [ ] `#[sse]` decorator for streaming responses
- [ ] Rate limiting decorators
- [ ] Redis adapter for clustering
- [ ] Compression support
- [ ] GraphQL subscription support

## Summary

Armature provides comprehensive real-time communication support:

✅ **WebSocket** - Full-duplex communication with rooms and broadcasting, authorized at the handshake
✅ **SSE** - Efficient server-to-client streaming
✅ **Type-Safe** - Compile-time verified message types
✅ **Easy to Use** - Simple, intuitive APIs
✅ **Scalable** - Designed for production use
✅ **Well-Tested** - Comprehensive test coverage

Both WebSocket and SSE integrate seamlessly with Armature's dependency injection system, making it easy to build real-time features into your applications.

