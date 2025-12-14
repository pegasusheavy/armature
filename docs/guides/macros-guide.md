# Armature Macros Guide

Comprehensive guide to all macros available in the Armature framework.

## Table of Contents

- [Overview](#overview)
- [Macro Crates](#macro-crates)
- [Route Decorators](#route-decorators)
- [Response Macros](#response-macros)
- [Validation Macros](#validation-macros)
- [Parameter Extraction](#parameter-extraction)
- [Error Handling](#error-handling)
- [Model Macros](#model-macros)
- [Test Helpers](#test-helpers)
- [Best Practices](#best-practices)

## Overview

Armature provides three macro crates:

- **`armature-macro`** - Procedural attribute macros (decorators)
- **`armature-macros`** - Declarative macros for common patterns
- **`armature-macros-utils`** - Utility procedural macros

## Macro Crates

### armature-macro (Proc Macros)

Attribute macros for decorating routes, controllers, and modules.

```toml
[dependencies]
armature-macro = { path = "../armature-macro" }
```

### armature-macros (Declarative)

Pattern-based macros for quick responses, validation, and utilities.

```toml
[dependencies]
armature-macros = { path = "../armature-macros" }
```

### armature-macros-utils (Proc Macros)

Additional utility procedural macros for responses and models.

```toml
[dependencies]
armature-macros-utils = { path = "../armature-macros-utils" }
```

## Route Decorators

### HTTP Method Decorators

From `armature-macro`:

```rust
use armature_macro::{get, post, put, delete, patch, controller};

#[controller("/api/users")]
pub struct UserController;

impl UserController {
    #[get("/:id")]
    async fn get_user(req: HttpRequest) -> Result<HttpResponse, Error> {
        let id = path_param!(req, "id")?;
        ok_json!({ "id": id })
    }

    #[post("/")]
    async fn create_user(req: HttpRequest) -> Result<HttpResponse, Error> {
        created_json!({ "message": "User created" })
    }

    #[put("/:id")]
    async fn update_user(req: HttpRequest) -> Result<HttpResponse, Error> {
        ok_json!({ "message": "User updated" })
    }

    #[delete("/:id")]
    async fn delete_user(req: HttpRequest) -> Result<HttpResponse, Error> {
        HttpResponse::no_content()
    }
}
```

### Additional Route Decorators

```rust
// Timeout decorator
#[timeout(5)]  // 5 seconds
#[get("/slow")]
async fn slow_endpoint(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Handler
}

// Body size limit
#[body_limit("10mb")]
#[post("/upload")]
async fn upload(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Handler
}

// Cache decorator
#[cache(ttl = 300)]
#[get("/expensive")]
async fn expensive_operation(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Result is cached for 5 minutes
}
```

## Response Macros

### JSON Responses

From `armature-macros`:

```rust
use armature_macros::*;

// 200 OK JSON response
#[get("/users")]
async fn list_users(req: HttpRequest) -> Result<HttpResponse, Error> {
    let users = vec!["Alice", "Bob", "Charlie"];
    ok_json!({ "users": users })
}

// 201 Created JSON response
#[post("/users")]
async fn create_user(req: HttpRequest) -> Result<HttpResponse, Error> {
    let id = 123;
    created_json!({ "id": id, "message": "User created" })
}

// Custom status JSON response
#[get("/status")]
async fn status(req: HttpRequest) -> Result<HttpResponse, Error> {
    json_response!(202, { "status": "processing" })
}
```

### Error Responses

```rust
// 400 Bad Request
#[post("/users")]
async fn create_user(req: HttpRequest) -> Result<HttpResponse, Error> {
    if name.is_empty() {
        return bad_request!("Name is required");
    }
    ok_json!({ "id": 1 })
}

// 404 Not Found
#[get("/users/:id")]
async fn get_user(req: HttpRequest) -> Result<HttpResponse, Error> {
    let id: i64 = path_param!(req, "id")?;

    match find_user(id).await {
        Some(user) => ok_json!(user),
        None => not_found!("User {} not found", id),
    }
}

// 500 Internal Server Error
#[get("/data")]
async fn get_data(req: HttpRequest) -> Result<HttpResponse, Error> {
    match load_data().await {
        Ok(data) => ok_json!(data),
        Err(e) => internal_error!("Failed to load data: {}", e),
    }
}
```

### Other Response Types

```rust
use armature_macros_utils::{html, text, redirect};

// HTML response
#[get("/page")]
async fn page(req: HttpRequest) -> Result<HttpResponse, Error> {
    html!("<h1>Welcome</h1><p>Hello, world!</p>")
}

// Plain text response
#[get("/text")]
async fn plain_text(req: HttpRequest) -> Result<HttpResponse, Error> {
    text!("Hello, world!")
}

// Redirect response
#[get("/old-url")]
async fn redirect_old(req: HttpRequest) -> Result<HttpResponse, Error> {
    redirect!("/new-url")
}
```

## Validation Macros

### Field Validation

```rust
use armature_macros::*;

#[post("/users")]
async fn create_user(req: HttpRequest) -> Result<HttpResponse, Error> {
    let name: String = extract_field(&req, "name")?;
    let email: String = extract_field(&req, "email")?;
    let age: u32 = extract_field(&req, "age")?;

    // Validate required fields
    validate_required!(name);
    validate_required!(email);

    // Validate email format
    validate_email!(email);

    // Validate age
    validate!(age >= 18);

    created_json!({ "message": "User created" })
}
```

### Guard Macro

```rust
use armature_macros::guard;

#[get("/admin")]
async fn admin_panel(req: HttpRequest) -> Result<HttpResponse, Error> {
    let user = get_current_user(&req).await?;

    // Guard condition - returns 403 if false
    guard!(user.is_admin(), "Admin access required");

    ok_json!({ "message": "Welcome, admin!" })
}
```

### Validation Error Helper

```rust
#[post("/users")]
async fn create_user(req: HttpRequest) -> Result<HttpResponse, Error> {
    let age: u32 = extract_field(&req, "age")?;

    if age < 18 {
        return validation_error!("Age must be at least 18");
    }

    created_json!({ "message": "User created" })
}
```

## Parameter Extraction

### Single Path Parameter

```rust
use armature_macros::path_param;

#[get("/users/:id")]
async fn get_user(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Extract and parse in one line
    let id: i64 = path_param!(req, "id")?;

    ok_json!({ "id": id })
}
```

### Multiple Path Parameters

```rust
use armature_macros::path_params;

#[get("/users/:user_id/posts/:post_id")]
async fn get_post(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Extract multiple parameters at once
    let (user_id, post_id) = path_params!(
        req,
        "user_id": i64,
        "post_id": i64
    )?;

    ok_json!({ "user_id": user_id, "post_id": post_id })
}
```

### Query Parameters

```rust
use armature_macros::query_param;

#[get("/search")]
async fn search(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Extract with default value
    let page: u32 = query_param!(req, "page").unwrap_or(1);
    let limit: u32 = query_param!(req, "limit").unwrap_or(20);
    let query: String = query_param!(req, "q").unwrap_or_default();

    ok_json!({ "page": page, "limit": limit, "query": query })
}
```

### Headers

```rust
use armature_macros::header;

#[get("/protected")]
async fn protected(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Extract header (returns error if missing)
    let auth: &String = header!(req, "Authorization")?;

    // Or with default
    let content_type = header!(req, "Content-Type").unwrap_or(&"text/plain".to_string());

    ok_json!({ "message": "Authorized" })
}
```

## Error Handling

### Log and Return Error

```rust
use armature_macros::log_error;

#[get("/data")]
async fn get_data(req: HttpRequest) -> Result<HttpResponse, Error> {
    match database.query().await {
        Ok(data) => ok_json!(data),
        Err(e) => log_error!("Database query failed: {}", e),
    }
}
```

## Model Macros

### Model Derive

From `armature-macros-utils`:

```rust
use armature_macros_utils::{Model, ApiModel, Resource};
use serde::{Serialize, Deserialize};

// Basic model with common traits
#[derive(Model, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}

// API model with field control
#[derive(ApiModel, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub name: String,
    #[api(skip)]  // Exclude from API response
    pub password_hash: String,
}

// Resource model for database operations
#[derive(Resource, Serialize, Deserialize)]
#[resource(table = "users")]
pub struct UserEntity {
    #[resource(primary_key)]
    pub id: i64,
    pub name: String,
    pub created_at: String,
}
```

## Test Helpers

### Test Request Creation

From `armature-macros-utils`:

```rust
use armature_macros_utils::{test_request, assert_json, assert_status};

#[tokio::test]
async fn test_get_user() {
    let req = test_request!(GET "/users/1");
    let resp = handler(req).await.unwrap();

    assert_status!(resp, 200);
    assert_json!(resp, { "id": 1, "name": "Alice" });
}

#[tokio::test]
async fn test_create_user() {
    let req = test_request!(
        POST "/users",
        json!({ "name": "Bob", "email": "bob@example.com" })
    );

    let resp = handler(req).await.unwrap();
    assert_status!(resp, 201);
}
```

## Best Practices

### Combining Macros

```rust
use armature_macro::{get, post, timeout, body_limit};
use armature_macros::*;

#[controller("/api/users")]
pub struct UserController;

impl UserController {
    // Combine multiple decorators
    #[timeout(30)]
    #[body_limit("5mb")]
    #[post("/")]
    async fn create_user(req: HttpRequest) -> Result<HttpResponse, Error> {
        // Extract and validate
        let name: String = path_param!(req, "name")?;
        let email: String = path_param!(req, "email")?;

        validate_required!(name);
        validate_email!(email);

        // Create user
        let user = User { id: 1, name, email };

        // Return created response
        created_json!({ "user": user })
    }

    #[get("/:id")]
    async fn get_user(req: HttpRequest) -> Result<HttpResponse, Error> {
        let id: i64 = path_param!(req, "id")?;

        match find_user(id).await {
            Some(user) => ok_json!(user),
            None => not_found!("User {} not found", id),
        }
    }
}
```

### Clean Handler Code

**Before macros:**

```rust
async fn get_user(req: HttpRequest) -> Result<HttpResponse, Error> {
    let id = req.path_params.get("id")
        .ok_or_else(|| Error::BadRequest("Missing id parameter".to_string()))?
        .parse::<i64>()
        .map_err(|e| Error::BadRequest(format!("Invalid id: {}", e)))?;

    match db.find_user(id).await {
        Ok(Some(user)) => {
            let json = serde_json::to_string(&user)
                .map_err(|e| Error::Serialization(e.to_string()))?;
            let mut response = HttpResponse::ok();
            response.body = json.into_bytes();
            response.headers.insert("Content-Type".to_string(), "application/json".to_string());
            Ok(response)
        }
        Ok(None) => {
            let error = serde_json::json!({
                "error": format!("User {} not found", id),
                "status": 404
            });
            HttpResponse::not_found().with_json(&error)
        }
        Err(e) => {
            Err(Error::InternalServerError(e.to_string()))
        }
    }
}
```

**After macros:**

```rust
async fn get_user(req: HttpRequest) -> Result<HttpResponse, Error> {
    let id: i64 = path_param!(req, "id")?;

    match db.find_user(id).await {
        Ok(Some(user)) => ok_json!(user),
        Ok(None) => not_found!("User {} not found", id),
        Err(e) => log_error!("Database error: {}", e),
    }
}
```

**Result:** 15 lines → 9 lines (40% reduction) with better readability!

### Type-Safe Parameter Extraction

```rust
#[get("/posts")]
async fn list_posts(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Automatic parsing with defaults
    let page: u32 = query_param!(req, "page").unwrap_or(1);
    let limit: u32 = query_param!(req, "limit").unwrap_or(20).min(100);
    let sort: String = query_param!(req, "sort").unwrap_or_else(|| "created_at".to_string());

    let posts = db.list_posts(page, limit, &sort).await?;

    paginated_response!(posts, page, posts.len())
}
```

### Consistent Error Handling

```rust
#[post("/users")]
async fn create_user(req: HttpRequest) -> Result<HttpResponse, Error> {
    let name: String = path_param!(req, "name")?;
    let email: String = path_param!(req, "email")?;

    // Validation with clear error messages
    validate_required!(name);
    validate_email!(email);
    guard!(name.len() >= 3, "Name must be at least 3 characters");

    let user = db.create_user(name, email).await
        .map_err(|e| log_error!("Failed to create user: {}", e))?;

    created_json!({ "user": user })
}
```

## Summary

### Available Macros

| Category | Macro | Purpose |
|----------|-------|---------|
| **Routes** | `#[get]`, `#[post]`, `#[put]`, `#[delete]`, `#[patch]` | HTTP method routing |
| **Routes** | `#[timeout]`, `#[body_limit]`, `#[cache]` | Route configuration |
| **Responses** | `ok_json!()`, `created_json!()` | Quick JSON responses |
| **Responses** | `json_response!()`, `html!()`, `text!()` | Custom responses |
| **Errors** | `bad_request!()`, `not_found!()`, `internal_error!()` | Error responses |
| **Params** | `path_param!()`, `query_param!()`, `header!()` | Extract parameters |
| **Params** | `path_params!()` | Extract multiple params |
| **Validation** | `validate!()`, `validate_required!()` | Field validation |
| **Validation** | `validate_email!()`, `guard!()` | Specific validators |
| **Utilities** | `json_object!{}`, `paginated_response!()` | Helpers |
| **Errors** | `log_error!()`, `validation_error!()` | Error handling |
| **Models** | `#[derive(Model)]`, `#[derive(ApiModel)]` | Model generation |
| **Testing** | `test_request!()`, `assert_json!()` | Test helpers |

### Benefits

- ✅ **Reduced Boilerplate** - 30-50% less code
- ✅ **Type Safety** - Compile-time validation
- ✅ **Consistency** - Uniform error handling
- ✅ **Readability** - Clear, expressive code
- ✅ **Maintainability** - Easier to refactor
- ✅ **Performance** - Zero runtime overhead

### When to Use Macros

**Use macros for:**
- ✓ Repetitive patterns
- ✓ Type-safe abstractions
- ✓ Quick prototyping
- ✓ Consistent error handling

**Don't use macros for:**
- ✗ Complex business logic
- ✗ One-off operations
- ✗ When clarity suffers

### Macro Composition

You can combine multiple macros for powerful, concise code:

```rust
#[timeout(30)]
#[body_limit("10mb")]
#[cache(ttl = 300)]
#[post("/process")]
async fn process_data(req: HttpRequest) -> Result<HttpResponse, Error> {
    let id: i64 = path_param!(req, "id")?;
    let data: String = path_param!(req, "data")?;

    guard!(!data.is_empty(), "Data cannot be empty");

    match process(id, data).await {
        Ok(result) => ok_json!(result),
        Err(e) => log_error!("Processing failed: {}", e),
    }
}
```

This combines:
- Timeout protection
- Body size limiting
- Result caching
- Parameter extraction
- Validation
- Logging
- Error handling

All in a clean, readable format! 🎉

