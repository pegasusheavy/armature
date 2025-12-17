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
| Grafana Dashboards | 1 | 🟡 |
| Internationalization | 4 | 🟠/🟡 |
| **Total Remaining** | **5** | |
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
