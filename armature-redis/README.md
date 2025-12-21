# armature-redis

Redis client integration for the Armature framework.

## Features

- **Connection Pooling** - Efficient connection management
- **Async Operations** - Non-blocking Redis commands
- **Pub/Sub** - Real-time messaging
- **Cluster Support** - Redis Cluster mode
- **Streams** - Redis Streams for event sourcing

## Installation

```toml
[dependencies]
armature-redis = "0.1"
```

## Quick Start

```rust
use armature_redis::RedisClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RedisClient::new("redis://localhost:6379").await?;

    // Basic operations
    client.set("key", "value").await?;
    let value: String = client.get("key").await?;

    // With expiration
    client.set_ex("temp_key", "value", 60).await?;

    Ok(())
}
```

## Pub/Sub

```rust
// Subscribe
let mut subscriber = client.subscribe("channel").await?;
while let Some(message) = subscriber.next().await {
    println!("Received: {:?}", message);
}

// Publish
client.publish("channel", "Hello!").await?;
```

## License

MIT OR Apache-2.0

