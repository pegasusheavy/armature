# Armature Benchmark Suite

Comprehensive performance benchmarks for all major components of the Armature framework.

## Overview

The benchmark suite measures performance across four categories:

1. **Core Benchmarks** - HTTP, routing, middleware, status codes
2. **Security Benchmarks** - JWT, CSRF, XSS protection
3. **Validation Benchmarks** - Form validation, email, URL, patterns
4. **Data Benchmarks** - Queue jobs, cron expressions, caching

## Running Benchmarks

### Run All Benchmarks

```bash
cargo bench
```

### Run Specific Benchmark Suite

```bash
# Core HTTP and routing
cargo bench --bench core_benchmarks

# Security (JWT, CSRF, XSS)
cargo bench --bench security_benchmarks

# Validation
cargo bench --bench validation_benchmarks

# Data processing (queue, cron)
cargo bench --bench data_benchmarks
```

### Run Specific Benchmark

```bash
# Run only JWT benchmarks
cargo bench --bench security_benchmarks jwt

# Run only email validation
cargo bench --bench validation_benchmarks email_validation

# Run only HTTP request creation
cargo bench --bench core_benchmarks http_request_new
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
- **Rate Limiting** - Rate limit checks

### Security Benchmarks (`security_benchmarks.rs`)

#### JWT Operations
- Token signing (HS256, HS384, HS512)
- Token verification
- Algorithm comparison

#### CSRF Operations
- Token generation
- Token encoding/decoding
- Secret generation

#### XSS Protection
- HTML sanitization (strict, default, permissive)
- HTML/JS/URL encoding
- Attack detection and validation
- Different payload sizes

### Validation Benchmarks (`validation_benchmarks.rs`)

- **Email Validation** - Valid and invalid emails
- **URL Validation** - Various URL formats
- **String Validators**
  - MinLength, MaxLength
  - IsAlpha, IsAlphanumeric, IsNumeric
  - Contains, StartsWith, EndsWith
- **Numeric Validators** - Min, Max, InRange, IsPositive
- **Pattern Matching** - Regex validation (phone, UUID)
- **Validation Rules** - Chained validation rules

### Data Benchmarks (`data_benchmarks.rs`)

- **Queue Jobs**
  - Job creation (new vs builder)
  - Serialization/deserialization
  - Priority scoring
- **Cron Expressions**
  - Parsing various expressions
  - Preset parsing
  - Next execution calculation

## Performance Targets

### Target Latencies (p50)

| Operation | Target | Notes |
|-----------|--------|-------|
| HTTP Request Creation | < 100ns | Minimal allocation |
| JSON Parsing (small) | < 1μs | Typical API payload |
| JWT Sign | < 10μs | HS256 algorithm |
| JWT Verify | < 20μs | Includes signature check |
| CSRF Token Gen | < 1μs | Random generation |
| XSS Sanitize | < 10μs | Simple HTML |
| Email Validation | < 500ns | Regex check |
| Middleware Chain (10) | < 5μs | Typical stack |

### Throughput Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| HTTP Requests | > 100K/s | Single core |
| JWT Operations | > 50K/s | Sign + verify |
| Validations | > 1M/s | Simple validators |
| Form Parsing | > 100K/s | Typical form |

## Interpreting Results

### Key Metrics

- **Mean** - Average time per operation
- **Std Dev** - Consistency of performance
- **Median** - 50th percentile (p50)
- **Outliers** - Operations outside normal range

### Performance Regression

Criterion automatically detects:
- ✅ **Improvement** - Green, faster than baseline
- ⚠️ **Regression** - Yellow/Red, slower than baseline
- 📊 **No change** - Within noise threshold

### Comparing Results

```bash
# Run baseline
git checkout main
cargo bench

# Test changes
git checkout feature-branch
cargo bench

# Criterion automatically compares with baseline
```

## Continuous Benchmarking

The GitHub Actions workflow (`.github/workflows/benchmark.yml`) runs benchmarks:
- On pull requests (to detect regressions)
- On main branch (to track historical performance)
- Results are saved as artifacts

## Adding New Benchmarks

### 1. Create Benchmark File

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| {
            // Code to benchmark
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

## Performance Tips

### HTTP Processing
- Pool allocations for headers
- Reuse buffers for body parsing
- Avoid unnecessary clones

### Security
- Cache compiled regexes
- Use constant-time comparisons for secrets
- Prefer native crypto over pure Rust when possible

### Validation
- Compile regexes once, use many times
- Short-circuit validation on first error
- Use type system to avoid runtime validation

### Data Processing
- Batch operations when possible
- Use async I/O for network operations
- Consider connection pooling

## Troubleshooting

### Benchmarks Won't Run

```bash
# Clean and rebuild
cargo clean
cargo bench
```

### Inconsistent Results

- Close other applications
- Disable CPU scaling: `sudo cpupower frequency-set --governor performance`
- Run multiple iterations: `cargo bench -- --sample-size 1000`

### Missing Dependencies

```bash
# Install required dependencies
cargo fetch
```

## Resources

- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Benchmarking Best Practices](https://easyperf.net/blog/)

## Summary

**Quick Commands:**

```bash
# Run all benchmarks
cargo bench

# Run and save baseline
cargo bench --save-baseline main

# Compare with baseline
cargo bench --baseline main

# Generate HTML report
cargo bench && open target/criterion/report/index.html
```

**Performance Expectations:**
- Sub-microsecond for core operations
- Sub-10μs for security operations
- Sub-microsecond for validation
- Minimal allocations across the board

**Continuous Improvement:**
- Run benchmarks before every release
- Track performance over time
- Investigate any regressions
- Document performance characteristics


