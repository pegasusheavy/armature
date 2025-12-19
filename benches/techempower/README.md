# TechEmpower Framework Benchmarks (TFB)

This directory contains Armature's implementation of the [TechEmpower Framework Benchmarks](https://www.techempower.com/benchmarks/).

## Benchmark Types

| Test | Description | Endpoint |
|------|-------------|----------|
| **JSON** | Serialize `{"message":"Hello, World!"}` | `GET /json` |
| **Plaintext** | Return "Hello, World!" | `GET /plaintext` |
| **DB** | Single database query | `GET /db` |
| **Queries** | Multiple database queries (1-500) | `GET /queries?queries=N` |
| **Fortunes** | Template rendering with DB data | `GET /fortunes` |
| **Updates** | Read-modify-write operations | `GET /updates?queries=N` |
| **Cached** | Cached database queries | `GET /cached-queries?queries=N` |

## Running Benchmarks

### Prerequisites

```bash
# Install wrk (HTTP benchmarking tool)
# Ubuntu/Debian
sudo apt-get install wrk

# macOS
brew install wrk

# Or use hey
go install github.com/rakyll/hey@latest
```

### Quick Start

```bash
# Start the benchmark server
cargo run --release --example techempower_server

# In another terminal, run benchmarks
./benches/techempower/run.sh
```

### Individual Tests

```bash
# JSON serialization
wrk -t4 -c256 -d15s http://127.0.0.1:8080/json

# Plaintext
wrk -t4 -c256 -d15s -H "Accept: text/plain" http://127.0.0.1:8080/plaintext

# Single DB query (requires PostgreSQL)
wrk -t4 -c256 -d15s http://127.0.0.1:8080/db

# Multiple queries
wrk -t4 -c256 -d15s http://127.0.0.1:8080/queries?queries=20
```

## Database Setup

For DB-related tests, you need PostgreSQL:

```bash
# Start PostgreSQL with Docker
docker run -d \
  --name tfb-postgres \
  -e POSTGRES_USER=benchmarkdbuser \
  -e POSTGRES_PASSWORD=benchmarkdbpass \
  -e POSTGRES_DB=hello_world \
  -p 5432:5432 \
  postgres:15

# Initialize the database
psql -h localhost -U benchmarkdbuser -d hello_world -f benches/techempower/create.sql
```

## Expected Results

Based on similar hardware (8-core, 32GB RAM):

| Test | Target RPS | Notes |
|------|------------|-------|
| JSON | 500k+ | Measures serialization speed |
| Plaintext | 1M+ | Raw framework overhead |
| DB | 100k+ | Single query latency |
| Queries (20) | 20k+ | Connection pool efficiency |
| Fortunes | 50k+ | Template rendering |

## Comparison with Other Frameworks

Run the comparison script to benchmark against Axum and Actix-web:

```bash
./benches/techempower/compare.sh
```

## TechEmpower Submission

To submit to the official TechEmpower benchmarks:

1. Fork the [TechEmpower/FrameworkBenchmarks](https://github.com/TechEmpower/FrameworkBenchmarks) repo
2. Copy our implementation to `frameworks/Rust/armature/`
3. Create the required `benchmark_config.json`
4. Submit a pull request

See [TFB Documentation](https://github.com/TechEmpower/FrameworkBenchmarks/wiki) for details.

