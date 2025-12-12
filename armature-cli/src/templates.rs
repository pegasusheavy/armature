//! Code generation templates for Armature CLI.

use handlebars::Handlebars;
use serde::Serialize;

/// Template registry for code generation.
pub struct TemplateRegistry {
    hbs: Handlebars<'static>,
}

impl TemplateRegistry {
    /// Create a new template registry with all templates registered.
    pub fn new() -> Self {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);

        // Register all templates
        hbs.register_template_string("controller", CONTROLLER_TEMPLATE)
            .expect("Failed to register controller template");
        hbs.register_template_string("controller_crud", CONTROLLER_CRUD_TEMPLATE)
            .expect("Failed to register controller CRUD template");
        hbs.register_template_string("controller_test", CONTROLLER_TEST_TEMPLATE)
            .expect("Failed to register controller test template");
        hbs.register_template_string("module", MODULE_TEMPLATE)
            .expect("Failed to register module template");
        hbs.register_template_string("middleware", MIDDLEWARE_TEMPLATE)
            .expect("Failed to register middleware template");
        hbs.register_template_string("middleware_test", MIDDLEWARE_TEST_TEMPLATE)
            .expect("Failed to register middleware test template");
        hbs.register_template_string("guard", GUARD_TEMPLATE)
            .expect("Failed to register guard template");
        hbs.register_template_string("guard_test", GUARD_TEST_TEMPLATE)
            .expect("Failed to register guard test template");
        hbs.register_template_string("service", SERVICE_TEMPLATE)
            .expect("Failed to register service template");
        hbs.register_template_string("service_test", SERVICE_TEST_TEMPLATE)
            .expect("Failed to register service test template");
        hbs.register_template_string("main_minimal", MAIN_MINIMAL_TEMPLATE)
            .expect("Failed to register main minimal template");
        hbs.register_template_string("cargo_toml", CARGO_TOML_TEMPLATE)
            .expect("Failed to register Cargo.toml template");
        hbs.register_template_string("env_example", ENV_EXAMPLE_TEMPLATE)
            .expect("Failed to register .env.example template");
        hbs.register_template_string("readme", README_TEMPLATE)
            .expect("Failed to register README template");

        Self { hbs }
    }

    /// Render a template with the given data.
    pub fn render<T: Serialize>(&self, template: &str, data: &T) -> Result<String, String> {
        self.hbs.render(template, data).map_err(|e| e.to_string())
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// CONTROLLER TEMPLATES
// =============================================================================

const CONTROLLER_TEMPLATE: &str = r#"//! {{name_pascal}} controller.

use armature::prelude::*;

/// {{name_pascal}} controller handles {{name_snake}} related endpoints.
#[controller("/{{base_path}}")]
#[derive(Default)]
pub struct {{name_pascal}}Controller;

impl {{name_pascal}}Controller {
    /// Get all {{name_snake}}s.
    #[get("/")]
    pub async fn index(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        HttpResponse::ok().with_json(&serde_json::json!({
            "message": "List all {{name_snake}}s"
        }))
    }

    /// Get a single {{name_snake}} by ID.
    #[get("/:id")]
    pub async fn show(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        let id = req.params.get("id").unwrap_or(&"0".to_string()).clone();
        HttpResponse::ok().with_json(&serde_json::json!({
            "message": format!("Get {{name_snake}} with id: {}", id)
        }))
    }
}
"#;

const CONTROLLER_CRUD_TEMPLATE: &str = r#"//! {{name_pascal}} controller with CRUD operations.

use armature::prelude::*;
use serde::{Deserialize, Serialize};

/// {{name_pascal}} data transfer object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {{name_pascal}}Dto {
    pub id: Option<u64>,
    pub name: String,
    // Add more fields as needed
}

/// Create {{name_pascal}} request.
#[derive(Debug, Deserialize)]
pub struct Create{{name_pascal}}Request {
    pub name: String,
}

/// Update {{name_pascal}} request.
#[derive(Debug, Deserialize)]
pub struct Update{{name_pascal}}Request {
    pub name: Option<String>,
}

/// {{name_pascal}} controller handles {{name_snake}} CRUD operations.
#[controller("/{{base_path}}")]
#[derive(Default)]
pub struct {{name_pascal}}Controller;

impl {{name_pascal}}Controller {
    /// List all {{name_snake}}s.
    ///
    /// GET /{{base_path}}
    #[get("/")]
    pub async fn index(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        // TODO: Implement listing logic
        let items: Vec<{{name_pascal}}Dto> = vec![];
        HttpResponse::ok().with_json(&items)
    }

    /// Get a single {{name_snake}} by ID.
    ///
    /// GET /{{base_path}}/:id
    #[get("/:id")]
    pub async fn show(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        let id = req.params.get("id")
            .ok_or_else(|| Error::BadRequest("Missing id parameter".to_string()))?;

        // TODO: Implement fetch logic
        let item = {{name_pascal}}Dto {
            id: Some(id.parse().unwrap_or(0)),
            name: "Example".to_string(),
        };

        HttpResponse::ok().with_json(&item)
    }

    /// Create a new {{name_snake}}.
    ///
    /// POST /{{base_path}}
    #[post("/")]
    pub async fn create(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        let body: Create{{name_pascal}}Request = serde_json::from_slice(&req.body)
            .map_err(|e| Error::BadRequest(format!("Invalid request body: {}", e)))?;

        // TODO: Implement create logic
        let item = {{name_pascal}}Dto {
            id: Some(1),
            name: body.name,
        };

        HttpResponse::created().with_json(&item)
    }

    /// Update an existing {{name_snake}}.
    ///
    /// PUT /{{base_path}}/:id
    #[put("/:id")]
    pub async fn update(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        let id = req.params.get("id")
            .ok_or_else(|| Error::BadRequest("Missing id parameter".to_string()))?;

        let body: Update{{name_pascal}}Request = serde_json::from_slice(&req.body)
            .map_err(|e| Error::BadRequest(format!("Invalid request body: {}", e)))?;

        // TODO: Implement update logic
        let item = {{name_pascal}}Dto {
            id: Some(id.parse().unwrap_or(0)),
            name: body.name.unwrap_or_else(|| "Updated".to_string()),
        };

        HttpResponse::ok().with_json(&item)
    }

    /// Delete a {{name_snake}}.
    ///
    /// DELETE /{{base_path}}/:id
    #[delete("/:id")]
    pub async fn destroy(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        let id = req.params.get("id")
            .ok_or_else(|| Error::BadRequest("Missing id parameter".to_string()))?;

        // TODO: Implement delete logic
        let _ = id;

        HttpResponse::no_content()
    }
}
"#;

const CONTROLLER_TEST_TEMPLATE: &str = r#"//! Tests for {{name_pascal}}Controller.

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_index() {
        let controller = {{name_pascal}}Controller::default();
        let req = HttpRequest::default();

        let response = controller.index(req).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_show() {
        let controller = {{name_pascal}}Controller::default();
        let mut req = HttpRequest::default();
        req.params.insert("id".to_string(), "1".to_string());

        let response = controller.show(req).await;
        assert!(response.is_ok());
    }
}
"#;

// =============================================================================
// MODULE TEMPLATES
// =============================================================================

const MODULE_TEMPLATE: &str = r#"//! {{name_pascal}} module.

use armature::prelude::*;

{{#each controllers}}
mod {{this}};
pub use {{this}}::{{this_pascal}}Controller;
{{/each}}

{{#each providers}}
mod {{this}};
pub use {{this}}::{{this_pascal}}Service;
{{/each}}

/// {{name_pascal}} module bundles related controllers and providers.
#[module(
    controllers: [{{controller_list}}],
    providers: [{{provider_list}}]
)]
#[derive(Default)]
pub struct {{name_pascal}}Module;
"#;

// =============================================================================
// MIDDLEWARE TEMPLATES
// =============================================================================

const MIDDLEWARE_TEMPLATE: &str = r#"//! {{name_pascal}} middleware.

use armature::prelude::*;
use async_trait::async_trait;

/// {{name_pascal}} middleware.
///
/// # Example
///
/// ```rust
/// use armature::prelude::*;
///
/// let middleware = {{name_pascal}}Middleware::new();
/// ```
pub struct {{name_pascal}}Middleware {
    // Add configuration fields here
}

impl {{name_pascal}}Middleware {
    /// Create a new {{name_pascal}}Middleware instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for {{name_pascal}}Middleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for {{name_pascal}}Middleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        // Pre-processing: Add logic before the request is handled
        tracing::debug!("{{name_pascal}}Middleware: Processing request to {}", req.path);

        // Call the next middleware/handler
        let response = next(req).await?;

        // Post-processing: Add logic after the response is generated
        tracing::debug!("{{name_pascal}}Middleware: Response status {}", response.status);

        Ok(response)
    }
}
"#;

const MIDDLEWARE_TEST_TEMPLATE: &str = r#"//! Tests for {{name_pascal}}Middleware.

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_middleware_passes_through() {
        let middleware = {{name_pascal}}Middleware::new();
        let req = HttpRequest::default();

        let next: Next = Box::new(|_req| {
            Box::pin(async { Ok(HttpResponse::ok()) })
        });

        let response = middleware.handle(req, next).await;
        assert!(response.is_ok());
        assert_eq!(response.unwrap().status, 200);
    }
}
"#;

// =============================================================================
// GUARD TEMPLATES
// =============================================================================

const GUARD_TEMPLATE: &str = r#"//! {{name_pascal}} guard.

use armature::prelude::*;
use async_trait::async_trait;

/// {{name_pascal}} guard for route protection.
///
/// # Example
///
/// ```rust
/// use armature::prelude::*;
///
/// let guard = {{name_pascal}}Guard::new();
/// ```
pub struct {{name_pascal}}Guard {
    // Add configuration fields here
}

impl {{name_pascal}}Guard {
    /// Create a new {{name_pascal}}Guard instance.
    pub fn new() -> Self {
        Self {}
    }

    /// Check if the request is authorized.
    fn is_authorized(&self, req: &HttpRequest) -> bool {
        // TODO: Implement authorization logic
        // Example: Check for a valid API key or JWT token
        req.headers.contains_key("authorization")
    }
}

impl Default for {{name_pascal}}Guard {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Guard for {{name_pascal}}Guard {
    async fn can_activate(&self, req: &HttpRequest) -> Result<bool, Error> {
        if self.is_authorized(req) {
            Ok(true)
        } else {
            Err(Error::Unauthorized("Access denied".to_string()))
        }
    }
}
"#;

const GUARD_TEST_TEMPLATE: &str = r#"//! Tests for {{name_pascal}}Guard.

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_guard_denies_unauthorized() {
        let guard = {{name_pascal}}Guard::new();
        let req = HttpRequest::default();

        let result = guard.can_activate(&req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_guard_allows_authorized() {
        let guard = {{name_pascal}}Guard::new();
        let mut req = HttpRequest::default();
        req.headers.insert("authorization".to_string(), "Bearer token".to_string());

        let result = guard.can_activate(&req).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
"#;

// =============================================================================
// SERVICE TEMPLATES
// =============================================================================

const SERVICE_TEMPLATE: &str = r#"//! {{name_pascal}} service.

use armature::prelude::*;
use std::sync::Arc;

/// {{name_pascal}} service provides business logic for {{name_snake}} operations.
///
/// # Example
///
/// ```rust
/// use armature::prelude::*;
///
/// let service = {{name_pascal}}Service::new();
/// ```
#[derive(Clone)]
#[injectable]
pub struct {{name_pascal}}Service {
    // Add dependencies here
}

impl {{name_pascal}}Service {
    /// Create a new {{name_pascal}}Service instance.
    pub fn new() -> Self {
        Self {}
    }

    /// Example method - replace with your business logic.
    pub async fn find_all(&self) -> Result<Vec<String>, Error> {
        // TODO: Implement business logic
        Ok(vec![])
    }

    /// Example method - replace with your business logic.
    pub async fn find_by_id(&self, id: u64) -> Result<Option<String>, Error> {
        // TODO: Implement business logic
        let _ = id;
        Ok(None)
    }
}

impl Default for {{name_pascal}}Service {
    fn default() -> Self {
        Self::new()
    }
}
"#;

const SERVICE_TEST_TEMPLATE: &str = r#"//! Tests for {{name_pascal}}Service.

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_all() {
        let service = {{name_pascal}}Service::new();
        let result = service.find_all().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let service = {{name_pascal}}Service::new();
        let result = service.find_by_id(1).await;
        assert!(result.is_ok());
    }
}
"#;

// =============================================================================
// PROJECT TEMPLATES
// =============================================================================

const MAIN_MINIMAL_TEMPLATE: &str = r#"//! {{name_pascal}} - Built with Armature Framework

use armature::prelude::*;

mod controllers;

use controllers::health::HealthController;

/// Application module.
#[module(controllers: [HealthController])]
#[derive(Default)]
struct AppModule;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create and run the application
    let app = Application::create::<AppModule>().await;

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    println!("🚀 Server running on http://127.0.0.1:{}", port);

    app.listen(port).await?;

    Ok(())
}
"#;

const CARGO_TOML_TEMPLATE: &str = r#"[package]
name = "{{name_kebab}}"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "{{description}}"

[dependencies]
armature = "0.1"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
async-trait = "0.1"
thiserror = "2.0"

[dev-dependencies]
tokio-test = "0.4"
"#;

const ENV_EXAMPLE_TEMPLATE: &str = r#"# {{name_pascal}} Environment Configuration

# Server
PORT=3000
HOST=127.0.0.1

# Logging
RUST_LOG=info

# Database (if needed)
# DATABASE_URL=postgres://user:password@localhost:5432/{{name_snake}}

# Redis (if needed)
# REDIS_URL=redis://localhost:6379

# JWT (if needed)
# JWT_SECRET=your-secret-key-here
# JWT_EXPIRATION=3600
"#;

const README_TEMPLATE: &str = r#"# {{name_pascal}}

{{description}}

Built with [Armature](https://github.com/pegasusheavy/armature) - A modern Rust web framework.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Cargo

### Installation

1. Clone the repository
2. Copy `.env.example` to `.env` and configure
3. Run the development server:

```bash
cargo run
```

Or with the Armature CLI:

```bash
armature dev
```

### Development

Generate new code:

```bash
# Generate a controller
armature generate controller users

# Generate a service
armature generate service users

# Generate a complete resource
armature generate resource products --crud
```

### Building for Production

```bash
cargo build --release
```

## Project Structure

```
{{name_kebab}}/
├── src/
│   ├── main.rs           # Application entry point
│   ├── controllers/      # Route handlers
│   ├── services/         # Business logic
│   ├── middleware/       # Request/response middleware
│   └── guards/           # Route guards
├── Cargo.toml
├── .env.example
└── README.md
```

## License

[Your License]
"#;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Template data for controller generation.
#[derive(Serialize)]
pub struct ControllerData {
    pub name_pascal: String,
    pub name_snake: String,
    pub name_kebab: String,
    pub base_path: String,
}

/// Template data for module generation.
#[derive(Serialize)]
pub struct ModuleData {
    pub name_pascal: String,
    pub name_snake: String,
    pub controllers: Vec<String>,
    pub providers: Vec<String>,
    pub controller_list: String,
    pub provider_list: String,
}

/// Template data for middleware/guard/service generation.
#[derive(Serialize)]
pub struct ComponentData {
    pub name_pascal: String,
    pub name_snake: String,
    pub name_kebab: String,
}

/// Template data for project generation.
#[derive(Serialize)]
pub struct ProjectData {
    pub name_pascal: String,
    pub name_snake: String,
    pub name_kebab: String,
    pub description: String,
}
