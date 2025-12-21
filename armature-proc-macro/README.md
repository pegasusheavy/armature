# armature-proc-macro

Procedural macros for the Armature framework.

This crate provides compile-time attribute macros for defining controllers, routes, and dependency injection.

## Features

- **Route Macros** - `#[get]`, `#[post]`, `#[put]`, `#[delete]`, `#[patch]`
- **Controller Macros** - `#[controller("/path")]`
- **Module Macros** - `#[module(providers: [...], controllers: [...])]`
- **DI Macros** - `#[injectable]`
- **Validation** - Compile-time route validation
- **Middleware** - `#[timeout]`, `#[cache]`, `#[body_limit]`

## Installation

```toml
[dependencies]
armature-proc-macro = "0.1"
```

## Quick Start

```rust
use armature_proc_macro::{get, post, controller};

#[controller("/users")]
impl UserController {
    #[get("/")]
    async fn list(&self) -> Json<Vec<User>> {
        // List users
    }

    #[get("/:id")]
    async fn get(&self, id: Path<i32>) -> Json<User> {
        // Get user by ID
    }

    #[post("/")]
    async fn create(&self, body: Json<CreateUser>) -> Json<User> {
        // Create user
    }
}
```

## Available Macros

| Macro | Purpose |
|-------|---------|
| `#[controller("/path")]` | Define a controller with base path |
| `#[module(...)]` | Define a module with providers/controllers |
| `#[injectable]` | Mark a struct for dependency injection |
| `#[get("/path")]` | HTTP GET route |
| `#[post("/path")]` | HTTP POST route |
| `#[put("/path")]` | HTTP PUT route |
| `#[delete("/path")]` | HTTP DELETE route |
| `#[patch("/path")]` | HTTP PATCH route |
| `#[routes]` | Generate route handlers for a controller |
| `#[timeout(ms)]` | Set request timeout |
| `#[cache(ttl)]` | Enable response caching |
| `#[body_limit(bytes)]` | Set max request body size |

## License

Apache-2.0
