# Armature Framework - Feature Roadmap

A comprehensive list of features to make Armature enterprise-grade, production-ready, and easy to use.

## Legend

- 🔴 **Critical** - Must have for production use
- 🟠 **High Priority** - Important for enterprise adoption
- 🟡 **Medium Priority** - Nice to have, improves DX
- 🟢 **Low Priority** - Future enhancements
- ✅ **Completed** - Already implemented

---

## 1. Core Framework Enhancements

### Request/Response Handling

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Request Timeout | Configurable request timeouts with graceful handling | `armature-core` |
| ✅ | Request Size Limits | Max body size, max header size configuration | `armature-core` |
| ✅ | Content Negotiation | Accept header parsing, response format selection | `armature-core` |
| ✅ | ETags & Conditional Requests | If-Match, If-None-Match, If-Modified-Since support | `armature-core` |
| ✅ | Response Caching Headers | Cache-Control, Expires, Vary header helpers | `armature-core` |
| ✅ | Streaming Responses | Chunked transfer encoding, streaming large files | `armature-core` |
| ✅ | Request Extractors | Body, Query, Path, Header extractors | `armature-core` |

### Routing & Controllers

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | API Versioning | URL-based, header-based, and query-based versioning | `armature-core` |
| ✅ | Route Groups | Group routes with shared middleware/guards | `armature-core` |
| ✅ | Route Constraints | Parameter validation at route level | `armature-core` |
| ✅ | `#[use_middleware]` Decorator | Apply middleware via decorator syntax | `armature-macro` |
| ✅ | `#[use_guard]` Decorator | Apply guards via decorator syntax | `armature-macro` |
| ✅ | Path Parameters | `:id` style path parameters | `armature-core` |
| ✅ | Query Parameters | Query string parsing | `armature-core` |

### Error Handling

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Global Exception Filters | Centralized error transformation | `armature-core` |
| ✅ | Problem Details (RFC 7807) | Standardized error response format | `armature-core` |
| ✅ | Error Correlation | Tie errors to request IDs for debugging | `armature-core` |
| ✅ | HTTP Status Errors | Type-safe error responses | `armature-core` |

---

## 3. Observability & Operations

### Health Checks

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Health Check Module | `/health`, `/ready`, `/live` endpoints | `armature-core` |
| ✅ | Custom Health Indicators | Register custom health checks | `armature-core` |
| ✅ | Kubernetes Probes | K8s-compatible probe endpoints | `armature-core` |
| ✅ | OpenTelemetry | Distributed tracing and metrics | `armature-opentelemetry` |
| ✅ | Logging | Structured logging | `armature-core` |

### Metrics & Monitoring

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Prometheus Metrics | `/metrics` endpoint with custom metrics | `armature-metrics` |
| ✅ | Request Metrics | Request count, latency, error rates | `armature-metrics` |
| ✅ | Business Metrics | Custom metric registration | `armature-metrics` |
| 🟡 | Grafana Dashboards | Pre-built dashboard templates | `docs/` |

### Audit & Compliance

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Audit Logging | Track who did what, when | `armature-audit` |
| ✅ | Request/Response Logging | Configurable payload logging | `armature-audit` |
| ✅ | Data Masking | Mask sensitive data in logs | `armature-audit` |
| ✅ | Retention Policies | Auto-cleanup old audit logs | `armature-audit` |

---

## 4. Resilience & Reliability

### Circuit Breaker

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🔴 | Circuit Breaker | Prevent cascade failures | `armature-resilience` |
| 🟠 | Retry with Backoff | Configurable retry strategies | `armature-resilience` |
| 🟠 | Bulkhead Pattern | Resource isolation | `armature-resilience` |
| 🟠 | Timeout Policies | Timeout configuration per endpoint | `armature-resilience` |
| 🟡 | Fallback Handlers | Graceful degradation | `armature-resilience` |

### Graceful Shutdown

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Connection Draining | Wait for in-flight requests | `armature-core` |
| ✅ | Shutdown Hooks | Custom cleanup on shutdown | `armature-core` |
| ✅ | Health Status Update | Mark unhealthy during shutdown | `armature-core` |
| ✅ | Lifecycle Hooks | OnApplicationShutdown | `armature-core` |

---

## 5. API Features

### Pagination & Filtering

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Pagination Helpers | Offset, cursor-based pagination | `armature-core` |
| ✅ | Sorting Helpers | Multi-field sorting | `armature-core` |
| ✅ | Filtering Helpers | Query parameter filtering | `armature-core` |
| ✅ | Search Helpers | Full-text search integration | `armature-core` |
| ✅ | Field Selection | Sparse fieldsets (GraphQL-like) | `armature-core` |

### File Handling

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🔴 | Multipart Upload | File upload handling | `armature-storage` |
| 🔴 | File Validation | Type, size, extension validation | `armature-storage` |
| 🟠 | S3 Integration | AWS S3 file storage | `armature-storage` |
| 🟠 | GCS Integration | Google Cloud Storage | `armature-storage` |
| 🟠 | Azure Blob | Azure Blob Storage | `armature-storage` |
| 🟡 | Local Storage | Filesystem storage with paths | `armature-storage` |


## 6. Communication & Integration

### Email

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🔴 | SMTP Integration | Email sending via SMTP | `armature-mail` |
| 🟠 | Email Templates | HTML email with templates | `armature-mail` |
| 🟠 | SendGrid Integration | SendGrid API support | `armature-mail` |
| 🟠 | AWS SES Integration | AWS SES support | `armature-mail` |
| 🟡 | Mailgun Integration | Mailgun API support | `armature-mail` |
| 🟡 | Email Queue | Async email sending | `armature-mail` |

### Messaging

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | RabbitMQ Integration | RabbitMQ message broker | `armature-messaging` |
| ✅ | Kafka Integration | Apache Kafka support | `armature-messaging` |
| ✅ | NATS Integration | NATS messaging | `armature-messaging` |
| ✅ | AWS SQS/SNS | AWS messaging services | `armature-messaging` |
| ✅ | Job Queue | Redis-based job queue | `armature-queue` |

### External APIs

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟠 | HTTP Client | Built-in HTTP client with retry | `armature-http-client` |
| 🟡 | gRPC Support | gRPC server and client | `armature-grpc` |
| 🟡 | GraphQL Client | GraphQL client for federation | `armature-graphql-client` |

---

## 7. Security Enhancements

### Additional Auth

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | API Key Management | API key generation/rotation | `armature-auth` |
| ✅ | Two-Factor Auth (2FA) | TOTP/HOTP support | `armature-auth` |
| ✅ | Passwordless Auth | Magic links, WebAuthn | `armature-auth` |
| ✅ | Social Auth Extensions | More OAuth providers | `armature-auth` |
| ✅ | JWT Authentication | JWT token management | `armature-jwt` |
| ✅ | OAuth2/OIDC | Google, Microsoft, etc. | `armature-auth` |
| ✅ | SAML 2.0 | Enterprise SSO | `armature-auth` |

### Security Headers & Protection

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | CORS Improvements | More granular CORS control | `armature-security` |
| ✅ | CSP Headers | Content Security Policy | `armature-security` |
| ✅ | HSTS | HTTP Strict Transport Security | `armature-security` |
| ✅ | Request Signing | HMAC request verification | `armature-security` |
| ✅ | Security Headers | Basic security headers | `armature-security` |
| ✅ | Rate Limiting | Token bucket, sliding window | `armature-ratelimit` |

## 8. Multi-tenancy & Enterprise

### Multi-tenancy

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Tenant Isolation | Request-scoped tenant context | `armature-tenancy` |
| ✅ | Database per Tenant | Separate database connections | `armature-tenancy` |
| ✅ | Schema per Tenant | PostgreSQL schema isolation | `armature-tenancy` |
| ✅ | Tenant Middleware | Auto tenant resolution | `armature-tenancy` |
| ✅ | Tenant-aware Caching | Cache key prefixing | `armature-tenancy` |

### Feature Flags

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Feature Flags | Toggle features at runtime | `armature-features` |
| ✅ | LaunchDarkly Integration | LaunchDarkly support | `armature-features` |
| ✅ | A/B Testing | Experiment framework | `armature-features` |
| ✅ | Gradual Rollout | Percentage-based rollout | `armature-features` |

### Internationalization

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟠 | i18n Support | Message translation | `armature-i18n` |
| 🟠 | Locale Detection | Accept-Language parsing | `armature-i18n` |
| 🟡 | Pluralization | Plural rules support | `armature-i18n` |
| 🟡 | Date/Number Formatting | Locale-aware formatting | `armature-i18n` |

---

## 9. Developer Experience

### CLI Improvements

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | REPL | Interactive Rust REPL | `armature-cli` |
| ✅ | Route List | `armature routes` - list all routes | `armature-cli` |
| ✅ | Config Validation | `armature config:check` | `armature-cli` | `armature-cli` |
| ✅ | Code Generation | Controllers, services, modules | `armature-cli` |
| ✅ | Project Templates | Starter templates | `armature-cli` |
| ✅ | Dev Server | Hot reloading development | `armature-cli` |

### Documentation & Tooling

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | API Playground | Interactive API testing UI | `armature-openapi` | `armature-openapi` |
| ✅ | OpenAPI Generation | Swagger/OpenAPI docs | `armature-openapi` |

### Testing

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Integration Test Helpers | Database setup/teardown | `armature-testing` |
| ✅ | Test Containers | Docker-based testing | `armature-testing` |
| ✅ | Load Testing | Performance test utilities | `armature-testing` |
| ✅ | Contract Testing | Pact/consumer-driven contracts | `armature-testing` |
| ✅ | Unit Test Helpers | Mocks, spies, assertions | `armature-testing` |

---

## 10. Advanced Patterns

### Event-Driven Architecture

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Event Bus | In-process event publishing | `armature-events` |
| ✅ | Event Handlers | Decorator-based event handling | `armature-events` |
| ✅ | Event Sourcing | Event-sourced aggregates | `armature-eventsourcing` |
| ✅ | CQRS Support | Command/Query separation | `armature-cqrs` |

### Distributed Systems

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Distributed Locks | Redis-based distributed locks | `armature-distributed` |
| ✅ | Request Correlation | Correlation ID propagation | `armature-core` |
| ✅ | Leader Election | Distributed leader election | `armature-distributed` |
| ✅ | Service Discovery | Consul/etcd integration | `armature-discovery` |

### Caching Improvements

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Cache Decorators | `#[cache]` method decorator | `armature-cache` |
| ✅ | Cache Invalidation | Tag-based invalidation | `armature-cache` |
| ✅ | Multi-tier Caching | L1/L2 cache layers | `armature-cache` |
| ✅ | Redis Cache | Redis caching | `armature-cache` |
| ✅ | Memcached Cache | Memcached caching | `armature-cache` |

---

## 11. Deployment & Infrastructure

### Containerization

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟠 | Dockerfile Templates | Optimized Dockerfiles | `templates/` |
| 🟠 | Docker Compose | Development environment | `templates/` |
| 🟡 | Kubernetes Manifests | K8s deployment templates | `templates/` |
| 🟡 | Helm Charts | Helm chart templates | `templates/` |

### CI/CD

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟠 | GitHub Actions | CI/CD workflow templates | `.github/` |
| 🟡 | Jenkins | Jenkinsfile templates | `templates/` |

### Cloud Providers

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟡 | AWS Lambda | Serverless deployment | `armature-lambda` |
| 🟡 | Google Cloud Run | GCR deployment | `armature-cloudrun` |
| 🟡 | Azure Functions | Azure serverless | `armature-azure-functions` |

---

## Implementation Priority Order

### Phase 1: Production Essentials (Q1)
1. ✅ Health Check Module
2. ✅ Request Timeout & Size Limits
3. 🔴 Global Exception Filters
4. 🔴 Pagination Helpers
5. 🔴 Circuit Breaker
6. 🔴 Connection Draining (Graceful Shutdown)
7. 🔴 Multipart Upload
8. 🔴 SMTP Integration

### Phase 2: Enterprise Features (Q2)
3. 🟠 Prometheus Metrics
4. ✅ API Versioning
5. 🟠 Audit Logging
6. 🟠 Multi-tenancy
7. 🟠 Feature Flags
8. 🟠 i18n Support

### Phase 3: Advanced Capabilities (Q3)
1. 🟠 RabbitMQ/Kafka Integration
2. 🟠 Event Bus
3. 🟠 S3/GCS Storage
5. 🟠 HTTP Client with Retry
6. 🟠 Distributed Locks
7. ✅ Request Correlation

### Phase 4: Developer Experience (Q4)
1. 🟡 Admin Dashboard
2. 🟡 VS Code Extension
3. 🟡 Test Containers
4. 🟡 gRPC Support
5. 🟡 Push Notifications
6. 🟡 Advanced Caching

---

## Contributing

We welcome contributions! Each feature should:

1. Have comprehensive documentation in `docs/`
2. Include working examples in `examples/`
3. Have full test coverage
4. Follow existing code patterns
5. Update the README and website

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## Already Completed ✅

For reference, these features are already implemented:

- **Core**: DI, Controllers, Modules, Routing, Route Groups, Route Constraints, Middleware, Guards, Interceptors, API Versioning, Content Negotiation, ETags/Conditional Requests, Streaming Responses, Response Caching Headers, Error Correlation, Request Correlation, Health Checks (liveness/readiness/full), Request Timeout (`#[timeout]` decorator), Request Size Limits (`#[body_limit]` decorator), **Graceful Shutdown (Connection Draining, Shutdown Hooks, Health Status Integration)**, **Pagination & Filtering (Offset/Cursor Pagination, Multi-field Sorting, Query Filtering, Search Integration, Field Selection)**
- **Auth**: JWT, OAuth2 (Google, Microsoft, Cognito, Okta, Auth0, GitHub, GitLab, Discord, LinkedIn), SAML 2.0, **API Key Management (with DI)**, **2FA (TOTP/HOTP)**, **Passwordless Auth (Magic Links, WebAuthn)**
- **Data**: Redis Cache, Memcached Cache, Session Storage
- **Background**: Job Queues, Cron Jobs
- **Messaging**: RabbitMQ, Kafka, NATS, AWS SQS/SNS (unified `armature-messaging` module)
- **API**: GraphQL, OpenAPI/Swagger, WebSocket, SSE, Webhooks
- **Security**: Rate Limiting, HTTPS/TLS, ACME Certificates, Security Headers, **Granular CORS**, **Content Security Policy (CSP)**, **HSTS (with preload)**, **Request Signing (HMAC-SHA256)**
- **Observability**: OpenTelemetry, Structured Logging, Prometheus Metrics (Counter, Gauge, Histogram), Request Metrics Middleware, Business Metrics
- **Audit & Compliance**: Audit Logging, Request/Response Logging, Data Masking (PII, passwords, credit cards), Retention Policies, Multiple Backends (File, Memory, Stdout)
- **Multi-Tenancy**: Tenant Isolation (request-scoped), **Database Per Tenant (with DI)**, **Schema Per Tenant (PostgreSQL)**, Tenant Middleware (auto resolution), Tenant-Aware Caching (key prefixing)
- **Feature Flags**: Runtime Feature Toggling, **Targeting Rules**, **A/B Testing (multivariate)**, **Gradual Rollout (percentage-based)**, LaunchDarkly Integration (optional)
- **DX**: CLI, Code Generation, Project Templates, Compression, `#[use_middleware]`, `#[use_guard]` decorators, **REPL (Interactive Rust)**, **Route Listing**, **Config Validation**, **API Playground (Interactive Testing UI)**
- **Testing**: Test Utilities, Validation Framework, **Integration Test Helpers (Database Setup/Teardown)**, **Docker Test Containers (Postgres, Redis, MongoDB)**, **Load Testing (Request/Duration-Based, Stress Testing)**, **Contract Testing (Pact-Compatible)**
- **Event-Driven**: **Event Bus (In-Process Publishing/Subscribing)**, **Event Handlers (Type-Safe, Async)**, **Event Sourcing (Aggregates, Event Store, Snapshots)**, **CQRS (Command/Query Separation, Projections)**
- **Distributed Systems**: **Distributed Locks (Redis-Based, TTL, RAII)**, **Leader Election (Automatic, Callbacks, TTL Renewal)**, **Service Discovery (Consul, etcd, Load Balancing)**
- **Caching Improvements**: **Cache Decorators (#[cache] Attribute)**, **Tag-Based Invalidation (Bulk Cache Busting)**, **Multi-Tier Caching (L1/L2 Layers, Auto-Promotion, Write-Through)**

