# Logging Guide

Comprehensive guide to Armature's logging system.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Log Formats](#log-formats)
- [Log Levels](#log-levels)
- [Output Destinations](#output-destinations)
- [Structured Logging](#structured-logging)
- [HTTP Request Logging](#http-request-logging)
- [Best Practices](#best-practices)
- [Performance](#performance)
- [Examples](#examples)

---

## Overview

Armature provides a powerful, highly configurable logging system built on the `tracing` ecosystem. The logging system is designed for production use with features like:

- **Multiple Formats:** JSON, Pretty, Plain, Compact
- **Multiple Outputs:** STDOUT, STDERR, File, Rolling Files
- **Log Levels:** TRACE, DEBUG, INFO, WARN, ERROR
- **Structured Logging:** Key-value pairs for context
- **HTTP Middleware:** Automatic request/response logging
- **Low Overhead:** Asynchronous, non-blocking logging

**Default Configuration:** JSON format to STDOUT at INFO level

---

## Quick Start

### Basic Logging

```rust
use armature_core::*;

#[tokio::main]
async fn main() {
    // Initialize with defaults (JSON to STDOUT)
    let _guard = Application::init_logging();

    info!("Application started");
    warn!("Warning message");
    error!("Error occurred");
}
```

### Custom Configuration

```rust
use armature_core::*;

#[tokio::main]
async fn main() {
    let config = LogConfig::new()
        .level(LogLevel::Debug)
        .format(LogFormat::Pretty)
        .with_colors(true);

    let _guard = Application::init_logging_with_config(config);

    debug!("Debug information");
    info!("Application running");
}
```

---

## Configuration

### LogConfig

The `LogConfig` struct provides a fluent API for configuring logging:

```rust
let config = LogConfig::new()
    .level(LogLevel::Info)           // Set log level
    .format(LogFormat::Json)         // Set output format
    .output(LogOutput::Stdout)       // Set output destination
    .with_timestamps(true)           // Include timestamps
    .with_thread_ids(false)          // Include thread IDs
    .with_targets(true)              // Include module paths
    .with_file_line(false)           // Include file/line numbers
    .with_spans(false)               // Include span information
    .with_colors(false);             // Enable ANSI colors

let _guard = config.init();
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `level` | `LogLevel` | `Info` | Minimum log level to display |
| `format` | `LogFormat` | `Json` | Output format |
| `output` | `LogOutput` | `Stdout` | Output destination |
| `timestamps` | `bool` | `true` | Include timestamps in logs |
| `thread_ids` | `bool` | `false` | Include thread IDs |
| `targets` | `bool` | `true` | Include module paths |
| `file_line` | `bool` | `false` | Include file and line numbers |
| `spans` | `bool` | `false` | Include tracing span information |
| `colors` | `bool` | `false` | Enable ANSI color codes |

---

## Log Formats

### JSON Format (Default)

Machine-readable, structured format ideal for production and log aggregators.

```json
{"timestamp":"2024-01-01T12:00:00.123Z","level":"INFO","target":"my_app","fields":{"message":"User logged in","user_id":123}}
```

**Use Cases:**
- Production environments
- Log aggregation (ELK, Splunk, Datadog)
- Automated log parsing
- Cloud environments

**Configuration:**
```rust
LogConfig::new().format(LogFormat::Json)
```

### Pretty Format

Formatted, colored output for development.

```
2024-01-01T12:00:00.123Z  INFO my_app: User logged in
    user_id: 123
    ip_address: "192.168.1.1"
```

**Use Cases:**
- Local development
- Debugging
- Interactive terminal use

**Configuration:**
```rust
LogConfig::new()
    .format(LogFormat::Pretty)
    .with_colors(true)
```

### Plain Format

Simple, human-readable format.

```
2024-01-01T12:00:00.123Z INFO my_app: User logged in
```

**Use Cases:**
- Simple applications
- Log files without parsing
- Quick debugging

**Configuration:**
```rust
LogConfig::new().format(LogFormat::Plain)
```

### Compact Format

Minimal output format.

```
INFO User logged in
```

**Use Cases:**
- Low-volume logging
- Development quick-checks
- Performance-critical paths

**Configuration:**
```rust
LogConfig::new().format(LogFormat::Compact)
```

---

## Log Levels

### Available Levels

| Level | Use Case | Example |
|-------|----------|---------|
| `TRACE` | Very detailed debugging | Function entry/exit, loop iterations |
| `DEBUG` | Development information | Variable values, state changes |
| `INFO` | General information | App start, config loaded, request processed |
| `WARN` | Potential issues | Deprecated API used, fallback activated |
| `ERROR` | Failures requiring attention | Database error, API call failed |

### Setting Log Level

```rust
// Configuration
LogConfig::new().level(LogLevel::Debug)

// Or use environment variable
// RUST_LOG=debug cargo run
```

### Logging Macros

```rust
use armature_core::*;

trace!("Entering function");
debug!("Processing item {}", id);
info!("User {} logged in", username);
warn!("Rate limit approaching: {}/100", count);
error!("Failed to connect to database: {}", err);
```

---

## Output Destinations

### STDOUT (Default)

```rust
LogConfig::new().output(LogOutput::Stdout)
```

### STDERR

```rust
LogConfig::new().output(LogOutput::Stderr)
```

### File

```rust
LogConfig::new().output(LogOutput::File("app.log".to_string()))
```

### Rolling Files

Automatic file rotation based on time.

```rust
LogConfig::new().output(LogOutput::RollingFile {
    directory: "logs".to_string(),
    prefix: "armature".to_string(),
    rotation: Rotation::Daily,
})
```

**Rotation Options:**
- `Rotation::Minutely` - Rotate every minute (testing)
- `Rotation::Hourly` - Rotate every hour
- `Rotation::Daily` - Rotate daily
- `Rotation::Never` - Never rotate

**Example File Names:**
```
logs/armature.2024-01-01
logs/armature.2024-01-02
logs/armature.2024-01-03
```

---

## Structured Logging

Add context to log messages with key-value pairs.

### Basic Structured Logging

```rust
info!(
    user_id = 123,
    action = "login",
    ip_address = "192.168.1.1",
    "User authentication successful"
);
```

**JSON Output:**
```json
{
  "timestamp": "2024-01-01T12:00:00.123Z",
  "level": "INFO",
  "message": "User authentication successful",
  "user_id": 123,
  "action": "login",
  "ip_address": "192.168.1.1"
}
```

### Complex Types

```rust
// Strings
info!(name = "Alice", "User created");

// Numbers
info!(count = 42, duration_ms = 150, "Query completed");

// Display format
info!(error = %err, "Operation failed");

// Debug format
info!(config = ?my_config, "Configuration loaded");
```

### Multiple Fields

```rust
error!(
    operation = "database_query",
    table = "users",
    query_id = 123,
    duration_ms = 1500,
    error_code = "TIMEOUT",
    error = %err,
    "Database query timeout"
);
```

---

## HTTP Request Logging

### Automatic Request Logging

Use the `LoggingMiddleware` to automatically log HTTP requests and responses.

```rust
use armature_core::*;

let mut chain = MiddlewareChain::new();
chain.use_middleware(LoggingMiddleware::new());

// Logs:
// INFO method="GET" path="/api/users" status=200 duration_ms=45 "HTTP request completed"
```

### Configuration

```rust
let logging = LoggingMiddleware::new()
    .with_request_body(true)   // Log request bodies
    .with_response_body(true)  // Log response bodies
    .with_max_body_size(1024); // Max body size to log

chain.use_middleware(logging);
```

### Example Logs

```json
{
  "timestamp": "2024-01-01T12:00:00.123Z",
  "level": "INFO",
  "message": "HTTP request completed",
  "method": "POST",
  "path": "/api/users",
  "status": 201,
  "duration_ms": 123,
  "body_size": 256
}
```

### Error Logging

```json
{
  "timestamp": "2024-01-01T12:00:05.456Z",
  "level": "ERROR",
  "message": "Request failed",
  "method": "GET",
  "path": "/api/data",
  "status": 500,
  "duration_ms": 12,
  "error": "Database connection failed"
}
```

---

## Best Practices

### 1. Use Appropriate Log Levels

```rust
// ✅ Good
info!("User {} logged in", user_id);              // General info
warn!("Rate limit exceeded for IP {}", ip);       // Potential issue
error!("Failed to save user: {}", err);           // Actual error

// ❌ Bad
info!("Database error occurred");                 // Should be ERROR
error!("User clicked button");                    // Should be DEBUG or none
debug!("Critical payment processing failed");     // Should be ERROR
```

### 2. Add Context with Structured Logging

```rust
// ✅ Good - provides context
error!(
    user_id = user.id,
    order_id = order.id,
    payment_method = "credit_card",
    error = %err,
    "Payment processing failed"
);

// ❌ Bad - missing context
error!("Payment failed");
```

### 3. Don't Log Sensitive Data

```rust
// ❌ Bad - logs sensitive data
info!(password = %user.password, "User logged in");
info!(credit_card = %card_number, "Payment processed");

// ✅ Good - masks or omits sensitive data
info!(user_id = user.id, "User logged in");
info!(card_last_4 = %last_4_digits, "Payment processed");
```

### 4. Use Consistent Field Names

```rust
// ✅ Good - consistent naming
info!(user_id = 123, "Action performed");
info!(user_id = 456, "Another action");

// ❌ Bad - inconsistent naming
info!(userId = 123, "Action performed");
info!(user_identifier = 456, "Another action");
```

### 5. Log at Application Boundaries

```rust
// Log when:
// - Request received
// - Request completed
// - External service called
// - External service responded
// - Database operation performed
// - Critical state change

info!(method = "GET", path = %path, "Request received");
// ... process request ...
info!(method = "GET", path = %path, status = 200, "Request completed");
```

### 6. Don't Log in Loops (Usually)

```rust
// ❌ Bad - logs 1000 times
for item in items {
    debug!("Processing item {}", item.id);
}

// ✅ Good - log once
debug!("Processing {} items", items.len());
// ... process items ...
info!("Processed {} items successfully", processed_count);
```

### 7. Initialize Logging Early

```rust
#[tokio::main]
async fn main() {
    // Initialize logging FIRST
    let _guard = Application::init_logging();

    info!("Application starting");
    // ... rest of application ...
}
```

---

## Performance

### Logging Overhead

Armature's logging system is designed for minimal overhead:

- **Async writes:** Non-blocking I/O
- **Lazy evaluation:** Only evaluates log statements that will be output
- **Efficient formatting:** Optimized for JSON output
- **No locks:** Lock-free write path

### Benchmarks

| Operation | Time | Overhead |
|-----------|------|----------|
| Filtered out log (TRACE when INFO) | ~5ns | Negligible |
| Simple info! message | ~200ns | Very low |
| Structured log (5 fields) | ~500ns | Low |
| JSON formatting | ~1μs | Low |

### Tips for Performance

1. **Use appropriate log levels** - DEBUG/TRACE disabled in production
2. **Avoid expensive operations** - Don't compute values if log is filtered
3. **Use lazy evaluation** - Use closures for expensive computations
4. **Batch logging** - Log summaries instead of individual items

```rust
// ✅ Good - lazy evaluation
debug!("Expensive computation: {}", || expensive_fn());

// ❌ Bad - always computed
debug!("Expensive computation: {}", expensive_fn());
```

---

## Examples

### Production Configuration

```rust
use armature_core::*;

#[tokio::main]
async fn main() {
    let config = LogConfig::new()
        .level(LogLevel::Info)
        .format(LogFormat::Json)
        .output(LogOutput::RollingFile {
            directory: "/var/log/myapp".to_string(),
            prefix: "app".to_string(),
            rotation: Rotation::Daily,
        })
        .with_timestamps(true)
        .with_targets(true);

    let _guard = config.init();

    info!("Production application started");
    // ... application code ...
}
```

### Development Configuration

```rust
let config = LogConfig::new()
    .level(LogLevel::Debug)
    .format(LogFormat::Pretty)
    .output(LogOutput::Stdout)
    .with_colors(true)
    .with_file_line(true);

let _guard = config.init();
```

### Environment-Based Configuration

```rust
use std::env;

let log_level = match env::var("LOG_LEVEL").unwrap_or_default().as_str() {
    "trace" => LogLevel::Trace,
    "debug" => LogLevel::Debug,
    "warn" => LogLevel::Warn,
    "error" => LogLevel::Error,
    _ => LogLevel::Info,
};

let config = LogConfig::new()
    .level(log_level)
    .format(if env::var("LOG_JSON").is_ok() {
        LogFormat::Json
    } else {
        LogFormat::Pretty
    });

let _guard = config.init();
```

### Custom Filter

```rust
// Advanced: Custom environment filter
let config = LogConfig::new()
    .with_env_filter("myapp=debug,hyper=info,tokio=warn");

let _guard = config.init();

// Now:
// - myapp logs at DEBUG and above
// - hyper logs at INFO and above
// - tokio logs at WARN and above
```

---

## Summary

### Key Features

✅ **Highly Configurable** - Multiple formats, outputs, and levels
✅ **JSON Default** - Production-ready out of the box
✅ **Structured Logging** - Rich context with key-value pairs
✅ **HTTP Middleware** - Automatic request logging
✅ **Low Overhead** - Async, non-blocking logging
✅ **Production Ready** - Battle-tested tracing ecosystem

### Quick Reference

```rust
// Initialize
let _guard = Application::init_logging();

// Log messages
trace!("Very detailed");
debug!("Development info");
info!("General information");
warn!("Potential issue");
error!("Failure occurred");

// Structured logging
info!(user_id = 123, action = "login", "User authenticated");

// HTTP logging
chain.use_middleware(LoggingMiddleware::new());
```

### Next Steps

- See `examples/comprehensive_logging.rs` for complete examples
- Read `armature-core/src/logging.rs` for implementation details
- Check out the [tracing documentation](https://docs.rs/tracing) for advanced features

---

**Happy logging!** 📝

