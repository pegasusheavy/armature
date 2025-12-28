# Armature Framework - TODO

## Status

**113 optimizations implemented** | Axum/Actix-competitive performance achieved

---

## Open Issues

### Priority 1: Memory Leaks (Fixed ✅)

| Issue | Location | Status |
|-------|----------|--------|
| Unbounded HashMap | `serialization_pool.rs:286` | ✅ Fixed with `LruCache` (max 256) |
| Unbounded Vec | `connection_manager.rs:238` | ✅ Fixed with hard cap (1000 samples) |

### Priority 2: Thread Safety

| Issue | Location | Status |
|-------|----------|--------|
| WebSocket `closed` race | `client.rs:67` | ✅ Fixed with `AtomicBool` |
| Room cleanup TOCTOU | `room.rs:95,165` | ⏳ Pending (use `remove_if()`) |

### Priority 3: Cleanup

| Issue | Location | Fix |
|-------|----------|-----|
| Object pool overhead | `memory_benchmarks.rs` | Use `crossbeam::ArrayQueue` |
| Dead code warnings | `memory_benchmarks.rs:15-44` | Remove `AllocationCounter` |

---

## Memory Benchmark Results

```
Allocation Patterns:
├── Vec (64B → 64KB):     5ns → 856ns
├── String (64B → 64KB):  24ns → 960ns
├── HashMap (100 entries): 4.4µs (with capacity) vs 5.2µs (without)
└── Object Pool:          19ns (2x overhead vs direct alloc)

Leak Patterns:
├── Unbounded cache:      88µs (grows without limit)
├── Bounded cache (LRU):  85µs (stable memory)
└── Weak references:      29ns (automatic cleanup)

Drop Timing:
├── Small Vec (100):      19-23ns
├── Large Vec (100K):     157-187ns
├── Nested structures:    10-11µs
└── HashMap (1000):       50-54µs
```

---

## Completed Work

### Performance (73 optimizations)

| Category | Highlights |
|----------|------------|
| Routing | `matchit` O(log n), LRU cache, static fast path |
| HTTP Parsing | SIMD via `httparse`/`memchr`, header interning |
| Serialization | `simd-json` (1.8x faster), zero-copy `Bytes`, pooled buffers |
| Memory | Arena allocator (6x faster), SmallVec, CompactString |
| Response | `LazyHeaders`, `FastResponse`, `FastBody` enum |

### Connection & I/O

| Category | Highlights |
|----------|------------|
| Pipelining | HTTP/1.1 request/response pipelining |
| I/O | `io_uring`, vectored I/O, TCP_CORK, epoll tuning |
| Buffers | Thread-local pools, zero-copy, adaptive sizing |
| Connections | Adaptive keep-alive, idle culling, branchless FSM |

### Architecture

| Category | Highlights |
|----------|------------|
| Workers | Per-worker routers, CPU affinity, NUMA-aware |
| State | Copy-on-write, cache-line alignment, `parking_lot::RwLock` |
| Runtime | Task spawning control, LocalSet, work-stealing |

### Tooling

| Category | Highlights |
|----------|------------|
| Profiling | CPU flamegraphs, memory profiling (DHAT, Valgrind) |
| Benchmarks | TechEmpower suite, framework comparisons |
| Build | PGO, LTO, release-native profiles |

### Integrations

| Crate | Description |
|-------|-------------|
| `armature-websocket` | WebSocket server/client with rooms |
| `armature-opensearch` | OpenSearch client |
| `armature-i18n` | i18n with pluralization |
| `armature-compression` | Streaming gzip/brotli/zstd |
| `armature-diesel` | Async Diesel with connection pools |
| `armature-seaorm` | SeaORM integration |

---

## Latest Benchmarks (December 2025)

| Benchmark | Time | Improvement |
|-----------|------|-------------|
| Health check | 386ns | -4% |
| GET with param | 692ns | -15% |
| POST with body | 778ns | -26% |
| Route first match | 51ns | -6% |
| Route middle | 343ns | -17% |
| JSON serialize (small) | 17ns | -14% |
| JSON serialize (large) | 14.4µs | -7% |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
