# armature-session

Session management for the Armature framework.

## Features

- **Multiple Backends** - Redis, in-memory, cookie-based
- **Secure by Default** - Cryptographic session IDs
- **TTL Support** - Automatic session expiration
- **Flash Messages** - One-time session data
- **Middleware** - Easy integration with Armature

## Installation

```toml
[dependencies]
armature-session = "0.1"
```

## Quick Start

```rust
use armature_session::{SessionStore, RedisSessionStore, Session};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create session store
    let store = RedisSessionStore::new("redis://localhost:6379").await?;

    // Create a new session
    let session = Session::new();
    session.set("user_id", "123");
    session.set("role", "admin");

    // Save session
    store.save(&session).await?;

    // Load session
    let loaded = store.get(&session.id).await?;

    // Delete session
    store.delete(&session.id).await?;

    Ok(())
}
```

## Middleware Usage

```rust
use armature_core::Application;
use armature_session::SessionMiddleware;

let app = Application::new()
    .with_middleware(SessionMiddleware::new(store))
    .get("/profile", |req| async move {
        let session = req.session();
        let user_id = session.get::<String>("user_id");
        Ok(HttpResponse::ok())
    });
```

## License

MIT OR Apache-2.0

