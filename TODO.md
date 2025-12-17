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
| 🟡 | SIMD HTTP Parser | Integrate `httparse` SIMD features or `picohttpparser` | `armature-core` |
| 🟡 | Header Interning | Intern common header names to avoid allocations | `armature-core` |

### Serialization

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟠 | SIMD JSON | Add optional `simd-json` or `sonic-rs` for faster JSON | `armature-core` |
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
| 🔴 | Replace Trie with `matchit` | Use `matchit` crate (same as Axum) for route matching | `armature-core/routing.rs` |
| 🔴 | Compile-time Route Validation | Validate routes at compile time, not runtime | `armature-macro` |
| 🟠 | Route Parameter Extraction | Zero-allocation parameter extraction like Axum | `armature-core/routing.rs` |
| 🟠 | Wildcard/Catch-all Optimization | Optimize `*path` and `/*rest` patterns | `armature-core/routing.rs` |

### Zero-Cost Abstractions (Critical - Axum's strength)

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🔴 | Inline Handler Dispatch | Ensure handlers are inlined via monomorphization | `armature-core` |
| 🔴 | Remove Runtime Type Checks | Eliminate `Any` downcasting in hot paths | `armature-core/di.rs` |
| 🟠 | Const Generic Extractors | Use const generics for zero-cost extractor chains | `armature-core/extractors.rs` |
| 🟠 | Static Dispatch Middleware | Replace `Box<dyn>` with static dispatch where possible | `armature-core/middleware.rs` |

### Memory & Allocation (Axum minimizes allocations)

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🔴 | Arena Allocator for Requests | Per-request arena to batch deallocations | `armature-core` |
| 🟠 | `SmallVec` for Headers | Use `SmallVec<[_; 16]>` for typical header counts | `armature-core` |
| 🟠 | `CompactString` for Paths | Use `compact_str` for short route paths | `armature-core/routing.rs` |
| 🟠 | Pre-sized Response Buffers | Avoid reallocations during response building | `armature-core/response.rs` |
| 🟡 | Object Pool for Requests | Reuse request/response objects across connections | `armature-core` |

### Hyper Integration (Axum is thin layer over Hyper)

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🔴 | Direct Hyper Body Passthrough | Avoid wrapping/unwrapping `hyper::Body` | `armature-core` |
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
| 🔴 | TechEmpower Benchmark Suite | Implement all TechEmpower tests (JSON, DB, Fortune) | `benches/techempower/` |
| 🟠 | Automated Regression Tests | CI pipeline to catch performance regressions | `.github/workflows/` |
| 🟠 | Axum Comparison Benchmark | Side-by-side benchmark vs Axum with same routes | `benches/comparison/` |
| 🟡 | Flame Graph CI Integration | Auto-generate flamegraphs on benchmark runs | `.github/workflows/` |

### Compiler Optimizations

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟠 | Profile-Guided Optimization | Add PGO build profile | `Cargo.toml` |
| 🟠 | LTO Thin/Fat Modes | Benchmark LTO impact on binary size vs speed | `Cargo.toml` |
| 🟡 | Target-specific Tuning | Enable `-C target-cpu=native` for benchmarks | `Cargo.toml` |
| 🟡 | Codegen Units = 1 | Single codegen unit for maximum optimization | `Cargo.toml` |

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
| Performance - HTTP Parsing | 2 | 🟡 |
| Performance - Serialization | 3 | 🟠/🟡 |
| Performance - Connections | 3 | 🟡 |
| **Axum-Competitive** | | |
| ↳ Router Optimization | 4 | 🔴/🟠 |
| ↳ Zero-Cost Abstractions | 4 | 🔴/🟠 |
| ↳ Memory & Allocation | 5 | 🔴/🟠/🟡 |
| ↳ Hyper Integration | 4 | 🔴/🟠/🟡 |
| ↳ Async Runtime | 4 | 🟠/🟡 |
| ↳ Benchmark Infrastructure | 4 | 🔴/🟠/🟡 |
| ↳ Compiler Optimizations | 4 | 🟠/🟡 |
| Internationalization | 4 | 🟠/🟡 |
| **Total Remaining** | **44** | |
| **Recently Completed** | **4** | ✅ |

---

## Contributing

We welcome contributions! Each feature should:

1. Have comprehensive documentation in `docs/`
2. Include working examples in `examples/`
3. Have full test coverage
4. Follow existing code patterns
5. Update the README and website

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.
