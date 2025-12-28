# Armature Framework - TODO

## Status

**113 optimizations implemented** | Axum/Actix-competitive performance achieved

---

## Open Issues

| Priority | Issue | Location | Status |
|----------|-------|----------|--------|
| P2 | Room cleanup TOCTOU race | `room.rs:95,165` | ⏳ Pending (use `remove_if()`) |

---

## Benchmark Reference (December 2025)

| Benchmark | Time |
|-----------|------|
| Health check | 386ns |
| GET with param | 692ns |
| POST with body | 778ns |
| Route first match | 51ns |
| JSON serialize (small) | 17ns |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
