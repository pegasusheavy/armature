# Armature Benchmark Suite

Comprehensive performance benchmarks for all major components of the Armature framework,
including comparisons with other popular Rust web frameworks.

## Overview

The benchmark suite measures performance across five categories:

1. **Core Benchmarks** - HTTP, routing, middleware, status codes
2. **Security Benchmarks** - JWT operations
3. **Validation Benchmarks** - Form validation, email, URL, patterns
4. **Data Benchmarks** - Queue jobs, cron expressions, caching
5. **Framework Comparison** - Comparison with Actix-web, Axum, Warp, Rocket

## Quick Start

```bash
# Run all benchmarks
cargo bench

# Run framework comparison benchmarks
cargo bench --bench framework_comparison

# Run HTTP benchmark server
cargo run --release --example benchmark_server

# Run comparison tool (requires oha or wrk)
cargo run --release --bin http-benchmark -- --framework armature
```

## Running Benchmarks

### Run All Benchmarks

```bash
cargo bench
```

### Run Specific Benchmark Suite

```bash
# Core HTTP and routing
cargo bench --bench core_benchmarks

# Security (JWT)
cargo bench --bench security_benchmarks

# Validation
cargo bench --bench validation_benchmarks

# Data processing (queue, cron)
cargo bench --bench data_benchmarks

# Framework comparison (micro-benchmarks)
cargo bench --bench framework_comparison
```

### Run Specific Benchmark

```bash
# Run only JWT benchmarks
cargo bench --bench security_benchmarks jwt

# Run only routing benchmarks
cargo bench --bench framework_comparison routing

# Run only JSON operations
cargo bench --bench framework_comparison json_operations
```

## Framework Comparison

### Micro-Benchmarks

The `framework_comparison` benchmark measures internal operations:

- **Request Creation** - Building HttpRequest objects
- **Response Creation** - Building HttpResponse with JSON
- **JSON Operations** - Serialize/deserialize performance
- **Routing** - Route matching with 10-500 routes
- **Middleware** - Middleware creation overhead
- **DI Resolution** - Dependency injection container performance
- **Handler Invocation** - Async handler execution

```bash
cargo bench --bench framework_comparison
```

### HTTP Benchmarks

For real HTTP performance, use the benchmark runner:

```bash
# Start Armature benchmark server
cargo run --release --example benchmark_server

# In another terminal, run benchmarks
cargo run --release --bin http-benchmark -- --framework armature

# Compare with other frameworks (start their servers first)
cargo run --release --bin http-benchmark -- --all
```

### Comparison Servers

Start comparison servers for each framework:

```bash
# Armature (port 3000)
cargo run --release --example benchmark_server

# Actix-web (port 3001)
cd benches/comparison_servers/actix_server && cargo run --release

# Axum (port 3002)
cd benches/comparison_servers/axum_server && cargo run --release

# Warp (port 3003)
cd benches/comparison_servers/warp_server && cargo run --release

# Rocket (port 3004)
cd benches/comparison_servers/rocket_server && cargo run --release

# Node.js Frameworks (for comparison)

# Express (port 3006)
cd benches/comparison_servers/express_server && npm install && npm start

# Koa (port 3007)
cd benches/comparison_servers/koa_server && npm install && npm start

# NestJS (port 3008)
cd benches/comparison_servers/nestjs_server && npm install && npm run benchmark

# Next.js (port 3005)
cd benches/comparison_servers/nextjs_api && npm install && npm run benchmark
```

### Benchmark with oha (Recommended)

```bash
# Install oha
cargo install oha

# Plaintext
oha -z 10s -c 50 http://localhost:3000/

# JSON
oha -z 10s -c 50 http://localhost:3000/json

# Path parameters
oha -z 10s -c 50 http://localhost:3000/users/123

# POST with body
oha -z 10s -c 50 -m POST -d '{"name":"test"}' -H "Content-Type: application/json" http://localhost:3000/api/users
```

### Benchmark with wrk

```bash
# Install wrk
# Ubuntu: apt install wrk
# macOS: brew install wrk

# Basic benchmark
wrk -t4 -c50 -d10s http://localhost:3000/

# With latency stats
wrk -t4 -c50 -d10s --latency http://localhost:3000/json
```

## Benchmark Results

Results are saved in `target/criterion/` with:
- HTML reports for visualization
- Statistical analysis (mean, std dev, outliers)
- Historical comparison (if run multiple times)
- Performance graphs

View HTML reports:

```bash
open target/criterion/report/index.html
```

## Benchmark Categories

### Core Benchmarks (`core_benchmarks.rs`)

- **HTTP Request Creation** - Creating HttpRequest instances
- **HTTP Response Creation** - ok(), with_json(), with_body()
- **JSON Parsing** - Deserializing request bodies
- **Form Parsing** - URL-encoded form data
- **Middleware Chain** - Processing with 1, 5, 10, 20 middleware
- **Routing** - Route matching with 100 routes
- **Status Codes** - Status code lookups and checks
- **Error Handling** - Error creation and status mapping

### Security Benchmarks (`security_benchmarks.rs`)

- Token signing (HS256, HS384, HS512)
- Token verification
- Algorithm comparison

### Validation Benchmarks (`validation_benchmarks.rs`)

- **Email Validation** - Valid and invalid emails
- **URL Validation** - Various URL formats
- **String Validators** - MinLength, MaxLength, IsAlpha, etc.
- **Numeric Validators** - Min, Max, InRange, IsPositive
- **Pattern Matching** - Regex validation

### Data Benchmarks (`data_benchmarks.rs`)

- **Queue Jobs** - Job creation, serialization
- **Cron Expressions** - Parsing, next execution

### Framework Comparison (`framework_comparison.rs`)

- **Request/Response** - Object creation overhead
- **JSON** - Serialization with small/medium/large payloads
- **Routing** - Route matching with 10/50/100/500 routes
- **DI** - Container operations, service resolution
- **Handlers** - Async handler invocation patterns
- **Full Cycle** - Complete request handling

## Performance Targets

### Target Latencies (p50)

| Operation | Target | Notes |
|-----------|--------|-------|
| HTTP Request Creation | < 100ns | Minimal allocation |
| JSON Parsing (small) | < 1μs | Typical API payload |
| JWT Sign | < 10μs | HS256 algorithm |
| JWT Verify | < 20μs | Includes signature check |
| Email Validation | < 500ns | Regex check |
| Route Match (100 routes) | < 1μs | Prefix tree |
| DI Resolution | < 50ns | DashMap lookup |

### Throughput Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| HTTP Requests (plaintext) | > 200K/s | Single core |
| HTTP Requests (JSON) | > 150K/s | With serialization |
| JWT Operations | > 50K/s | Sign + verify |
| Validations | > 1M/s | Simple validators |

## Expected HTTP Performance

Typical performance on modern hardware (varies by configuration):

### Rust Frameworks

| Framework | Plaintext (req/s) | JSON (req/s) | Relative |
|-----------|------------------|--------------|----------|
| Actix-web | 400K-600K | 300K-450K | 100% |
| Axum | 350K-500K | 280K-400K | ~85% |
| Warp | 300K-450K | 250K-350K | ~75% |
| Armature | 250K-400K | 200K-300K | ~65% |
| Rocket | 200K-350K | 150K-250K | ~55% |

### Node.js Frameworks (for comparison)

| Framework | Plaintext (req/s) | JSON (req/s) | Relative |
|-----------|------------------|--------------|----------|
| Express | 25K-50K | 20K-45K | ~8% |
| Koa | 30K-55K | 25K-50K | ~10% |
| NestJS | 20K-45K | 18K-40K | ~7% |
| Next.js | 15K-40K | 12K-35K | ~5% |

**Note:** Armature prioritizes developer experience, type safety, and features
(DI, validation, middleware, etc.) alongside raw performance.

**Rust vs Node.js:** Rust frameworks typically achieve 10-15x higher throughput than
Node.js frameworks. Node.js frameworks are included for real-world comparison when evaluating
Armature as a backend for JavaScript/TypeScript frontends.

See [Armature vs Next.js Benchmark Guide](../docs/guides/armature-vs-nextjs-benchmark.md) for detailed comparison.

## Interpreting Results

### Key Metrics

- **Mean** - Average time per operation
- **Std Dev** - Consistency of performance
- **Median** - 50th percentile (p50)
- **Outliers** - Operations outside normal range
- **Throughput** - Operations per second

### Performance Regression

Criterion automatically detects:
- ✅ **Improvement** - Green, faster than baseline
- ⚠️ **Regression** - Yellow/Red, slower than baseline
- 📊 **No change** - Within noise threshold

### Comparing Results

```bash
# Run baseline
git checkout main
cargo bench -- --save-baseline main

# Test changes
git checkout feature-branch
cargo bench -- --baseline main
```

## Adding New Benchmarks

### 1. Create Benchmark File

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| {
            my_function(black_box(input))
        })
    });
}

criterion_group!(benches, bench_my_feature);
criterion_main!(benches);
```

### 2. Add to `Cargo.toml`

```toml
[[bench]]
name = "my_benchmarks"
harness = false
```

### 3. Run

```bash
cargo bench --bench my_benchmarks
```

## Best Practices

### DO

✅ Use `black_box()` to prevent compiler optimizations
✅ Benchmark realistic workloads
✅ Measure multiple input sizes
✅ Run benchmarks on consistent hardware
✅ Check for regressions before merging
✅ Use `--release` for HTTP benchmarks

### DON'T

❌ Benchmark trivial operations
❌ Include setup in benchmark loop
❌ Run benchmarks with debug builds
❌ Compare results across different machines
❌ Ignore performance regressions

## Profiling

For detailed profiling:

```bash
# CPU profiling with flamegraph
cargo flamegraph --bench core_benchmarks

# Memory profiling
cargo bench --bench core_benchmarks -- --profile-time=10

# Cachegrind
valgrind --tool=cachegrind target/release/deps/core_benchmarks-*
```

## Troubleshooting

### Benchmarks Won't Run

```bash
cargo clean
cargo bench
```

### Inconsistent Results

- Close other applications
- Disable CPU scaling: `sudo cpupower frequency-set --governor performance`
- Run multiple iterations: `cargo bench -- --sample-size 1000`

### HTTP Benchmark Issues

- Ensure server is running: `curl http://localhost:3000/health`
- Check for port conflicts: `lsof -i :3000`
- Verify tool installation: `oha --version` or `wrk --version`

## Resources

- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [TechEmpower Benchmarks](https://www.techempower.com/benchmarks/)
- [oha - HTTP load generator](https://github.com/hatoo/oha)

## Summary

**Quick Commands:**

```bash
# Run all benchmarks
cargo bench

# Run framework comparison
cargo bench --bench framework_comparison

# HTTP benchmarks
cargo run --release --example benchmark_server
oha -z 10s -c 50 http://localhost:3000/

# Full comparison
cargo run --release --bin http-benchmark -- --all

# Generate HTML report
cargo bench && open target/criterion/report/index.html
```

**Performance Expectations:**
- Sub-microsecond for core operations
- Sub-10μs for security operations
- Competitive with other Rust frameworks
- Excellent developer experience trade-off
