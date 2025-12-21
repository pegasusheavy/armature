# armature-lambda

AWS Lambda runtime adapter for the Armature framework.

## Features

- **Lambda Runtime** - Run Armature apps on Lambda
- **API Gateway** - HTTP event handling
- **ALB** - Application Load Balancer support
- **Cold Start Optimization** - Minimal startup time
- **Layers** - Shared dependencies

## Installation

```toml
[dependencies]
armature-lambda = "0.1"
```

## Quick Start

```rust
use armature_lambda::LambdaRuntime;
use armature_core::Application;

#[tokio::main]
async fn main() {
    let app = Application::new()
        .get("/", |_| async { Ok(HttpResponse::ok().with_text("Hello!")) });

    LambdaRuntime::new(app).run().await;
}
```

## API Gateway Integration

```rust
// Handles API Gateway proxy events
let runtime = LambdaRuntime::api_gateway(app);
runtime.run().await;
```

## Build for Lambda

```bash
# Install cargo-lambda
cargo install cargo-lambda

# Build
cargo lambda build --release

# Deploy
cargo lambda deploy my-function
```

## License

MIT OR Apache-2.0

