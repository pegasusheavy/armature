# Framework Comparison Servers

This directory contains minimal server implementations for different web frameworks,
designed for fair performance comparisons. Includes both Rust frameworks and Next.js
for Rust-vs-Node.js API benchmarking.

## Overview

Each server implements identical endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` or `/api` | GET | Plaintext "Hello, World!" |
| `/json` or `/api/json` | GET | JSON response |
| `/users/:id` or `/api/users/:id` | GET | Path parameter extraction |
| `/api/users` | POST | JSON body parsing |
| `/health` or `/api/health` | GET | Health check |
| `/data` or `/api/data` | GET | Complex nested JSON (supports `?size=small\|medium\|large\|xlarge`) |

## Port Assignments

| Framework | Port | Runtime |
|-----------|------|---------|
| Armature | 3000 | Rust/Tokio |
| Actix-web | 3001 | Rust/Tokio |
| Axum | 3002 | Rust/Tokio |
| Warp | 3003 | Rust/Tokio |
| Rocket | 3004 | Rust/Tokio |
| Next.js | 3005 | Node.js |
| Express | 3006 | Node.js |
| Koa | 3007 | Node.js |
| NestJS | 3008 | Node.js |

## Running the Servers

### Armature

```bash
cargo run --release --example benchmark_server
```

### Rust Frameworks

Each Rust framework has its own directory with a standalone Cargo project:

```bash
# Actix-web
cd benches/comparison_servers/actix_server && cargo run --release

# Axum
cd benches/comparison_servers/axum_server && cargo run --release

# Warp
cd benches/comparison_servers/warp_server && cargo run --release

# Rocket
cd benches/comparison_servers/rocket_server && cargo run --release
```

### Node.js Frameworks

All Node.js frameworks require Node.js (18+) and npm/pnpm:

```bash
# Express (port 3006)
cd benches/comparison_servers/express_server
npm install && npm start

# Koa (port 3007)
cd benches/comparison_servers/koa_server
npm install && npm start

# NestJS (port 3008)
cd benches/comparison_servers/nestjs_server
npm install && npm run benchmark

# Next.js (port 3005)
cd benches/comparison_servers/nextjs_api
npm install && npm run benchmark
```

## Running Benchmarks

### Using oha (Recommended)

```bash
# Install oha
cargo install oha

# Benchmark Armature
oha -z 10s -c 50 http://localhost:3000/
oha -z 10s -c 50 http://localhost:3000/json

# Compare with Actix-web
oha -z 10s -c 50 http://localhost:3001/
oha -z 10s -c 50 http://localhost:3001/json
```

### Using wrk

```bash
# Install wrk
# Ubuntu: apt install wrk
# macOS: brew install wrk

# Benchmark
wrk -t4 -c50 -d10s http://localhost:3000/
wrk -t4 -c50 -d10s http://localhost:3000/json
```

### Using the Benchmark Runner

```bash
# From the armature directory
cargo run --release --bin http-benchmark -- --all
```

## Methodology

For fair comparisons:

1. **Build Configuration**: All servers built with `--release`
2. **Warmup**: 2-second warmup before measurements
3. **Duration**: 10-second benchmark runs
4. **Connections**: 50 concurrent connections
5. **Consistent Responses**: Identical response bodies

## Expected Results

Performance varies based on:
- Hardware (CPU cores, memory)
- OS and kernel configuration
- Network stack tuning
- Connection keep-alive settings

Typical relative performance (varies by workload):

### Rust Frameworks

| Framework | Typical RPS Range | Notes |
|-----------|------------------|-------|
| Actix-web | 400K-600K | Highly optimized, actor model |
| Axum | 350K-500K | Tower-based, good ergonomics |
| Warp | 300K-450K | Filter-based, composable |
| Armature | 250K-400K | Full-featured, DI support |
| Rocket | 200K-350K | Macro-heavy, developer-friendly |

### Node.js Frameworks

| Framework | Typical RPS Range | Notes |
|-----------|------------------|-------|
| Express | 25K-50K | Minimal, battle-tested |
| Koa | 30K-55K | Lighter than Express, async/await native |
| NestJS | 20K-45K | Full-featured, Angular-inspired |
| Next.js | 15K-40K | Full-stack React framework |

Note: Node.js frameworks have lower raw throughput but offer different advantages:
- Familiar JavaScript/TypeScript ecosystem
- Excellent for full-stack applications
- Large community and ecosystem
- Faster development iteration

Note: These are rough estimates. Actual performance depends heavily on use case.

## Creating New Comparison Servers

To add a new framework:

1. Create a new directory: `benches/comparison_servers/{framework}_server/`
2. Add `Cargo.toml` with the framework dependency
3. Implement `src/main.rs` with the standard endpoints
4. Use the assigned port (next available: 3005+)
5. Update the framework list in `http_benchmark_runner.rs`

## Results Interpretation

When comparing frameworks, consider:

- **Raw Throughput**: Requests per second
- **Latency Percentiles**: p50, p90, p99
- **Memory Usage**: Check with `htop` or similar
- **Error Rate**: Non-2xx responses or timeouts
- **Developer Experience**: Code complexity, type safety
- **Feature Set**: DI, middleware, validation, etc.

Armature prioritizes developer experience and feature richness alongside performance.

