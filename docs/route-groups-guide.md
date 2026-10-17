# Route Groups Guide

Comprehensive guide to organizing routes with Route Groups in Armature.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Basic Usage](#basic-usage)
- [Registering Routes](#registering-routes)
- [Shared Configuration](#shared-configuration)
- [Nested Groups](#nested-groups)
- [Best Practices](#best-practices)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Summary](#summary)

---

## Overview

Route Groups allow you to organize routes with shared configuration, making your routing code more maintainable and DRY (Don't Repeat Yourself).

Route groups provide:
- **Path prefixes** - Automatic prefix for all routes in the group
- **Shared middleware** - Apply middleware to all routes in the group
- **Shared guards** - Apply authorization to all routes in the group
- **Nested configuration** - Groups can inherit from parent groups

---

## Features

- ✅ Path prefix inheritance
- ✅ Shared middleware application
- ✅ Shared guard application
- ✅ Nested groups with configuration merging
- ✅ Fluent builder API
- ✅ Type-safe configuration

---

## Basic Usage

### Creating a Route Group

```rust
use armature_core::*;

// Create a basic API group with prefix
let api_group = RouteGroup::new()
    .prefix("/api/v1");

// All routes in this group will have /api/v1 prefix
let user_route = api_group.apply_prefix("/users");
// Result: "/api/v1/users"
```

### With Middleware

```rust
use armature_core::*;
use std::sync::Arc;

let api_group = RouteGroup::new()
    .prefix("/api/v1")
    .middleware(Arc::new(LoggerMiddleware::new()))
    .middleware(Arc::new(CorsMiddleware::default()));

// All routes in this group will have logging and CORS enabled
```

### With Guards

```rust
use armature_core::*;

let protected_group = RouteGroup::new()
    .prefix("/api/v1/admin")
    .guard(Box::new(AuthenticationGuard))
    .guard(Box::new(RolesGuard::new(vec!["admin".to_string()])));

// All routes require authentication AND admin role
```

The built-in guards fail closed: `AuthenticationGuard` and `RolesGuard` need the verified
identity an authentication middleware (such as `armature_auth::JwtAuthMiddleware`) attaches.
Register that middleware globally with `Application::use_middleware`, or on the group with
`.middleware(...)`; group middleware always runs before group guards.

---

## Registering Routes

A group's prefix, middleware and guards apply to the routes you register **through** it with
`RouteGroup::route` or `RouteGroup::add_route`. The middleware and guards are bound to each
route, so they run however the client spells the path (`/api/v1/admin/users`,
`//api/v1/admin/users`, `/api/v1/admin//users/`):

```rust
use armature_core::*;

async fn list_users(_req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok())
}

async fn delete_user(_req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::no_content())
}

let mut router = Router::new();

let admin = RouteGroup::new()
    .prefix("/api/v1/admin")
    .guard(Box::new(AuthenticationGuard))
    .guard(Box::new(RolesGuard::new(vec!["admin".to_string()])));

// Handler form: method, path (prefixed by the group) and handler.
admin
    .route(&mut router, HttpMethod::GET, "/users", list_users)
    .route(&mut router, HttpMethod::DELETE, "/users/:id", delete_user);

// Or hand over a prebuilt `Route` (its path is prefixed too).
admin.add_route(&mut router, Route::new(HttpMethod::GET, "/stats", list_users));
```

Registering a route directly on the router (`router.get(admin.apply_prefix("/users"), ...)`)
applies **only** the prefix: the group's guards and middleware are not attached.

Within a route, the order is: group middleware (in the order added) → group guards (in the
order added) → handler. A guard returning `Ok(false)` answers `403`; an `Err` is propagated
(for example `401` from `AuthenticationGuard` without a verified identity).

---

## Shared Configuration

### Path Prefixes

Route groups automatically prepend prefixes to all routes:

```rust
use armature_core::*;

let api = RouteGroup::new().prefix("/api/v1");

// Apply prefix to routes
assert_eq!(api.apply_prefix("/users"), "/api/v1/users");
assert_eq!(api.apply_prefix("/posts"), "/api/v1/posts");
assert_eq!(api.apply_prefix("/comments"), "/api/v1/comments");
```

### Multiple Middleware

Middleware are applied in the order they're added:

```rust
let group = RouteGroup::new()
    .middleware(Arc::new(LoggerMiddleware::new()))
    .middleware(Arc::new(CorsMiddleware::default()))
    .middleware(Arc::new(CompressionMiddleware::new()));

// Execution order: Logger → CORS → Compression → (group guards) → Handler
```

### Multiple Guards

All guards must pass for access to be granted (AND logic):

```rust
let group = RouteGroup::new()
    .guard(Box::new(AuthenticationGuard))
    .guard(Box::new(RolesGuard::new(vec!["admin".to_string()])))
    .guard(Box::new(ApiKeyGuard::new(api_keys_from_config())));

// Request must pass ALL guards
```

---

## Nested Groups

Groups can inherit configuration from parent groups:

### Basic Nesting

```rust
use armature_core::*;

let api = RouteGroup::new()
    .prefix("/api")
    .middleware(Arc::new(LoggerMiddleware::new()));

let v1 = RouteGroup::new()
    .prefix("/v1")
    .with_parent(&api);

// v1 inherits:
// - Prefix: "/api/v1"
// - Middleware: LoggerMiddleware

let admin = RouteGroup::new()
    .prefix("/admin")
    .guard(Box::new(AdminGuard))
    .with_parent(&v1);

// admin has:
// - Prefix: "/api/v1/admin"
// - Middleware: LoggerMiddleware (from api)
// - Guard: AdminGuard
```

### Configuration Merging Rules

When using `with_parent()`:

1. **Prefixes are concatenated** - parent prefix + child prefix
2. **Middleware are combined** - parent middleware execute first
3. **Guards are combined** - parent guards are checked first, then the child's

---

## Best Practices

### 1. Organize by API Version

```rust
let v1 = RouteGroup::new()
    .prefix("/api/v1")
    .middleware(Arc::new(LoggerMiddleware::new()));

let v2 = RouteGroup::new()
    .prefix("/api/v2")
    .middleware(Arc::new(LoggerMiddleware::new()))
    .middleware(Arc::new(TimeoutMiddleware::new(30)));
```

### 2. Group by Authentication Level

```rust
let public = RouteGroup::new()
    .prefix("/api/public");

let authenticated = RouteGroup::new()
    .prefix("/api/auth")
    .guard(Box::new(AuthenticationGuard));

let admin = RouteGroup::new()
    .prefix("/api/admin")
    .guard(Box::new(AuthenticationGuard))
    .guard(Box::new(AdminGuard));
```

### 3. Combine Strategies

```rust
// Base API group
let api = RouteGroup::new()
    .prefix("/api")
    .middleware(Arc::new(LoggerMiddleware::new()));

// Version groups
let v1 = RouteGroup::new()
    .prefix("/v1")
    .with_parent(&api);

// Resource groups within v1
let v1_users = RouteGroup::new()
    .prefix("/users")
    .guard(Box::new(AuthenticationGuard))
    .with_parent(&v1);

let v1_admin = RouteGroup::new()
    .prefix("/admin")
    .guard(Box::new(AuthenticationGuard))
    .guard(Box::new(AdminGuard))
    .with_parent(&v1);
```

---

## API Reference

### RouteGroup Methods

| Method | Description |
|--------|-------------|
| `prefix(path)` | Set the path prefix for this group |
| `middleware(mw)` | Add a single middleware |
| `with_middleware(mws)` | Add multiple middleware |
| `guard(guard)` | Add a single guard |
| `with_guards(guards)` | Add multiple guards |
| `route(router, method, path, handler)` | Register a handler through the group (prefix, middleware and guards applied) |
| `add_route(router, route)` | Register a prebuilt `Route` through the group |
| `get_prefix()` | Get the current prefix |
| `apply_prefix(path)` | Apply prefix to a path |
| `get_middleware()` | Get all middleware |
| `get_guards()` | Get all guards |
| `with_parent(parent)` | Inherit from parent group |

---

## Summary

**Key Points:**

1. **RouteGroup organizes routes** with shared configuration
2. **Prefixes are automatically applied** and normalized
3. **Register routes with `route()` / `add_route()`** so the group's middleware and guards are enforced
4. **Middleware stack in order** they're added and run before the guards
5. **All guards must pass** (AND logic)
6. **Nest groups with `with_parent()`** for inheritance
7. **Use for API versions, auth levels, resources**

**Quick Reference:**

```rust
// Basic group
let group = RouteGroup::new()
    .prefix("/api/v1")
    .middleware(Arc::new(LoggerMiddleware::new()))
    .guard(Box::new(AuthenticationGuard));

// Nested group
let child = RouteGroup::new()
    .prefix("/admin")
    .guard(Box::new(AdminGuard))
    .with_parent(&group);

// Register a route through the group (prefix + middleware + guards)
child.route(&mut router, HttpMethod::GET, "/users", list_users);
// Serves GET /api/v1/admin/users
```

**Benefits:**

- 📦 **DRY** - Don't repeat middleware/guards
- 🎯 **Organized** - Clear route structure
- 🔒 **Secure** - Consistent auth application
- 📈 **Scalable** - Easy to add new groups
- 🔧 **Maintainable** - Change once, apply everywhere

