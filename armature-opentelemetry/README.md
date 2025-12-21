# armature-opentelemetry

OpenTelemetry integration for the Armature framework.

## Features

- **Distributed Tracing** - Trace requests across services
- **Auto Instrumentation** - HTTP, database, cache spans
- **Multiple Exporters** - Jaeger, Zipkin, OTLP
- **Context Propagation** - W3C Trace Context, B3
- **Baggage** - Pass context across services

## Installation

```toml
[dependencies]
armature-opentelemetry = "0.1"
```

## Quick Start

```rust
use armature_opentelemetry::{init_tracer, TracingMiddleware};

#[tokio::main]
async fn main() {
    // Initialize tracer
    init_tracer("my-service", "http://localhost:4317").await;

    let app = Application::new()
        .with_middleware(TracingMiddleware::new())
        .get("/", handler);

    app.listen("0.0.0.0:3000").await.unwrap();
}
```

## Custom Spans

```rust
use armature_opentelemetry::tracer;

async fn my_handler(req: HttpRequest) -> Result<HttpResponse, Error> {
    let span = tracer().start("process_request");

    // Do work...

    span.end();
    Ok(HttpResponse::ok())
}
```

## Exporters

### Jaeger

```rust
init_tracer_jaeger("my-service", "localhost:6831").await;
```

### OTLP (OpenTelemetry Protocol)

```rust
init_tracer_otlp("my-service", "http://localhost:4317").await;
```

## License

MIT OR Apache-2.0

