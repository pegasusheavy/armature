# Testing Guide

Comprehensive testing utilities for Armature framework applications.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Integration Test Helpers](#integration-test-helpers)
- [Docker Test Containers](#docker-test-containers)
- [Load Testing](#load-testing)
- [Contract Testing](#contract-testing)
- [Basic Test Utilities](#basic-test-utilities)
- [Best Practices](#best-practices)
- [Examples](#examples)
- [API Reference](#api-reference)

## Overview

The `armature-testing` crate provides a comprehensive suite of testing utilities:

- **Integration Helpers** - Database setup/teardown
- **Docker Containers** - Isolated test environments
- **Load Testing** - Performance and stress testing
- **Contract Testing** - Consumer-driven contracts (Pact)
- **Test App** - HTTP test client
- **Mocks** - Service mocking and spies
- **Assertions** - Fluent test assertions

## Features

✅ **Integration Test Helpers**
- Database setup/teardown automation
- Test fixtures with lifecycle management
- Database seeding utilities
- Migration support
- Before/after hooks

✅ **Docker Test Containers**
- Automatic container lifecycle
- Built-in configurations for Postgres, Redis, MongoDB
- Custom container support
- Auto-cleanup on drop
- Health checks

✅ **Load Testing**
- Request count-based testing
- Duration-based testing
- Concurrent load generation
- Stress testing (gradual ramp-up)
- Detailed statistics (RPS, latency percentiles)

✅ **Contract Testing**
- Pact-compatible contracts
- Consumer-driven design
- Contract versioning
- Verification utilities
- JSON serialization

✅ **Basic Utilities**
- HTTP test client
- Service mocking
- Fluent assertions
- Test app builder

## Integration Test Helpers

### Database Setup/Teardown

Implement `DatabaseTestHelper` trait for your database:

```rust
use armature_testing::integration::*;
use async_trait::async_trait;

struct MyDbHelper {
    connection_string: String,
}

#[async_trait]
impl DatabaseTestHelper for MyDbHelper {
    async fn setup(&self) -> Result<(), IntegrationTestError> {
        // Connect to database
        // Run migrations
        // Seed test data
        Ok(())
    }

    async fn teardown(&self) -> Result<(), IntegrationTestError> {
        // Drop tables
        // Clean up test data
        Ok(())
    }

    async fn migrate(&self) -> Result<(), IntegrationTestError> {
        // Run database migrations
        Ok(())
    }

    async fn seed(&self) -> Result<(), IntegrationTestError> {
        // Insert test data
        Ok(())
    }
}
```

### Test Fixtures

Use `TestFixture` for automatic setup/teardown:

```rust
use std::sync::Arc;

let helper = Arc::new(MyDbHelper::new("postgres://localhost/test"));
let fixture = TestFixture::new(helper);

// Automatic setup and teardown
fixture.run_test(|| async {
    // Your test code
    // Database is ready to use
    Ok(())
}).await?;
```

### Manual Control

```rust
let fixture = TestFixture::new(helper)
    .without_auto_cleanup();

fixture.setup().await?;
// Run tests
fixture.teardown().await?;
```

### Database Seeding

```rust
let seeder = DatabaseSeeder::new()
    .add_fixture("users")
    .add_fixture("posts")
    .add_fixture("comments");

for fixture_name in seeder.fixtures() {
    println!("Loading fixture: {}", fixture_name);
}
```

### Integration Test Builder

```rust
let test_suite = IntegrationTestBuilder::new("User API Tests")
    .before_each(|| async {
        // Runs before each test
    })
    .after_each(|| async {
        // Runs after each test
    });
```

## Docker Test Containers

### Checking Docker Availability

```rust
use armature_testing::docker::*;

if !DockerContainer::is_docker_available() {
    println!("Docker not available");
    return;
}
```

### PostgreSQL Container

```rust
let config = PostgresContainer::config("testdb", "user", "pass");
let mut container = DockerContainer::new(config);

container.start().await?;
// Connection: postgres://user:pass@localhost:5432/testdb

// Run tests...

container.stop().await?;
// Or let it drop for auto-cleanup
```

### Redis Container

```rust
let config = RedisContainer::config();
let mut container = DockerContainer::new(config);

container.start().await?;
// Connection: redis://localhost:6379

// Container auto-stops on drop
```

### MongoDB Container

```rust
let config = MongoContainer::config("testdb");
let mut container = DockerContainer::new(config);

container.start().await?;
// Connection: mongodb://localhost:27017/testdb
```

### Custom Containers

```rust
let config = ContainerConfig::new("nginx", "alpine")
    .with_name("test-nginx")
    .with_port(8080, 80)
    .with_env("NGINX_HOST", "localhost")
    .with_wait_timeout(5);

let mut container = DockerContainer::new(config);
container.start().await?;
```

### Container Lifecycle

```rust
// Check if running
if container.is_running() {
    println!("Container is active");
}

// Get container ID
if let Some(id) = container.container_id() {
    println!("Container ID: {}", id);
}

// Manual stop
container.stop().await?;

// Auto-stop on drop (RAII)
drop(container);
```

## Load Testing

### Basic Load Test

```rust
use armature_testing::load::*;
use std::time::Duration;

let config = LoadTestConfig::new(10, 1000); // 10 concurrent, 1000 requests

let runner = LoadTestRunner::new(config, || async {
    // Your test code (e.g., HTTP request)
    Ok(())
});

let stats = runner.run().await?;
stats.print();
```

### Duration-Based Load Test

```rust
let config = LoadTestConfig::new(20, u64::MAX)
    .with_duration(Duration::from_secs(60))  // Run for 60 seconds
    .with_timeout(Duration::from_secs(10));

let runner = LoadTestRunner::new(config, || async {
    // Your test code
    Ok(())
});

let stats = runner.run().await?;
```

### Rate-Limited Load Test

```rust
let config = LoadTestConfig::new(10, 1000)
    .with_rate_limit(50.0);  // Max 50 requests/sec

let runner = LoadTestRunner::new(config, || async {
    Ok(())
});
```

### Stress Test (Gradual Ramp-Up)

```rust
let stress_runner = StressTestRunner::new(
    1,                          // Start with 1 concurrent
    100,                        // Max 100 concurrent
    10,                         // Step by 10
    Duration::from_secs(10),    // 10 seconds per step
    || async {
        Ok(())
    },
);

let results = stress_runner.run().await?;

for (concurrency, stats) in results {
    println!("Concurrency {}: {} RPS", concurrency, stats.rps);
}
```

### Load Test Statistics

The `LoadTestStats` struct provides:

- `total_requests` - Total number of requests
- `successful` - Successful requests
- `failed` - Failed requests
- `duration` - Total test duration
- `rps` - Requests per second
- `min_response_time` - Minimum latency
- `max_response_time` - Maximum latency
- `avg_response_time` - Average latency
- `median_response_time` - Median (p50)
- `p95_response_time` - 95th percentile
- `p99_response_time` - 99th percentile

### Real-World Example

```rust
use reqwest;

let config = LoadTestConfig::new(50, 10000)
    .with_timeout(Duration::from_secs(30));

let runner = LoadTestRunner::new(config, || async {
    let response = reqwest::get("http://localhost:3000/api/users")
        .await
        .map_err(|e| LoadTestError::TestFailed(e.to_string()))?;

    if !response.status().is_success() {
        return Err(LoadTestError::TestFailed("Request failed".to_string()));
    }

    Ok(())
});

let stats = runner.run().await?;
stats.print();
```

## Contract Testing

### Creating a Contract

```rust
use armature_testing::contract::*;

let mut builder = ContractBuilder::new("Frontend", "UserAPI");

// Define interaction
let request = ContractRequest::new(ContractMethod::Get, "/api/users/1")
    .with_header("Accept", "application/json");

let response = ContractResponse::new(200)
    .with_header("Content-Type", "application/json")
    .with_body(serde_json::json!({
        "id": 1,
        "name": "Alice"
    }));

builder.add_interaction(
    ContractInteraction::new(
        "get user by ID",
        request,
        response,
    )
    .with_provider_state("user with ID 1 exists")
);

let contract = builder.build();
```

### Saving Contracts

```rust
use std::path::PathBuf;

let manager = ContractManager::new(PathBuf::from("./pacts"));
manager.save(&contract)?;
// Saves to: ./pacts/frontend-userapi.json
```

### Loading Contracts

```rust
let contract = manager.load("Frontend", "UserAPI")?;
```

### Verifying Contracts

```rust
let actual_response = ContractResponse::new(200)
    .with_body(serde_json::json!({"id": 1, "name": "Alice"}));

match ContractVerifier::verify_interaction(&interaction, &actual_response) {
    Ok(()) => println!("✅ Contract verified"),
    Err(e) => println!("❌ Verification failed: {}", e),
}
```

### Contract Request Methods

- `ContractMethod::Get`
- `ContractMethod::Post`
- `ContractMethod::Put`
- `ContractMethod::Delete`
- `ContractMethod::Patch`
- `ContractMethod::Head`
- `ContractMethod::Options`

### Provider States

Provider states set up test conditions:

```rust
let interaction = ContractInteraction::new(
    "delete user",
    request,
    response,
)
.with_provider_state("user with ID 1 exists");
```

### Multiple Interactions

```rust
let mut builder = ContractBuilder::new("Frontend", "API");

builder
    .add_interaction(get_user_interaction)
    .add_interaction(create_user_interaction)
    .add_interaction(update_user_interaction)
    .add_interaction(delete_user_interaction);

let contract = builder.build();
```

## Basic Test Utilities

### Test App

```rust
use armature_testing::*;

let app = TestAppBuilder::new()
    .with_route("/hello", |_req| async {
        Ok(HttpResponse::ok().with_body(b"Hello!".to_vec()))
    })
    .build();

let client = app.client();
let response = client.get("/hello").await;
assert_eq!(response.status(), Some(200));
```

### Mock Services

```rust
use armature_testing::MockService;

let mock = MockService::<String>::new();
mock.record_call("get_user");

assert_eq!(mock.call_count(), 1);
assert!(mock.was_called("get_user"));
```

### Assertions

```rust
use armature_testing::*;

// Assert status
assert_status(&response, 200);

// Assert header
assert_header(&response, "Content-Type", "application/json");

// Assert JSON
assert_json(&response, &serde_json::json!({"status": "ok"}));
```

## Best Practices

### Integration Testing

1. **Use Fixtures** - Automate setup/teardown
2. **Isolate Tests** - Each test should be independent
3. **Clean Up** - Always clean up test data
4. **Use Transactions** - Rollback after each test
5. **Seed Minimal Data** - Only what's needed for the test

### Docker Containers

1. **Check Availability** - Always check if Docker is available
2. **Use RAII** - Let containers auto-cleanup
3. **Wait for Ready** - Use wait timeouts
4. **Unique Names** - Use UUIDs for container names
5. **Resource Limits** - Set appropriate limits

### Load Testing

1. **Start Small** - Begin with low concurrency
2. **Gradual Increase** - Use stress tests to find limits
3. **Monitor Metrics** - Track p95/p99, not just average
4. **Realistic Tests** - Use production-like data
5. **CI Integration** - Run on every PR

### Contract Testing

1. **Consumer-Driven** - Let consumers define contracts
2. **Version Contracts** - Track contract versions
3. **Share Contracts** - Use shared repository
4. **Verify Often** - Run verification in CI
5. **Provider States** - Use for test setup

### General

1. **Fast Tests** - Keep unit tests under 100ms
2. **Parallel Execution** - Run tests in parallel
3. **Clear Names** - Use descriptive test names
4. **One Assert** - Focus each test on one thing
5. **No Flakiness** - Eliminate flaky tests immediately

## Examples

See the `examples/` directory:

- `testing_integration.rs` - Integration test helpers
- `testing_docker.rs` - Docker test containers
- `testing_load.rs` - Load testing
- `testing_contract.rs` - Contract testing

Run examples:

```bash
cargo run --example testing_integration
cargo run --example testing_docker
cargo run --example testing_load
cargo run --example testing_contract
```

## API Reference

### Integration Module

- `DatabaseTestHelper` - Trait for database helpers
- `TestFixture<T>` - Test lifecycle manager
- `IntegrationTestBuilder` - Test suite builder
- `DatabaseSeeder` - Data seeding utility
- `IntegrationTestError` - Error type

### Docker Module

- `DockerContainer` - Docker container manager
- `ContainerConfig` - Container configuration
- `PostgresContainer` - Postgres helper
- `RedisContainer` - Redis helper
- `MongoContainer` - MongoDB helper
- `DockerError` - Error type

### Load Module

- `LoadTestRunner<F, Fut>` - Load test runner
- `LoadTestConfig` - Load test configuration
- `LoadTestStats` - Test statistics
- `StressTestRunner<F, Fut>` - Stress test runner
- `LoadTestError` - Error type

### Contract Module

- `ContractBuilder` - Contract builder
- `Contract` - Consumer contract
- `ContractInteraction` - Single interaction
- `ContractRequest` - Request specification
- `ContractResponse` - Response specification
- `ContractManager` - Contract file manager
- `ContractVerifier` - Verification utility
- `ContractMethod` - HTTP methods
- `ContractError` - Error type

## Troubleshooting

### Docker Not Available

**Problem:** `Docker not available` error

**Solution:**
- Install Docker: https://docs.docker.com/get-docker/
- Start Docker daemon
- Check with: `docker --version`

### Container Start Fails

**Problem:** Container fails to start

**Solutions:**
- Ensure port is not in use
- Check Docker logs: `docker logs <container_id>`
- Pull image manually: `docker pull <image>`
- Check Docker daemon is running

### Load Tests Timeout

**Problem:** Load tests timing out

**Solutions:**
- Increase timeout: `.with_timeout(Duration::from_secs(60))`
- Reduce concurrency
- Check target service capacity
- Verify network connectivity

### Contract Verification Fails

**Problem:** Contract verification fails unexpectedly

**Solutions:**
- Check JSON structure matches exactly
- Verify header names (case-sensitive)
- Check status codes
- Review provider states
- Use `println!` for debugging

## Summary

The `armature-testing` crate provides comprehensive testing utilities:

- ✅ **Integration Helpers** - Automate database setup/teardown
- ✅ **Docker Containers** - Isolated, reproducible environments
- ✅ **Load Testing** - Find performance limits
- ✅ **Contract Testing** - Consumer-driven API design
- ✅ **Test Utilities** - Mock, assert, test clients

**Key Benefits:**

- **Productivity** - Less boilerplate, more testing
- **Reliability** - Isolated, reproducible tests
- **Performance** - Find bottlenecks early
- **Confidence** - Comprehensive test coverage

**Next Steps:**

1. Add integration tests with fixtures
2. Use Docker containers for databases
3. Run load tests in CI
4. Implement contract testing
5. Measure test coverage

Happy Testing! 🧪

