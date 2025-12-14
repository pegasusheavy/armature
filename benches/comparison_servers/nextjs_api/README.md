# Next.js API Benchmark Server

Next.js API routes implementation for framework benchmarking.

## Port

- **Development:** 3005
- **Production:** 3005

## Setup

```bash
# Install dependencies
npm install
# or
pnpm install

# Development mode
npm run dev

# Production mode (for benchmarking)
npm run build && npm run start
# or
npm run benchmark
```

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api` | Plaintext "Hello, World!" |
| GET | `/api/json` | JSON response with message and timestamp |
| GET | `/api/health` | Health check with uptime and memory |
| GET | `/api/users` | List mock users |
| POST | `/api/users` | Create user (accepts JSON body) |
| GET | `/api/users/:id` | Get user by ID |
| PUT | `/api/users/:id` | Update user by ID |
| DELETE | `/api/users/:id` | Delete user by ID |
| GET | `/api/data` | Complex nested JSON (supports ?size=small\|medium\|large\|xlarge) |
| POST | `/api/data` | Process JSON data |

## Benchmark Commands

```bash
# Quick test
curl http://localhost:3005/api/json

# Benchmark with oha
oha -z 10s -c 50 http://localhost:3005/api
oha -z 10s -c 50 http://localhost:3005/api/json
oha -z 10s -c 50 http://localhost:3005/api/users/123

# Benchmark with wrk
wrk -t4 -c50 -d10s http://localhost:3005/api/json

# POST benchmark
oha -z 10s -c 50 -m POST \
  -H "Content-Type: application/json" \
  -d '{"name":"Test"}' \
  http://localhost:3005/api/users
```

## Comparison with Armature

To compare Next.js API routes with Armature as a backend:

1. Start both servers:
   ```bash
   # Terminal 1: Armature (port 3000)
   cargo run --release --example benchmark_server

   # Terminal 2: Next.js (port 3005)
   cd benches/comparison_servers/nextjs_api
   npm run benchmark
   ```

2. Run identical benchmarks:
   ```bash
   # Armature
   oha -z 30s -c 100 http://localhost:3000/json

   # Next.js
   oha -z 30s -c 100 http://localhost:3005/api/json
   ```

3. Compare results for:
   - Requests per second
   - Latency (p50, p90, p99)
   - Memory usage
   - CPU utilization

## Performance Notes

- **Cold Start:** Next.js has longer cold start times due to JIT compilation
- **Memory:** Node.js typically uses more memory than Rust
- **Throughput:** Rust/Armature generally achieves higher throughput
- **Development:** Next.js offers faster iteration with hot reload

## Use Cases

Next.js API routes are best for:
- Full-stack applications with React frontend
- Rapid prototyping
- Edge function deployments
- Serverless architectures

Armature is best for:
- High-performance API backends
- Microservices
- Real-time applications
- CPU-intensive workloads

