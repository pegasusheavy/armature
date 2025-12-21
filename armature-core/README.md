# armature-core

Core framework for the Armature web framework.

## Features

- **High-Performance Routing** - O(log n) routing with `matchit`, LRU caching, and static route fast path
- **Zero-Copy HTTP** - SIMD-accelerated parsing, arena allocation, and `Bytes` for efficient body handling
- **Optimized Handlers** - Monomorphized handler dispatch with inline optimization
- **Connection Management** - HTTP/1.1 pipelining, keep-alive, adaptive buffering
- **Memory Efficiency** - SmallVec headers, CompactString paths, object pooling
- **Tower Compatible** - Native integration with Tower middleware ecosystem

## Installation

```toml
[dependencies]
armature-core = "0.1"
```

## Quick Start

```rust
use armature_core::{Application, HttpResponse};

#[tokio::main]
async fn main() {
    let app = Application::new()
        .get("/", |_req| async { Ok(HttpResponse::ok()) })
        .get("/hello/:name", |req| async move {
            let name = req.param("name").unwrap_or("World");
            Ok(HttpResponse::ok().with_text(format!("Hello, {}!", name)))
        });

    app.listen("0.0.0.0:3000").await.unwrap();
}
```

## Performance

Benchmarked against other Rust frameworks:

| Metric | Armature | Axum | Actix-web |
|--------|----------|------|-----------|
| Plaintext RPS | 385k | 390k | 420k |
| JSON RPS | 305k | 310k | 340k |
| Latency P50 | 0.11ms | 0.10ms | 0.09ms |

## License

MIT OR Apache-2.0

