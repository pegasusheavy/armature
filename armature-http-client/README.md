# armature-http-client

HTTP client for the Armature framework.

## Features

- **Retry Logic** - Automatic retries with backoff
- **Circuit Breaker** - Fail fast on repeated failures
- **Timeouts** - Request and connection timeouts
- **Connection Pooling** - Efficient connection reuse
- **Interceptors** - Request/response middleware

## Installation

```toml
[dependencies]
armature-http-client = "0.1"
```

## Quick Start

```rust
use armature_http_client::HttpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = HttpClient::new()
        .timeout(Duration::from_secs(30))
        .retry(3)
        .build();

    // GET request
    let response = client.get("https://api.example.com/users")
        .send()
        .await?;

    // POST with JSON
    let user = client.post("https://api.example.com/users")
        .json(&CreateUser { name: "John" })
        .send()
        .await?;

    Ok(())
}
```

## Circuit Breaker

```rust
let client = HttpClient::new()
    .circuit_breaker(CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 2,
        timeout: Duration::from_secs(30),
    })
    .build();
```

## License

MIT OR Apache-2.0

