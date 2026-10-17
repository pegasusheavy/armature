# Rate Limiting Guide

Rate limiting protects your API from abuse and ensures fair usage across all clients.
The `armature-ratelimit` crate provides token bucket, sliding window and fixed window
limiters with in-memory and Redis storage, plus an HTTP middleware.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Quick Start](#quick-start)
- [Algorithms](#algorithms)
- [Storage Backends](#storage-backends)
- [Key Extraction](#key-extraction)
- [Middleware Integration](#middleware-integration)
- [Configuration](#configuration)
- [Best Practices](#best-practices)
- [Common Pitfalls](#common-pitfalls)
- [API Reference](#api-reference)

## Overview

Rate limiting controls how many requests a client can make within a time window.
This prevents:

- **Abuse**: Malicious actors overwhelming your API
- **Resource exhaustion**: A single client consuming all server resources
- **Cascading failures**: Overload propagating through your system
- **Cost overruns**: Excessive usage driving up infrastructure costs

## Features

- ✅ **Multiple Algorithms**: Token bucket, sliding window log, fixed window
- ✅ **Distributed Support**: Redis backend for multi-instance deployments
- ✅ **Flexible Key Extraction**: By client address, verified user ID, validated API key, or custom function
- ✅ **Spoof-Resistant Client Address**: Socket peer by default; `X-Forwarded-For` only with an explicit trusted proxy depth
- ✅ **Fail-Closed for Unkeyed Requests**: A request no key can be derived from gets a 429 by default
- ✅ **Standard Headers**: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`
- ✅ **Per-Route Limits**: Different limits for different endpoints
- ✅ **Bypass Rules**: Whitelist specific clients or API keys
- ✅ **Fail-Open Mode**: Continue serving requests if rate limit storage fails

## Quick Start

Add the dependency:

```toml
[dependencies]
armature-ratelimit = "0.5"
```

Basic usage:

```rust
use armature_ratelimit::{RateLimiter, Algorithm};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a rate limiter with token bucket algorithm
    let limiter = RateLimiter::builder()
        .algorithm(Algorithm::TokenBucket {
            capacity: 100,        // Maximum burst size
            refill_rate: 10.0,    // 10 tokens per second
        })
        .build()
        .await?;

    // Check if a request is allowed
    let result = limiter.check("client_ip_123").await?;

    if result.allowed {
        println!("Request allowed! {} remaining", result.remaining);
    } else {
        println!("Rate limited. Retry after {:?}", result.retry_after);
    }

    Ok(())
}
```

## Algorithms

### Token Bucket

The token bucket algorithm provides smooth rate limiting with burst capacity.

**How it works:**
1. A bucket starts full with `capacity` tokens
2. Each request consumes one token
3. Tokens are refilled at `refill_rate` per second
4. Requests are denied when the bucket is empty

**Best for:** APIs that allow occasional bursts but need average rate control.

```rust
use armature_ratelimit::Algorithm;

let algo = Algorithm::TokenBucket {
    capacity: 100,      // Allow bursts up to 100 requests
    refill_rate: 10.0,  // Steady rate of 10 requests/second
};
```

**Example scenario:** A user can make up to 100 rapid requests, then must wait
for tokens to refill at 10/second.

### Sliding Window Log

The sliding window log algorithm provides precise rate limiting by tracking
individual request timestamps.

**How it works:**
1. Each request timestamp is logged
2. On each request, count timestamps within the window
3. Deny if count exceeds `max_requests`
4. Old timestamps are automatically cleaned up

**Best for:** Strict rate limiting where accuracy is critical.

```rust
use armature_ratelimit::Algorithm;
use std::time::Duration;

let algo = Algorithm::SlidingWindowLog {
    max_requests: 100,              // 100 requests...
    window: Duration::from_secs(60), // ...per minute
};
```

**Trade-offs:**
- ✅ Most accurate algorithm
- ✅ No boundary issues
- ❌ Higher memory usage (stores all timestamps)

### Fixed Window

The fixed window algorithm divides time into fixed intervals and counts
requests per window.

**How it works:**
1. Time is divided into fixed windows (e.g., every minute)
2. Each request increments the counter for the current window
3. Counter resets when a new window starts
4. Deny if counter exceeds `max_requests`

**Best for:** Simple use cases, lowest resource usage.

```rust
use armature_ratelimit::Algorithm;
use std::time::Duration;

let algo = Algorithm::FixedWindow {
    max_requests: 100,
    window: Duration::from_secs(60),
};
```

**Trade-offs:**
- ✅ Simple and efficient
- ✅ Lowest memory usage
- ❌ Boundary burst issue: clients can make 2x requests at window boundaries

### Algorithm Comparison

| Algorithm | Accuracy | Memory | Complexity | Burst Handling |
|-----------|----------|--------|------------|----------------|
| Token Bucket | Medium | Low | Low | Controlled bursts |
| Sliding Window | High | Medium | Medium | No bursts |
| Fixed Window | Low | Very Low | Very Low | Boundary bursts |

## Storage Backends

### In-Memory Store (Default)

Uses DashMap for thread-safe concurrent access. Suitable for single-instance
deployments or development.

```rust
let limiter = RateLimiter::builder()
    .algorithm(Algorithm::token_bucket_default())
    .memory_store()  // This is the default
    .build()
    .await?;
```

**Pros:**
- Zero latency
- No external dependencies
- Simple setup

**Cons:**
- Not shared across instances
- State lost on restart

### Redis Store

Uses Redis for distributed rate limiting. Required for multi-instance deployments.

```rust
let limiter = RateLimiter::builder()
    .algorithm(Algorithm::token_bucket_default())
    .redis_store("redis://localhost:6379")
    .build()
    .await?;
```

**Pros:**
- Shared across all instances
- Persistent state
- Atomic operations via Lua scripts

**Cons:**
- Network latency
- Requires Redis infrastructure

**Enable the feature:**

```toml
[dependencies]
armature-ratelimit = { version = "0.5", features = ["redis"] }
```

The Redis store names its keys by the SHA-256 digest of the rate-limit key,
so raw client addresses, user IDs and API keys never appear in Redis. The
connection URL, which may hold a password, is not logged. Upgrading from an
earlier release starts every bucket fresh, because the key names changed.

## Key Extraction

Rate limits are applied per-key. The key extraction strategy determines how
clients are identified.

### By Client Address (Default)

```rust
use armature_ratelimit::{RateLimitMiddleware, KeyExtractor};

let middleware = RateLimitMiddleware::new(limiter)
    .with_extractor(KeyExtractor::Ip);
```

The middleware takes the address from
`HttpRequest::client_address(trusted_proxy_depth)`:

- **Depth `0` (default)**: the socket peer. `X-Forwarded-For` is ignored, so
  a client cannot pick its own bucket by sending the header.
- **Depth `N`**: the `N`-th `X-Forwarded-For` hop from the right, read across
  every `X-Forwarded-For` field line. Set `N` to the number of reverse proxies
  you run in front of the app. If the chain is shorter than `N`, the
  request has no address and is treated as unkeyed.
- `X-Real-IP` is never consulted.

IPv6 clients are bucketed by `/64`, because one host usually controls a
whole `/64`. IPv4-mapped IPv6 addresses are bucketed as IPv4.

```rust
// Behind exactly one load balancer that appends to X-Forwarded-For
let middleware = RateLimitMiddleware::new(limiter)
    .with_trusted_proxy_depth(1);

// Or set it on the limiter config
let limiter = RateLimiter::builder()
    .token_bucket(100, 10.0)
    .trusted_proxy_depth(1)
    .build()
    .await?;
```

### By User ID

`KeyExtractor::UserId` keys **only** on the `armature_core::VerifiedIdentity`
request extension that an authentication middleware (for example
`armature_auth::JwtAuthMiddleware`) inserts after verifying a credential.
Client headers such as `X-User-Id` are ignored. The rate-limit middleware must
therefore run **after** the authentication middleware. Without a verified
identity the request is unkeyed and, by default, denied.

```rust
let app = Application::create::<AppModule>()
    .await
    .use_middleware(JwtAuthMiddleware::new(jwt_manager))   // inserts VerifiedIdentity
    .use_middleware(
        RateLimitMiddleware::new(limiter).with_extractor(KeyExtractor::UserId),
    );
```

### By API Key

Extracts the key from a header (e.g., `X-API-Key`) and keys on it **only if
the validator accepts it**. The bucket is named `apikey:<sha256 of the key>`.
Clients rotating junk keys therefore cannot mint unlimited buckets, and the
plaintext key is never stored.

```rust
use std::collections::HashSet;
use std::sync::Arc;

let known: Arc<HashSet<String>> = Arc::new(load_api_keys());
let middleware = RateLimitMiddleware::new(limiter)
    .with_extractor(KeyExtractor::api_key("X-API-Key", move |key: &str| {
        known.contains(key)
    }));
```

### By IP and Path

Different limits per endpoint.

```rust
let middleware = RateLimitMiddleware::new(limiter)
    .with_extractor(KeyExtractor::IpAndPath);
```

The path is normalized (`//`, `.`, `..`, a trailing `/` and the query string
are removed), so `/api/users`, `/api//users/` and `/api/users?x=1` share one
bucket. `KeyExtractor::UserIdAndPath` does the same for a verified user ID.

### Custom Extractor

Build complex extraction logic:

```rust
use armature_ratelimit::extractor::KeyExtractorBuilder;

let extractor = KeyExtractorBuilder::new()
    .prefer_user_id()                                      // Verified user ID first
    .prefer_api_key("X-API-Key", |key: &str| is_known(key)) // Then a validated API key
    // Falls back to the client address automatically
    .build();
```

### Unkeyed Requests

A request is **unkeyed** when the extractor has nothing to work with: no
client address, no `VerifiedIdentity` under `UserId`, or no API key that
passed validation under `ApiKey`. Letting such requests through would let
any caller bypass the limiter by withholding the key, so the default
`UnkeyedRequestPolicy::Deny` answers them with the same 429 as an exceeded
limit.

```rust
use armature_ratelimit::middleware::UnkeyedRequestPolicy;

// Only where another layer protects the route, and never on login,
// password reset or OTP endpoints
let middleware = RateLimitMiddleware::new(limiter)
    .with_extractor(KeyExtractor::UserId)
    .with_unkeyed_policy(UnkeyedRequestPolicy::Allow);

// Requests let through this way are counted
let allowed_unkeyed = middleware.unkeyed_allow_count();
```

### Logging

Rate-limit keys are never logged verbatim. Logs carry a 12-hex-character
SHA-256 fingerprint (`extractor::key_fingerprint`), so client addresses, user
IDs and API keys stay out of log storage.

## Middleware Integration

### Basic Middleware

```rust
use armature_ratelimit::{RateLimiter, RateLimitMiddleware, Algorithm};
use std::sync::Arc;

let limiter = Arc::new(
    RateLimiter::builder()
        .token_bucket(100, 10.0)
        .build()
        .await?
);

let middleware = RateLimitMiddleware::new(limiter)
    .with_headers(true)  // Include X-RateLimit-* headers
    .with_error_message("Too many requests. Please slow down.");
```

### Checking Requests

```rust
use armature_ratelimit::extractor::RequestInfo;
use std::net::{IpAddr, Ipv4Addr};

// Extract request info from your HTTP framework
let info = RequestInfo::new("/api/users", "GET")
    .with_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)))
    .with_user_id("user_123");

// Check rate limit
let response = middleware.check(&info).await;

match response {
    RateLimitCheckResponse::Allowed { headers } => {
        // Add headers to response and continue
        if let Some(h) = headers {
            // Add X-RateLimit-Limit, X-RateLimit-Remaining, etc.
        }
    }
    RateLimitCheckResponse::Limited { headers, message, retry_after } => {
        // Return 429 Too Many Requests
        // Include Retry-After header
    }
}
```

## Configuration

### Builder Options

```rust
let limiter = RateLimiter::builder()
    // Algorithm (required)
    .algorithm(Algorithm::TokenBucket {
        capacity: 100,
        refill_rate: 10.0,
    })

    // Or use convenience methods
    .token_bucket(100, 10.0)
    .sliding_window(100, Duration::from_secs(60))
    .fixed_window(100, Duration::from_secs(60))

    // Storage backend
    .memory_store()
    .redis_store("redis://localhost:6379")

    // Key prefix for storage
    .key_prefix("api:ratelimit")

    // Include headers in responses
    .include_headers(true)

    // Fail open on storage errors
    .skip_on_error(true)

    // Custom error message
    .error_message("Rate limit exceeded")

    // Bypass specific keys
    .bypass_key("admin_api_key")
    .bypass_keys(["internal_service", "monitoring"])

    .build()
    .await?;
```

### Response Headers

When enabled, these headers are included in responses:

| Header | Description |
|--------|-------------|
| `X-RateLimit-Limit` | Maximum requests allowed |
| `X-RateLimit-Remaining` | Remaining requests in current window |
| `X-RateLimit-Reset` | Unix timestamp when the limit resets |
| `Retry-After` | Seconds until the client can retry (only on 429) |

## Best Practices

### 1. Choose the Right Algorithm

- **Token Bucket**: Most APIs—allows bursts, smooth average rate
- **Sliding Window**: Financial/gaming APIs—strict, no burst exploitation
- **Fixed Window**: High-volume, latency-sensitive—simple and fast

### 2. Use Redis for Production

```rust
// Single instance: memory is fine
let limiter = RateLimiter::builder()
    .token_bucket(100, 10.0)
    .build()
    .await?;

// Multiple instances: use Redis
let limiter = RateLimiter::builder()
    .token_bucket(100, 10.0)
    .redis_store("redis://redis-cluster:6379")
    .build()
    .await?;
```

### 3. Implement Tiered Limits

```rust
// Different limits for different user tiers
async fn check_rate_limit(user: &User, limiter: &RateLimiter) -> bool {
    let key = match user.tier {
        Tier::Free => format!("free:{}", user.id),
        Tier::Pro => format!("pro:{}", user.id),
        Tier::Enterprise => return true, // No limit
    };

    limiter.check(&key).await.map(|r| r.allowed).unwrap_or(true)
}
```

### 4. Include Helpful Headers

Always include rate limit headers so clients can self-regulate:

```rust
let middleware = RateLimitMiddleware::new(limiter)
    .with_headers(true);
```

### 5. Use Bypass for Internal Services

```rust
let limiter = RateLimiter::builder()
    .token_bucket(100, 10.0)
    .bypass_key("internal_service_key")
    .bypass_key("health_check_key")
    .build()
    .await?;
```

## Common Pitfalls

### ❌ Don't: Leave the proxy depth at 0 behind a proxy

```rust
// Every request comes from the proxy's address: one shared bucket
let middleware = RateLimitMiddleware::new(limiter)
    .with_extractor(KeyExtractor::Ip);
```

### ❌ Don't: Trust the leftmost X-Forwarded-For entry or X-Real-IP

```rust
// The client writes the leftmost entry itself, so it can choose
// a new bucket per request or exhaust someone else's
fn get_real_ip(headers: &Headers) -> Option<IpAddr> {
    headers.get("X-Forwarded-For")
        .and_then(|h| h.split(',').next())
        .and_then(|ip| ip.trim().parse().ok())
}
```

### ✅ Do: Set the trusted proxy depth to your proxy count

```rust
// One proxy in front of the app: key on the hop it observed
let middleware = RateLimitMiddleware::new(limiter)
    .with_extractor(KeyExtractor::Ip)
    .with_trusted_proxy_depth(1);
```

### ❌ Don't: Fail closed on errors

```rust
// If Redis is down, all requests will be denied!
let limiter = RateLimiter::builder()
    .skip_on_error(false)  // Bad for availability
    .build()
    .await?;
```

### ✅ Do: Fail open (default)

```rust
let limiter = RateLimiter::builder()
    .skip_on_error(true)  // Default, allows requests on storage failure
    .build()
    .await?;
```

### ❌ Don't: Use fixed window for strict limits

```rust
// Client can make 200 requests in 2 seconds by timing window boundaries
let algo = Algorithm::FixedWindow {
    max_requests: 100,
    window: Duration::from_secs(60),
};
```

### ✅ Do: Use sliding window for strict limits

```rust
// Accurate limiting, no boundary exploitation
let algo = Algorithm::SlidingWindowLog {
    max_requests: 100,
    window: Duration::from_secs(60),
};
```

## API Reference

### Core Types

- `RateLimiter` - Main rate limiter struct
- `RateLimiterBuilder` - Builder for configuring rate limiters
- `Algorithm` - Rate limiting algorithm enum
- `RateLimitCheckResult` - Result of a rate limit check

### Stores

- `MemoryStore` - In-memory storage using DashMap
- `RedisStore` - Redis-backed distributed storage (requires `redis` feature)

### Middleware

- `RateLimitMiddleware` - HTTP middleware for rate limiting
- `KeyExtractor` - Strategies for extracting rate limit keys
- `ApiKeyValidator` - Accepts only genuine API keys for `KeyExtractor::ApiKey`
- `UnkeyedRequestPolicy` - What to do with requests no key can be derived from (default `Deny`)
- `RequestInfo` - Request information for key extraction

### Helpers (`armature_ratelimit::extractor`)

- `key_digest` / `key_fingerprint` - SHA-256 digest and 12-hex-character log fingerprint of a key
- `api_key_bucket` / `ip_bucket` - Bucket names for API keys and client addresses (IPv6 by /64)
- `normalize_path` - Path normalization used by the `*AndPath` extractors

### Errors

- `RateLimitError` - Error types for rate limiting operations
- `RateLimitHeaders` - Standard rate limit response headers

## Summary

Key takeaways:

1. **Choose the right algorithm** for your use case
2. **Use Redis** for multi-instance deployments
3. **Include headers** so clients can self-regulate
4. **Fail open** to maintain availability
5. **Use tiered limits** for different user classes
6. **Handle proxy IPs** correctly: set `trusted_proxy_depth` to your proxy count; never trust `X-Real-IP` or the leftmost `X-Forwarded-For` entry
7. **Run after authentication** when keying on `UserId`, since only a `VerifiedIdentity` is used
8. **Validate API keys** in the `ApiKey` extractor so junk keys cannot mint buckets

