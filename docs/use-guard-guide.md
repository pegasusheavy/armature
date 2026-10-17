# Use Guard Decorator Guide

Armature provides the `#[use_guard]` and `#[guard]` decorators for protecting routes with authorization checks. Guards determine whether a request should be allowed to proceed before the handler executes.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Guards vs Middleware](#guards-vs-middleware)
- [Authentication Comes First](#authentication-comes-first)
- [Basic Usage](#basic-usage)
- [Multiple Guards](#multiple-guards)
- [Guard with Configuration](#guard-with-configuration)
- [Controller-Level Guards](#controller-level-guards)
- [Built-in Guards](#built-in-guards)
- [Custom Guards](#custom-guards)
- [Best Practices](#best-practices)
- [API Reference](#api-reference)
- [Summary](#summary)

## Overview

Guards are authorization checks that run before a route handler. They answer the question: "Should this request be allowed to proceed?"

- **Return `Ok(true)`** → Request proceeds to handler
- **Return `Ok(false)`** → Request is denied (403 Forbidden)
- **Return `Err(e)`** → Request fails with the error

## Features

- ✅ Route-level guard protection
- ✅ Controller-level guard inheritance
- ✅ Multiple guard chaining (all must pass)
- ✅ Type-safe guard configuration
- ✅ Access to request context (headers, params)
- ✅ Works with all HTTP method decorators

## Guards vs Middleware

| Aspect | Guards | Middleware |
|--------|--------|------------|
| Purpose | Authorization (allow/deny) | Request/Response processing |
| Return | `Result<bool, Error>` | `Result<HttpResponse, Error>` |
| Response modification | No | Yes |
| Short-circuit on deny | Yes | Optional |
| Use case | Auth, permissions, rate limiting | Logging, CORS, compression |

**Use guards when** you need to allow or deny access.
**Use middleware when** you need to process requests/responses.

Middleware runs **before** guards (NestJS order: middleware → guards → handler). That is
what lets an authentication middleware attach a verified identity for the guards to check.

## Authentication Comes First

The built-in guards **fail closed**. They never trust an `Authorization` header (or headers
such as `x-user-id` / `x-user-role`) on its own; they admit a request only when an
authentication layer has verified a credential and attached an identity extension:

- `armature_core::AuthenticationGuard` and an empty-role `RolesGuard` need an
  `armature_core::VerifiedIdentity` (or `RequestRoles`).
- `armature_core::RolesGuard` needs `armature_core::RequestRoles`.
- `armature_auth::{AuthGuard, RoleGuard, PermissionGuard}` need `armature_auth::UserContext`.

`armature_auth::JwtAuthMiddleware` verifies a JWT access token and inserts all three.
Register it globally so it runs before every guard:

```rust
use armature_framework::prelude::*;
use armature_auth::JwtAuthMiddleware;
use armature_jwt::{JwtConfig, JwtManager};

// The secret must be >= 32 bytes; JwtConfig::new returns a Result.
let jwt = JwtManager::new(JwtConfig::new(std::env::var("JWT_SECRET")?)?)?;

let app = Application::create::<AppModule>()
    .await
    .use_middleware(JwtAuthMiddleware::new(jwt));
```

Without it, every route behind these guards answers `401 Unauthorized`.

## Basic Usage

The guard attributes live in `armature_proc_macro`.

### Guard Instance

`#[guard(...)]` takes guard *expressions*. A unit struct such as `AuthenticationGuard` is
its own value:

```rust
use armature_framework::get;
use armature_proc_macro::guard;
use armature_core::{HttpRequest, HttpResponse, Error, guard::AuthenticationGuard};

#[guard(AuthenticationGuard)]
#[get("/protected")]
async fn protected_endpoint(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok().with_json(&serde_json::json!({
        "message": "You have access!"
    }))?)
}
```

### Simple Type-Based Guard

`#[use_guard(...)]` takes guard *types* and constructs each with `Default::default()`, so
the type must implement `Default` (`AuthenticationGuard`, `RolesGuard` and `ApiKeyGuard` do
not; use `#[guard(...)]` for them):

```rust
use armature_framework::get;
use armature_proc_macro::use_guard;

#[derive(Default)]
struct PremiumGuard; // implements Guard, see below

#[use_guard(PremiumGuard)]
#[get("/premium")]
async fn premium(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok())
}
```

### Guard Order

Guards are checked in order. If any guard fails, subsequent guards are not checked:

```rust
#[guard(AuthenticationGuard, AdminGuard)]
#[get("/admin")]
async fn admin_endpoint(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Only reaches here if BOTH guards pass
    Ok(HttpResponse::ok())
}
```

## Multiple Guards

Chain multiple guards - all must pass:

```rust
use armature_framework::get;
use armature_proc_macro::guard;
use armature_core::{
    HttpRequest, HttpResponse, Error, RequestRoles,
    guard::AuthenticationGuard,
};

#[derive(Default)]
struct PremiumGuard;

#[async_trait::async_trait]
impl armature_core::guard::Guard for PremiumGuard {
    async fn can_activate(
        &self,
        context: &armature_core::guard::GuardContext
    ) -> Result<bool, Error> {
        // Read the roles the authentication middleware verified, never a
        // client-supplied header such as `x-premium-user`.
        let premium = context
            .request
            .extensions
            .get::<RequestRoles>()
            .is_some_and(|roles| roles.contains("premium"));
        if premium {
            Ok(true)
        } else {
            Err(Error::Forbidden("Premium subscription required".into()))
        }
    }
}

#[guard(AuthenticationGuard, PremiumGuard)]
#[get("/premium-content")]
async fn premium_content(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok())
}
```

## Guard with Configuration

Use `#[guard(...)]` for guards that need configuration:

```rust
use armature_framework::get;
use armature_proc_macro::guard;
use armature_core::{HttpRequest, HttpResponse, Error, guard::ApiKeyGuard};

fn api_keys() -> Vec<String> {
    // Long random keys from configuration, never literals in source.
    std::env::var("API_KEYS")
        .expect("API_KEYS must be set")
        .split(',')
        .map(str::to_string)
        .collect()
}

#[guard(ApiKeyGuard::new(api_keys()))]
#[get("/api/data")]
async fn api_data(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok())
}
```

### Combining Guards

```rust
use armature_framework::get;
use armature_proc_macro::guard;
use armature_core::{
    HttpRequest, HttpResponse, Error,
    guard::{AuthenticationGuard, RolesGuard}
};

#[guard(
    AuthenticationGuard,
    RolesGuard::new(vec!["admin".into(), "moderator".into()])
)]
#[get("/moderation")]
async fn moderation_panel(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok())
}
```

## Controller-Level Guards

Apply guards to all routes in a controller. On a controller **struct**, `#[guard(...)]`
takes guard *types* constructed with `Default`, so use a guard that implements it, such as
`armature_auth::AuthGuard` (which requires the `UserContext` that `JwtAuthMiddleware`
attaches):

```rust
use armature_framework::{controller, get, post};
use armature_proc_macro::guard;
use armature_auth::AuthGuard;
use armature_core::{HttpRequest, HttpResponse, Error};

#[guard(AuthGuard)]
#[controller("/api/users")]
struct UserController;

impl UserController {
    // All routes require authentication

    #[get("")]
    async fn list_users(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::ok())
    }

    #[get("/:id")]
    async fn get_user(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::ok())
    }

    #[post("")]
    async fn create_user(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::created())
    }
}
```

### Combining Controller and Route Guards

Route guards add to controller guards:

```rust
use armature_framework::{controller, get};
use armature_proc_macro::guard;
use armature_auth::AuthGuard;
use armature_core::{HttpRequest, HttpResponse, Error, guard::RolesGuard};

#[guard(AuthGuard)]  // All routes require a verified identity
#[controller("/api")]
struct ApiController;

impl ApiController {
    #[get("/profile")]
    async fn profile(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // Only needs AuthGuard
        Ok(HttpResponse::ok())
    }

    #[guard(RolesGuard::new(vec!["admin".into()]))]  // Additional guard
    #[get("/admin")]
    async fn admin_data(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // Needs AuthGuard + the verified `admin` role
        Ok(HttpResponse::ok())
    }
}
```

### Controller Middleware Runs Before Controller Guards

When a controller has both `#[middleware(...)]` and `#[guard(...)]`, the middleware runs
first, so an authentication middleware declared on the controller attaches the identity
before the guards check it. The middleware also sees (and can post-process) a guard's
`403`.

```rust
use armature_proc_macro::{guard, middleware};
use armature_auth::{AuthGuard, JwtAuthMiddleware};
use armature_jwt::{JwtConfig, JwtManager};

// Struct-level middleware arguments are expressions, evaluated once when the
// module registers the controller's routes.
fn jwt_auth() -> JwtAuthMiddleware {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let config = JwtConfig::new(secret).expect("JWT_SECRET must be >= 32 random bytes");
    JwtAuthMiddleware::new(JwtManager::new(config).expect("valid JWT config"))
}

#[controller("/admin")]
#[middleware(jwt_auth())] // 1. verifies the token, attaches UserContext
#[guard(AuthGuard)]       // 2. then the guard checks it
struct AdminController;
```

At the **handler** level there is no reordering: each attribute macro wraps the expansion
of the attributes below it, so the bottom-most attribute becomes the outermost runtime
layer. To have a handler's `#[middleware(...)]` run before its `#[guard(...)]`, list the
middleware **below** the guard:

```rust
#[get("/admin")]
#[guard(RolesGuard::new(vec!["admin".to_string()]))]
#[middleware(AttachRolesMiddleware)] // runs first
async fn admin_endpoint(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok())
}
```

Prefer registering authentication once, globally or on the controller, over per-handler
middleware.

## Built-in Guards

Armature provides several built-in guards:

| Guard | Purpose | Usable with `#[use_guard]` |
|-------|---------|---------|
| `AuthenticationGuard` | Require a verified identity | ❌ (no `Default`; use `#[guard(AuthenticationGuard)]`) |
| `RolesGuard` | Require verified roles | ❌ (needs roles) |
| `ApiKeyGuard` | Validate API keys (constant time) | ❌ (needs keys) |
| `CustomGuard<F>` | Custom predicate | ❌ (needs fn) |

### AuthenticationGuard

Admits only requests carrying a `VerifiedIdentity` (or `RequestRoles`) extension attached
by an authentication layer. The `Authorization` header itself is not consulted; without a
verified identity the request gets `401 Unauthorized`:

```rust
use armature_core::VerifiedIdentity;

#[guard(AuthenticationGuard)]
#[get("/me")]
async fn get_current_user(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Present after the guard passes (it was attached by JwtAuthMiddleware).
    let subject = req
        .extensions
        .get::<VerifiedIdentity>()
        .map(|id| id.subject.clone());

    Ok(HttpResponse::ok())
}
```

### RolesGuard

Requires a verified `RequestRoles` extension with at least one of the listed roles
(`JwtAuthMiddleware` builds it from the token's `roles` claim). An empty list means "any
authenticated caller", never "everyone":

```rust
#[guard(RolesGuard::new(vec!["admin".into()]))]
#[get("/admin/settings")]
async fn admin_settings(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok())
}
```

### ApiKeyGuard

Validates the `X-API-Key` header, comparing against every configured key in constant
time:

```rust
#[guard(ApiKeyGuard::new(api_keys()))] // keys loaded from configuration
#[get("/api/v1/data")]
async fn api_data(req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok())
}
```

## Custom Guards

Create custom guards by implementing the `Guard` trait:

```rust
use armature_core::{Error, RequestRoles, VerifiedIdentity, guard::{Guard, GuardContext}};
use async_trait::async_trait;

/// Rate limiting guard
pub struct RateLimitGuard {
    max_requests: u32,
}

impl RateLimitGuard {
    pub fn new(max_requests: u32) -> Self {
        Self { max_requests }
    }
}

#[async_trait]
impl Guard for RateLimitGuard {
    async fn can_activate(&self, context: &GuardContext) -> Result<bool, Error> {
        // Key on something the client cannot choose: the socket peer
        // (`client_address(0)`), or the verified identity. A header such as
        // `x-client-id` would let each request pick a fresh bucket.
        let client_id = context
            .request
            .client_address(0)
            .map(|ip| ip.to_string())
            .ok_or_else(|| Error::Forbidden("Cannot determine client".into()))?;

        // Check rate limit (implement your logic)
        let request_count = get_request_count(&client_id).await;

        if request_count >= self.max_requests {
            Err(Error::TooManyRequests(format!(
                "Rate limit exceeded: {} requests",
                self.max_requests
            )))
        } else {
            increment_request_count(&client_id).await;
            Ok(true)
        }
    }
}
```

### Guard with Request Data Access

Guards have full access to the request context:

```rust
pub struct OwnershipGuard;

impl Default for OwnershipGuard {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Guard for OwnershipGuard {
    async fn can_activate(&self, context: &GuardContext) -> Result<bool, Error> {
        // Access path parameters
        let resource_id = context.get_param("id")
            .ok_or_else(|| Error::BadRequest("Missing resource ID".into()))?;

        // The caller's identity comes from the verified extension, never
        // from a client-supplied header such as `x-user-id`.
        let user_id = &context
            .request
            .extensions
            .get::<VerifiedIdentity>()
            .ok_or_else(|| Error::Unauthorized("Not authenticated".into()))?
            .subject;

        // Check ownership
        let resource = fetch_resource(resource_id).await?;

        if resource.owner_id == *user_id {
            Ok(true)
        } else {
            Err(Error::Forbidden("You don't own this resource".into()))
        }
    }
}
```

### IP Allowlist Guard

```rust
use std::net::IpAddr;

pub struct IpAllowlistGuard {
    allowed_ips: Vec<IpAddr>,
    /// How many reverse proxies in front of the app append to
    /// `X-Forwarded-For`. `0` = use the socket peer and ignore the header.
    trusted_proxy_depth: usize,
}

impl IpAllowlistGuard {
    pub fn new(ips: Vec<IpAddr>, trusted_proxy_depth: usize) -> Self {
        Self { allowed_ips: ips, trusted_proxy_depth }
    }
}

#[async_trait]
impl Guard for IpAllowlistGuard {
    async fn can_activate(&self, context: &GuardContext) -> Result<bool, Error> {
        // Never read the leftmost X-Forwarded-For hop or X-Real-IP: the client
        // sets them. client_address walks the chain from the right.
        let client_ip = context
            .request
            .client_address(self.trusted_proxy_depth)
            .ok_or_else(|| Error::BadRequest("Cannot determine client IP".into()))?;

        if self.allowed_ips.contains(&client_ip) {
            Ok(true)
        } else {
            Err(Error::Forbidden("Client address not allowed".into()))
        }
    }
}
```

### Time-Based Guard

```rust
use chrono::{Local, Timelike};

pub struct BusinessHoursGuard;

impl Default for BusinessHoursGuard {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Guard for BusinessHoursGuard {
    async fn can_activate(&self, _context: &GuardContext) -> Result<bool, Error> {
        let now = Local::now();
        let hour = now.hour();

        // Allow access only during business hours (9 AM - 5 PM)
        if hour >= 9 && hour < 17 {
            Ok(true)
        } else {
            Err(Error::Forbidden(
                "This endpoint is only available during business hours".into()
            ))
        }
    }
}
```

## Best Practices

### 1. Keep Guards Focused

Each guard should check one thing:

```rust
// ✅ Good: Single responsibility
pub struct AuthenticationGuard;  // Only checks auth
pub struct AdminRoleGuard;       // Only checks admin role
pub struct ResourceOwnerGuard;   // Only checks ownership

// ❌ Bad: Multiple responsibilities
pub struct EverythingGuard;  // Checks auth + role + ownership + ...
```

### 2. Use Descriptive Errors

```rust
// ✅ Good: Helpful error messages
Err(Error::Forbidden("Admin role required to access this resource".into()))
Err(Error::Unauthorized("API key expired".into()))

// ❌ Bad: Generic errors
Err(Error::Forbidden("Access denied".into()))
```

### 3. Order Guards Logically

```rust
// ✅ Good: Check auth before permissions
#[guard(AuthenticationGuard, AdminGuard)]

// ❌ Bad: Checking permissions before auth
#[guard(AdminGuard, AuthenticationGuard)]
```

### 4. Implement Default for Simple Guards

```rust
// ✅ Good: Allows #[use_guard(MyGuard)]
impl Default for MyGuard {
    fn default() -> Self {
        Self
    }
}

// Then use as:
#[use_guard(MyGuard)]
```

### 5. Use Configuration for Complex Guards

```rust
// ✅ Good: Use #[guard(...)] for configured guards
#[guard(RolesGuard::new(vec!["admin".into()]))]

// Instead of:
// #[use_guard(RolesGuard)]  // Won't work without Default
```

## Common Pitfalls

### ❌ Guard Without Default

```rust
// RolesGuard and AuthenticationGuard don't implement Default
#[use_guard(RolesGuard)]  // Error!

// Use #[guard(...)] instead for configured guards
#[guard(RolesGuard::new(vec!["admin".into()]))]  // Works!
```

### ❌ Returning `Ok(false)` Without Error

```rust
// Bad: Returns generic 403
async fn can_activate(&self, _ctx: &GuardContext) -> Result<bool, Error> {
    Ok(false)  // User gets "Access denied by guard"
}

// Good: Return specific error
async fn can_activate(&self, _ctx: &GuardContext) -> Result<bool, Error> {
    Err(Error::Forbidden("Premium subscription required".into()))
}
```

### ❌ Heavy Operations in Guards

```rust
// Bad: Database query on every request
async fn can_activate(&self, ctx: &GuardContext) -> Result<bool, Error> {
    let user = db.fetch_user_with_all_relations().await?;  // Expensive!
    Ok(user.has_permission("admin"))
}

// Good: Use the verified JWT claims instead of the database
async fn can_activate(&self, ctx: &GuardContext) -> Result<bool, Error> {
    // RequestRoles was attached by JwtAuthMiddleware after verifying the token.
    // Never read roles from a client-supplied header like `x-user-role`.
    Ok(ctx
        .request
        .extensions
        .get::<RequestRoles>()
        .is_some_and(|roles| roles.contains("admin")))
}
```

### ❌ Guards Without an Authentication Layer

```rust
// Every request gets 401: nothing attached a verified identity.
let app = Application::create::<AppModule>().await;

// Register the authentication middleware so it runs before the guards.
let app = Application::create::<AppModule>()
    .await
    .use_middleware(JwtAuthMiddleware::new(jwt_manager));
```

## API Reference

### Decorators

| Decorator | Target | Description |
|-----------|--------|-------------|
| `#[use_guard(Type, ...)]` | Function | Apply guards by type (requires Default) |
| `#[guard(expr, ...)]` | Function | Apply guard instances with configuration |
| `#[guard(Type, ...)]` | Controller struct | Guard every route of the controller (types require Default); runs after controller `#[middleware]` |

### GuardContext Methods

| Method | Description |
|--------|-------------|
| `get_header(name)` | Get request header by name |
| `get_param(name)` | Get path parameter by name |
| `request` | Access full `HttpRequest` |

### Built-in Guard Types

| Type | Constructor | Description |
|------|-------------|-------------|
| `AuthenticationGuard` | (unit struct) | Verified identity required (`401` without) |
| `RolesGuard` | `new(roles: Vec<String>)` | Verified role required; empty = any authenticated caller |
| `ApiKeyGuard` | `new(keys: Vec<String>)` | Constant-time API key validation |
| `CustomGuard<F>` | `new(predicate)` | Custom predicate |

## Summary

**Key Points:**

1. **`#[use_guard(Type)]`** for guards implementing `Default`
2. **`#[guard(expr)]`** for guards with configuration (and unit structs like `AuthenticationGuard`)
3. Guards fail closed: register an authentication middleware (`JwtAuthMiddleware`) with
   `Application::use_middleware` so it runs before them
4. Guards return `Result<bool, Error>`:
   - `Ok(true)` → Allow
   - `Ok(false)` → Deny (generic 403)
   - `Err(e)` → Deny with specific error
5. All guards must pass (AND logic)
6. Guards run in order; first failure stops execution
7. Middleware runs before guards; use guards for authorization, middleware for processing

**Quick Reference:**

```rust
// Unit-struct guard instance
#[guard(AuthenticationGuard)]
#[get("/protected")]
async fn protected(req: HttpRequest) -> Result<HttpResponse, Error> { ... }

// Configured guard
#[guard(ApiKeyGuard::new(api_keys()))]
#[get("/api")]
async fn api(req: HttpRequest) -> Result<HttpResponse, Error> { ... }

// Multiple guards
#[guard(AuthenticationGuard, RolesGuard::new(vec!["admin".into()]))]
#[get("/admin")]
async fn admin(req: HttpRequest) -> Result<HttpResponse, Error> { ... }

// Controller-level guard (type with Default)
#[guard(AuthGuard)]
#[controller("/api")]
struct ApiController;
```


