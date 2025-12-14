# Route Groups Guide

Comprehensive guide to organizing routes with Route Groups in Armature.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Basic Usage](#basic-usage)
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
    .middleware(Arc::new(LoggerMiddleware))
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

**Prefix Normalization:**
- Leading `/` is added if missing
- Trailing `/` is removed
- Empty paths become just the prefix

```rust
let group = RouteGroup::new().prefix("api/v1/");
assert_eq!(group.get_prefix(), "/api/v1"); // Normalized

assert_eq!(group.apply_prefix("/users"), "/api/v1/users");
assert_eq!(group.apply_prefix(""), "/api/v1"); // Empty path
```

### Multiple Middleware

Middleware are applied in the order they're added:

```rust
use armature_core::*;
use std::sync::Arc;

let group = RouteGroup::new()
    .middleware(Arc::new(LoggerMiddleware))
    .middleware(Arc::new(CorsMiddleware::default()))
    .middleware(Arc::new(CompressionMiddleware::new()));

// Execution order: Logger → CORS → Compression → Handler
```

**Using `with_middleware` for bulk addition:**

```rust
use armature_core::*;
use std::sync::Arc;

let middleware_stack = vec![
    Arc::new(LoggerMiddleware) as Arc<dyn Middleware>,
    Arc::new(CorsMiddleware::default()) as Arc<dyn Middleware>,
];

let group = RouteGroup::new()
    .with_middleware(middleware_stack);
```

### Multiple Guards

All guards must pass for access to be granted (AND logic):

```rust
use armature_core::*;

let group = RouteGroup::new()
    .guard(Box::new(AuthenticationGuard))
    .guard(Box::new(RolesGuard::new(vec!["admin".to_string()])))
    .guard(Box::new(ApiKeyGuard::new(vec!["key123".to_string()])));

// Request must pass ALL guards
```

**Using `with_guards` for bulk addition:**

```rust
use armature_core::*;

let guards = vec![
    Box::new(AuthenticationGuard) as Box<dyn Guard>,
    Box::new(RolesGuard::new(vec!["admin".to_string()])) as Box<dyn Guard>,
];

let group = RouteGroup::new()
    .with_guards(guards);
```

---

## Nested Groups

Groups can inherit configuration from parent groups:

### Basic Nesting

```rust
use armature_core::*;

let api = RouteGroup::new()
    .prefix("/api")
    .middleware(Arc::new(LoggerMiddleware));

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

// admin inherits:
// - Prefix: "/api/v1/admin"
// - Middleware: LoggerMiddleware
// - Guard: AdminGuard
```

### Configuration Merging Rules

When using `with_parent()`:

1. **Prefixes are concatenated** - parent prefix + child prefix
2. **Middleware are combined** - parent middleware execute first
3. **Guards are from child only** - cannot clone Box<dyn Guard>

```rust
let parent = RouteGroup::new()
    .prefix("/api")
    .middleware(Arc::new(LoggerMiddleware));

let child = RouteGroup::new()
    .prefix("/v1")
    .middleware(Arc::new(CorsMiddleware::default()))
    .with_parent(&parent);

// Result:
// - Prefix: "/api/v1"
// - Middleware: [LoggerMiddleware, CorsMiddleware]
```

---

## Best Practices

### 1. Organize by API Version

```rust
use armature_core::*;
use std::sync::Arc;

let v1 = RouteGroup::new()
    .prefix("/api/v1")
    .middleware(Arc::new(LoggerMiddleware));

let v2 = RouteGroup::new()
    .prefix("/api/v2")
    .middleware(Arc::new(LoggerMiddleware))
    .middleware(Arc::new(RateLimitMiddleware::new()));
```

### 2. Group by Authentication Level

```rust
use armature_core::*;

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

### 3. Group by Resource

```rust
use armature_core::*;

let users = RouteGroup::new()
    .prefix("/api/users")
    .middleware(Arc::new(UserMiddleware));

let posts = RouteGroup::new()
    .prefix("/api/posts")
    .middleware(Arc::new(PostMiddleware));
```

### 4. Combine Strategies

```rust
use armature_core::*;
use std::sync::Arc;

// Base API group
let api = RouteGroup::new()
    .prefix("/api")
    .middleware(Arc::new(LoggerMiddleware));

// Version groups
let v1 = RouteGroup::new()
    .prefix("/v1")
    .with_parent(&api);

let v2 = RouteGroup::new()
    .prefix("/v2")
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

### RouteGroup

#### Constructor

```rust
pub fn new() -> Self
```

Create a new empty route group.

#### Methods

| Method | Description |
|--------|-------------|
| `prefix(path)` | Set the path prefix for this group |
| `middleware(mw)` | Add a single middleware |
| `with_middleware(mws)` | Add multiple middleware |
| `guard(guard)` | Add a single guard |
| `with_guards(guards)` | Add multiple guards |
| `get_prefix()` | Get the current prefix |
| `apply_prefix(path)` | Apply prefix to a path |
| `get_middleware()` | Get all middleware |
| `get_guards()` | Get all guards |
| `with_parent(parent)` | Inherit from parent group |

---

## Examples

### Example 1: Basic REST API

```rust
use armature_core::*;
use std::sync::Arc;

// Create API group with logging
let api = RouteGroup::new()
    .prefix("/api/v1")
    .middleware(Arc::new(LoggerMiddleware))
    .middleware(Arc::new(CorsMiddleware::default()));

// Apply to routes
let users_path = api.apply_prefix("/users");
let posts_path = api.apply_prefix("/posts");

// Results:
// users_path = "/api/v1/users"
// posts_path = "/api/v1/posts"
```

### Example 2: Protected Admin Section

```rust
use armature_core::*;
use std::sync::Arc;

// Public API
let public_api = RouteGroup::new()
    .prefix("/api/public")
    .middleware(Arc::new(LoggerMiddleware));

// Admin API (protected)
let admin_api = RouteGroup::new()
    .prefix("/api/admin")
    .middleware(Arc::new(LoggerMiddleware))
    .guard(Box::new(AuthenticationGuard))
    .guard(Box::new(RolesGuard::new(vec!["admin".to_string()])));

let users_path = admin_api.apply_prefix("/users");
// Result: "/api/admin/users" (requires auth + admin role)
```

### Example 3: Multi-Version API

```rust
use armature_core::*;
use std::sync::Arc;

// Base API configuration
let api = RouteGroup::new()
    .prefix("/api")
    .middleware(Arc::new(LoggerMiddleware));

// Version 1
let v1 = RouteGroup::new()
    .prefix("/v1")
    .with_parent(&api);

// Version 2 with additional middleware
let v2 = RouteGroup::new()
    .prefix("/v2")
    .middleware(Arc::new(RateLimitMiddleware::new()))
    .with_parent(&api);

// V1 routes
let v1_users = v1.apply_prefix("/users");
// Result: "/api/v1/users"

// V2 routes (with rate limiting)
let v2_users = v2.apply_prefix("/users");
// Result: "/api/v2/users"
```

### Example 4: Nested Resource Groups

```rust
use armature_core::*;
use std::sync::Arc;

// API base
let api = RouteGroup::new()
    .prefix("/api/v1")
    .middleware(Arc::new(LoggerMiddleware));

// Users resource
let users = RouteGroup::new()
    .prefix("/users")
    .guard(Box::new(AuthenticationGuard))
    .with_parent(&api);

// User posts (nested under users)
let user_posts = RouteGroup::new()
    .prefix("/:user_id/posts")
    .with_parent(&users);

let path = user_posts.apply_prefix("");
// Result: "/api/v1/users/:user_id/posts"
```

---

## Common Pitfalls

### ❌ Don't: Manually Concatenate Prefixes

```rust
// BAD
let path = format!("{}{}", group1_prefix, group2_prefix);
```

### ✅ Do: Use `with_parent()`

```rust
// GOOD
let child = RouteGroup::new()
    .prefix("/v1")
    .with_parent(&parent);
```

### ❌ Don't: Duplicate Middleware

```rust
// BAD - middleware applied twice
let group1 = RouteGroup::new().middleware(Arc::new(LoggerMiddleware));
let group2 = RouteGroup::new().middleware(Arc::new(LoggerMiddleware));
```

### ✅ Do: Use Parent Groups

```rust
// GOOD - middleware applied once
let base = RouteGroup::new().middleware(Arc::new(LoggerMiddleware));
let group1 = RouteGroup::new().with_parent(&base);
let group2 = RouteGroup::new().with_parent(&base);
```

---

## Troubleshooting

### Issue: Prefix Not Applied

**Symptom:** Routes don't have the expected prefix

**Cause:** Forgot to call `apply_prefix()`

**Solution:**
```rust
let group = RouteGroup::new().prefix("/api");
let path = group.apply_prefix("/users"); // Don't forget this!
```

### Issue: Middleware Not Executing

**Symptom:** Group middleware doesn't run

**Cause:** Need to integrate with router to actually apply middleware

**Solution:** Use with controller or router integration (future feature)

### Issue: Guards Not Inherited

**Symptom:** Child group doesn't have parent guards

**Cause:** Guards cannot be cloned (Box<dyn Guard> limitation)

**Solution:** Re-add guards to child groups explicitly

---

## Summary

**Key Points:**

1. **RouteGroup organizes routes** with shared configuration
2. **Prefixes are automatically applied** and normalized
3. **Middleware stack in order** they're added
4. **All guards must pass** (AND logic)
5. **Nest groups with `with_parent()`** for inheritance
6. **Use for API versions, auth levels, resources**

**Quick Reference:**

```rust
// Basic group
let group = RouteGroup::new()
    .prefix("/api/v1")
    .middleware(Arc::new(LoggerMiddleware))
    .guard(Box::new(AuthenticationGuard));

// Nested group
let child = RouteGroup::new()
    .prefix("/admin")
    .guard(Box::new(AdminGuard))
    .with_parent(&group);

// Apply prefix
let path = child.apply_prefix("/users");
// Result: "/api/v1/admin/users"
```

**Benefits:**

- 📦 **DRY** - Don't repeat middleware/guards
- 🎯 **Organized** - Clear route structure
- 🔒 **Secure** - Consistent auth application
- 📈 **Scalable** - Easy to add new groups
- 🔧 **Maintainable** - Change once, apply everywhere

