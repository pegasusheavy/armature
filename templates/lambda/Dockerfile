# =============================================================================
# Armature AWS Lambda - Container Image Deployment
# =============================================================================
# Features:
# - AWS Lambda container image support
# - Multi-stage build for minimal size
# - cargo-lambda for optimal Lambda builds
# - x86_64 and ARM64 (Graviton2) support
# - Lambda Runtime Interface Client (RIC) included
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Builder - Build with cargo-lambda
# -----------------------------------------------------------------------------
# Base images are pinned by exact tag AND digest (Rust 1.99.1 >= the 1.94.1
# MSRV on Alpine 3.24; the Lambda provided:al2023 runtime). The Lambda base
# image is published under a floating tag, so the digest is what makes the
# build reproducible; Dependabot's "docker" ecosystem proposes bumps.
FROM rust:1.99.1-alpine3.24@sha256:557a94c82b485a14b569120e00867267108c734efe33f10cc31ca0cd5dd1eca9 AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    curl \
    gcc

# Install cargo-lambda: exact release, verified against a hardcoded SHA256.
# The build FAILS if the download does not match -- there is no "skip the
# check" path. To upgrade, change both values together, taking the hash from
# the release's own .sha256 asset and re-computing it yourself, e.g.
#   gh release download v1.9.2 -R cargo-lambda/cargo-lambda \
#     -p 'cargo-lambda-v1.9.2.x86_64-unknown-linux-musl.tar.gz*'
#   sha256sum cargo-lambda-v1.9.2.x86_64-unknown-linux-musl.tar.gz
ARG CARGO_LAMBDA_VERSION=1.9.2
ARG CARGO_LAMBDA_SHA256=b5c258c424ebaef39541c3641f33c51137ee6c426016f365628fedb2c737271d
RUN set -eu \
    && curl --proto '=https' --tlsv1.2 -fsSL -o cargo-lambda.tar.gz \
         "https://github.com/cargo-lambda/cargo-lambda/releases/download/v${CARGO_LAMBDA_VERSION}/cargo-lambda-v${CARGO_LAMBDA_VERSION}.x86_64-unknown-linux-musl.tar.gz" \
    && echo "${CARGO_LAMBDA_SHA256}  cargo-lambda.tar.gz" | sha256sum -c - \
    && tar xzf cargo-lambda.tar.gz -C /usr/local/bin cargo-lambda \
    && rm cargo-lambda.tar.gz \
    && cargo lambda --version

# Copy dependency files first (for caching)
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
# Intentionally allowed to fail: this dummy build only exists to cache
# dependency compilation in a Docker layer; a failure here just means no
# cache was warmed, not a real problem.
RUN cargo lambda build --release --target x86_64-unknown-linux-musl || true

# Copy source and build
COPY src ./src
RUN cargo lambda build --release --target x86_64-unknown-linux-musl

# -----------------------------------------------------------------------------
# Stage 2: Runtime - AWS Lambda base image
# -----------------------------------------------------------------------------
FROM public.ecr.aws/lambda/provided:al2023@sha256:f42e771ae36a9d34ab2e594ef5860fff8ace80aff5ad7c75fca28c06cbc7c6d5 AS runtime

# Note: no non-root USER is set here, unlike the Cloud Run Dockerfile.
# The AWS Lambda "provided" base image's bootstrap/init process and Lambda
# Runtime Interface Client are designed to run as root inside the managed
# Lambda execution environment; switching to a non-root user is not a
# supported/documented configuration for this base image and risks breaking
# the runtime's ability to start. Isolation here is instead provided by
# AWS Lambda's own Firecracker microVM sandboxing, not by an in-container
# non-root user.

# Copy the bootstrap binary
COPY --from=builder /app/target/lambda/*/bootstrap ${LAMBDA_RUNTIME_DIR}/bootstrap

# Set the handler
CMD ["bootstrap"]

# =============================================================================
# Alternative: ARM64 (Graviton2) Build
# =============================================================================
# To build for ARM64, use:
#   docker buildx build --platform linux/arm64 -f Dockerfile.arm64 .
#
# Or use cargo-lambda with:
#   cargo lambda build --release --arm64
# =============================================================================

# =============================================================================
# Deployment Instructions
# =============================================================================
# 1. Build the image:
#    docker build -t my-lambda-app -f Dockerfile .
#
# 2. Tag for ECR:
#    docker tag my-lambda-app:latest 123456789.dkr.ecr.us-east-1.amazonaws.com/my-lambda-app:latest
#
# 3. Push to ECR:
#    aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin 123456789.dkr.ecr.us-east-1.amazonaws.com
#    docker push 123456789.dkr.ecr.us-east-1.amazonaws.com/my-lambda-app:latest
#
# 4. Create/Update Lambda function:
#    aws lambda create-function \
#      --function-name my-function \
#      --package-type Image \
#      --code ImageUri=123456789.dkr.ecr.us-east-1.amazonaws.com/my-lambda-app:latest \
#      --role arn:aws:iam::123456789:role/lambda-role
# =============================================================================

