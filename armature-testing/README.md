# armature-testing

Testing utilities for the Armature framework.

## Features

- **Test Client** - HTTP client for testing handlers
- **Mock Services** - Mock external dependencies
- **Fixtures** - Database and state fixtures
- **Assertions** - HTTP response assertions
- **Integration Tests** - Full application testing

## Installation

```toml
[dev-dependencies]
armature-testing = "0.1"
```

## Quick Start

```rust
use armature_testing::{TestClient, assert_status};

#[tokio::test]
async fn test_hello_endpoint() {
    let app = create_test_app();
    let client = TestClient::new(app);

    let response = client.get("/hello").send().await;

    assert_status!(response, 200);
    assert_eq!(response.text().await, "Hello, World!");
}
```

## Test Client

```rust
let client = TestClient::new(app);

// GET request
let resp = client.get("/users").send().await;

// POST with JSON
let resp = client.post("/users")
    .json(&user)
    .send()
    .await;

// With headers
let resp = client.get("/api/data")
    .header("Authorization", "Bearer token")
    .send()
    .await;
```

## Assertions

```rust
assert_status!(response, 200);
assert_json!(response, {"id": 1, "name": "Test"});
assert_header!(response, "Content-Type", "application/json");
```

## License

MIT OR Apache-2.0

