# Armature Framework - TODO

## Status: 99% Complete ✅

**112 optimizations implemented** | **1 remaining** | Axum/Actix-competitive performance achieved

---

## Remaining Work

| Priority | Feature | Description | Location |
|----------|---------|-------------|----------|
| 🟡 | Read-Optimized State | `parking_lot::RwLock` for read-heavy workloads | `armature-core` |

---

## Completed Work Summary

### Performance (73 optimizations)

| Category | Highlights |
|----------|------------|
| **Routing** | `matchit` O(log n), LRU cache, static fast path, compile-time validation |
| **HTTP Parsing** | SIMD via `httparse`/`memchr`, header interning (32+ headers) |
| **Serialization** | `simd-json` feature (1.8x faster), zero-copy `Bytes`, pooled buffers |
| **Memory** | Arena allocator (6x faster), SmallVec headers, CompactString, object pools |
| **Response** | `LazyHeaders` (zero-alloc empty), `FastResponse`, `FastBody` enum |
| **Collections** | SmallVec for QueryParams/PathParams/FormFields/Cookies |

### Connection & I/O

| Category | Highlights |
|----------|------------|
| **Pipelining** | HTTP/1.1 request/response pipelining, request batching |
| **I/O** | `io_uring` backend, vectored I/O (`writev`), TCP_CORK, epoll tuning |
| **Buffers** | Thread-local pools, zero-copy parsing, adaptive sizing, auto-tuning |
| **Connections** | Adaptive keep-alive, idle culling, branchless FSM, recycling pools |

### Architecture

| Category | Highlights |
|----------|------------|
| **Workers** | Per-worker routers, CPU affinity, NUMA-aware allocation, load balancing |
| **State** | Copy-on-write, cache-line alignment, hot/cold separation |
| **Runtime** | Task spawning control, LocalSet, work-stealing tuning |
| **Hyper 1.0** | Native `http` types, Tower Service compatibility |

### Tooling

| Category | Highlights |
|----------|------------|
| **Profiling** | CPU flamegraphs, `pprof`, automated CI flamegraphs |
| **Benchmarks** | TechEmpower suite, framework comparisons, regression tests |
| **Logging** | `armature-log` with JSON default, env/runtime config |
| **Fuzzing** | 8 fuzz targets, cargo-fuzz integration |
| **Build** | PGO, LTO (thin/fat), release-native, target-cpu aliases |

### Integrations

| Crate | Description |
|-------|-------------|
| `armature-opensearch` | OpenSearch client, queries, bulk ops, index management |
| `armature-toon` | LLM-optimized serialization (30-60% token reduction) |
| `armature-i18n` | i18n with pluralization, locale detection, formatting |
| `armature-compression` | Streaming gzip/brotli/zstd compression |
| `armature-diesel` | Async Diesel with deadpool/bb8/mobc pools |
| `armature-seaorm` | SeaORM with active record, pagination, query helpers |

---

## Latest Benchmarks (December 2024)

| Benchmark | Time | Improvement |
|-----------|------|-------------|
| Health check | 386ns | -4% |
| GET with param | 692ns | -15% |
| POST with body | 778ns | -26% |
| Route first match | 51ns | -6% |
| Route middle | 343ns | -17% |
| JSON serialize (small) | 17ns | -14% |
| JSON serialize (large) | 14.4µs | -7% |

**All previous regressions fixed** via `LazyHeaders`, `FastResponse`, and `small_vec.rs`.

---

## Contributing

1. Documentation in `docs/`
2. Working examples in `examples/`
3. Full test coverage
4. Follow existing patterns

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
