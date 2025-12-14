# Armature 🦾

A modern, type-safe HTTP framework for Rust heavily inspired by **Angular** and **NestJS**.

Armature brings the elegant decorator syntax and powerful dependency injection from the TypeScript/JavaScript ecosystem to Rust, combining the developer experience of NestJS with Rust's performance and safety guarantees.

## Features

- **Completely Stateless**: No server-side sessions, fully stateless JWT-based authentication
- **Decorator Syntax**: Use Angular-style decorators via procedural macros
- **Full Dependency Injection**: Automatic service injection into controllers based on field types
- **Type-Safe DI Container**: Compile-time verified dependency resolution
- **Modular Architecture**: Organize your application into modules with providers and controllers
- **Service Dependencies**: Services can depend on other services with automatic resolution
- **Singleton Pattern**: Services are created once and shared across the application
- **Lifecycle Hooks**: NestJS-style hooks (OnModuleInit, OnModuleDestroy, OnApplicationBootstrap, OnApplicationShutdown) for resource management
- **Authentication & Authorization**: Optional comprehensive auth system with guards, RBAC, and strategies
- **OAuth2/OIDC Providers**: Built-in support for Google, Microsoft, AWS Cognito, Okta, and Auth0
- **SAML 2.0 Support**: Enterprise SSO with Service Provider implementation
- **Password Hashing**: Bcrypt and Argon2 support with auto-detection
- **JWT Authentication**: Optional JWT token management with HS256/RS256/ES256 support
- **Configuration Management**: Optional NestJS-style config system with env, .env, JSON, and TOML support
- **GraphQL Support**: Optional type-safe GraphQL API with queries, mutations, and subscriptions
- **Rate Limiting**: Token bucket, sliding window, and fixed window algorithms with Redis support
- **Response Compression**: Automatic gzip, brotli, and zstd compression with content-type awareness
- **Comprehensive Logging**: Highly configurable structured logging with JSON/Pretty/Plain formats, multiple outputs, and HTTP middleware
- **Testing Utilities**: Comprehensive testing framework with mocks, spies, and assertions
- **Validation Framework**: Powerful validation with built-in validators and custom rules
- **WebSocket Support**: Full-duplex real-time communication with rooms and broadcasting
- **Server-Sent Events (SSE)**: Efficient server-to-client streaming for live updates
- **HTTPS/TLS Support**: Built-in TLS support with certificate management and automatic HTTP to HTTPS redirect
- **OpenTelemetry Integration**: Distributed tracing, metrics, and observability with OTLP, Jaeger, Zipkin, and Prometheus support
- **Async-First**: Built on Tokio and Hyper for high-performance async I/O
- **Type-Safe Routing**: Path parameters and query string parsing with compile-time validation
- **JSON Serialization**: Built-in support for JSON request/response handling

## Quick Start

### Using the CLI (Recommended)

Install the Armature CLI for the best development experience:

```bash
# Install the CLI
cargo install armature-cli

# Create a new project
armature new my-api

# Navigate to your project
cd my-api

# Start the development server with hot reloading
armature dev
```

### Generate Code

```bash
# Generate a controller
armature generate controller users

# Generate a service
armature generate service users

# Generate a complete resource (controller + service + module)
armature generate resource products --crud

# Generate middleware, guards, and more
armature generate middleware auth
armature generate guard admin
```

### Manual Setup

```rust
use armature::prelude::*;
use serde::{Deserialize, Serialize};

// Define your domain model
#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
}

// Create an injectable service
#[injectable]
#[derive(Default, Clone)]
struct DatabaseService;

#[injectable]
#[derive(Default, Clone)]
struct UserService {
    database: DatabaseService,  // Automatically injected!
}

impl UserService {
    fn get_users(&self) -> Vec<User> {
        vec![User { id: 1, name: "Alice".to_string() }]
    }
}

// Create a controller with automatic DI
#[controller("/users")]
#[derive(Default, Clone)]
struct UserController {
    user_service: UserService,  // Automatically injected!
}

impl UserController {
    // Use injected service in methods
    fn get_all(&self) -> Result<Json<Vec<User>>, Error> {
        Ok(Json(self.user_service.get_users()))
    }

    fn get_one(&self, id: u32) -> Result<Json<User>, Error> {
        // Use self.user_service throughout the controller
        Ok(Json(User { id, name: "Alice".to_string() }))
    }
}

// Define your application module
#[module(
    providers: [DatabaseService, UserService],
    controllers: [UserController]
)]
#[derive(Default)]
struct AppModule;

// Bootstrap your application - DI happens automatically!
#[tokio::main]
async fn main() {
    let app = Application::create::<AppModule>();
    app.listen(3000).await.unwrap();
}
```

## Architecture

The framework is organized into three main crates:

### `armature-core`
Core runtime functionality including:
- Traits (`Provider`, `Controller`, `Module`, `RequestHandler`)
- DI Container with type-based resolution
- Router with path parameter extraction
- HTTP request/response types
- Error handling

### `armature-macro`
Procedural macros for decorator syntax:
- `#[injectable]` - Mark structs as injectable services
- `#[controller("/path")]` - Define controllers with base paths
- `#[get]`, `#[post]`, `#[put]`, `#[delete]`, `#[patch]` - HTTP route decorators
- `#[module(...)]` - Organize components into modules
- `#[derive(Body)]`, `#[derive(Param)]`, `#[derive(Query)]` - Request parameter extraction

### `armature`
Main library that re-exports everything from core and macros

## Decorators

### Service Decorators

#### `#[injectable]`
Marks a struct as injectable, allowing it to be registered in the DI container:

```rust
#[injectable]
struct DatabaseService {
    connection: DbConnection,
}
```

### Controller Decorators

#### `#[controller("/path")]`
Marks a struct as a controller with a base path:

```rust
#[controller("/api/users")]
struct UserController {
    user_service: UserService,
}
```

### Route Decorators

HTTP method decorators for defining routes:

```rust
#[get("/")]           // GET /api/users/
#[get("/:id")]        // GET /api/users/:id
#[post("/")]          // POST /api/users/
#[put("/:id")]        // PUT /api/users/:id
#[delete("/:id")]     // DELETE /api/users/:id
#[patch("/:id")]      // PATCH /api/users/:id
```

### Module Decorator

#### `#[module(...)]`
Defines a module with providers, controllers, and imports:

```rust
#[module(
    providers: [UserService, DatabaseService],
    controllers: [UserController],
    imports: [CommonModule],
    exports: [UserService]
)]
struct UserModule;
```

## Dependency Injection

The DI container uses Rust's type system for service resolution:

```rust
// Register a service
container.register(MyService::default());

// Resolve a service
let service = container.resolve::<MyService>()?;
```

Services are singletons by default and shared across the application.

## Lifecycle Hooks

Armature provides NestJS-inspired lifecycle hooks for managing service initialization and cleanup:

```rust
use armature::prelude::*;
use armature::lifecycle::{OnModuleInit, OnModuleDestroy, LifecycleResult};
use async_trait::async_trait;

#[injectable]
struct DatabaseService {
    connected: Arc<RwLock<bool>>,
}

#[async_trait]
impl OnModuleInit for DatabaseService {
    async fn on_module_init(&self) -> LifecycleResult {
        println!("Connecting to database...");
        *self.connected.write().await = true;
        Ok(())
    }
}

#[async_trait]
impl OnModuleDestroy for DatabaseService {
    async fn on_module_destroy(&self) -> LifecycleResult {
        println!("Closing database connection...");
        *self.connected.write().await = false;
        Ok(())
    }
}
```

Available hooks:
- `OnModuleInit` - Called after module initialization
- `OnModuleDestroy` - Called before module destruction
- `OnApplicationBootstrap` - Called after full application bootstrap
- `OnApplicationShutdown` - Called during graceful shutdown
- `BeforeApplicationShutdown` - Called before shutdown hooks

See the [Lifecycle Hooks Guide](docs/lifecycle-hooks.md) for complete documentation.

## Routing

The router supports:
- Static paths: `/users`
- Path parameters: `/users/:id`
- Query parameters: `/users?page=1&limit=10`
- Multiple HTTP methods per route

Path parameters are extracted automatically:

```rust
#[get("/:id")]
async fn get_user(req: HttpRequest) -> Result<Json<User>, Error> {
    let id = req.param("id").unwrap();
    // ...
}
```

## HTTP Status Codes and Error Handling

Comprehensive HTTP status code support with type-safe error handling:

```rust
use armature::{Error, HttpStatus};

// Type-safe status codes
let status = HttpStatus::NotFound;
assert_eq!(status.code(), 404);
assert_eq!(status.reason(), "Not Found");

// Structured errors for all 4xx/5xx codes
return Err(Error::NotFound("User not found".to_string()));
return Err(Error::TooManyRequests("Rate limit exceeded".to_string()));
return Err(Error::ServiceUnavailable("Database down".to_string()));

// Error helpers
let error = Error::Unauthorized("Invalid token".to_string());
assert_eq!(error.status_code(), 401);
assert!(error.is_client_error());
```

See the [HTTP Status & Errors Guide](docs/http-status-errors.md) for complete documentation.

## Guards and Interceptors

Protect and transform your routes with Guards and Interceptors:

```rust
use armature::{Guard, AuthenticationGuard, GuardContext};

// Apply authentication guard
let guard = AuthenticationGuard;
let context = GuardContext::new(request);
match guard.can_activate(&context).await {
    Ok(true) => { /* proceed */ },
    _ => { /* unauthorized */ }
}

// Built-in guards: AuthenticationGuard, RolesGuard, ApiKeyGuard
// Built-in interceptors: LoggingInterceptor, TransformInterceptor, CacheInterceptor
```

See the [Guards & Interceptors Guide](docs/guards-interceptors.md) for detailed documentation.

## Request/Response Handling

### Request
The `HttpRequest` type provides:
- `json::<T>()` - Parse body as JSON
- `param(name)` - Get path parameter
- `query(name)` - Get query parameter
- Access to headers and raw body

### Response
The `HttpResponse` type provides:
- Status code setting
- Header management
- JSON serialization via `with_json()`
- Helper constructors: `ok()`, `created()`, `not_found()`, etc.

### JSON Helper
Use the `Json<T>` wrapper for automatic serialization:

```rust
#[get("/users")]
async fn get_users() -> Result<Json<Vec<User>>, Error> {
    Ok(Json(vec![...]))
}
```

## Error Handling

The framework provides a comprehensive `Error` type:

```rust
pub enum Error {
    Http(String),
    RouteNotFound(String),
    MethodNotAllowed(String),
    DependencyInjection(String),
    ProviderNotFound(String),
    Serialization(String),
    Deserialization(String),
    Validation(String),
    Internal(String),
    Io(std::io::Error),
}
```

Errors are automatically converted to HTTP responses with appropriate status codes.

## Testing

Run the test suite:

```bash
cargo test
```

Build the library:

```bash
cargo build --lib
```

Run example applications:

```bash
# Full-featured example with CRUD operations
cargo run --example full_example

# Simple routing example
cargo run --example simple

# REST API example
cargo run --example rest_api
```

Test the endpoints (when running full_example):

```bash
# Health check
curl http://localhost:3000/health

# Get all users
curl http://localhost:3000/users

# Get user by ID
curl http://localhost:3000/users/1

# Create a user
curl -X POST http://localhost:3000/users \
  -H 'Content-Type: application/json' \
  -d '{"name":"Charlie","email":"charlie@example.com"}'
```

## Project Structure

```
armature/
├── armature-core/       # Core runtime library
│   └── src/
│       ├── traits.rs    # Core traits
│       ├── container.rs # DI container
│       ├── routing.rs   # Router implementation
│       ├── http.rs      # HTTP types
│       ├── error.rs     # Error types
│       └── application.rs # Application bootstrap
├── armature-macro/      # Procedural macros
│   └── src/
│       ├── injectable.rs   # #[injectable] macro
│       ├── controller.rs   # #[controller] macro
│       ├── routes.rs       # Route macros
│       ├── module.rs       # #[module] macro
│       └── params.rs       # Parameter extraction
├── armature-compression/ # HTTP response compression
│   └── src/
│       ├── algorithm.rs  # Compression algorithms (gzip, brotli, zstd)
│       ├── config.rs     # Configuration builder
│       └── middleware.rs # Compression middleware
├── src/
│   └── lib.rs          # Main library (re-exports)
├── examples/           # Example applications
│   ├── full_example.rs # Complete CRUD example
│   ├── simple.rs       # Basic routing
│   └── rest_api.rs     # REST API demo
└── Cargo.toml          # Workspace manifest
```

## Armature CLI

The Armature CLI provides powerful code generation and development tools:

### Installation

```bash
cargo install armature-cli
```

### Commands

| Command | Description |
|---------|-------------|
| `armature new <name>` | Create a new project from templates |
| `armature generate controller <name>` | Generate a controller |
| `armature generate service <name>` | Generate a service/provider |
| `armature generate module <name>` | Generate a module |
| `armature generate middleware <name>` | Generate middleware |
| `armature generate guard <name>` | Generate a guard |
| `armature generate resource <name>` | Generate controller + service + module |
| `armature dev` | Start development server with file watching |
| `armature build` | Build for production |
| `armature info` | Display project information |

### Project Templates

```bash
# Minimal API (default)
armature new my-api

# Full-featured API with auth, validation, Docker
armature new my-api --template full

# Microservice with queue worker
armature new my-api --template microservice
```

### Development Server

The `armature dev` command starts a development server with automatic rebuilding:

```bash
# Start with default settings (port 3000)
armature dev

# Custom port
armature dev --port 8080

# Uses cargo-watch if installed for better performance
```

## Design Principles

1. **Compile-Time Safety**: All metadata is captured at compile time via macros
2. **Zero-Cost Abstractions**: Minimal runtime overhead
3. **Type-Driven**: Leverage Rust's type system for DI and routing
4. **Async-First**: Built on Tokio for efficient async I/O
5. **Modular**: Organize code into reusable modules

## Comparison with Other Frameworks

### vs Actix-Web
- Armature provides Angular-style decorators and DI
- More opinionated structure
- Built-in module system

### vs Axum
- Armature uses decorators instead of extractors
- Explicit DI container vs implicit FromRequest
- Modular architecture by default

### vs Rocket
- Similar decorator syntax but with async support
- Type-safe DI without macros
- More flexible module system

## Roadmap

- [x] Full DI integration with auto-wiring
- [x] Middleware support
- [x] Guards and interceptors
- [x] WebSocket support
- [x] GraphQL integration
- [x] OpenAPI/Swagger generation
- [x] Authentication/authorization modules (JWT, OAuth2, SAML)
- [x] Testing utilities
- [x] Response compression (gzip, brotli, zstd)
- [ ] Database integration modules
- [ ] `#[use_middleware]` decorator syntax

## Acknowledgments

This project is heavily inspired by:
- **[Angular](https://angular.io/)** by Google - For pioneering decorator-based DI and modular architecture
- **[NestJS](https://nestjs.com/)** by Kamil Myśliwiec - For bringing Angular patterns to the server-side

We're grateful to these projects and their communities for showing what great developer experience looks like. Armature aims to bring these same patterns to the Rust ecosystem with the added benefits of memory safety and native performance.

## License

MIT

## Documentation

🌐 **Live Documentation Website**: [https://pegasusheavy.github.io/armature/](https://pegasusheavy.github.io/armature/)

Comprehensive documentation is available in the [`docs/`](docs/) directory:

**Getting Started:**
- **[Dependency Injection Guide](docs/di-guide.md)** - Complete DI system documentation
- **[Configuration Guide](docs/config-guide.md)** - Configuration management system

**Core Features:**
- **[Lifecycle Hooks Guide](docs/lifecycle-hooks.md)** - Service lifecycle management with hooks
- **[Authentication Guide](docs/auth-guide.md)** - JWT, OAuth2, and SAML authentication
- **[Guards & Interceptors](docs/guards-interceptors.md)** - Request processing and authorization
- **[Rate Limiting Guide](docs/rate-limiting-guide.md)** - API rate limiting with multiple algorithms
- **[Compression Guide](docs/modules/compression.md)** - HTTP response compression middleware

**Advanced:**
- **[GraphQL Guide](docs/graphql-guide.md)** - GraphQL API development
- **[WebSocket & SSE Guide](docs/websocket-sse-guide.md)** - Real-time communication guide
- **[Logging Guide](docs/logging-guide.md)** - Structured logging with tracing
- **[Parallel Processing Guide](docs/parallel-processing-guide.md)** - Multithreading and optimization

**And more guides covering testing, security, and deployment!**

## Website Development

The documentation website is an Angular 21 application located in the [`web/`](web/) directory.

**Local Development:**

```bash
cd web
pnpm install
pnpm start
```

Then open [http://localhost:4200](http://localhost:4200) in your browser.

**Building for Production:**

```bash
cd web
pnpm run build
```

The built website will be in `web/dist/web/browser/`.

**GitHub Pages Deployment:**

The website automatically deploys to GitHub Pages when changes are merged to the `main` branch.

## Contributing

Contributions are welcome! Please read our [Contributing Guide](docs/contributing.md) and feel free to submit a Pull Request.


