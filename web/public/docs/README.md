# Armature Documentation

Welcome to the Armature framework documentation! This directory contains comprehensive guides and references for building applications with Armature.

## Getting Started

Start with the main [README](../README.md) in the project root for a quick introduction and setup guide.

## Documentation Index

### Core Guides

- **[DI_GUIDE.md](DI_GUIDE.md)** - Complete guide to dependency injection
  - Service injection
  - Module system
  - Best practices
  - Testing strategies

- **[AUTH_GUIDE.md](AUTH_GUIDE.md)** - Authentication and authorization
  - Password hashing (Bcrypt, Argon2)
  - JWT integration
  - Guards and RBAC
  - Authentication strategies

- **[OAUTH2_PROVIDERS_GUIDE.md](OAUTH2_PROVIDERS_GUIDE.md)** - OAuth2/OIDC providers
  - Google, Microsoft, AWS, Okta, Auth0
  - Setup and configuration
  - OAuth2 flow
  - Best practices

- **[CONFIG_GUIDE.md](CONFIG_GUIDE.md)** - Configuration management
  - Environment variables and .env files
  - Multiple configuration formats
  - Type-safe configuration
  - Validation

- **[GRAPHQL_GUIDE.md](GRAPHQL_GUIDE.md)** - GraphQL API development
  - Schema definition
  - Queries, mutations, and subscriptions
  - DI integration
  - Best practices

- **[WEBSOCKET_SSE_GUIDE.md](WEBSOCKET_SSE_GUIDE.md)** - Real-time communication guide
  - WebSocket rooms and broadcasting
  - Server-Sent Events streaming
  - Usage examples
  - Performance tips

- **[RATE_LIMITING_GUIDE.md](RATE_LIMITING_GUIDE.md)** - API rate limiting
  - Token bucket, sliding window, fixed window algorithms
  - Redis-backed distributed rate limiting
  - Key extraction strategies
  - Best practices

- **[GUARDS_INTERCEPTORS.md](GUARDS_INTERCEPTORS.md)** - Request processing
  - Authentication guards
  - Role-based access control
  - Request/response interceptors
  - Custom implementations

### Reference Documentation

- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Contributing guidelines
  - Development setup
  - Code style
  - Testing requirements
  - Pull request process

## Quick Links

### Examples

See the [examples directory](../examples/) for working code samples:
- `full_example.rs` - Complete CRUD application
- `dependency_injection.rs` - DI patterns
- `websocket_chat.rs` - WebSocket chat room
- `server_sent_events.rs` - SSE streaming
- `rate_limiting.rs` - Rate limiting example

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

## Additional Resources

- [Cargo.toml](../Cargo.toml) - Dependencies and workspace configuration
- [Examples README](../examples/README.md) - Detailed example descriptions
- [Tests](../armature-core/tests/) - Test suite for reference

## Community & Support

- Report issues on GitHub
- Check existing documentation before asking questions
- Contribute improvements via pull requests

## Documentation Structure

```
docs/
├── README.md                    # This file
├── DI_GUIDE.md                 # Dependency Injection guide
├── WEBSOCKET_SSE_GUIDE.md      # WebSocket & SSE guide
├── RATE_LIMITING_GUIDE.md      # Rate limiting guide
└── CONTRIBUTING.md              # Contributing guidelines
```

## Version

This documentation is for Armature version 0.1.0.

---

For the latest updates and more information, visit the main [README](../README.md).
