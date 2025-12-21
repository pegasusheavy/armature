# armature-openapi

OpenAPI/Swagger documentation for the Armature framework.

## Features

- **Auto Generation** - Generate OpenAPI spec from routes
- **Swagger UI** - Built-in documentation UI
- **Type Inference** - Infer types from handlers
- **Validation** - Validate requests against schema
- **Export** - JSON/YAML spec export

## Installation

```toml
[dependencies]
armature-openapi = "0.1"
```

## Quick Start

```rust
use armature_openapi::{OpenApi, swagger_ui};
use armature_core::Application;

#[derive(OpenApi)]
struct ApiDoc;

let app = Application::new()
    .get("/users", list_users)
    .post("/users", create_user)
    .get("/swagger-ui/*", swagger_ui(ApiDoc::openapi()))
    .get("/openapi.json", openapi_spec(ApiDoc::openapi()));
```

## Annotations

```rust
/// List all users
#[openapi(
    summary = "List users",
    responses(
        (status = 200, description = "List of users", body = Vec<User>)
    )
)]
async fn list_users() -> Json<Vec<User>> {
    // ...
}
```

## Request Body

```rust
#[derive(ToSchema)]
struct CreateUser {
    /// User's name
    name: String,
    /// User's email
    email: String,
}
```

## License

MIT OR Apache-2.0

