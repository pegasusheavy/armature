# armature-macro

Procedural macros for the Armature framework.

## Features

- **Route Macros** - `#[get]`, `#[post]`, etc.
- **Controller Macros** - `#[controller]`
- **Validation** - Compile-time route validation
- **DI Integration** - Dependency injection macros

## Installation

```toml
[dependencies]
armature-macro = "0.1"
```

## Quick Start

```rust
use armature_macro::{get, post, controller};

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

## License

MIT OR Apache-2.0

