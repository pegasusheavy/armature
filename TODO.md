# Armature Framework - TODO

## Status: Memory Profiling Findings

**113 optimizations implemented** | **5 memory issues identified** | Axum/Actix-competitive performance achieved

---

## Memory Issues Identified (December 2024)

### Priority 1: Potential Memory Leaks

| Issue | Location | Description | Fix |
|-------|----------|-------------|-----|
| **Unbounded HashMap** | `serialization_pool.rs:286` | `SizeTracker::type_sizes` can grow unbounded with many type names | Replace with LRU cache (max 256 entries) |
| **BufferHistory samples** | `connection_manager.rs:219` | `BufferHistory::samples` prunes by time but not count; can grow under burst traffic | Add hard cap (1000 samples max) |

### Priority 2: Thread-Safety Fixes (Completed ✅)

| Issue | Location | Status |
|-------|----------|--------|
| WebSocket `closed` flag race | `armature-websocket/src/client.rs:67` | ✅ Fixed with `AtomicBool` |
| Room cleanup TOCTOU race | `armature-websocket/src/room.rs:95,165` | ✅ Fixed with `remove_if()` |

### Priority 3: Optimization Opportunities

| Issue | Location | Description | Recommendation |
|-------|----------|-------------|----------------|
| **Object pool overhead** | `benches/memory_benchmarks.rs` | Mutex-based pool (19ns) slower than direct alloc (9.5ns) | Use `crossbeam::ArrayQueue` for lock-free pooling |
| **Unused AllocationCounter** | `benches/memory_benchmarks.rs:15-44` | Dead code generating warnings | Remove or integrate with DHAT |

### Memory Benchmark Results

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

### Recommended Actions

1. **`serialization_pool.rs`** - Add LRU bound to `type_sizes`:
   ```rust
   // Before
   type_sizes: HashMap<String, TypeSizeInfo>,

   // After
   type_sizes: lru::LruCache<String, TypeSizeInfo>,  // max 256
   ```

2. **`connection_manager.rs`** - Cap `BufferHistory::samples`:
   ```rust
   fn record(&mut self, size: usize, was_sufficient: bool) {
       // Existing prune by time...

       // Add: prune by count if over capacity
       while self.samples.len() >= 1000 {
           self.samples.remove(0);
       }
   }
   ```

3. **`memory_benchmarks.rs`** - Clean up dead code:
   - Remove unused `AllocationCounter` struct
   - Remove unused `ALLOC_COUNTER` static

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
| **State** | Copy-on-write, cache-line alignment, hot/cold separation, `parking_lot::RwLock` |
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

### Recently Completed

| Feature | Description | Location |
|---------|-------------|----------|
| Read-Optimized State | `parking_lot::RwLock` for read-heavy workloads | `read_state.rs` |

**`read_state.rs` provides:**
- `ReadState<T>` - General-purpose read-optimized state
- `ReadCache<K, V>` - Read-optimized concurrent HashMap
- `ReadConfig<T>` - Configuration state with change detection
- `ArcSwapState<T>` - Ultra-fast reads via Arc swapping
- Upgradeable read locks (read → write without releasing)
- Version tracking for cache invalidation
- Global statistics tracking

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
