# Deployment Guide

This guide covers deploying Armature applications to production environments.

## Table of Contents

- [Overview](#overview)
- [Deployment Options](#deployment-options)
- [Environment Configuration](#environment-configuration)
- [Reverse Proxy Setup](#reverse-proxy-setup)
- [Containerization](#containerization)
- [Health Checks](#health-checks)
- [Monitoring](#monitoring)
- [Scaling](#scaling)
- [Security Checklist](#security-checklist)
- [Best Practices](#best-practices)

## Overview

Armature applications are compiled to native binaries, providing excellent performance and simple deployment. The typical production setup includes:

1. **Application Binary** - Your compiled Armature application
2. **Reverse Proxy** - Ferron, Nginx, or Caddy for TLS termination and load balancing
3. **Database** - PostgreSQL, MySQL, or other database
4. **Cache** - Redis for caching and sessions
5. **Monitoring** - Prometheus, Grafana, or cloud monitoring

## Deployment Options

### Bare Metal / VM

```bash
# Build release binary
cargo build --release

# Copy to server
scp target/release/my-api user@server:/opt/my-api/

# Run with systemd
sudo systemctl enable my-api
sudo systemctl start my-api
```

### Docker

See the [Docker Guide](docker-guide.md) for detailed containerization instructions.

```bash
docker build -t my-api .
docker run -p 3000:3000 my-api
```

### Kubernetes

See the [Kubernetes Guide](kubernetes-guide.md) for orchestration details.

```bash
kubectl apply -f k8s/
```

### Serverless

Armature supports serverless deployment to:
- **AWS Lambda** with `armature-lambda`
- **Google Cloud Run** with `armature-cloudrun`
- **Azure Functions** with `armature-azure-functions`

## Environment Configuration

### Required Environment Variables

```bash
# Application
PORT=3000
HOST=0.0.0.0
RUST_LOG=info

# Database
DATABASE_URL=postgres://user:pass@localhost/mydb

# Redis (optional)
REDIS_URL=redis://localhost:6379

# Security: at least 32 random bytes; JwtConfig::new rejects shorter,
# placeholder or low-entropy secrets. Generate with: openssl rand -base64 48
JWT_SECRET=
```

Never commit real values. Load secrets from your platform's secret store.

### Configuration Files

```rust
// Requires the `config` feature of `armature`.
use armature::armature_config::{ConfigService, FileFormat};

// `with_prefix` must come first: it resets the builder's config store.
// Sources load in this order, later ones overriding earlier ones:
// `.env`, then APP_* environment variables, then the files.
let config = ConfigService::builder()
    .with_prefix("APP".to_string())
    .load_env()
    .add_file("config/production.toml".to_string(), FileFormat::Toml)
    .build()?;
```

## Reverse Proxy Setup

### Using Ferron (Recommended)

See the [Ferron Guide](ferron-guide.md) for detailed integration.

```rust
use armature_ferron::{FerronConfig, Location, RateLimitConfig};

let config = FerronConfig::builder()
    .domain("api.example.com")
    .backend_url("http://localhost:3000")
    .tls_auto(true)
    .location(
        Location::new("/api")
            .proxy("http://localhost:3000/api")
            .rate_limit(RateLimitConfig::new(100))
    )
    .build()?;
```

### Using Nginx

```nginx
server {
    listen 443 ssl http2;
    server_name api.example.com;

    ssl_certificate /etc/letsencrypt/live/api.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.example.com/privkey.pem;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Bind the application to loopback (`app.listen_on(([127, 0, 0, 1], 3000))`) so
clients cannot bypass the proxy. With one proxy in front, read the client
address with `HttpRequest::client_address(1)`; middleware that identifies
clients (`armature-ratelimit`, `armature-audit`, `armature-analytics`) takes the
same `trusted_proxy_depth` setting, which defaults to `0` (socket peer only,
`X-Forwarded-For` ignored). `X-Real-IP` is never consulted.

### Using Caddy

```caddyfile
api.example.com {
    reverse_proxy localhost:3000

    header {
        X-Frame-Options "DENY"
        X-Content-Type-Options "nosniff"
    }
}
```

## Health Checks

### Implementing Health Endpoints

```rust
#[controller("")]
#[derive(Default, Clone)]
struct HealthController;

#[routes]
impl HealthController {
    #[get("/health")]
    async fn health() -> Result<HttpResponse, Error> {
        // Check dependencies
        HttpResponse::json(&serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    #[get("/ready")]
    async fn ready() -> Result<HttpResponse, Error> {
        // Check if ready to serve traffic
        HttpResponse::json(&serde_json::json!({ "ready": true }))
    }

    #[get("/live")]
    async fn live() -> Result<HttpResponse, Error> {
        HttpResponse::ok().with_body(b"OK".to_vec())
    }
}
```

## Monitoring

### Prometheus Metrics

```rust
// `armature-metrics` is a separate crate (not re-exported by `armature`).
use armature_metrics::{RequestMetricsMiddleware, create_metrics_handler};

// Record per-request metrics, and serve them in Prometheus text format.
let metrics_middleware = RequestMetricsMiddleware::new();
let metrics_handler = create_metrics_handler(); // mount at GET /metrics
```

### OpenTelemetry

```rust
// Requires the `opentelemetry` feature of `armature`.
use armature::armature_opentelemetry::TelemetryBuilder;

let telemetry = TelemetryBuilder::new("my-service")
    .with_tracing()
    .with_otlp_endpoint("http://otel-collector:4317")
    .build()
    .await?;
let telemetry_middleware = telemetry.middleware();
```

## Security Checklist

Before deploying to production:

- [ ] **TLS enabled** - All traffic encrypted. If Armature serves the HTTP→HTTPS
  redirect itself, set `Application::with_https_redirect_hosts`
- [ ] **Secrets in environment** - No hardcoded credentials; JWT/signing/CSRF
  secrets are at least 32 random bytes
- [ ] **Authentication middleware registered** - `JwtAuthMiddleware` added with
  `Application::use_middleware`, so guards see a verified identity (guards fail
  closed without one)
- [ ] **Proxy depth configured** - `trusted_proxy_depth` matches the number of
  proxies in front of the app
- [ ] **Developer tooling off** - GraphQL playground/introspection, HMR and
  debug endpoints disabled (they are off by default)
- [ ] **Rate limiting configured** - Prevent abuse
- [ ] **CORS properly set** - Allow only trusted origins
- [ ] **Security headers** - X-Frame-Options, CSP, etc.
- [ ] **Input validation** - Validate all user input
- [ ] **Authentication required** - Protect sensitive endpoints
- [ ] **Logging configured** - Structured JSON logging
- [ ] **Health checks working** - /health, /ready, /live
- [ ] **Graceful shutdown** - Handle SIGTERM properly

## Best Practices

### 1. Use Release Builds

```bash
cargo build --release
```

### 2. Enable Security Headers

Register `armature-security`'s `SecurityMiddleware` globally. `new()` already
sends `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`,
`Referrer-Policy: no-referrer`, `Cross-Origin-Opener-Policy: same-origin`,
`Cross-Origin-Resource-Policy: same-origin` and a restrictive
`Permissions-Policy`; add HSTS and CSP for your deployment. Headers a handler
sets itself are never overwritten.

```rust,ignore
use armature_security::{
    SecurityMiddleware, content_security_policy::CspConfig, hsts::HstsConfig,
};

let security = SecurityMiddleware::new()
    .with_hsts(HstsConfig::new(31_536_000).include_subdomains(true))
    .with_csp(CspConfig::default())?; // validated; returns Result

let app = app.use_middleware(security);
```

### 3. Configure Graceful Shutdown

See the [Graceful Shutdown guide](graceful-shutdown-guide.md). Make sure your
orchestrator's termination grace period is longer than the shutdown timeout.

### 4. Use Connection Pooling

```rust
let pool = PgPoolOptions::new()
    .max_connections(100)
    .min_connections(10)
    .connect(&database_url)
    .await?;
```

### 5. Set Resource Limits

```yaml
# Kubernetes
resources:
  limits:
    cpu: "2"
    memory: "2Gi"
  requests:
    cpu: "500m"
    memory: "512Mi"
```

## Summary

Key deployment considerations:

1. **Use Ferron** for reverse proxy with automatic TLS
2. **Configure health checks** for load balancer integration
3. **Enable metrics and tracing** for observability
4. **Follow security checklist** before going live
5. **Test graceful shutdown** to prevent dropped requests
6. **Use containers** for consistent deployments

