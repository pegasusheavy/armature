# Use Middleware Decorator Guide

Armature provides the `#[use_middleware]` decorator for applying middleware to individual route handlers or entire controllers. This enables declarative middleware configuration at the route level.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Global Middleware](#global-middleware)
- [Basic Usage](#basic-usage)
- [Multiple Middleware](#multiple-middleware)
- [Controller-Level Middleware](#controller-level-middleware)
- [Built-in Middleware](#built-in-middleware)
- [Custom Middleware](#custom-middleware)
- [Execution Order](#execution-order)
- [Best Practices](#best-practices)
- [API Reference](#api-reference)
- [Summary](#summary)

## Overview

The `#[use_middleware]` decorator wraps route handlers with a middleware chain, allowing you to:

- Apply middleware to specific routes without affecting others
- Chain multiple middleware together
- Keep middleware configuration close to the routes they affect
- Create reusable middleware combinations

## Features

- ✅ Route-level middleware application
- ✅ Controller-level middleware inheritance
- ✅ Multiple middleware chaining
- ✅ Works with all HTTP method decorators
- ✅ Compatible with request extractors
- ✅ Type-safe middleware configuration

## Global Middleware

Middleware that must run for every request, authentication in particular, belongs on the
application. `Application::use_middleware` adds global middleware, run in the order added
(first = outermost):

```rust
use armature::prelude::*;
use armature::{LoggingMiddleware, RequestIdMiddleware};
use armature_auth::JwtAuthMiddleware;
use armature_jwt::{JwtConfig, JwtManager};

// JwtConfig::new returns a Result and rejects secrets shorter than 32 bytes.
let jwt = JwtManager::new(JwtConfig::new(std::env::var("JWT_SECRET")?)?)?;

let app = Application::create::<AppModule>()
    .await
    .use_middleware(RequestIdMiddleware)
    .use_middleware(LoggingMiddleware::new())
    .use_middleware(JwtAuthMiddleware::new(jwt));
```

Global middleware sees the canonical request path (a path with `.`/`..` segments or an
encoded slash has already been refused with `400`) and also runs for requests that match no
route.

### The Request Pipeline

Armature follows the NestJS order:

1. Global middleware (`Application::use_middleware`)
2. Global guards (`Application::with_guard`)
3. Route-scoped middleware, then route-scoped guards (controller `#[middleware]` runs
   **before** controller `#[guard]`; `RouteGroup` middleware before group guards)
4. The handler

Because middleware runs before guards, `JwtAuthMiddleware` can verify the token and attach
`VerifiedIdentity` / `RequestRoles` / `UserContext` before `AuthenticationGuard`,
`RolesGuard`, `RoleGuard` or `PermissionGuard` inspect them. Middleware also sees, and can
post-process, the `401`/`403` a guard produces. A middleware that previously relied on only
running for already-authorized requests must check authorization itself.

## Basic Usage

The middleware attributes are re-exported by the framework crate
(`use armature::{middleware, use_middleware};`). Their expansion names `armature_core::`
paths, so keep `armature-core` as a direct dependency next to `armature`.

### Single Middleware

Apply a single middleware to a route:

```rust
use armature::get;
use armature::use_middleware;
use armature_core::{HttpRequest, HttpResponse, Error, LoggerMiddleware};

#[use_middleware(LoggerMiddleware::new())]
#[get("/users")]
async fn get_users(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok().with_json(&serde_json::json!({
        "users": []
    }))?)
}
```

### With Configuration

Middleware can be configured inline:

```rust
use armature::get;
use armature::use_middleware;
use armature_core::{HttpRequest, HttpResponse, Error, CorsMiddleware};

#[use_middleware(CorsMiddleware::new().allow_origin("https://example.com"))]
#[get("/api/data")]
async fn get_data(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok())
}
```

## Multiple Middleware

Chain multiple middleware together by separating them with commas:

```rust
use armature::get;
use armature::use_middleware;
use armature_core::{
    HttpRequest, HttpResponse, Error,
    LoggerMiddleware, CorsMiddleware, SecurityHeadersMiddleware
};

#[use_middleware(
    LoggerMiddleware::new(),
    CorsMiddleware::new(),
    SecurityHeadersMiddleware::new()
)]
#[get("/protected")]
async fn protected_endpoint(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok())
}
```

Middleware executes in the order specified:
1. `LoggerMiddleware` runs first (logs incoming request)
2. `CorsMiddleware` runs second (adds CORS headers)
3. `SecurityHeadersMiddleware` runs third (adds security headers)
4. Handler executes
5. Middleware runs in reverse order for response processing

## Controller-Level Middleware

Apply middleware to all routes in a controller using the `#[middleware]` decorator:

```rust
use armature::{controller, get, post};
use armature::middleware;
use armature_core::{HttpRequest, HttpResponse, Error, LoggerMiddleware, CorsMiddleware};

#[middleware(LoggerMiddleware::new(), CorsMiddleware::new())]
#[controller("/api")]
struct ApiController;

impl ApiController {
    // All routes in this controller automatically have logging and CORS

    #[get("/users")]
    async fn get_users(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::ok())
    }

    #[post("/users")]
    async fn create_user(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::created())
    }
}
```

### Controller Middleware and Guards

A controller's `#[middleware(...)]` runs before its `#[guard(...)]`s, so authentication can
live on the controller:

```rust
use armature::{guard, middleware};
use armature_auth::{AuthGuard, JwtAuthMiddleware};

#[controller("/admin")]
#[middleware(jwt_auth())] // runs first: verifies the token, attaches UserContext
#[guard(AuthGuard)]       // runs second: requires that UserContext
struct AdminController;
```

(`jwt_auth()` is any function returning a configured `JwtAuthMiddleware`; struct-level
arguments are expressions evaluated once when the routes are registered.)

### Combining Controller and Route Middleware

Route-level middleware adds to controller-level middleware:

```rust
use armature::{controller, get};
use armature::{middleware, use_middleware};
use armature_core::{
    HttpRequest, HttpResponse, Error,
    LoggerMiddleware, TimeoutMiddleware
};

#[middleware(LoggerMiddleware::new())]  // Applied to all routes
#[controller("/api")]
struct ApiController;

impl ApiController {
    #[get("/fast")]
    async fn fast_endpoint(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // Only has LoggerMiddleware
        Ok(HttpResponse::ok())
    }

    #[use_middleware(TimeoutMiddleware::new(30))]  // Additional middleware
    #[get("/slow")]
    async fn slow_endpoint(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // Has LoggerMiddleware + TimeoutMiddleware
        Ok(HttpResponse::ok())
    }
}
```

## Built-in Middleware

Armature provides several built-in middleware:

| Middleware | Purpose |
|------------|---------|
| `LoggerMiddleware` | Log requests and responses |
| `LoggingMiddleware` | Structured logging with tracing |
| `CorsMiddleware` | Handle CORS headers |
| `SecurityHeadersMiddleware` | Add security headers (HSTS, XSS, etc.) |
| `TimeoutMiddleware` | Request timeout handling |
| `BodySizeLimitMiddleware` | Limit request body size |
| `RequestIdMiddleware` | Add unique request IDs |
| `CompressionMiddleware` | Response compression hints |

### Example: Common Middleware Stack

```rust
#[use_middleware(
    RequestIdMiddleware,
    LoggingMiddleware::new(),
    SecurityHeadersMiddleware::new(),
    CorsMiddleware::new().allow_credentials(true),
    TimeoutMiddleware::new(30)
)]
#[get("/api/secure")]
async fn secure_endpoint(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok())
}
```

## Custom Middleware

Create custom middleware by implementing the `Middleware` trait:

```rust
use armature_core::{HttpRequest, HttpResponse, Error};
use armature_core::middleware::{Middleware, Next};
use async_trait::async_trait;

/// Authentication middleware
pub struct AuthMiddleware {
    api_key: String,
}

impl AuthMiddleware {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self { api_key: api_key.into() }
    }
}

#[async_trait]
impl Middleware for AuthMiddleware {
    async fn handle(&self, mut req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        // Header lookup is case-insensitive.
        let auth_header = req.headers.get("x-api-key").map(str::to_owned);

        match auth_header {
            // Compare secrets in constant time.
            Some(key) if armature_core::crypto::constant_time_eq(
                key.as_bytes(),
                self.api_key.as_bytes(),
            ) => {
                // Verified: tell the guards who the caller is, then proceed.
                req.extensions
                    .insert(armature_core::VerifiedIdentity::new("api-client"));
                next(req).await
            }
            Some(_) => {
                // Invalid key
                Ok(HttpResponse::new(401)
                    .with_json(&serde_json::json!({
                        "error": "Invalid API key"
                    }))?)
            }
            None => {
                // Missing key
                Ok(HttpResponse::new(401)
                    .with_json(&serde_json::json!({
                        "error": "Missing API key"
                    }))?)
            }
        }
    }
}
```

Use your custom middleware:

```rust
use armature::get;
use armature::use_middleware;
use my_app::AuthMiddleware;

#[use_middleware(AuthMiddleware::new(
    std::env::var("API_KEY").expect("API_KEY must be set"),
))]
#[get("/protected")]
async fn protected(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok())
}
```

### Middleware with State

Middleware can maintain state:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct RateLimitMiddleware {
    requests: Arc<AtomicU64>,
    max_requests: u64,
}

impl RateLimitMiddleware {
    pub fn new(max_requests: u64) -> Self {
        Self {
            requests: Arc::new(AtomicU64::new(0)),
            max_requests,
        }
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        let current = self.requests.fetch_add(1, Ordering::Relaxed);

        if current >= self.max_requests {
            return Ok(HttpResponse::new(429)
                .with_json(&serde_json::json!({
                    "error": "Too many requests"
                }))?);
        }

        next(req).await
    }
}
```

## Execution Order

Understanding middleware execution order is crucial. Within one chain, middleware runs in
the order listed; across layers, global middleware wraps global guards, which wrap
route-scoped middleware, then route-scoped guards, then the handler (see
[The Request Pipeline](#the-request-pipeline)):

```
Request Flow:
┌─────────────────────────────────────────────────────┐
│  Incoming Request                                    │
├─────────────────────────────────────────────────────┤
│  ↓ Middleware 1 (before)                            │
│  ↓ Middleware 2 (before)                            │
│  ↓ Middleware 3 (before)                            │
├─────────────────────────────────────────────────────┤
│  → Handler Executes                                  │
├─────────────────────────────────────────────────────┤
│  ↑ Middleware 3 (after)                             │
│  ↑ Middleware 2 (after)                             │
│  ↑ Middleware 1 (after)                             │
├─────────────────────────────────────────────────────┤
│  Outgoing Response                                   │
└─────────────────────────────────────────────────────┘
```

### Example: Logging Timing

```rust
pub struct TimingMiddleware;

#[async_trait]
impl Middleware for TimingMiddleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        let start = std::time::Instant::now();

        // Execute next middleware/handler
        let response = next(req).await?;

        // After handler completes
        let duration = start.elapsed();
        println!("Request took {:?}", duration);

        Ok(response)
    }
}
```

## Best Practices

### 1. Order Middleware Carefully

Place middleware in logical order:

```rust
#[use_middleware(
    RequestIdMiddleware,       // First: assign request ID
    LoggingMiddleware::new(),  // Second: log with request ID
    AuthMiddleware::new(key),  // Third: authenticate
    TimeoutMiddleware::new(30) // Fourth: enforce timeout
)]
```

### 2. Keep Middleware Focused

Each middleware should do one thing well:

```rust
// ✅ Good: Single responsibility
pub struct AuthMiddleware;  // Only handles authentication
pub struct LoggingMiddleware;  // Only handles logging

// ❌ Bad: Multiple responsibilities
pub struct EverythingMiddleware;  // Auth + Logging + CORS + ...
```

### 3. Use Controller-Level for Common Middleware

```rust
// ✅ Good: Common middleware at controller level
#[middleware(LoggingMiddleware::new(), CorsMiddleware::new())]
#[controller("/api")]
struct ApiController;

// ❌ Bad: Repeating on every route
impl ApiController {
    #[use_middleware(LoggingMiddleware::new(), CorsMiddleware::new())]
    #[get("/users")]
    async fn get_users(&self, req: HttpRequest) -> Result<HttpResponse, Error> { ... }

    #[use_middleware(LoggingMiddleware::new(), CorsMiddleware::new())]
    #[get("/posts")]
    async fn get_posts(&self, req: HttpRequest) -> Result<HttpResponse, Error> { ... }
}
```

### 4. Handle Errors Gracefully

Middleware should handle its own errors:

```rust
#[async_trait]
impl Middleware for MyMiddleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        // Handle potential errors
        match self.validate(&req) {
            Ok(()) => next(req).await,
            Err(e) => {
                // Return error response instead of propagating
                Ok(HttpResponse::bad_request()
                    .with_json(&serde_json::json!({
                        "error": e.to_string()
                    }))?)
            }
        }
    }
}
```

### 5. Avoid Cloning Large Data

```rust
// ✅ Good: Only clone what's needed
let path = req.path.clone();
let method = req.method.clone();
let response = next(req).await?;

// ❌ Bad: Cloning entire request
let req_clone = req.clone();
let response = next(req).await?;
// req_clone is now stale
```

## Common Pitfalls

### ❌ Forgetting to Call `next`

```rust
// Bad: Handler never executes
async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
    // Missing: next(req).await
    Ok(HttpResponse::ok())  // Short-circuits the chain
}

// Good: Always call next (or return error response)
async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
    if !self.validate(&req) {
        return Ok(HttpResponse::unauthorized());
    }
    next(req).await  // Continue to handler
}
```

### ❌ Modifying Response Without Returning It

```rust
// Bad: Modifications lost
async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
    let response = next(req).await?;
    response.headers.insert("X-Custom".to_string(), "value".to_string());
    // Missing: return modified response
    Ok(HttpResponse::ok())
}

// Good: Return modified response
async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
    let mut response = next(req).await?;
    response.headers.insert("X-Custom".to_string(), "value".to_string());
    Ok(response)
}
```

## API Reference

### Decorators

| Decorator | Target | Description |
|-----------|--------|-------------|
| `#[use_middleware(...)]` | Function | Apply middleware to a route handler |
| `#[middleware(...)]` | Struct | Apply middleware to a controller (runs before the controller's guards) |
| `Application::use_middleware(m)` | Application | Global middleware, before every guard |

### Built-in Middleware Types

| Type | Constructor | Description |
|------|-------------|-------------|
| `LoggerMiddleware` | `new()` | Simple console logging |
| `LoggingMiddleware` | `new()` | Structured tracing logs |
| `CorsMiddleware` | `new()` | CORS header management |
| `SecurityHeadersMiddleware` | `new()` | Security headers (HSTS, XSS, etc.) |
| `TimeoutMiddleware` | `new(seconds)` | Request timeout |
| `BodySizeLimitMiddleware` | `new(bytes)` | Body size limits |
| `RequestIdMiddleware` | (unit struct) | Request ID generation |
| `CompressionMiddleware` | `new()` | Compression hints |

## Summary

**Key Points:**

1. **`#[use_middleware]`** applies middleware to individual routes
2. **`#[middleware]`** applies middleware to entire controllers, ahead of their guards
3. **`Application::use_middleware`** applies middleware globally; register authentication here
4. Middleware executes in **order specified** (before handler) and **reverse order** (after handler); middleware runs before guards
5. Always call `next(req).await` unless short-circuiting
6. Use controller-level middleware for common functionality
7. Keep middleware focused on single responsibilities

**Quick Reference:**

```rust
// Route-level
#[use_middleware(LoggerMiddleware::new())]
#[get("/users")]
async fn get_users(req: HttpRequest) -> Result<HttpResponse, Error> { ... }

// Controller-level
#[middleware(LoggerMiddleware::new())]
#[controller("/api")]
struct ApiController;

// Multiple middleware
#[use_middleware(
    LoggerMiddleware::new(),
    CorsMiddleware::new(),
    TimeoutMiddleware::new(30)
)]
#[get("/protected")]
async fn protected(req: HttpRequest) -> Result<HttpResponse, Error> { ... }
```

