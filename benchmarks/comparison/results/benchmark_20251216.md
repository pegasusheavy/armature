# Framework Comparison Benchmarks - Complete Cross-Language Results

**Date:** 2025-12-16
**System:** Linux 5.15.167.4 (WSL2)
**Tool:** oha v1.12.1
**Parameters:** 100 concurrent connections, 10 second duration
**Endpoint:** GET /json returning `{"message":"Hello, World!"}`

**Versions:**
- Rust: 1.83
- Go: 1.24.2
- Node.js: 22.13.1
- .NET: 8.0.122

---

## Complete Results - All Frameworks Ranked

| Rank | Framework | Language | Requests/sec | Avg Latency | P99 Latency | vs NestJS |
|------|-----------|----------|-------------|-------------|-------------|-----------|
| 1 | **Actix-web 4** | Rust | 589,565 | 0.17ms | 1.04ms | 74x |
| 2 | **Axum 0.7** | Rust | 434,567 | 0.23ms | 0.77ms | 55x |
| 3 | **Armature 0.1** | Rust | 421,323 | 0.23ms | 0.75ms | **53x** |
| 4 | **Fiber v2** | Go | 320,189 | 0.31ms | 3.57ms | 40x |
| 5 | **ASP.NET Core 8** | C# | 306,217 | 0.32ms | 1.29ms | 38x |
| 6 | **Gin** | Go | 202,882 | 0.49ms | 3.65ms | 25x |
| 7 | **Fastify 4** | Node.js | 29,170 | 3.42ms | 8.06ms | 3.7x |
| 8 | **NestJS (Fastify)** | Node.js | 28,099 | 3.55ms | 8.25ms | 3.5x |
| 9 | **Express.js** | Node.js | 13,345 | 7.48ms | 13.41ms | 1.7x |
| 10 | **NestJS (Express)** | Node.js | 7,961 | 12.53ms | 25.06ms | baseline |

---

## By Language

### Rust Frameworks

| Framework | Requests/sec | Avg Latency | P99 Latency |
|-----------|-------------|-------------|-------------|
| Actix-web 4 | 589,565 | 0.17ms | 1.04ms |
| Axum 0.7 | 434,567 | 0.23ms | 0.77ms |
| **Armature 0.1** | 421,323 | 0.23ms | 0.75ms |

### Go Frameworks

| Framework | Requests/sec | Avg Latency | P99 Latency |
|-----------|-------------|-------------|-------------|
| Fiber v2 | 320,189 | 0.31ms | 3.57ms |
| Gin | 202,882 | 0.49ms | 3.65ms |

### C# Frameworks

| Framework | Requests/sec | Avg Latency | P99 Latency |
|-----------|-------------|-------------|-------------|
| ASP.NET Core 8 (Minimal) | 306,217 | 0.32ms | 1.29ms |

### Node.js Frameworks

| Framework | Requests/sec | Avg Latency | P99 Latency |
|-----------|-------------|-------------|-------------|
| Fastify 4 | 29,170 | 3.42ms | 8.06ms |
| NestJS (Fastify) | 28,099 | 3.55ms | 8.25ms |
| Express.js | 13,345 | 7.48ms | 13.41ms |
| NestJS (Express) | 7,961 | 12.53ms | 25.06ms |

---

## Detailed Results

### Armature (Rust)

```
Summary:
  Success rate:	100.00%
  Total:	10004.0309 ms
  Slowest:	11.3934 ms
  Fastest:	0.0111 ms
  Average:	0.2349 ms
  Requests/sec:	421323.5692

Response time distribution:
  50.00% in 0.2064 ms
  95.00% in 0.4539 ms
  99.00% in 0.7467 ms
  99.99% in 4.7146 ms

Status code distribution:
  [200] 4214850 responses
```

### Fiber (Go)

```
Summary:
  Success rate:	100.00%
  Total:	10005.2647 ms
  Slowest:	23.7856 ms
  Fastest:	0.0078 ms
  Average:	0.3100 ms
  Requests/sec:	320189.6283

Response time distribution:
  50.00% in 0.1215 ms
  95.00% in 1.0767 ms
  99.00% in 3.5715 ms
  99.99% in 18.8660 ms

Status code distribution:
  [200] 3203490 responses
```

### ASP.NET Core 8 (C#)

```
Summary:
  Success rate:	100.00%
  Total:	10007.1674 ms
  Slowest:	103.0407 ms
  Fastest:	0.0230 ms
  Average:	0.3239 ms
  Requests/sec:	306217.6201

Response time distribution:
  50.00% in 0.2607 ms
  95.00% in 0.7118 ms
  99.00% in 1.2938 ms
  99.99% in 7.8553 ms

Status code distribution:
  [200] 3064275 responses
```

### Gin (Go)

```
Summary:
  Success rate:	100.00%
  Total:	10004.9229 ms
  Slowest:	28.0379 ms
  Fastest:	0.0096 ms
  Average:	0.4900 ms
  Requests/sec:	202882.8228

Response time distribution:
  50.00% in 0.1626 ms
  95.00% in 2.1281 ms
  99.00% in 3.6484 ms
  99.99% in 10.6100 ms

Status code distribution:
  [200] 2029730 responses
```

### Fastify (Node.js)

```
Summary:
  Success rate:	100.00%
  Total:	10002.0256 ms
  Slowest:	331.5032 ms
  Fastest:	0.5644 ms
  Average:	3.4195 ms
  Requests/sec:	29170.2911

Response time distribution:
  50.00% in 2.8351 ms
  95.00% in 6.3325 ms
  99.00% in 8.0573 ms
  99.99% in 206.3342 ms

Status code distribution:
  [200] 291663 responses
```

### NestJS with Fastify Adapter (Node.js)

```
Summary:
  Success rate:	100.00%
  Total:	10002.9510 ms
  Slowest:	361.9140 ms
  Fastest:	0.4826 ms
  Average:	3.5465 ms
  Requests/sec:	28099.0080

Response time distribution:
  50.00% in 3.0120 ms
  95.00% in 6.5396 ms
  99.00% in 8.2483 ms
  99.99% in 205.3112 ms

Status code distribution:
  [200] 280973 responses
```

### Express.js (Node.js)

```
Summary:
  Success rate:	100.00%
  Total:	10002.6840 ms
  Slowest:	579.4127 ms
  Fastest:	0.3640 ms
  Average:	7.4829 ms
  Requests/sec:	13345.3182

Response time distribution:
  50.00% in 6.9422 ms
  95.00% in 10.8167 ms
  99.00% in 13.4080 ms
  99.99% in 471.8649 ms

Status code distribution:
  [200] 133389 responses
```

### NestJS with Express Adapter (Node.js)

```
Summary:
  Success rate:	100.00%
  Total:	10003.0971 ms
  Slowest:	958.0108 ms
  Fastest:	5.9844 ms
  Average:	12.5276 ms
  Requests/sec:	7961.3343

Response time distribution:
  50.00% in 11.7185 ms
  95.00% in 19.1021 ms
  99.00% in 25.0576 ms
  99.99% in 865.0715 ms

Status code distribution:
  [200] 79538 responses
```

---

## Analysis

### Armature vs Go Frameworks

| Comparison | Armature Advantage |
|------------|-------------------|
| vs Fiber | **1.32x faster** (421K vs 320K req/sec) |
| vs Gin | **2.08x faster** (421K vs 203K req/sec) |

**Key insight:** Armature with full enterprise features (DI, modules, guards) beats Go's fastest framework (Fiber) by 32%. This is significant because Go is often chosen for performance.

### Armature vs ASP.NET Core

| Comparison | Armature Advantage |
|------------|-------------------|
| vs ASP.NET Core 8 | **1.38x faster** (421K vs 306K req/sec) |
| Latency | **28% lower** (0.23ms vs 0.32ms) |

**Key insight:** ASP.NET Core 8 with Minimal APIs is already one of the fastest mainstream frameworks, and Armature beats it by 38%.

### Armature vs Node.js Ecosystem

| Comparison | Armature Advantage |
|------------|-------------------|
| vs NestJS (Express) | **53x faster** |
| vs NestJS (Fastify) | **15x faster** |
| vs Fastify (bare) | **14x faster** |
| vs Express (bare) | **31x faster** |

**Key insight:** The most relevant comparison is Armature vs NestJS since both provide the same architectural patterns. Armature is 53x faster with the Express adapter and 15x faster with Fastify.

### Framework Architectural Overhead

Comparing frameworks with their "bare" counterparts:

| Transition | Overhead |
|------------|----------|
| Express → NestJS | 40% slower |
| Fastify → NestJS | 4% slower |
| **Axum → Armature** | **3% slower** |

**Key insight:** Armature has the lowest architectural overhead of any enterprise framework tested.

---

## Reproduction

```bash
cd benchmarks/comparison

# === RUST ===
cargo build --release
PORT=8080 ./target/release/armature_bench &
oha -c 100 -z 10s http://localhost:8080/json
pkill armature_bench

# === GO ===
cd go_gin && go build -o gin_bench && PORT=8090 ./gin_bench &
oha -c 100 -z 10s http://localhost:8090/json
pkill gin_bench

cd ../go_fiber && go build -o fiber_bench && PORT=8091 ./fiber_bench &
oha -c 100 -z 10s http://localhost:8091/json
pkill fiber_bench

# === C# ===
cd ../aspnet_core && dotnet publish -c Release -o ./publish
ASPNETCORE_URLS="http://0.0.0.0:8092" dotnet ./publish/aspnet_bench.dll &
oha -c 100 -z 10s http://localhost:8092/json
pkill -f aspnet_bench

# === NODE.JS ===
cd .. && npm install express fastify
PORT=8081 node express_bench.js &
oha -c 100 -z 10s http://localhost:8081/json
pkill -f express_bench

PORT=8084 node fastify_bench.js &
oha -c 100 -z 10s http://localhost:8084/json
pkill -f fastify_bench

cd nestjs_express && npm install && npx tsc
PORT=8085 node dist/main.js &
oha -c 100 -z 10s http://localhost:8085/json
pkill -f "node dist/main"

cd ../nestjs_fastify && npm install && npx tsc
PORT=8086 node dist/main.js &
oha -c 100 -z 10s http://localhost:8086/json
pkill -f "node dist/main"
```
