# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Rate Limiting Module (`armature-ratelimit`)
- New `armature-ratelimit` crate for comprehensive API rate limiting
- **Algorithms**:
  - Token Bucket - smooth rate limiting with burst capacity
  - Sliding Window Log - precise rate limiting with timestamp tracking
  - Fixed Window - simple fixed time window counters
- **Storage Backends**:
  - `MemoryStore` - in-memory storage using DashMap (default)
  - `RedisStore` - Redis-backed distributed storage (optional `redis` feature)
- **Key Extraction**:
  - By IP address, user ID, API key, or custom headers
  - `KeyExtractorBuilder` for complex extraction logic
  - Per-endpoint rate limiting with `IpAndPath` extractor
- **Middleware**:
  - `RateLimitMiddleware` ready for HTTP integration
  - Standard headers: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`, `Retry-After`
  - Bypass keys for whitelisting specific clients
  - Fail-open mode for high availability
- Rate limiting example (`examples/rate_limiting.rs`)
- Comprehensive documentation (`docs/rate-limiting-guide.md`)

#### Armature CLI (`armature-cli`)
- New `armature-cli` crate for code generation and development tools
- **Commands**:
  - `armature new <name>` - Create new projects from templates (minimal, full, microservice)
  - `armature generate controller <name>` - Generate controllers with optional CRUD
  - `armature generate service <name>` - Generate injectable services
  - `armature generate module <name>` - Generate modules with controllers and providers
  - `armature generate middleware <name>` - Generate middleware
  - `armature generate guard <name>` - Generate route guards
  - `armature generate resource <name>` - Generate complete resource (controller + service + module)
  - `armature dev` - Development server with file watching and hot reloading
  - `armature build` - Production build with size reporting
  - `armature info` - Display project information
- **Features**:
  - Template-based code generation using Handlebars
  - Automatic `mod.rs` updates when generating code
  - Test file generation (optional)
  - Progress indicators and colored output
  - Uses `cargo-watch` if installed for better performance

#### Project Templates
- New `templates/` directory with starter templates:
  - **api-minimal** - Single-file REST API for learning and prototyping
  - **api-full** - Production-ready API with JWT auth, validation, Docker, health checks
  - **microservice** - Queue-connected worker with Prometheus metrics and graceful shutdown
- Template documentation (`docs/project-templates.md`)
- Each template includes:
  - `Cargo.toml` with appropriate dependencies
  - `.env.example` for configuration
  - `Dockerfile` and `docker-compose.yml` where applicable

#### Core Framework
- Initial release of Armature framework
- Core framework with dependency injection and decorators
- Authentication support (JWT, OAuth2, SAML)
- GraphQL support
- Validation framework
- Testing utilities
- OpenAPI/Swagger integration
- Caching (Redis, Memcached)
- Job queue system
- Cron scheduling
- OpenTelemetry observability
- Security middleware (Helmet-like)
- HTTPS/TLS support
- Static asset serving with compression
- Comprehensive debug logging throughout the framework
- 30+ working examples refactored to use module/controller pattern
- Angular 21 documentation website with:
  - Tailwind CSS 4 styling
  - SPA routing with 404.html fallback for GitHub Pages
  - Vitest for unit testing
  - API documentation integration at `/api/`

#### Documentation
- `docs/rate-limiting-guide.md` - Rate limiting usage and best practices
- `docs/project-templates.md` - Template usage and customization guide
- `docs/debug-logging-guide.md` - Debug logging configuration
- `docs/logging-guide.md` - Logging system documentation
- GitHub Pages deployment with Angular website at `https://pegasusheavy.github.io/armature/`
- `examples/handlebars_templates.rs` - Demonstrates using Handlebars with DI container

### Changed
- GitHub Actions CI/CD workflows
- GitHub Pages deployment for documentation website with Angular SPA support
- GitHub Security Advisories for vulnerability reporting
- **All examples now use module/controller pattern** instead of manual route registration
- All middleware implementations now implement `armature_core::Middleware` trait
- `RateLimitMiddleware` now implements `armature_core::Middleware` for `MiddlewareChain` integration
- `SecurityMiddleware` now implements `armature_core::Middleware` for `MiddlewareChain` integration
- Updated `Application::create::<Module>()` as the standard app bootstrapping pattern

### Deprecated
- N/A

### Removed
- **SSR Modules** - Removed `armature-angular`, `armature-react`, `armature-vue`, `armature-svelte` crates
- **Security Modules** - Removed `armature-csrf` and `armature-xss` crates (use `armature-security` instead)
- **Handlebars Plugin** - Removed `armature-handlebars` crate (use `handlebars` crate directly with DI container - see `examples/handlebars_templates.rs`)

### Fixed
- Fixed `CertificateParams::new` type mismatch in TLS module
- Fixed clippy warnings across multiple crates
- Code formatting standardized with `cargo fmt`

### Security
- Added cargo-husky for Git hooks (pre-commit, pre-push, commit-msg)
- Branch protection via Git hooks
- Automated linting and testing on commits

## [0.1.0] - TBD

### Added
- Initial public release

---

## Version History

### Versioning Strategy

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** version when making incompatible API changes
- **MINOR** version when adding functionality in a backward compatible manner
- **PATCH** version when making backward compatible bug fixes

### Release Schedule

- **Major releases**: When significant breaking changes are necessary
- **Minor releases**: Every 2-3 months with new features
- **Patch releases**: As needed for bug fixes and security updates

### Upgrade Guide

See [docs/migration.md](docs/migration.md) for detailed upgrade instructions between major versions.

---

[Unreleased]: https://github.com/pegasusheavy/armature/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/pegasusheavy/armature/releases/tag/v0.1.0


