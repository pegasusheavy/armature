# Framework Comparison Benchmarks

This directory contains benchmark code for comparing Armature against other web frameworks.

## Frameworks Tested

| Framework | Language | Description |
|-----------|----------|-------------|
| **Armature** | Rust | Enterprise web framework with DI/modules |
| **Actix-web** | Rust | High-performance minimal framework |
| **Axum** | Rust | Ergonomic, modular framework from Tokio |
| **Express.js** | Node.js | Most popular Node.js web framework |

## Latest Results (2025-12-16)

| Framework | Requests/sec | Avg Latency | vs Express |
|-----------|-------------|-------------|------------|
| **Actix-web 4** | 589,565 | 0.17ms | 44x faster |
| **Axum 0.7** | 434,567 | 0.23ms | 32x faster |
| **Armature 0.1** | 421,323 | 0.23ms | **31x faster** |
| **Express.js** | 13,345 | 7.48ms | baseline |

See [results/benchmark_20251216.md](results/benchmark_20251216.md) for detailed analysis.

## Running Benchmarks

### Prerequisites

```bash
# Install oha (HTTP load generator)
cargo install oha

# For Express.js benchmarks
npm install express
```

### Build

```bash
cd benchmarks/comparison
cargo build --release
```

### Run Individual Benchmarks

```bash
# Armature
PORT=8080 ./target/release/armature_bench &
oha -c 100 -z 10s http://localhost:8080/json
pkill armature_bench

# Actix-web
PORT=8082 ./target/release/actix_bench &
oha -c 100 -z 10s http://localhost:8082/json
pkill actix_bench

# Axum
PORT=8083 ./target/release/axum_bench &
oha -c 100 -z 10s http://localhost:8083/json
pkill axum_bench

# Express.js
PORT=8081 node express_bench.js &
oha -c 100 -z 10s http://localhost:8081/json
pkill -f express_bench
```

### Run All Benchmarks

```bash
chmod +x run_benchmarks.sh
./run_benchmarks.sh [connections] [duration]

# Example: 200 connections for 30 seconds
./run_benchmarks.sh 200 30s
```

## Test Endpoints

All servers implement the same endpoints:

- `GET /json` - Returns `{"message":"Hello, World!"}`
- `GET /plaintext` - Returns `Hello, World!`

## Benchmark Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| Connections | 100 | Concurrent HTTP connections |
| Duration | 10s | Test duration |
| Warmup | 2s | Server warmup before benchmark |

## Files

```
benchmarks/comparison/
├── Cargo.toml              # Rust dependencies
├── README.md               # This file
├── armature_bench.rs       # Armature benchmark server
├── actix_bench.rs          # Actix-web benchmark server
├── axum_bench.rs           # Axum benchmark server
├── express_bench.js        # Express.js benchmark server
├── run_benchmarks.sh       # Automated benchmark runner
└── results/                # Benchmark results
    └── benchmark_*.md      # Timestamped results
```

## Notes

1. **Release Mode**: Always build with `--release` for accurate results
2. **System Load**: Run benchmarks on an idle system for consistent results
3. **Multiple Runs**: Run multiple times and take the median for accuracy
4. **Same Machine**: All frameworks must be benchmarked on the same machine

## Analysis

### Why Armature is ~3% slower than Axum

The overhead comes from:
- **DI Container**: Runtime service resolution
- **Module System**: Module lifecycle hooks
- **Route Registration**: Macro-based route collection

This is acceptable overhead for the architectural benefits.

### Why Armature is 31x faster than Express.js

- **Compiled vs Interpreted**: Rust is compiled to native code
- **No GC**: No garbage collection pauses
- **Async Runtime**: Tokio's work-stealing scheduler vs Node's event loop
- **Memory**: Lower memory footprint and better cache utilization

