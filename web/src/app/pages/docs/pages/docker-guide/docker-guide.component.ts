import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-docker-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class DockerGuideComponent {
  page: DocPage = {
    title: 'Docker',
    subtitle: 'Containerize your Armature application with optimized multi-stage builds and best practices.',
    icon: '🐳',
    badge: 'Deployment',
    features: [
      { icon: '📦', title: 'Multi-Stage', description: 'Optimized build process' },
      { icon: '🏔️', title: 'Alpine-Based', description: 'Minimal ~5MB images' },
      { icon: '🔐', title: 'Secure', description: 'Non-root user, minimal attack surface' },
      { icon: '🚀', title: 'Fast Builds', description: 'Layer caching with cargo-chef' }
    ],
    sections: [
      {
        id: 'basic-dockerfile',
        title: 'Basic Dockerfile',
        content: `<p>A simple multi-stage Dockerfile for Armature:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            filename: 'Dockerfile',
            code: `# Build stage
FROM rust:1.75-alpine AS builder
WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev

# Copy source
COPY . .

# Build release binary
RUN cargo build --release

# Runtime stage
FROM alpine:3.19
WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Copy binary from builder
COPY --from=builder /app/target/release/my-app /app/my-app

# Create non-root user
RUN adduser -D -u 1000 appuser
USER appuser

EXPOSE 3000
CMD ["./my-app"]`
          }
        ]
      },
      {
        id: 'optimized-dockerfile',
        title: 'Optimized with Cargo Chef',
        content: `<p>Use cargo-chef for faster rebuilds by caching dependencies:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            filename: 'Dockerfile',
            code: `# Chef stage - prepare dependency cache
FROM rust:1.75-alpine AS chef
RUN apk add --no-cache musl-dev
RUN cargo install cargo-chef
WORKDIR /app

# Planner stage - analyze dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage - build with cached dependencies
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies (cached unless Cargo.toml changes)
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# Runtime stage
FROM alpine:3.19
WORKDIR /app
RUN apk add --no-cache ca-certificates
RUN adduser -D -u 1000 appuser

COPY --from=builder /app/target/release/my-app /app/my-app
RUN chown appuser:appuser /app/my-app

USER appuser
EXPOSE 3000
ENV RUST_LOG=info
CMD ["./my-app"]`
          }
        ]
      },
      {
        id: 'docker-compose',
        title: 'Docker Compose',
        content: `<p>Development environment with dependencies:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            filename: 'docker-compose.yml',
            code: `version: '3.8'

services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://user:pass@postgres:5432/mydb
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=debug
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_started

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: user
      POSTGRES_PASSWORD: pass
      POSTGRES_DB: mydb
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U user -d mydb"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:`
          }
        ]
      },
      {
        id: 'env-configuration',
        title: 'Environment Configuration',
        content: `<p>Pass configuration via environment variables:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            filename: '.env.docker',
            code: `# Application
APP_NAME=my-armature-app
APP_ENV=production
PORT=3000

# Database
DATABASE_URL=postgres://user:pass@postgres:5432/mydb
DATABASE_POOL_SIZE=10

# Redis
REDIS_URL=redis://redis:6379

# Security
JWT_SECRET=your-production-secret-here

# Logging
RUST_LOG=info`
          },
          {
            language: 'bash',
            code: `# Run with env file
$ docker run --env-file .env.docker my-app

# Or with docker-compose
$ docker-compose --env-file .env.docker up`
          }
        ]
      },
      {
        id: 'health-checks',
        title: 'Health Checks',
        content: `<p>Add health checks for container orchestration:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            filename: 'Dockerfile',
            code: `# ... build stages ...

FROM alpine:3.19
# ... setup ...

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \\
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

CMD ["./my-app"]`
          },
          {
            language: 'bash',
            filename: 'docker-compose.yml',
            code: `services:
  app:
    # ...
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s`
          }
        ]
      },
      {
        id: 'building-running',
        title: 'Building & Running',
        content: `<p>Common Docker commands:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `# Build image
$ docker build -t my-app:latest .

# Run container
$ docker run -d -p 3000:3000 --name my-app my-app:latest

# View logs
$ docker logs -f my-app

# Stop and remove
$ docker stop my-app && docker rm my-app

# Docker Compose
$ docker-compose up -d
$ docker-compose logs -f
$ docker-compose down`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Use multi-stage builds</strong> — Keep final image small</li>
          <li><strong>Run as non-root</strong> — Security best practice</li>
          <li><strong>Use specific tags</strong> — Don't use <code>:latest</code> in production</li>
          <li><strong>Layer caching</strong> — Order Dockerfile commands for efficient caching</li>
          <li><strong>Minimize layers</strong> — Combine RUN commands where sensible</li>
          <li><strong>Use .dockerignore</strong> — Exclude unnecessary files from build context</li>
          <li><strong>Set resource limits</strong> — Use <code>--memory</code> and <code>--cpus</code> flags</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'deployment-guide', title: 'Deployment', description: 'Production deployment strategies' },
      { id: 'kubernetes-guide', title: 'Kubernetes', description: 'Container orchestration' }
    ],
    seeAlso: [
      { title: 'Health Checks', id: 'health-check' },
      { title: 'Configuration', id: 'config-guide' }
    ]
  };
}

