# Armature Framework - Remaining TODO

Only features that are **not yet completed**.

## Legend

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

## Observability & Operations

### Metrics & Monitoring

| Priority | Feature | Description | Module |
|----------|---------|-------------|--------|
| 🟡 | Grafana Dashboards | Pre-built dashboard templates | `templates/grafana/` |

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
| Grafana Dashboards | 1 | 🟡 |
| Internationalization | 4 | 🟠/🟡 |
| **Total Remaining** | **16** | |
| **Recently Completed** | **3** | ✅ |

---

## Contributing

We welcome contributions! Each feature should:

1. Have comprehensive documentation in `docs/`
2. Include working examples in `examples/`
3. Have full test coverage
4. Follow existing code patterns
5. Update the README and website

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.
