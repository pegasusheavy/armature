# Armature Examples

This directory contains example applications demonstrating various features of the Armature framework.

## Running Examples

```bash
# Full-featured example with complete CRUD operations
cargo run --example full_example

# Simple routing and JSON responses
cargo run --example simple

# REST API with task management
cargo run --example rest_api

# Dependency injection demonstration
cargo run --example dependency_injection

# Automatic DI with module system
cargo run --example automatic_di
```

## Examples

### full_example.rs

A comprehensive example demonstrating:
- User CRUD operations (Create, Read, Update, Delete)
- Health check endpoint
- Injectable services (`#[injectable]`)
- Controller with dependency injection (`#[controller]`)
- Multiple HTTP methods (GET, POST)
- Path parameters (`/users/:id`)
- JSON request/response handling
- Error handling
- Module system (`#[module]`)

**Endpoints:**
- `GET /health` - Health check
- `GET /users` - Get all users
- `GET /users/:id` - Get user by ID
- `POST /users` - Create new user

**Port:** 3000

### simple.rs

A minimal example showing:
- Basic routing
- JSON responses
- Echo endpoint

**Endpoints:**
- `GET /api/hello` - Hello world message
- `POST /api/echo` - Echo back the request body

**Port:** 3001

### rest_api.rs

A REST API example with:
- Full CRUD operations for tasks
- Resource-based routing
- Service layer pattern
- Request/response DTOs

**Endpoints:**
- `GET /tasks` - List all tasks
- `GET /tasks/:id` - Get task by ID
- `POST /tasks` - Create new task
- `PUT /tasks/:id` - Update task
- `DELETE /tasks/:id` - Delete task

**Port:** 3002

### dependency_injection.rs

A comprehensive example demonstrating the full dependency injection system:
- Multi-level service dependencies (DatabaseService → ProductService → ProductController)
- Automatic dependency resolution
- Service reuse across the application
- Clean separation of concerns

**Features:**
- DatabaseService (base service)
- ProductService (depends on DatabaseService)
- ProductController (depends on ProductService)
- Full CRUD operations with injected dependencies

**Endpoints:**
- `GET /health` - Health check
- `GET /products` - List all products
- `GET /products/:id` - Get product by ID
- `POST /products` - Create new product

**Port:** 3003

### automatic_di.rs

Shows how the module system automatically handles DI:
- Service registration in correct order
- Automatic dependency resolution
- Module-based organization

**Features:**
- LoggerService → BookService → BookController chain
- Demonstrates `#[module]` with providers and controllers
- Shows how Application::create() would work with full DI

**Endpoints:**
- `GET /books` - List all books

**Port:** 3004

### websocket_chat.rs

WebSocket chat room demonstration:
- WebSocket connection management
- Room-based broadcasting
- Multi-user chat support
- Connection statistics

**Features:**
- WebSocketManager for room management
- WebSocketRoom for broadcasting
- Chat message broadcasting
- Room statistics endpoint

**Endpoints:**
- `POST /chat/:room/message` - Send message to room
- `GET /chat/:room/stats` - Get room statistics

**Port:** 3005

### server_sent_events.rs

Server-Sent Events streaming example:
- Real-time stock price updates
- News feed streaming
- Automatic keep-alive
- Multiple concurrent subscribers

**Features:**
- SseBroadcaster for multi-client streaming
- Automatic reconnection support
- JSON event streaming
- Keep-alive mechanism

**Endpoints:**
- `GET /events/stocks` - Stock price SSE stream
- `GET /events/news` - News updates SSE stream
- `GET /events/stats` - Subscriber statistics

**Port:** 3006

### graphql_api.rs

GraphQL API with full query and mutation support:
- Type-safe GraphQL schema
- Queries for data retrieval
- Mutations for data modification
- Built-in GraphiQL playground

**Features:**
- Book catalog with authors
- Search functionality
- CRUD operations via GraphQL
- Schema introspection

**Endpoints:**
- `POST /graphql` - GraphQL API endpoint
- `GET /playground` - GraphiQL playground
- `GET /graphql/schema` - Schema SDL

**Port:** 3007

**Note:** Requires the `graphql` feature flag:
```bash
cargo run --example graphql_api --features graphql
```

### config_example.rs

Configuration management with multiple sources:
- Environment variables
- .env file loading
- Type-safe configuration
- Validation support

**Features:**
- Load from env, .env, JSON, TOML
- Prefix support for namespacing
- Hierarchical configuration
- Injectable config service

**Endpoints:**
- `GET /config/info` - Application info
- `GET /config/database` - Database config

**Port:** 3008

**Note:** Requires the `config` feature flag:
```bash
cargo run --example config_example --features config
```

## Testing Endpoints

Once an example is running, you can test it with curl:

```bash
# For full_example (port 3000)
curl http://localhost:3000/health
curl http://localhost:3000/users
curl http://localhost:3000/users/1
curl -X POST http://localhost:3000/users \
  -H 'Content-Type: application/json' \
  -d '{"name":"Charlie","email":"charlie@example.com"}'

# For simple (port 3001)
curl http://localhost:3001/api/hello
curl -X POST http://localhost:3001/api/echo \
  -H 'Content-Type: application/json' \
  -d '{"text":"Hello Armature!"}'

# For rest_api (port 3002)
curl http://localhost:3002/tasks
curl http://localhost:3002/tasks/1
curl -X POST http://localhost:3002/tasks \
  -H 'Content-Type: application/json' \
  -d '{"id":0,"title":"New Task","completed":false}'
```

## Using Armature in Your Project

To use Armature in your own project, add it to your `Cargo.toml`:

```toml
[dependencies]
armature = { path = "../armature" }  # Or use git/version when published
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

Then import the prelude:

```rust
use armature::prelude::*;
```

