# armature-ratelimit

Rate limiting middleware for the Armature framework.

## Features

- **Multiple Algorithms** - Token bucket, sliding window, fixed window
- **Redis Backend** - Distributed rate limiting
- **Flexible Keys** - Rate limit by IP, user, API key, etc.
- **Custom Responses** - Configurable 429 responses
- **Headers** - Standard rate limit headers (X-RateLimit-*)

## Installation

```toml
[dependencies]
armature-ratelimit = "0.1"
```

## Quick Start

```rust
use armature_ratelimit::{RateLimiter, RateLimitConfig};

let limiter = RateLimiter::new(RateLimitConfig {
    requests: 100,
    window: Duration::from_secs(60),
    key_extractor: |req| req.header("X-API-Key").unwrap_or("anonymous"),
});

let app = Application::new()
    .with_middleware(limiter)
    .get("/api/data", handler);
```

## Algorithms

### Token Bucket

```rust
let limiter = RateLimiter::token_bucket(100, Duration::from_secs(1));
```

### Sliding Window

```rust
let limiter = RateLimiter::sliding_window(100, Duration::from_secs(60));
```

### Fixed Window

```rust
let limiter = RateLimiter::fixed_window(100, Duration::from_secs(60));
```

## Response Headers

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640000000
```

## License

MIT OR Apache-2.0

