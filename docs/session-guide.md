# Session Storage Guide

Server-side session storage for Armature framework with Redis, Memcached, and CouchDB backends.

## Table of Contents

- [Important: Stateless Architecture Preferred](#important-stateless-architecture-preferred)
- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [When to Use Sessions](#when-to-use-sessions)
- [Quick Start](#quick-start)
- [Session Backends](#session-backends)
- [Session API](#session-api)
- [Integration with Handlers](#integration-with-handlers)
- [Best Practices](#best-practices)
- [Migration from Sessions to JWT](#migration-from-sessions-to-jwt)
- [API Reference](#api-reference)
- [Summary](#summary)

## Important: Stateless Architecture Preferred

> ⚠️ **Armature strongly recommends stateless architecture using JWT tokens instead of server-side sessions.**

### Why Stateless?

| Aspect | Stateless (JWT) | Sessions |
|--------|-----------------|----------|
| **Scalability** | ✅ Any server handles any request | ❌ Requires shared session store |
| **Performance** | ✅ No session lookups | ❌ External store on every request |
| **Reliability** | ✅ No session store to fail | ❌ Session store is SPOF |
| **Cloud-Native** | ✅ Perfect for K8s/serverless | ❌ Requires stateful infrastructure |
| **Security** | ✅ No session hijacking | ❌ Session vulnerabilities possible |

### Preferred Approach: JWT Authentication

```rust
use armature_jwt::JwtManager;

// Create token at login - contains all user info
let token = jwt_manager.sign(&UserClaims {
    user_id: user.id,
    email: user.email,
    roles: user.roles,
})?;

// Client stores token (localStorage, cookie, etc.)
// Server validates on each request - no session lookup needed.
// `verify_access` rejects refresh tokens presented as access tokens.
let claims: UserClaims = jwt_manager.verify_access(&token)?;
```

See the [Stateless Architecture Guide](./stateless-architecture.md) for comprehensive details.

## Overview

The `armature-session` module provides server-side session storage for cases where sessions are absolutely necessary. It supports multiple backends with a unified API.

**Use this module only when:**
- Integrating with legacy systems requiring sessions
- Compliance mandates server-side session tracking
- You need immediate session invalidation
- Storing large amounts of temporary user data

## Features

- ✅ **Redis backend** - High-performance, default option
- ✅ **Memcached backend** - Optional, feature-gated
- ✅ **CouchDB backend** - Document-based sessions
- ✅ Unified `SessionStore` trait across all backends
- ✅ 256-bit session IDs from the OS CSPRNG (64 lowercase hex characters), validated before every backend command
- ✅ `regenerate` to rotate the ID after login (session fixation defence)
- ✅ Sliding expiry, a fixed absolute lifetime and an idle timeout, all enforced on load
- ✅ `SessionCookie` helper with secure defaults and an optional `__Host-` prefix
- ✅ Session metadata (IP, user agent)
- ✅ Typed session data storage
- ✅ Credentials redacted from `SessionConfig`'s `Debug` output

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
# Redis only (default)
armature-session = "0.3"

# With Memcached support
armature-session = { version = "0.3", features = ["memcached"] }

# With CouchDB support
armature-session = { version = "0.3", features = ["couchdb"] }

# All backends
armature-session = { version = "0.3", features = ["full"] }
```

`armature-session` is a standalone crate; `armature-framework` does not
re-export it behind a feature flag.

> **`memcached-tls` is an opt-in OpenSSL exception.** `memcache` 0.21 only
> implements TLS through OpenSSL, so this feature links OpenSSL, contrary to
> Armature's rustls-only TLS policy. It is not part of `full`. Prefer a
> TLS-terminating sidecar (e.g. stunnel) or a private network and leave the
> feature off.

## When to Use Sessions

### ✅ Appropriate Use Cases

1. **Legacy System Integration**
   ```rust
   // Old system expects session IDs
   let session = store.create(None).await?;
   legacy_api.set_session_id(&session.id)?;
   ```

2. **Compliance Requirements**
   ```rust
   // Audit trail requires server-side session tracking
   session.set("audit_trail", audit_entry)?;
   store.save(&session).await?;
   ```

3. **Immediate Logout (All Devices)**
   ```rust
   // Delete all sessions for a user
   for session_id in user_sessions {
       store.delete(&session_id).await?;
   }
   ```

4. **Large Temporary Data**
   ```rust
   // Shopping cart, multi-step wizard data
   session.set("cart", large_cart_data)?;
   session.set("wizard_step", 3)?;
   ```

### ❌ Inappropriate Use Cases

- **Authentication only** → Use JWT
- **Simple user identification** → Use JWT
- **API authentication** → Use JWT/API keys
- **Microservices** → Use JWT for service-to-service

## Quick Start

### Redis Session Store (Default)

```rust
use armature_session::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), SessionError> {
    // Configure Redis session store
    let config = SessionConfig::redis("redis://localhost:6379")?
        .with_namespace("myapp:session")
        .with_default_ttl(Duration::from_secs(3600))   // 1 hour sliding expiry
        .with_max_ttl(Duration::from_secs(86400))      // 24 hour absolute lifetime
        .with_idle_timeout(Duration::from_secs(1800)); // 30 min of inactivity

    // Every store constructor runs `config.validate()`
    let store = RedisSessionStore::new(config.clone()).await?;

    // Create a new session (ID: 64 lowercase hex characters)
    let mut session = store.create(None).await?;

    // Store typed data
    session.set("user_id", 123)?;
    session.set("username", "alice")?;
    session.set("roles", vec!["admin", "user"])?;
    store.save(&session).await?;

    // Retrieve session later
    if let Some(session) = store.get(&session.id).await? {
        let user_id: Option<i32> = session.get("user_id");
        let roles: Option<Vec<String>> = session.get("roles");
        println!("User: {:?}, Roles: {:?}", user_id, roles);
    }

    // After login (or any privilege change), rotate the ID
    let session = store.regenerate(&session.id).await?;

    // Hand the new ID to the browser in a hardened cookie
    let cookie = SessionCookie::new("sid").host_prefix(true).max_age_from(&config);
    let set_cookie = cookie.to_header_value(&session.id)?;
    println!("Set-Cookie: {}", set_cookie);

    // Delete session (logout)
    store.delete(&session.id).await?;

    Ok(())
}
```

## Session Backends

### Redis (Default)

High-performance, recommended for most use cases.

```rust
use armature_session::*;

let config = SessionConfig::redis("redis://localhost:6379")?
    .with_namespace("myapp:session")
    .with_default_ttl(Duration::from_secs(3600));

let store = RedisSessionStore::new(config).await?;
```

**Pros:**
- Very fast (in-memory)
- Automatic expiration (TTL)
- Clustering support
- Persistence options

**Cons:**
- Additional infrastructure
- Memory-based (limited by RAM)

### Memcached (Feature-gated)

Requires `memcached` feature flag.

```toml
[dependencies]
armature-session = { version = "0.1", features = ["memcached"] }
```

```rust
use armature_session::*;

let config = SessionConfig::memcached("memcache://localhost:11211")?
    .with_namespace("myapp:session")
    .with_default_ttl(Duration::from_secs(3600));

let store = MemcachedSessionStore::new(config).await?;
```

**Pros:**
- Simple protocol
- Very fast
- Distributed by design

**Cons:**
- No persistence (volatile)
- No prefix scanning (count/clear_all limited)
- Memory-only

### CouchDB

Document-based sessions for environments already using CouchDB.

```toml
[dependencies]
armature-session = { version = "0.1", features = ["couchdb"] }
```

```rust
use armature_session::*;

let config = SessionConfig::couchdb("http://localhost:5984", "sessions")?
    .with_namespace("myapp")
    .with_default_ttl(Duration::from_secs(3600))
    .with_auth("admin", "password");

let store = CouchDbSessionStore::new(config).await?;
```

**Setup CouchDB database:**

```bash
# Create database
curl -X PUT http://admin:password@localhost:5984/sessions

# Create view for expiration cleanup
curl -X PUT http://admin:password@localhost:5984/sessions/_design/sessions \
  -H "Content-Type: application/json" \
  -d '{
    "views": {
      "by_expiration": {
        "map": "function(doc) { if(doc.expires_at) emit(doc.expires_at, null); }"
      }
    }
  }'
```

**Pros:**
- Persistent storage
- Query capabilities
- Good for document-oriented data

**Cons:**
- Slower than Redis/Memcached
- More complex setup
- Higher resource usage

## Session API

### Session Data Structure

```rust
pub struct Session {
    pub id: String,                              // 64 lowercase hex chars
    pub data: HashMap<String, serde_json::Value>, // Session data
    pub created_at: DateTime<Utc>,               // Creation time
    pub last_accessed_at: DateTime<Utc>,         // Last access time
    pub expires_at: DateTime<Utc>,               // Sliding expiration
    pub absolute_expires_at: DateTime<Utc>,      // Hard cap, fixed at creation
    pub user_agent: Option<String>,              // Browser/client info
    pub ip_address: Option<String>,              // Client IP
}
```

### Lifetimes

A session has three limits, all checked whenever it is loaded:

| Limit | Set by | Behaviour |
|-------|--------|-----------|
| Sliding expiry (`expires_at`) | `default_ttl` / the `ttl` passed to `create` | Moved forward by `extend`, never past the absolute expiry |
| Absolute expiry (`absolute_expires_at`) | `max_ttl` (default 7 days) | Fixed at creation. `extend`, `touch` and `regenerate` never move it |
| Idle timeout | `idle_timeout` (default 1 hour) | A session not accessed for longer than this is dropped |

A session that is expired, idle, or stored under a different ID is deleted
and `get` returns `None`. `exists` applies the same rules.
`SessionConfig::validate()` runs in every store constructor and rejects an
empty namespace or one containing whitespace or control characters, a
namespace too long for memcached's 250-byte key limit, zero lifetimes, and
`default_ttl > max_ttl`.

Sessions written by earlier `armature-session` releases (UUID IDs, no
`absolute_expires_at`) cannot be read, so users log in again once after
upgrading.

### Session Methods

```rust
// Get typed value
let user_id: Option<i32> = session.get("user_id");

// Set typed value
session.set("user_id", 123)?;

// Remove value
session.remove("temp_data");

// Check if key exists
if session.contains("user_id") { ... }

// Get all keys
let keys = session.keys();

// Clear all data
session.clear();

// Update last accessed time
session.touch();

// Extend expiration (clamped to absolute_expires_at)
session.extend(Duration::from_secs(3600));

// Check if expired (sliding or absolute)
if session.is_expired() { ... }

// Check if idle longer than a timeout
if session.is_idle(Duration::from_secs(1800)) { ... }

// Build a session by hand: the absolute lifetime defaults to `ttl`;
// allow sliding up to a longer hard cap with `with_absolute_lifetime`
let session = Session::new(generate_session_id(), ttl)
    .with_absolute_lifetime(Duration::from_secs(86400))
    .with_user_agent("Mozilla/5.0...")
    .with_ip_address("192.168.1.1");
```

### SessionStore Trait

```rust
#[async_trait]
pub trait SessionStore: Send + Sync {
    // Create new session
    async fn create(&self, ttl: Option<Duration>) -> SessionResult<Session>;

    // Get session by ID
    async fn get(&self, session_id: &str) -> SessionResult<Option<Session>>;

    // Save/update session
    async fn save(&self, session: &Session) -> SessionResult<()>;

    // Delete session
    async fn delete(&self, session_id: &str) -> SessionResult<()>;

    // Check if session exists
    async fn exists(&self, session_id: &str) -> SessionResult<bool>;

    // Extend session TTL
    async fn extend(&self, session_id: &str, ttl: Duration) -> SessionResult<()>;

    // Touch session (update last accessed)
    async fn touch(&self, session_id: &str) -> SessionResult<()>;

    // Clear all sessions (dangerous!)
    async fn clear_all(&self) -> SessionResult<()>;

    // Count active sessions
    async fn count(&self) -> SessionResult<usize>;

    // Cleanup expired sessions
    async fn cleanup_expired(&self) -> SessionResult<usize>;

    // Issue a fresh ID, carry the data over, delete the old ID
    // (provided method; returns SessionError::NotFound for an unknown ID)
    async fn regenerate(&self, old_id: &str) -> SessionResult<Session>;
}
```

Every method that takes a `session_id` validates it first with
`validate_session_id` (exactly 64 lowercase hex characters) and returns
`SessionError::InvalidSessionId` otherwise. A client-supplied value such as
`"a\r\nset x ..."` therefore never reaches a backend command. The rejected
value is not echoed in the error.

## Integration with Handlers

### Middleware Approach

```rust
use armature::prelude::*;
use armature_session::*;

pub struct SessionMiddleware {
    store: RedisSessionStore,
    cookie: SessionCookie,
}

#[async_trait]
impl Middleware for SessionMiddleware {
    async fn handle(&self, mut req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        // Extract the session ID from the `Cookie` header
        let session_id = req
            .headers
            .cookie()
            .and_then(|h| self.cookie.value_from_cookie_header(h))
            .map(str::to_owned);

        if let Some(id) = session_id {
            // A malformed ID is an error and an expired/idle session is
            // `None`; treat both as "no session".
            if let Ok(Some(session)) = self.store.get(&id).await {
                // Attach the loaded session as a request extension. Never
                // copy it into request headers: handlers cannot tell those
                // apart from headers the client sent.
                req.extensions.insert(session);
            }
        }

        next(req).await
    }
}
```

### Controller with Sessions

```rust
#[controller("/api")]
struct UserController {
    session_store: RedisSessionStore,
}

impl UserController {
    #[post("/login")]
    async fn login(
        &self,
        #[body] credentials: Body<LoginRequest>,
    ) -> Result<HttpResponse, Error> {
        // Validate credentials...
        let user = validate_user(&credentials)?;

        let internal = |e: SessionError| Error::Internal(e.to_string());

        // Create a fresh session at login. Never reuse an ID the client
        // brought with it; if it already had one, `regenerate` it instead.
        let mut session = self.session_store.create(None).await.map_err(internal)?;
        session.set("user_id", user.id).map_err(internal)?;
        session.set("roles", user.roles).map_err(internal)?;
        self.session_store.save(&session).await.map_err(internal)?;

        // Return the session ID in a hardened `__Host-` cookie
        // (Secure, HttpOnly, SameSite=Lax, Path=/, no Domain)
        let set_cookie = SessionCookie::new("sid")
            .host_prefix(true)
            .max_age(Duration::from_secs(3600))
            .to_header_value(&session.id)
            .map_err(internal)?;

        let mut response = HttpResponse::ok().with_json(&LoginResponse { success: true })?;
        response.cookies.push(set_cookie);
        Ok(response)
    }

    #[post("/logout")]
    async fn logout(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        let cookie = SessionCookie::new("sid").host_prefix(true);

        if let Some(session_id) = req
            .headers
            .cookie()
            .and_then(|h| cookie.value_from_cookie_header(h))
        {
            let _ = self.session_store.delete(session_id).await;
        }

        // Overwrite the cookie with an immediately expiring one
        let clear = cookie
            .clear_header_value()
            .map_err(|e| Error::Internal(e.to_string()))?;
        let mut response = HttpResponse::ok().with_json(&LogoutResponse { success: true })?;
        response.cookies.push(clear);
        Ok(response)
    }
}
```

### Session Cookies

`SessionCookie` builds the `Set-Cookie` value for a session ID. Its defaults
are `Secure`, `HttpOnly`, `SameSite=Lax` and `Path=/`, with `Max-Age` taken
from `max_age` or `max_age_from(&config)` (the config's `max_ttl`, the
absolute lifetime; the store enforces the shorter sliding and idle expiries).
`host_prefix(true)` names the cookie `__Host-<name>` and forces `Secure`,
`Path=/` and no `Domain`, so the browser pins it to the exact host that set
it and a sibling subdomain cannot overwrite (fixate) it. Building fails on
cookie- or header-injection characters, and on `SameSite=None` without
`Secure`.

```rust
use armature_session::{SessionConfig, SessionCookie, generate_session_id};

let config = SessionConfig::default();
let cookie = SessionCookie::new("sid").host_prefix(true).max_age_from(&config);

let value = cookie.to_header_value(&generate_session_id()).unwrap();
assert!(value.starts_with("__Host-sid="));
assert!(value.contains("; Secure"));
assert!(value.contains("; HttpOnly"));
assert!(value.contains("; SameSite=Lax"));

// On logout
assert!(cookie.clear_header_value().unwrap().contains("Max-Age=0"));
```

For other cookies, use `armature_core::Cookie` with `HttpResponse::set_cookie`.
It validates names, values, `Path` and `Domain` the same way.

## Best Practices

### 1. Always Set Reasonable TTLs

```rust
// ✅ Good - Short default, enforced maximum
let config = SessionConfig::redis("redis://localhost:6379")?
    .with_default_ttl(Duration::from_secs(3600))   // 1 hour
    .with_max_ttl(Duration::from_secs(86400));     // 24 hour max

// ❌ Bad - Very long sessions are security risks
let config = SessionConfig::redis("redis://localhost:6379")?
    .with_default_ttl(Duration::from_secs(86400 * 30)); // 30 days!
```

### 2. Store Minimal Data in Sessions

```rust
// ✅ Good - Only essential data
session.set("user_id", user.id)?;
session.set("roles", user.roles)?;

// ❌ Bad - Full user object with sensitive data
session.set("user", full_user_object)?; // Contains password hash!
```

### 3. Use Secure Session IDs

```rust
// Session IDs are 256 bits from the OS CSPRNG, encoded as
// 64 lowercase hex characters. Never allow user-chosen session IDs.
let session = store.create(None).await?; // ID auto-generated

// Any other shape is rejected before it reaches the backend
assert!(validate_session_id("not-a-session-id").is_err());
```

### 4. Regenerate Sessions on Login and Privilege Changes

`regenerate` issues a new ID, carries the data, `created_at` and absolute
expiry over, and deletes the old ID. An attacker who planted or observed the
old ID is left holding a dead one.

```rust
async fn elevate_privileges(
    store: &impl SessionStore,
    old_session_id: &str,
) -> SessionResult<Session> {
    // New ID; the old ID stops working
    let mut session = store.regenerate(old_session_id).await?;

    session.set("is_admin", true)?;
    store.save(&session).await?;

    Ok(session)
}
```

Send the new ID back to the client in a fresh `Set-Cookie`.

### 5. Clean Up Expired Sessions

```rust
// Run periodically (cron job)
async fn cleanup_sessions(store: &impl SessionStore) {
    match store.cleanup_expired().await {
        Ok(count) => println!("Cleaned up {} expired sessions", count),
        Err(e) => eprintln!("Session cleanup failed: {}", e),
    }
}
```

## Migration from Sessions to JWT

If you're currently using sessions and want to migrate to stateless JWT:

### Step 1: Add JWT Support

```rust
use armature_jwt::{JwtConfig, JwtManager};

// The secret must be at least 32 random bytes (e.g. `openssl rand -base64 48`)
let config = JwtConfig::new(std::env::var("JWT_SECRET")?)?
    .with_expiration(Duration::from_secs(3600));
let jwt_manager = JwtManager::new(config)?;
```

### Step 2: Dual Authentication Period

```rust
async fn authenticate(
    req: &HttpRequest,
    session_store: &impl SessionStore,
    jwt_manager: &JwtManager,
) -> Result<UserId, Error> {
    // Try JWT first (new method). `verify_access` rejects refresh tokens.
    if let Some(token) = extract_bearer_token(req) {
        if let Ok(claims) = jwt_manager.verify_access::<UserClaims>(token) {
            return Ok(claims.user_id);
        }
    }

    // Fall back to session (legacy)
    let cookie = SessionCookie::new("sid").host_prefix(true);
    if let Some(session_id) = req
        .headers
        .cookie()
        .and_then(|h| cookie.value_from_cookie_header(h))
    {
        if let Ok(Some(session)) = session_store.get(session_id).await {
            if let Some(user_id) = session.get("user_id") {
                return Ok(user_id);
            }
        }
    }

    Err(Error::Unauthorized("authentication required".into()))
}
```

### Step 3: Issue JWT on Login

```rust
async fn login(
    credentials: LoginRequest,
    jwt_manager: &JwtManager,
) -> Result<LoginResponse, Error> {
    let user = validate_user(&credentials)?;

    // Issue JWT instead of creating session
    let token = jwt_manager.sign(&UserClaims {
        user_id: user.id,
        roles: user.roles,
    })?;

    Ok(LoginResponse { token })
}
```

### Step 4: Remove Session Support

Once all clients are migrated, remove session dependencies.

## Common Pitfalls

- ❌ **Don't** use sessions for simple authentication - use JWT
- ❌ **Don't** store sensitive data (passwords, keys) in sessions
- ❌ **Don't** use very long session TTLs
- ❌ **Don't** trust session IDs from untrusted sources
- ❌ **Don't** copy session data into request headers
- ✅ **Do** call `store.regenerate()` after login and privilege changes
- ✅ **Do** use `SessionCookie` (ideally with `host_prefix(true)`) for the session cookie
- ✅ **Do** implement session cleanup
- ✅ **Do** consider JWT for new applications

## API Reference

### Types

| Type | Description |
|------|-------------|
| `Session` | Session data structure |
| `SessionConfig` | Configuration for session stores |
| `SessionBackend` | Enum of backend types |
| `SessionCookie` | Hardened `Set-Cookie` builder for session IDs |
| `SameSite` | `SameSite` attribute for `SessionCookie` |
| `SessionError` | Error type for session operations |
| `SessionResult<T>` | Result type alias |

### Traits

| Trait | Description |
|-------|-------------|
| `SessionStore` | Main trait for session storage backends |

### Structs

| Struct | Feature | Description |
|--------|---------|-------------|
| `RedisSessionStore` | `redis` (default) | Redis-backed sessions |
| `MemcachedSessionStore` | `memcached` | Memcached-backed sessions |
| `CouchDbSessionStore` | `couchdb` | CouchDB-backed sessions |

### Functions

| Function | Description |
|----------|-------------|
| `generate_session_id()` | Generate a new 256-bit session ID (64 lowercase hex chars) |
| `validate_session_id(id)` | Check that an ID has exactly that shape |

## Summary

**Key Takeaways:**

1. ⚠️ **Prefer stateless JWT authentication** - Sessions add complexity
2. Use sessions only when absolutely necessary (legacy, compliance, immediate logout)
3. Redis is the recommended backend for performance
4. Memcached requires `memcached` feature flag
5. Always set reasonable TTLs; `max_ttl` is a hard absolute lifetime and `idle_timeout` drops inactive sessions
6. Call `regenerate` after login and send the ID in a `SessionCookie`
7. Store minimal data in sessions
8. Consider migrating to JWT for new applications

**Quick Reference:**

```rust
use armature_session::prelude::*;

// Redis (default)
let store = RedisSessionStore::new(
    SessionConfig::redis("redis://localhost:6379")?
).await?;

// Create session
let mut session = store.create(None).await?;

// Store data
session.set("user_id", 123)?;
store.save(&session).await?;

// Retrieve
if let Some(session) = store.get(&session.id).await? {
    let user_id: Option<i32> = session.get("user_id");
}

// Delete (logout)
store.delete(&session.id).await?;
```

**Remember:** If you're starting a new project, use JWT. Sessions are for legacy compatibility and specific use cases only.

