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
- **TOON**: `armature-toon` crate for LLM-optimized serialization (30-60% token reduction)
- **Publishing**: Automated crates.io publishing scripts with dependency ordering

---

## Remaining Work

### Performance Regressions (from December 2024 benchmarks)

| Priority | Issue | Impact | Status |
|----------|-------|--------|--------|
| ✅ | Response Creation Overhead | +9% empty, +23% status codes | Fixed via `FastResponse` |
| ✅ | Small JSON Response Allocation | +7% for small JSON responses | Fixed via `FastResponse` |
| 🟡 | Vec Small Allocation | +7% regression on small vectors | `armature-core` |

**Fix:** Added `armature-core/src/fast_response.rs` with:
- `FastResponse`: Zero-alloc response creation using const constructors
- `FastHeaders`: SmallVec-based inline header storage (≤8 headers on stack)
- `FastBody`: Enum for Empty/Static/Bytes/Owned bodies (no alloc for empty)
- `fast::ok()`, `fast::not_found()`, etc. for common status codes

### Buffer & Connection Tuning

| Priority | Feature | Description | Status |
|----------|---------|-------------|--------|
| ✅ | Buffer Size Auto-Tuning | Dynamic adjustment based on traffic | `connection_manager.rs` |
| ✅ | Adaptive Keep-Alive | Adjust based on server load | `connection_manager.rs` |
| ✅ | Idle Connection Culling | Drop idle connections under pressure | `connection_manager.rs` |

**Implemented via `ConnectionManager`:**
- **Buffer Auto-Tuning**: Tracks buffer usage history, computes p95 optimal size
- **Adaptive Keep-Alive**: Reduces timeout as load increases, configurable thresholds
- **Idle Culling**: Drops oldest idle connections under memory/connection pressure

### Streaming & Compression

| Priority | Feature | Description | Status |
|----------|---------|-------------|--------|
| ✅ | Backpressure Handling | Flow control for slow clients | `streaming.rs` |
| ✅ | Streaming Compression | Compress chunks as generated | `armature-compression` |

**Implemented:**
- **BackpressureController**: High/low watermarks, pause/resume, drop strategies, async wait
- **StreamingCompressor**: gzip/brotli/zstd, incremental compression, configurable flush intervals

### State Management

| Priority | Feature | Description | Location |
|----------|---------|-------------|----------|
| 🟡 | Read-Optimized State | `parking_lot::RwLock` for read-heavy | `armature-core` |

### Benchmarking

| Priority | Feature | Description | Status |
|----------|---------|-------------|--------|
| ✅ | Flame Graph CI | Auto-generate flamegraphs | `flamegraph.yml` |
| ✅ | Database Benchmark | Single/multiple query tests | `database_benchmarks.rs` |

**Implemented:**
- **Flame Graph CI**: Auto-generates SVGs on PRs, differential analysis, weekly trends
- **Database Benchmark**: TechEmpower patterns (single/multi query, fortunes, updates, async)

### Internationalization

| Priority | Feature | Description | Status |
|----------|---------|-------------|--------|
| ✅ | i18n Support | Message translation | `armature-i18n` |
| ✅ | Locale Detection | Accept-Language parsing | `locale.rs` |
| ✅ | Pluralization | Plural rules support | `plural.rs` |
| ✅ | Date/Number Formatting | Locale-aware formatting | `format.rs` |

**Implemented (`armature-i18n` crate):**
- **I18n**: Thread-safe translation with locale fallback, JSON/Fluent support
- **Locale**: BCP 47 parsing, Accept-Language negotiation, match scoring
- **Pluralization**: CLDR rules for 10+ language families (English, French, Slavic, Arabic, etc.)
- **Formatting**: Numbers, currencies, dates, percentages with locale-aware separators

---

## Summary

| Category | Remaining | Completed |
|----------|-----------|-----------|
| Performance Regressions | 1 | 2 |
| Compiler Optimizations | - | 4 |
| Buffer/Connection Tuning | 0 | 18+ |
| Streaming/Compression | 0 | 6 |
| State Management | 1 | 4 |
| Benchmarking | 0 | 9 |
| Testing & Fuzzing | - | 8 |
| Internationalization | 0 | 4 |
| Integrations | - | 3 |
| **Total** | **3** | **106** |

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
