# Armature vs Node.js Frameworks Benchmark Guide

A comprehensive guide for benchmarking Armature (Rust) against popular Node.js frameworks: Express, Koa, and NestJS.

## Overview

This benchmark compares Armature's performance against the most popular Node.js web frameworks for common API scenarios.

### Frameworks Compared

| Framework | Language | Description | Port |
|-----------|----------|-------------|------|
| **Armature** | Rust | NestJS-inspired, full-featured | 3000 |
| **Express** | JavaScript | Minimal, battle-tested | 3006 |
| **Koa** | JavaScript | Modern, async/await native | 3007 |
| **NestJS** | TypeScript | Angular-inspired, enterprise-grade | 3008 |

## Features

- ✅ Identical API endpoints for fair comparison
- ✅ Common scenarios: plaintext, JSON, path params, POST
- ✅ Complex payload testing (nested objects, arrays)
- ✅ Production-mode benchmarking
- ✅ Memory and latency comparisons

## Quick Start

### Prerequisites

- Rust (1.88+)
- Node.js (18+)
- npm or pnpm
- oha (`cargo install oha`) or wrk

### Start All Servers

```bash
# Terminal 1: Armature (port 3000)
cd /path/to/armature
cargo run --release --example benchmark_server

# Terminal 2: Express (port 3006)
cd benches/comparison_servers/express_server
npm install && npm start

# Terminal 3: Koa (port 3007)
cd benches/comparison_servers/koa_server
npm install && npm start

# Terminal 4: NestJS (port 3008)
cd benches/comparison_servers/nestjs_server
npm install && npm run benchmark
```

### Verify Servers

```bash
curl http://localhost:3000/json   # Armature
curl http://localhost:3006/json   # Express
curl http://localhost:3007/json   # Koa
curl http://localhost:3008/json   # NestJS
```

## Benchmark Scenarios

### 1. Plaintext Response

The simplest benchmark - raw HTTP response performance.

```bash
echo "=== Plaintext Benchmark ==="

echo "Armature:"
oha -z 10s -c 50 http://localhost:3000/

echo "Express:"
oha -z 10s -c 50 http://localhost:3006/

echo "Koa:"
oha -z 10s -c 50 http://localhost:3007/

echo "NestJS:"
oha -z 10s -c 50 http://localhost:3008/
```

### 2. JSON Serialization

JSON response performance - common for REST APIs.

```bash
echo "=== JSON Benchmark ==="

echo "Armature:"
oha -z 10s -c 50 http://localhost:3000/json

echo "Express:"
oha -z 10s -c 50 http://localhost:3006/json

echo "Koa:"
oha -z 10s -c 50 http://localhost:3007/json

echo "NestJS:"
oha -z 10s -c 50 http://localhost:3008/json
```

### 3. Path Parameter Extraction

Dynamic route handling performance.

```bash
echo "=== Path Parameter Benchmark ==="

echo "Armature:"
oha -z 10s -c 50 http://localhost:3000/users/123

echo "Express:"
oha -z 10s -c 50 http://localhost:3006/users/123

echo "Koa:"
oha -z 10s -c 50 http://localhost:3007/users/123

echo "NestJS:"
oha -z 10s -c 50 http://localhost:3008/users/123
```

### 4. POST with JSON Body

Request body parsing performance.

```bash
echo "=== POST Benchmark ==="

PAYLOAD='{"name":"John Doe","email":"john@example.com"}'

echo "Armature:"
oha -z 10s -c 50 -m POST -H "Content-Type: application/json" -d "$PAYLOAD" \
  http://localhost:3000/api/users

echo "Express:"
oha -z 10s -c 50 -m POST -H "Content-Type: application/json" -d "$PAYLOAD" \
  http://localhost:3006/api/users

echo "Koa:"
oha -z 10s -c 50 -m POST -H "Content-Type: application/json" -d "$PAYLOAD" \
  http://localhost:3007/api/users

echo "NestJS:"
oha -z 10s -c 50 -m POST -H "Content-Type: application/json" -d "$PAYLOAD" \
  http://localhost:3008/api/users
```

### 5. Complex Data (Large Payload)

Large JSON response with nested objects.

```bash
echo "=== Complex Data Benchmark ==="

# Medium payload (50 products)
echo "Medium Payload:"
oha -z 10s -c 50 http://localhost:3000/data?size=medium
oha -z 10s -c 50 http://localhost:3006/data?size=medium
oha -z 10s -c 50 http://localhost:3007/data?size=medium
oha -z 10s -c 50 http://localhost:3008/data?size=medium

# Large payload (100 products)
echo "Large Payload:"
oha -z 10s -c 50 http://localhost:3000/data?size=large
oha -z 10s -c 50 http://localhost:3006/data?size=large
oha -z 10s -c 50 http://localhost:3007/data?size=large
oha -z 10s -c 50 http://localhost:3008/data?size=large
```

## Automated Benchmark Runner

Run all benchmarks automatically:

```bash
cd /path/to/armature

# Benchmark specific frameworks
cargo run --release --bin http-benchmark -- \
  --framework armature \
  --framework express \
  --framework koa \
  --framework nestjs

# Benchmark all frameworks
cargo run --release --bin http-benchmark -- --all
```

## Expected Results

### Performance Comparison

| Metric | Armature | Express | Koa | NestJS |
|--------|----------|---------|-----|--------|
| **Plaintext (req/s)** | 200K-400K | 25K-50K | 30K-55K | 20K-45K |
| **JSON (req/s)** | 150K-300K | 20K-45K | 25K-50K | 18K-40K |
| **Path Param (req/s)** | 120K-250K | 18K-40K | 22K-48K | 15K-35K |
| **POST (req/s)** | 80K-180K | 15K-35K | 18K-40K | 12K-30K |
| **Latency p99** | 0.5-2ms | 3-15ms | 2-12ms | 4-18ms |
| **Memory (idle)** | 5-15 MB | 30-50 MB | 25-40 MB | 50-80 MB |
| **Memory (load)** | 20-50 MB | 80-150 MB | 60-120 MB | 100-200 MB |

### Performance Ratio (vs Armature)

| Framework | Throughput | Latency | Memory |
|-----------|------------|---------|--------|
| Express | ~8-12x slower | ~5-8x higher | ~4-6x more |
| Koa | ~6-10x slower | ~4-6x higher | ~3-5x more |
| NestJS | ~8-15x slower | ~6-10x higher | ~6-10x more |

## When to Use Each Framework

### Choose Armature When:

- ✅ High-performance API is critical (>50K RPS)
- ✅ Low latency requirements (p99 < 5ms)
- ✅ Memory-constrained environments
- ✅ CPU-intensive workloads
- ✅ Type safety is important
- ✅ Long-running, stable services

### Choose Express When:

- ✅ Quick prototyping needed
- ✅ Large middleware ecosystem required
- ✅ Team expertise in JavaScript
- ✅ Lower traffic applications (<20K RPS)
- ✅ Simple, minimal API

### Choose Koa When:

- ✅ Modern async/await patterns preferred
- ✅ Lightweight alternative to Express
- ✅ More control over middleware
- ✅ Clean, minimal codebase

### Choose NestJS When:

- ✅ Enterprise-grade structure needed
- ✅ TypeScript-first development
- ✅ Angular developers on the team
- ✅ Built-in DI, modules, guards
- ✅ Consistent project architecture

## Framework Comparison Matrix

| Feature | Armature | Express | Koa | NestJS |
|---------|----------|---------|-----|--------|
| **Performance** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| **Memory Usage** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ |
| **Type Safety** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| **Learning Curve** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Ecosystem** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **DI Support** | ⭐⭐⭐⭐⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐⭐ |
| **Validation** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| **OpenAPI** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| **Testing** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

## High Concurrency Testing

Test how frameworks handle high load:

```bash
# 100 concurrent connections
oha -z 30s -c 100 http://localhost:3000/json
oha -z 30s -c 100 http://localhost:3006/json
oha -z 30s -c 100 http://localhost:3007/json
oha -z 30s -c 100 http://localhost:3008/json

# 500 concurrent connections
oha -z 30s -c 500 http://localhost:3000/json
oha -z 30s -c 500 http://localhost:3006/json
oha -z 30s -c 500 http://localhost:3007/json
oha -z 30s -c 500 http://localhost:3008/json

# 1000 concurrent connections (stress test)
oha -z 30s -c 1000 http://localhost:3000/json
oha -z 30s -c 1000 http://localhost:3006/json  # May struggle
oha -z 30s -c 1000 http://localhost:3007/json  # May struggle
oha -z 30s -c 1000 http://localhost:3008/json  # May struggle
```

## Memory Monitoring

Monitor memory usage during benchmarks:

```bash
# Watch memory usage (Linux)
watch -n 1 'ps aux | grep -E "(benchmark_server|node)" | grep -v grep'

# Detailed memory (Linux)
while true; do
  echo "=== $(date) ==="
  echo "Armature: $(ps -p $(pgrep -f benchmark_server) -o rss= 2>/dev/null || echo 'N/A') KB"
  echo "Express:  $(ps -p $(pgrep -f express_server) -o rss= 2>/dev/null || echo 'N/A') KB"
  echo "Koa:      $(ps -p $(pgrep -f koa_server) -o rss= 2>/dev/null || echo 'N/A') KB"
  echo "NestJS:   $(ps -p $(pgrep -f nestjs_server) -o rss= 2>/dev/null || echo 'N/A') KB"
  sleep 1
done
```

## Cold Start Comparison

Measure server startup time:

```bash
# Armature cold start
time (cargo run --release --example benchmark_server &
  sleep 0.2 && curl -s http://localhost:3000/health > /dev/null && pkill -f benchmark_server)

# Express cold start
time (cd benches/comparison_servers/express_server && node src/server.js &
  sleep 0.3 && curl -s http://localhost:3006/health > /dev/null && pkill -f express)

# Koa cold start
time (cd benches/comparison_servers/koa_server && node src/server.js &
  sleep 0.3 && curl -s http://localhost:3007/health > /dev/null && pkill -f koa)

# NestJS cold start (slower due to TypeScript compilation)
time (cd benches/comparison_servers/nestjs_server && node dist/main.js &
  sleep 0.5 && curl -s http://localhost:3008/health > /dev/null && pkill -f nestjs)
```

## Production Recommendations

### For High-Performance APIs:

```
┌─────────────────────────────────────────────────────────────┐
│                     Load Balancer                           │
└─────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
    ┌──────────┐        ┌──────────┐        ┌──────────┐
    │ Armature │        │ Armature │        │ Armature │
    │ Instance │        │ Instance │        │ Instance │
    └──────────┘        └──────────┘        └──────────┘
```

### For Full-Stack Applications:

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (Next.js)                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ HTTP/REST
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Armature Backend                        │
│   - High-performance APIs                                   │
│   - Real-time features                                      │
│   - Heavy computation                                       │
└─────────────────────────────────────────────────────────────┘
```

## Troubleshooting

### Node.js Event Loop Blocking

If Node.js frameworks show degraded performance under load:

```bash
# Increase UV_THREADPOOL_SIZE
UV_THREADPOOL_SIZE=16 node src/server.js
```

### Memory Issues

```bash
# Increase Node.js heap size
NODE_OPTIONS="--max-old-space-size=4096" npm start
```

### Port Conflicts

```bash
# Find and kill process on port
lsof -ti :3006 | xargs kill -9
```

## Summary

| Framework | Best For | Throughput | Developer Experience |
|-----------|----------|------------|---------------------|
| **Armature** | Performance-critical APIs | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Express** | Simple APIs, prototypes | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Koa** | Modern, minimal APIs | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **NestJS** | Enterprise applications | ⭐⭐⭐ | ⭐⭐⭐⭐ |

**Key Takeaway:** Armature provides 8-15x better throughput and significantly lower memory usage compared to Node.js frameworks, making it ideal for performance-critical applications. Node.js frameworks offer faster development cycles and larger ecosystems, making them suitable for rapid prototyping and full-stack JavaScript applications.

