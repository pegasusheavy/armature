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
- Comprehensive documentation (`docs/RATE_LIMITING_GUIDE.md`)

#### Project Templates
- New `templates/` directory with starter templates:
  - **api-minimal** - Single-file REST API for learning and prototyping
  - **api-full** - Production-ready API with JWT auth, validation, Docker, health checks
  - **microservice** - Queue-connected worker with Prometheus metrics and graceful shutdown
- Template documentation (`docs/PROJECT_TEMPLATES.md`)
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
- 30+ working examples

#### Documentation
- `docs/RATE_LIMITING_GUIDE.md` - Rate limiting usage and best practices
- `docs/PROJECT_TEMPLATES.md` - Template usage and customization guide
- `docs/DEBUG_LOGGING_GUIDE.md` - Debug logging configuration
- `docs/LOGGING_GUIDE.md` - Logging system documentation

### Changed
- GitHub Actions CI/CD workflows
- GitHub Pages deployment for documentation website
- GitHub Security Advisories for vulnerability reporting

### Deprecated
- N/A

### Removed
- N/A

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

See [docs/MIGRATION.md](docs/MIGRATION.md) for detailed upgrade instructions between major versions.

---

[Unreleased]: https://github.com/pegasusheavy/armature/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/pegasusheavy/armature/releases/tag/v0.1.0


