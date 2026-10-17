# Docker Guide

This guide covers containerizing Armature applications with Docker for consistent, portable deployments.

## Table of Contents

- [Overview](#overview)
- [Basic Dockerfile](#basic-dockerfile)
- [Multi-Stage Build](#multi-stage-build)
- [Docker Compose](#docker-compose)
- [Best Practices](#best-practices)
- [Common Patterns](#common-patterns)

## Overview

Docker provides consistent deployment environments for Armature applications. Benefits include:

- **Consistent environments** across development, staging, and production
- **Easy scaling** with container orchestration
- **Isolation** from host system
- **Reproducible builds** with multi-stage Dockerfiles

## Basic Dockerfile

```dockerfile
FROM rust:1.99.1-slim-trixie AS builder

WORKDIR /app
COPY . .

RUN cargo build --release --locked

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --system --gid 10001 app \
    && useradd --system --uid 10001 --gid app --no-create-home --shell /usr/sbin/nologin app

COPY --from=builder /app/target/release/my-api /usr/local/bin/

USER 10001:10001
EXPOSE 3000

CMD ["my-api"]
```

Armature is rustls-only, so the runtime image needs `ca-certificates` but no
OpenSSL. Any Rust image at or above the 1.94.1 MSRV works.

## Multi-Stage Build

Optimized Dockerfile with caching for faster builds:

This mirrors the Dockerfile `armature new` generates. Pin base images by tag
**and** digest (`FROM rust:1.99.1-slim-trixie@sha256:...`); get the digest with
`docker buildx imagetools inspect <image>:<tag>` and let Dependabot's `docker`
ecosystem propose bumps. Never use `latest`.

```dockerfile
# Stage 1: Build dependencies
FROM rust:1.99.1-slim-trixie AS builder

WORKDIR /app

# Create a dummy project to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
# --locked: never resolve new dependency versions inside an image build
RUN cargo build --release --locked && rm -rf src target/release/my-api*

# Stage 2: Build application
COPY src ./src
RUN cargo build --release --locked

# Stage 3: Runtime
FROM debian:trixie-slim

WORKDIR /app

# TLS is rustls-based: no OpenSSL needed
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Unprivileged user with a fixed numeric UID so orchestrators can enforce
# runAsNonRoot
RUN groupadd --system --gid 10001 app \
    && useradd --system --uid 10001 --gid app --no-create-home --shell /usr/sbin/nologin app

# The binary stays root-owned and read-only for the app user
COPY --from=builder /app/target/release/my-api /app/my-api

USER 10001:10001

EXPOSE 3000

# Configure health checks in your orchestrator (Kubernetes probes, Compose
# `healthcheck`) against /health; the slim runtime ships no curl.

CMD ["/app/my-api"]
```

## Docker Compose

### Development Setup

```yaml
version: '3.8'

services:
  app:
    build:
      context: .
      dockerfile: Dockerfile.dev
    ports:
      - "127.0.0.1:3000:3000"
    volumes:
      - ./src:/app/src
    environment:
      - RUST_LOG=debug
      - DATABASE_URL=postgres://app:${POSTGRES_PASSWORD:?set POSTGRES_PASSWORD in .env}@db:5432/app
      - REDIS_URL=redis://:${REDIS_PASSWORD:?set REDIS_PASSWORD in .env}@redis:6379
    depends_on:
      - db
      - redis

  db:
    image: postgres:15
    environment:
      POSTGRES_USER: app
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:?set POSTGRES_PASSWORD in .env}
      POSTGRES_DB: app
    ports:
      - "127.0.0.1:5432:5432"   # loopback only
    volumes:
      - pgdata:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    command: ["redis-server", "--requirepass", "${REDIS_PASSWORD:?set REDIS_PASSWORD in .env}"]
    ports:
      - "127.0.0.1:6379:6379"   # loopback only
    volumes:
      - redisdata:/data

volumes:
  pgdata:
  redisdata:
```

Passwords come from a git-ignored `.env` file; `${VAR:?message}` makes Compose
refuse to start when one is unset, so no default credential ever ships. This is
what `armature new` generates.

### Production Setup with Ferron

```yaml
version: '3.8'

services:
  ferron:
    image: ferronweb/ferron:<version>  # pin a version, never latest
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./ferron.conf:/etc/ferron/ferron.conf:ro
      - certs:/var/lib/ferron/certs
    depends_on:
      - app

  app:
    build:
      context: .
      dockerfile: Dockerfile
    expose:
      - "3000"
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgres://app:${POSTGRES_PASSWORD:?set POSTGRES_PASSWORD}@db:5432/app
      - REDIS_URL=redis://:${REDIS_PASSWORD:?set REDIS_PASSWORD}@redis:6379
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '1'
          memory: 512M
    depends_on:
      db:
        condition: service_healthy
      redis:
        condition: service_healthy

  db:
    image: postgres:15
    environment:
      POSTGRES_USER: app
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:?set POSTGRES_PASSWORD}
      POSTGRES_DB: app
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U app -d app"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    command: ["redis-server", "--requirepass", "${REDIS_PASSWORD:?set REDIS_PASSWORD}"]
    volumes:
      - redisdata:/data
    healthcheck:
      test: ["CMD-SHELL", "redis-cli -a \"$$REDIS_PASSWORD\" ping"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  certs:
  pgdata:
  redisdata:
```

## Best Practices

### 1. Use Multi-Stage Builds

Separate build and runtime stages for smaller images:

```dockerfile
# Build stage
FROM rust:1.99.1-slim-trixie AS builder
# ... build steps

# Runtime stage
FROM debian:trixie-slim
# ... only runtime files
```

### 2. Run as Non-Root User

```dockerfile
RUN groupadd --system --gid 10001 app \
    && useradd --system --uid 10001 --gid app --no-create-home --shell /usr/sbin/nologin app
USER 10001:10001
```

A numeric `USER` lets Kubernetes verify `runAsNonRoot` without resolving names.

### 3. Add Health Checks

Slim runtime images ship no `curl`. Configure probes in the orchestrator
(Kubernetes `livenessProbe`/`readinessProbe`, Compose `healthcheck`) against
`/health`, or install a minimal client in the image if you need a Docker
`HEALTHCHECK`.

### 4. Use .dockerignore

```dockerignore
target/
.git/
.env
.env.*
*.pem
*.key
*.md
tests/
docs/
```

### 5. Set Resource Limits

```yaml
deploy:
  resources:
    limits:
      cpus: '1'
      memory: 512M
    reservations:
      cpus: '0.25'
      memory: 256M
```

### 6. Use Slim Base Images

Prefer `debian:trixie-slim` (or distroless) over full images, pinned by digest.

### 7. Cache Dependencies

Copy `Cargo.toml` and `Cargo.lock` first, then build deps before copying source.

## Common Patterns

### Development Hot Reload

```dockerfile
# Dockerfile.dev
FROM rust:1.99.1-slim-trixie

RUN cargo install cargo-watch --locked

WORKDIR /app
COPY Cargo.toml Cargo.lock ./

CMD ["cargo", "watch", "-x", "run"]
```

`cargo watch` rebuilds the server. Armature's browser hot-module-reload
(`armature_core::hmr`) is a separate, development-only feature: its WebSocket
server binds `127.0.0.1` inside the container, requires the per-process token
embedded in the injected client script, accepts only loopback `Origin`s (add
others with `HmrConfig::allow_origin`), and refuses to start in release builds
unless `HmrConfig::allow_in_release(true)`. Never ship an image with it enabled.
See the [Debug Logging guide](debug-logging-guide.md#hot-module-reload-hmr).

### With Static Assets

```dockerfile
# Build frontend
FROM node:20 AS frontend
WORKDIR /web
COPY web/package*.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

# Build backend
FROM rust:1.99.1-slim-trixie AS backend
WORKDIR /app
COPY . .
RUN cargo build --release --locked

# Runtime
FROM debian:trixie-slim
RUN groupadd --system --gid 10001 app \
    && useradd --system --uid 10001 --gid app --no-create-home --shell /usr/sbin/nologin app
COPY --from=backend /app/target/release/my-api /app/
COPY --from=frontend /web/dist /app/static/
USER 10001:10001
CMD ["/app/my-api"]
```

### With Ferron Sidecar

```dockerfile
# ferron.Dockerfile
# Pin a version (and digest), never latest
FROM ferronweb/ferron:<version>
COPY ferron.conf /etc/ferron/ferron.conf
CMD ["ferron", "-c", "/etc/ferron/ferron.conf"]
```

## Summary

- Use **multi-stage builds** for smaller images
- Run as **non-root user** for security
- Add **health checks** for orchestration
- Use **Docker Compose** for local development
- Set **resource limits** in production
- Separate **development and production** Dockerfiles

