# Armature Framework - TODO

## Legend

- 🟠 **High Priority** - Important for enterprise adoption
- 🟡 **Medium Priority** - Nice to have, improves DX

---

## Completed Work Summary

**73 performance optimizations implemented** across the following areas:

### Core Performance (Axum/Actix-competitive)
- **Routing**: `matchit` O(log n) router, route cache with LRU + static fast path, compile-time validation
- **HTTP Parsing**: SIMD-optimized (`httparse` + `memchr`), header interning (32+ headers)
- **Serialization**: `simd-json` feature, zero-copy `Bytes`, pooled serialization buffers
- **Memory**: Arena allocator (6x faster), SmallVec headers (12 inline), CompactString paths, object pools

### Connection & I/O
- **Pipelining**: HTTP/1.1 request/response pipelining, request batching
- **I/O**: `io_uring` backend, vectored I/O (`writev`), TCP_CORK, epoll tuning
- **Buffers**: Thread-local `BytesMut` pools, zero-copy body parsing, adaptive sizing

### Worker Architecture
- **Workers**: Per-worker routers, CPU affinity, NUMA-aware allocation, load balancing
- **State**: Copy-on-write, cache-line alignment, hot/cold separation
- **Connections**: Branchless FSM, recycling pools, streaming responses

### Framework Integration
- **Hyper 1.0**: Native `http` crate types, Tower Service compatibility
- **Abstractions**: Const generic extractors, static dispatch middleware, inline handlers
- **Runtime**: Task spawning control, LocalSet, work-stealing tuning

### Tooling & Benchmarks
- **Profiling**: CPU flamegraphs, `pprof` integration, profiling guide
- **Benchmarks**: TechEmpower suite, framework comparisons, CI regression tests
- **Infrastructure**: Grafana dashboards, Ferron reverse proxy, all CI passing
- **Logging**: `armature-log` crate with JSON default, env config, runtime config
- **Fuzzing**: 8 fuzz targets (HTTP, routing, JSON, URLs, headers, params), cargo-fuzz setup

### Compiler Optimizations
- **Build Profiles**: PGO, LTO (thin/fat), release-native, profiling
- **Cargo Config**: target-cpu aliases, codegen-units=1, PGO workflow script

### Integrations
- **OpenSearch**: `armature-opensearch` crate with client, queries, bulk ops, index management
- **Publishing**: Automated crates.io publishing scripts with dependency ordering

---

## Remaining Work

### Performance Regressions (from December 2024 benchmarks)

| Priority | Issue | Impact | Location |
|----------|-------|--------|----------|
| 🟠 | Response Creation Overhead | +9% empty, +23% status codes | `armature-core/response.rs` |
| 🟠 | Small JSON Response Allocation | +7% for small JSON responses | `armature-core/response.rs` |
| 🟡 | Vec Small Allocation | +7% regression on small vectors | `armature-core` |

### Buffer & Connection Tuning

| Priority | Feature | Description | Location |
|----------|---------|-------------|----------|
| 🟡 | Buffer Size Auto-Tuning | Dynamic adjustment based on traffic | `armature-core` |
| 🟡 | Adaptive Keep-Alive | Adjust based on server load | `armature-core` |
| 🟡 | Idle Connection Culling | Drop idle connections under pressure | `armature-core` |

### Streaming & Compression

| Priority | Feature | Description | Location |
|----------|---------|-------------|----------|
| 🟡 | Backpressure Handling | Flow control for slow clients | `armature-core` |
| 🟡 | Streaming Compression | Compress chunks as generated | `armature-compression` |

### State Management

| Priority | Feature | Description | Location |
|----------|---------|-------------|----------|
| 🟡 | Read-Optimized State | `parking_lot::RwLock` for read-heavy | `armature-core` |

### Benchmarking

| Priority | Feature | Description | Location |
|----------|---------|-------------|----------|
| 🟡 | Flame Graph CI | Auto-generate flamegraphs | `.github/workflows/` |
| 🟡 | Database Benchmark | Single/multiple query tests | `benches/database/` |

### Internationalization

| Priority | Feature | Description | Location |
|----------|---------|-------------|----------|
| 🟠 | i18n Support | Message translation | `armature-i18n` |
| 🟠 | Locale Detection | Accept-Language parsing | `armature-i18n` |
| 🟡 | Pluralization | Plural rules support | `armature-i18n` |
| 🟡 | Date/Number Formatting | Locale-aware formatting | `armature-i18n` |

---

## Summary

| Category | Remaining | Completed |
|----------|-----------|-----------|
| Performance Regressions | 3 | - |
| Compiler Optimizations | - | 4 |
| Buffer/Connection Tuning | 3 | 15+ |
| Streaming/Compression | 2 | 4 |
| State Management | 1 | 4 |
| Benchmarking | 2 | 7 |
| Testing & Fuzzing | - | 8 |
| Internationalization | 4 | - |
| Integrations | - | 2 |
| **Total** | **15** | **85** |

### Performance Status

| Metric | Status |
|--------|--------|
| Axum parity | ✅ Achieved (routing, zero-cost abstractions) |
| Actix-competitive | ✅ Core optimizations complete |
| TechEmpower ready | ✅ Benchmark suite implemented |

### Latest Benchmark Results (December 2024)

| Benchmark | Time | Change |
|-----------|------|--------|
| **Full Cycle** | | |
| Health check | 386ns | **-4%** ✅ |
| GET with param | 692ns | **-15%** ✅ |
| POST with body | 778ns | **-26%** ✅ |
| **Routing (100 routes)** | | |
| First match | 51ns | **-6%** ✅ |
| Middle match | 343ns | **-17%** ✅ |
| Not found | 1.3µs | **-16%** ✅ |
| **JSON Operations** | | |
| Serialize small | 17ns | **-14%** ✅ |
| Serialize large | 14.4µs | **-7%** ✅ |
| Deserialize medium | 204ns | **-2%** ✅ |
| **Regressions** | | |
| Empty response | 2.2ns | +9% ⚠️ |
| Status codes | 11ns | +23% ⚠️ |
| Small JSON response | 59ns | +7% ⚠️ |

---

## Contributing

Each feature should:

1. Have documentation in `docs/`
2. Include working examples in `examples/`
3. Have full test coverage
4. Follow existing code patterns

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
