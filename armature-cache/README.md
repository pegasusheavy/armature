# armature-cache

Cache management for the Armature framework.

## Features

- **Multiple Backends** - Redis, in-memory, and custom stores
- **TTL Support** - Automatic expiration
- **Async API** - Non-blocking cache operations
- **Serialization** - Automatic JSON serialization/deserialization
- **Connection Pooling** - Efficient Redis connections via bb8

## Installation

```toml
[dependencies]
armature-cache = "0.1"
```

## Quick Start

```rust
use armature_cache::{Cache, RedisCache};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Redis
    let cache = RedisCache::new("redis://localhost:6379").await?;

    // Set a value with TTL
    cache.set("key", "value", Some(Duration::from_secs(300))).await?;

    // Get a value
    let value: Option<String> = cache.get("key").await?;

    // Delete a value
    cache.delete("key").await?;

    Ok(())
}
```

## Backends

### Redis

```rust
let cache = RedisCache::new("redis://localhost:6379").await?;
```

### In-Memory

```rust
let cache = MemoryCache::new();
```

## License

MIT OR Apache-2.0

