# Armature Project Templates

Starter templates for building applications with the Armature framework.

## Available Templates

| Template | Description | Features |
|----------|-------------|----------|
| [api-minimal](./api-minimal/) | Bare-bones REST API | Basic routing, JSON responses |
| [api-full](./api-full/) | Full-featured API | Auth, validation, OpenAPI, Docker |
| [microservice](./microservice/) | Queue-connected microservice | Job worker, health checks, Docker |

## Quick Start

### Using a Template

1. Copy the template directory:
   ```bash
   cp -r templates/api-minimal my-project
   cd my-project
   ```

2. Update `Cargo.toml` with your project name:
   ```toml
   [package]
   name = "my-project"
   ```

3. Copy `.env.example` to `.env` and configure:
   ```bash
   cp .env.example .env
   ```

4. Run the project:
   ```bash
   cargo run
   ```

## Template Details

### api-minimal

The simplest starting point. Perfect for:
- Learning Armature
- Small APIs
- Prototyping

Features:
- Single-file implementation
- Basic health check endpoint
- JSON response helpers

### api-full

Production-ready API template. Perfect for:
- SaaS backends
- Public APIs
- Enterprise applications

Features:
- JWT authentication
- Request validation
- OpenAPI documentation
- Structured logging
- Docker support
- Health checks
- CORS configuration

### microservice

Background job processor. Perfect for:
- Email/notification services
- Data processing pipelines
- Async task handlers

Features:
- Redis job queue
- Retry with backoff
- Health endpoints
- Graceful shutdown
- Docker support

## Customization

All templates are designed to be customized. Common modifications:

### Adding Database Support

```toml
# Cargo.toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres"] }
```

### Adding Authentication

```toml
# Cargo.toml
[dependencies]
armature = { version = "0.1", features = ["auth"] }
```

### Adding Rate Limiting

```toml
# Cargo.toml
[dependencies]
armature = { version = "0.1", features = ["ratelimit"] }
```

## Contributing

To add a new template:

1. Create a new directory under `templates/`
2. Include at minimum:
   - `Cargo.toml`
   - `src/main.rs`
   - `.env.example`
   - `README.md` (optional but recommended)
3. Update this README with the new template
4. Test the template works out of the box

## License

All templates are provided under the same license as Armature (Apache-2.0).

