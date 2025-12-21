# armature-cloudrun

Google Cloud Run deployment utilities for the Armature framework.

## Features

- **Container Ready** - Optimized for Cloud Run containers
- **Health Checks** - Built-in health endpoints
- **Graceful Shutdown** - Handle SIGTERM properly
- **Port Configuration** - Respect PORT environment variable

## Installation

```toml
[dependencies]
armature-cloudrun = "0.1"
```

## Quick Start

```rust
use armature_cloudrun::CloudRunApp;
use armature_core::Application;

#[tokio::main]
async fn main() {
    let app = Application::new()
        .get("/", |_| async { Ok(HttpResponse::ok()) });

    CloudRunApp::new(app)
        .with_health_check("/health")
        .run()
        .await;
}
```

## Dockerfile

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/my-app /app/my-app
CMD ["/app/my-app"]
```

## License

MIT OR Apache-2.0

