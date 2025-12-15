# =============================================================================
# Armature Microservice - Optimized Multi-Stage Dockerfile (Alpine)
# =============================================================================
# Features:
# - Multi-stage build for minimal image size (~10-15MB)
# - Cargo chef for dependency caching
# - Alpine Linux for tiny footprint
# - Non-root user for security
# - Health checks built-in
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Chef - Prepare dependency recipe
# -----------------------------------------------------------------------------
FROM lukemathwalker/cargo-chef:latest-rust-1.85-alpine AS chef
WORKDIR /app

# -----------------------------------------------------------------------------
# Stage 2: Planner - Create dependency recipe
# -----------------------------------------------------------------------------
FROM chef AS planner
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# -----------------------------------------------------------------------------
# Stage 3: Builder - Build dependencies and application
# -----------------------------------------------------------------------------
FROM chef AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

# Build dependencies (cached layer)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Build application
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl --bin microservice

# -----------------------------------------------------------------------------
# Stage 4: Runtime - Minimal Alpine image
# -----------------------------------------------------------------------------
FROM alpine:3.20 AS runtime

WORKDIR /app

# Install runtime dependencies (curl for health checks)
RUN apk add --no-cache ca-certificates curl \
    && adduser -D -u 1000 appuser

# Copy binary from builder
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/microservice /app/service

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health/live || exit 1

# Run
CMD ["/app/service"]

