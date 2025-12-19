# Armature Framework - Remaining TODO

Only features that are **not yet completed**.

## Legend

- 🔴 **Critical Priority** - Required for Axum-competitive performance
- 🟠 **High Priority** - Important for enterprise adoption
- 🟡 **Medium Priority** - Nice to have, improves DX
- ✅ **Completed** - Recently finished

---

## Recently Completed

| Feature | Description | Module/Location |
|---------|-------------|-----------------|
| ✅ CPU Profiling | Flamegraph generation with pprof | `examples/profiling_server.rs` |
| ✅ Profiling Script | Automated profiling workflow | `scripts/profile.sh` |
| ✅ Profiling Docs | Documentation website guide | `web/src/app/pages/docs/pages/profiling-guide/` |
| ✅ Grafana Dashboards | Pre-built dashboard templates | `templates/grafana/` |
| ✅ Replace Trie with `matchit` | High-performance router using `matchit` crate | `armature-core/src/router.rs` |
| ✅ Compile-time Route Validation | Validate routes at compile time via proc macros | `armature-macro/src/route_validation.rs` |
| ✅ Remove Runtime Type Checks | Zero-cost `State<T>` extractor without `Any` downcasting | `armature-core/src/extractors.rs` |
| ✅ TechEmpower Benchmark Suite | JSON, DB, and Fortunes benchmark implementations | `benches/techempower/` |
| ✅ Framework Comparison Benchmarks | Side-by-side benchmarks vs Axum, Actix, Express, etc. | `benchmarks/comparison/` |
| ✅ Ferron Integration | Reverse proxy integration with Ferron | `armature-ferron/` |
| ✅ CI Pipeline Fixes | All 16 CI jobs passing (format, clippy, tests, benchmarks) | `.github/workflows/` |
| ✅ Inline Handler Dispatch | Handler trait with monomorphization and `#[inline]` hints | `armature-core/src/handler.rs` |
| ✅ SIMD HTTP Parser | Integrated `httparse` + `memchr` for SIMD-optimized parsing | `armature-core/src/simd_parser.rs` |
| ✅ SIMD JSON | Optional `simd-json` feature for SIMD-accelerated JSON | `armature-core/src/json.rs` |
| ✅ Arena Allocator | Per-request arena for batch allocations (~6x faster) | `armature-core/src/arena.rs` |
| ✅ Hyper Body Passthrough | Zero-copy Bytes-based body handling (~4x faster clone) | `armature-core/src/body.rs` |
| ✅ Automated Regression Tests | CI pipeline with benchmark regression detection | `.github/workflows/benchmark.yml` |
| ✅ HTTP/1.1 Pipelining | Pipeline config, stats, TCP_NODELAY, keep-alive | `armature-core/src/pipeline.rs` |
| ✅ Request Batching | Batch-read multiple requests from socket buffer | `armature-core/src/batch.rs` |
| ✅ `io_uring` Backend | Linux io_uring support for reduced syscall overhead | `armature-core/src/io_uring.rs` |
| ✅ Thread-local `BytesMut` Pool | Buffer pool for reduced allocation overhead | `armature-core/src/buffer_pool.rs` |
| ✅ Zero-Copy Body Parsing | Lazy body, streaming, pooled buffer integration | `armature-core/src/body_parser.rs` |
| ✅ SmallVec Headers | Stack-allocated headers (12 inline, no heap for typical requests) | `armature-core/src/headers.rs` |
| ✅ Pre-allocated Response Buffer | 512-byte default buffer to avoid reallocations | `armature-core/src/response_buffer.rs` |
| ✅ Vectored I/O | writev() support for headers+body in single syscall | `armature-core/src/vectored_io.rs` |
| ✅ Per-Worker Router | Thread-local router to avoid Arc cloning overhead | `armature-core/src/worker.rs` |
| ✅ CPU Core Affinity | Pin workers to CPU cores for cache locality | `armature-core/src/worker.rs` |
| ✅ Response Pipelining | Queue responses for batch-write to socket | `armature-core/src/response_pipeline.rs` |
| ✅ Read Buffer Sizing | Tune read buffer sizes based on payload patterns | `armature-core/src/read_buffer.rs` |
| ✅ Write Buffer Coalescing | Combine small writes into single buffer flush | `armature-core/src/write_coalesce.rs` |
| ✅ Per-Worker State | Thread-local state to avoid Arc contention | `armature-core/src/worker.rs` |
| ✅ NUMA-Aware Allocation | Allocate memory on same NUMA node as worker | `armature-core/src/numa.rs` |
| ✅ Optimized State Transitions | Branchless connection FSM with lookup tables | `armature-core/src/connection.rs` |

---

## Performance Optimizations

Based on CPU profiling analysis (flamegraph from `examples/profiling_server.rs`):

### Routing & Request Handling (~28% CPU)

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟠 | Route Matching Cache | Cache compiled routes to avoid repeated trie traversal | `armature-core/routing.rs` |
| 🟠 | Static Route Fast Path | Bypass trie for exact-match static routes using HashMap | `armature-core/routing.rs` |
| 🟡 | Header Map Optimization | Use `smallvec` or pre-allocated headers for common cases | `armature-core` |

### HTTP Parsing (~7% CPU)

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | SIMD HTTP Parser | Integrated `httparse` + `memchr` with SIMD query parsing | `armature-core/src/simd_parser.rs` |
| ✅ | Header Interning | Intern 32+ common header names to avoid allocations | `armature-core/src/simd_parser.rs` |

### Serialization

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | SIMD JSON | Added optional `simd-json` feature flag | `armature-core/src/json.rs` |
| 🟡 | Zero-Copy Responses | Use `Bytes` for zero-copy response bodies | `armature-core` |
| 🟡 | Pre-allocated Buffers | Buffer pool for response serialization | `armature-core` |

### Connection Handling

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟡 | HTTP/2 Priority | Optimize HTTP/2 stream prioritization | `armature-core` |
| 🟡 | TCP_NODELAY Tuning | Fine-tune TCP settings for low latency | `armature-core` |
| 🟡 | Connection Keep-Alive | Optimize keep-alive timeout handling | `armature-core` |

---

## Axum-Competitive Benchmarking

Goal: Achieve comparable performance to Axum on standard benchmarks (TechEmpower, wrk, hey).

### Router Optimization (Critical - Axum uses `matchit`)

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Replace Trie with `matchit` | Use `matchit` crate (same as Axum) for route matching | `armature-core/src/router.rs` |
| ✅ | Compile-time Route Validation | Validate routes at compile time, not runtime | `armature-macro/src/route_validation.rs` |
| 🟠 | Route Parameter Extraction | Zero-allocation parameter extraction like Axum | `armature-core/routing.rs` |
| 🟠 | Wildcard/Catch-all Optimization | Optimize `*path` and `/*rest` patterns | `armature-core/routing.rs` |

### Zero-Cost Abstractions (Critical - Axum's strength)

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Inline Handler Dispatch | Handler trait with monomorphization and `#[inline]` hints | `armature-core/src/handler.rs` |
| ✅ | Remove Runtime Type Checks | Zero-cost `State<T>` extractor with `Extensions` | `armature-core/src/extractors.rs` |
| 🟠 | Const Generic Extractors | Use const generics for zero-cost extractor chains | `armature-core/extractors.rs` |
| 🟠 | Static Dispatch Middleware | Replace `Box<dyn>` with static dispatch where possible | `armature-core/middleware.rs` |

### Memory & Allocation (Axum minimizes allocations)

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Arena Allocator for Requests | Per-request arena to batch deallocations (~6x faster) | `armature-core/src/arena.rs` |
| 🟠 | `SmallVec` for Headers | Use `SmallVec<[_; 16]>` for typical header counts | `armature-core` |
| 🟠 | `CompactString` for Paths | Use `compact_str` for short route paths | `armature-core/routing.rs` |
| 🟠 | Pre-sized Response Buffers | Avoid reallocations during response building | `armature-core/response.rs` |
| 🟡 | Object Pool for Requests | Reuse request/response objects across connections | `armature-core` |

### Hyper Integration (Axum is thin layer over Hyper)

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Direct Hyper Body Passthrough | Zero-copy Bytes-based body handling | `armature-core/src/body.rs` |
| 🟠 | Native `http` Crate Types | Use `http::Request`/`Response` directly | `armature-core` |
| 🟠 | Tower Service Compatibility | Implement `tower::Service` for composability | `armature-core` |
| 🟡 | Hyper 1.0 Full Support | Ensure all Hyper 1.0 features are utilized | `armature-core` |

### Async Runtime Optimization

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟠 | Reduce Task Spawning | Inline simple handlers instead of spawning tasks | `armature-core` |
| 🟠 | `tokio::task::LocalSet` Option | Single-threaded mode for maximum cache locality | `armature-core` |
| 🟡 | Custom Executor Tuning | Expose tokio runtime configuration | `armature-core` |
| 🟡 | Work-Stealing Optimization | Tune work-stealing for HTTP workloads | `armature-core` |

### Benchmark Infrastructure

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | TechEmpower Benchmark Suite | Implement all TechEmpower tests (JSON, DB, Fortune) | `benches/techempower/` |
| ✅ | Automated Regression Tests | CI pipeline with threshold alerts | `.github/workflows/benchmark.yml` |
| ✅ | Axum Comparison Benchmark | Side-by-side benchmark vs Axum with same routes | `benchmarks/comparison/` |
| 🟡 | Flame Graph CI Integration | Auto-generate flamegraphs on benchmark runs | `.github/workflows/` |

### Compiler Optimizations

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟠 | Profile-Guided Optimization | Add PGO build profile | `Cargo.toml` |
| 🟠 | LTO Thin/Fat Modes | Benchmark LTO impact on binary size vs speed | `Cargo.toml` |
| 🟡 | Target-specific Tuning | Enable `-C target-cpu=native` for benchmarks | `Cargo.toml` |
| 🟡 | Codegen Units = 1 | Single codegen unit for maximum optimization | `Cargo.toml` |

---

## Actix-web Competitive Performance

Goal: Match Actix-web's TechEmpower-leading performance through low-level optimizations.

### Benchmark Results (December 2024)

**Current Performance Gap**:
| Framework | Requests/sec | vs Armature |
|-----------|-------------|-------------|
| Actix-web 4 | 589,565 | +40% faster |
| Axum 0.7 | 434,567 | +3% faster |
| **Armature 0.1** | 421,323 | baseline |

**Micro-benchmark Analysis** (per-operation latency):

| Operation | Armature | Notes |
|-----------|----------|-------|
| Handler dispatch (simple) | 112 ns | Good - monomorphized |
| Handler dispatch (JSON) | 172 ns | +60ns for JSON response |
| Handler dispatch (params) | 292 ns | HashMap param extraction overhead |
| Handler dispatch (body parse) | 533 ns | JSON deserialization dominates |
| Route match (10 routes) | 55-150 ns | O(n) linear scan |
| Route match (50 routes) | 54-489 ns | Degrades with route count |
| Route match (100 routes) | 57-400+ ns | Scaling issue |
| Request creation (minimal) | 24 ns | Good |
| Request creation (headers) | 192 ns | HashMap allocation overhead |
| Response creation (empty) | 2 ns | Excellent |
| Response (small JSON) | 55 ns | Good |
| JSON serialize (small) | 20 ns | Good - serde_json |
| JSON serialize (large) | 15.5 µs | Consider simd-json |

### Critical Bottlenecks Identified

1. **Routing is O(n)** - Current implementation uses linear search
   - Actix uses radix trie with O(log n) lookup
   - Solution: Implement `matchit` crate properly or custom radix trie

2. **HashMap Allocations** - Headers/params use std HashMap
   - Each request allocates 2+ HashMaps
   - Solution: SmallVec or pre-allocated fixed-size arrays

3. **No Buffer Pooling** - Request/response allocate fresh buffers
   - Actix reuses BytesMut from thread-local pools
   - Solution: Thread-local buffer pool with BytesMut

4. **JSON Serialization** - Using standard serde_json
   - 15µs for large payloads
   - Solution: Optional simd-json or sonic-rs feature

5. **Router Cloning** - Arc<Router> cloned per connection
   - Actix avoids this with shared state
   - Solution: Arc-free routing or per-worker routers

### Actix Performance Gap Roadmap

**Phase 1: Low-Hanging Fruit (Expected: +15% throughput)**

| Priority | Task | Estimated Impact | Effort |
|----------|------|------------------|--------|
| ✅ | Use `matchit` crate for O(log n) routing | +8-10% | Low |
| ✅ | Replace HashMap with `SmallVec<[_; 12]>` for headers | +3-5% | Medium |
| ✅ | Add `simd-json` feature flag for JSON | +2-3% | Low |
| ✅ | Pre-allocate response buffer (512 bytes default) | +1-2% | Low |

**Phase 2: Buffer Management (Expected: +10% throughput)**

| Priority | Task | Estimated Impact | Effort |
|----------|------|------------------|--------|
| ✅ | Thread-local `BytesMut` buffer pool | +4-5% | Medium |
| ✅ | Zero-copy request body parsing | +3-4% | High |
| ✅ | Vectored I/O for responses (writev) | +2-3% | Medium |

**Phase 3: Connection Optimization (Expected: +10% throughput)**

| Priority | Task | Estimated Impact | Effort |
|----------|------|------------------|--------|
| ✅ | HTTP/1.1 request pipelining | +5-7% | High |
| ✅ | Per-worker routing tables (avoid Arc clone) | +2-3% | Medium |
| ✅ | CPU core affinity for workers | +1-2% | Low |

**Phase 4: Advanced Optimizations (Expected: +5% throughput)**

| Priority | Task | Estimated Impact | Effort |
|----------|------|------------------|--------|
| ✅ | `io_uring` backend for Linux | +3-5% | Very High |
| 🟡 | Object pool for request/response structs | +1-2% | Medium |
| 🟡 | PGO (Profile-Guided Optimization) build | +2-3% | Low |

---

### HTTP/1.1 Optimizations (Actix excels here)

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | HTTP/1.1 Pipelining | Process multiple requests per connection without waiting | `armature-core/src/pipeline.rs` |
| ✅ | Request Batching | Batch-read multiple requests from socket buffer | `armature-core/src/batch.rs` |
| ✅ | Response Pipelining | Queue responses for batch-write to socket | `armature-core/src/response_pipeline.rs` |
| 🟠 | Vectored I/O (writev) | Use `writev()` to send headers+body in single syscall | `armature-core/http.rs` |

### Buffer Management (Actix's key advantage)

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | `BytesMut` Buffer Pool | Thread-local pool of pre-allocated `BytesMut` buffers | `armature-core/src/buffer_pool.rs` |
| ✅ | Zero-Copy Request Body | Parse directly into pooled buffers without copying | `armature-core/src/body_parser.rs` |
| ✅ | Read Buffer Sizing | Tune read buffer sizes based on typical payload | `armature-core/src/read_buffer.rs` |
| ✅ | Write Buffer Coalescing | Combine small writes into single buffer flush | `armature-core/src/write_coalesce.rs` |
| 🟡 | Buffer Size Auto-Tuning | Dynamically adjust buffer sizes based on traffic | `armature-core/buffer.rs` |

### Worker Architecture (Actix's Arbiter pattern)

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Per-Worker State | Thread-local state to avoid Arc contention | `armature-core/src/worker.rs` |
| 🟠 | CPU Core Affinity | Pin worker threads to CPU cores for cache locality | `armature-core/runtime.rs` |
| ✅ | NUMA-Aware Allocation | Allocate memory on same NUMA node as worker | `armature-core/src/numa.rs` |
| 🟡 | Worker Load Balancing | Round-robin or least-connections distribution | `armature-core/worker.rs` |

### Connection State Machine

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Optimized State Transitions | Minimize branching in connection FSM | `armature-core/src/connection.rs` |
| 🟠 | Connection Recycling | Reset and reuse connection objects | `armature-core/connection.rs` |
| 🟡 | Adaptive Keep-Alive | Adjust keep-alive based on server load | `armature-core/connection.rs` |
| 🟡 | Idle Connection Culling | Efficiently drop idle connections under pressure | `armature-core/connection.rs` |

### Streaming & Chunked Transfer

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟠 | Streaming Response Body | Send response while still generating body | `armature-core/response.rs` |
| 🟠 | Chunk Size Optimization | Optimal chunk sizes for chunked encoding | `armature-core/response.rs` |
| 🟡 | Backpressure Handling | Flow control when client reads slowly | `armature-core/response.rs` |
| 🟡 | Streaming Compression | Compress chunks as they're generated | `armature-compression` |

### Application State Optimization

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟠 | Copy-on-Write State | Use `Arc<T>` patterns that avoid cloning | `armature-core/state.rs` |
| 🟠 | State Locality | Keep frequently-accessed state in cache | `armature-core/state.rs` |
| 🟡 | Read-Optimized State | Use `parking_lot::RwLock` for read-heavy state | `armature-core/state.rs` |

### Syscall Optimization

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | `io_uring` Support | Use io_uring for async I/O on Linux 5.1+ | `armature-core/src/io_uring.rs` |
| 🟠 | `epoll` Tuning | Optimize epoll flags and batch sizes | `armature-core/io.rs` |
| 🟠 | Reduce `recv`/`send` Calls | Batch socket operations where possible | `armature-core/io.rs` |
| 🟡 | `TCP_CORK` Usage | Cork TCP for header+body combining | `armature-core/io.rs` |

### Actix-specific Benchmark Comparison

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| ✅ | Actix Comparison Benchmark | Direct A/B benchmark against Actix-web | `benchmarks/comparison/actix_bench.rs` |
| ✅ | JSON Serialization Benchmark | Compare JSON endpoint performance | `benchmarks/comparison/` |
| ✅ | Plaintext Benchmark | Raw "Hello World" throughput test | `benchmarks/comparison/` |
| 🟡 | Database Query Benchmark | Single/multiple query performance | `benches/database/` |

---

## Multi-tenancy & Enterprise

### Internationalization

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟠 | i18n Support | Message translation | `armature-i18n` |
| 🟠 | Locale Detection | Accept-Language parsing | `armature-i18n` |
| 🟡 | Pluralization | Plural rules support | `armature-i18n` |
| 🟡 | Date/Number Formatting | Locale-aware formatting | `armature-i18n` |

---

## Summary

| Category | Remaining | Priority |
|----------|-----------|----------|
| Performance - Routing | 3 | 🟠/🟡 |
| Performance - HTTP Parsing | 0 | ✅ |
| Performance - Serialization | 2 | 🟡 |
| Performance - Connections | 3 | 🟡 |
| **Axum-Competitive** | | |
| ↳ Router Optimization | 2 | 🟠 |
| ↳ Zero-Cost Abstractions | 2 | 🟠 |
| ↳ Memory & Allocation | 4 | 🟠/🟡 |
| ↳ Hyper Integration | 3 | 🟠/🟡 |
| ↳ Async Runtime | 4 | 🟠/🟡 |
| ↳ Benchmark Infrastructure | 1 | 🟡 |
| ↳ Compiler Optimizations | 4 | 🟠/🟡 |
| **Actix-web Competitive** | | |
| ↳ Actix Performance Roadmap | 8 | 🟠/🟡 |
| ↳ HTTP/1.1 Optimizations | 2 | 🟠 |
| ↳ Buffer Management | 3 | 🟠/🟡 |
| ↳ Worker Architecture | 4 | 🟠/🟡 |
| ↳ Connection State Machine | 4 | 🟠/🟡 |
| ↳ Streaming & Chunked | 4 | 🟠/🟡 |
| ↳ State Optimization | 3 | 🟠/🟡 |
| ↳ Syscall Optimization | 3 | 🟠/🟡 |
| ↳ Actix Benchmarks | 1 | 🟡 |
| Internationalization | 4 | 🟠/🟡 |
| **Total Remaining** | **79** | |
| **Recently Completed** | **35** | ✅ |

### Performance Target

**Goal**: Close the 40% gap to Actix-web through systematic optimization.

| Phase | Tasks | Expected Gain | Cumulative |
|-------|-------|---------------|------------|
| Phase 1 | Routing, Headers, JSON | +15% | 485k req/s |
| Phase 2 | Buffer pools, zero-copy | +10% | 534k req/s |
| Phase 3 | Pipelining, workers | +10% | 587k req/s |
| Phase 4 | io_uring, PGO | +5% | 617k req/s |

Target: **~590k req/s** (Actix-equivalent performance)

---

## Contributing

We welcome contributions! Each feature should:

1. Have comprehensive documentation in `docs/`
2. Include working examples in `examples/`
3. Have full test coverage
4. Follow existing code patterns
5. Update the README and website

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.
