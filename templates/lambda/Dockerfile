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
FROM rust:1.85-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    curl \
    gcc

# Install cargo-lambda
RUN curl -sSL https://github.com/cargo-lambda/cargo-lambda/releases/latest/download/cargo-lambda-x86_64-unknown-linux-musl.tar.gz | tar xz -C /usr/local/bin

# Copy dependency files first (for caching)
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo lambda build --release --target x86_64-unknown-linux-musl || true

# Copy source and build
COPY src ./src
RUN cargo lambda build --release --target x86_64-unknown-linux-musl

# -----------------------------------------------------------------------------
# Stage 2: Runtime - AWS Lambda base image
# -----------------------------------------------------------------------------
FROM public.ecr.aws/lambda/provided:al2023 AS runtime

# Copy the bootstrap binary
COPY --from=builder /app/target/lambda/*/bootstrap ${LAMBDA_RUNTIME_DIR}/bootstrap

# Set the handler
CMD ["bootstrap"]

# =============================================================================
# Alternative: ARM64 (Graviton2) Build
# =============================================================================
# To build for ARM64, use:
#   docker build --platform linux/arm64 -f Dockerfile.lambda-arm64 .
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

