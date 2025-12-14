# Armature Documentation

Welcome to the Armature framework documentation! This directory contains comprehensive guides and references for building applications with Armature.

## Getting Started

Start with the main [README](../README.md) in the project root for a quick introduction and setup guide.

## Documentation Index

### Core Guides

- **[di-guide.md](di-guide.md)** - Complete guide to dependency injection
  - Service injection
  - Module system
  - Best practices
  - Testing strategies

- **[auth-guide.md](auth-guide.md)** - Authentication and authorization
  - Password hashing (Bcrypt, Argon2)
  - JWT integration
  - Guards and RBAC
  - Authentication strategies

- **[oauth2-providers-guide.md](oauth2-providers-guide.md)** - OAuth2/OIDC providers
  - Google, Microsoft, AWS, Okta, Auth0
  - Setup and configuration
  - OAuth2 flow
  - Best practices

- **[config-guide.md](config-guide.md)** - Configuration management
  - Environment variables and .env files
  - Multiple configuration formats
  - Type-safe configuration
  - Validation

### API & Data

- **[graphql-guide.md](graphql-guide.md)** - GraphQL API development
- **[graphql-configuration.md](graphql-configuration.md)** - GraphQL configuration options
- **[openapi-guide.md](openapi-guide.md)** - OpenAPI/Swagger documentation
- **[request-extractors.md](request-extractors.md)** - Request data extraction
- **[api-versioning-guide.md](api-versioning-guide.md)** - API versioning strategies
- **[content-negotiation-guide.md](content-negotiation-guide.md)** - Content type negotiation

### Real-Time & Communication

- **[websocket-sse-guide.md](websocket-sse-guide.md)** - WebSocket & SSE guide
- **[webhooks.md](webhooks.md)** - Webhook handling
- **[streaming-responses-guide.md](streaming-responses-guide.md)** - Streaming responses

### Security

- **[security-guide.md](security-guide.md)** - Security best practices
- **[https-guide.md](https-guide.md)** - HTTPS configuration
- **[acme-certificates.md](acme-certificates.md)** - Let's Encrypt/ACME certificates
- **[session-guide.md](session-guide.md)** - Session management
- **[use-guard-guide.md](use-guard-guide.md)** - Using guards for authorization
- **[guards-interceptors.md](guards-interceptors.md)** - Guards and interceptors

### Performance & Caching

- **[queue-guide.md](queue-guide.md)** - Job queue system
- **[cron-guide.md](cron-guide.md)** - Scheduled tasks/cron jobs
- **[rate-limiting-guide.md](rate-limiting-guide.md)** - Rate limiting
- **[response-caching-guide.md](response-caching-guide.md)** - Response caching
- **[compression.md](compression.md)** - Response compression
- **[request-timeouts-guide.md](request-timeouts-guide.md)** - Request timeouts
- **[etag-conditional-requests-guide.md](etag-conditional-requests-guide.md)** - ETag and conditional requests

### Observability

- **[opentelemetry-guide.md](opentelemetry-guide.md)** - OpenTelemetry integration
- **[logging-guide.md](logging-guide.md)** - Logging configuration
- **[debug-logging-guide.md](debug-logging-guide.md)** - Debug logging
- **[health-check-guide.md](health-check-guide.md)** - Health checks
- **[error-correlation-guide.md](error-correlation-guide.md)** - Error correlation

### Architecture

- **[lifecycle-hooks.md](lifecycle-hooks.md)** - Lifecycle hooks
- **[use-middleware-guide.md](use-middleware-guide.md)** - Middleware usage
- **[stateless-architecture.md](stateless-architecture.md)** - Stateless design patterns
- **[server-integration.md](server-integration.md)** - Server integration
- **[http-status-errors.md](http-status-errors.md)** - HTTP status and error handling
- **[error-transformation-guide.md](error-transformation-guide.md)** - Error transformation

### Macros & Code Generation

- **[macro-overview.md](macro-overview.md)** - Overview of Armature macros
- **[project-templates.md](project-templates.md)** - Project templates

### Testing & Quality

- **[testing-documentation.md](testing-documentation.md)** - Testing guide
- **[testing-coverage.md](testing-coverage.md)** - Test coverage
- **[documentation-testing.md](documentation-testing.md)** - Documentation testing

### Feature Guides (in guides/)

- **[guides/route-groups-guide.md](guides/route-groups-guide.md)** - Route groups
- **[guides/route-constraints-guide.md](guides/route-constraints-guide.md)** - Route constraints
- **[guides/metrics-guide.md](guides/metrics-guide.md)** - Prometheus metrics
- **[guides/audit-guide.md](guides/audit-guide.md)** - Audit logging
- **[guides/graceful-shutdown-guide.md](guides/graceful-shutdown-guide.md)** - Graceful shutdown
- **[guides/pagination-filtering-guide.md](guides/pagination-filtering-guide.md)** - Pagination & filtering
- **[guides/security-advanced-guide.md](guides/security-advanced-guide.md)** - Advanced security
- **[guides/cache-improvements-guide.md](guides/cache-improvements-guide.md)** - Cache improvements
- **[guides/macros-guide.md](guides/macros-guide.md)** - Macros in depth
- **[guides/testing-guide.md](guides/testing-guide.md)** - Testing utilities

### Benchmarks

- **[guides/armature-vs-nodejs-benchmark.md](guides/armature-vs-nodejs-benchmark.md)** - Armature vs Node.js frameworks
- **[guides/armature-vs-nextjs-benchmark.md](guides/armature-vs-nextjs-benchmark.md)** - Armature vs Next.js

## Quick Links

### Examples

See the [examples directory](../examples/) for working code samples:
- `full_example.rs` - Complete CRUD application
- `dependency_injection.rs` - DI patterns
- `websocket_chat.rs` - WebSocket chat room
- `server_sent_events.rs` - SSE streaming

### Key Concepts

#### Dependency Injection
Armature provides automatic service injection based on field types:
```rust
#[injectable]
#[derive(Default, Clone)]
struct UserService { }

#[controller("/users")]
#[derive(Default, Clone)]
struct UserController {
    user_service: UserService,  // Auto-injected!
}
```

#### Module System
Organize your application into modules:
```rust
#[module(
    providers: [UserService],
    controllers: [UserController]
)]
#[derive(Default)]
struct AppModule;
```

#### WebSocket & SSE
Built-in real-time communication:
```rust
// WebSocket broadcasting
let room = WebSocketRoom::new("chat".to_string());
room.broadcast_json(&message).await?;

// Server-Sent Events
let broadcaster = SseBroadcaster::new();
broadcaster.broadcast_message("Update".to_string()).await?;
```

## Documentation Structure

```
docs/
├── README.md                         # This file (index)
├── guides/                           # Feature-specific guides
│   ├── route-groups-guide.md
│   ├── route-constraints-guide.md
│   ├── metrics-guide.md
│   ├── audit-guide.md
│   ├── graceful-shutdown-guide.md
│   ├── pagination-filtering-guide.md
│   ├── security-advanced-guide.md
│   ├── cache-improvements-guide.md
│   ├── macros-guide.md
│   ├── testing-guide.md
│   ├── armature-vs-nodejs-benchmark.md
│   └── armature-vs-nextjs-benchmark.md
├── auth-guide.md                     # Authentication
├── config-guide.md                   # Configuration
├── di-guide.md                       # Dependency injection
├── graphql-guide.md                  # GraphQL
├── openapi-guide.md                  # OpenAPI/Swagger
├── queue-guide.md                    # Job queues
├── security-guide.md                 # Security
├── websocket-sse-guide.md            # Real-time
└── ...                               # Other guides
```

## Naming Conventions

All documentation files follow these conventions:
- **lowercase with hyphens**: `my-feature-guide.md`
- **descriptive names**: `oauth2-providers-guide.md` not `oauth.md`
- **.md extension** for all Markdown files

## Version

This documentation is for Armature version 0.1.0.

---

For the latest updates and more information, visit the main [README](../README.md).
