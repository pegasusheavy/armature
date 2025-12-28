# Armature Framework - TODO

## Status

**113 optimizations implemented** | Axum/Actix-competitive performance achieved

---

## Open Issues

None - all issues resolved! ✅

---

## Feature Roadmap (Product Manager Analysis)

### P0: Critical Gaps (vs Competitors)

| Feature | RICE Score | Description | Effort | Status |
|---------|------------|-------------|--------|--------|
| **HTTP/2 Support** | 8.0 | Actix/Axum support HTTP/2; required for modern deployments | M | ✅ Done |
| **Database Migrations** | 7.5 | CLI-driven migrations like `armature migrate` (NestJS, Rails pattern) | M | ⏳ |
| **OpenAPI Client Gen** | 6.0 | Generate TypeScript/Rust clients from OpenAPI spec | S | ⏳ |

### P1: High-Value Enterprise Features

| Feature | RICE Score | Description | Effort |
|---------|------------|-------------|--------|
| **Admin Dashboard Generator** | 7.2 | Auto-generate CRUD admin UI from models (like Django Admin) | L |
| **GraphQL Federation** | 6.8 | Apollo Federation for microservices architecture | M |
| **API Analytics Module** | 6.5 | Built-in usage tracking, rate limit insights, error rates | M |
| **Payment Processing** | 6.0 | Stripe, PayPal, Braintree integration module | M |

### P2: Developer Experience

| Feature | RICE Score | Description | Effort |
|---------|------------|-------------|--------|
| **Mock Server Mode** | 5.5 | `armature mock` to run API with fake data for frontend dev | S |
| **Database Seeding** | 5.0 | `armature db:seed` with factories and fixtures | S |
| **VS Code Extension** | 4.8 | Syntax highlighting, snippets, route navigation | M |
| **Interactive Docs** | 4.5 | Embedded try-it-out in generated OpenAPI docs | S |

### P3: Advanced Capabilities

| Feature | RICE Score | Description | Effort |
|---------|------------|-------------|--------|
| **HTTP/3 (QUIC)** | 4.0 | Next-gen HTTP protocol support | L |
| **File Processing Pipeline** | 3.8 | Image resize, PDF gen, format conversion | M |
| **Real-time Collaboration** | 3.5 | CRDTs/OT for collaborative features | L |
| **ML Model Serving** | 3.0 | ONNX/TensorFlow Lite inference endpoints | L |

---

## RICE Scoring Details

```
Score = (Reach × Impact × Confidence) / Effort

Reach: Users affected (1-10)
Impact: Experience improvement (0.25=minimal, 0.5=low, 1=medium, 2=high, 3=massive)
Confidence: Certainty (0.5=low, 0.8=medium, 1.0=high)
Effort: S=1, M=2, L=4, XL=8 (person-weeks)
```

### Top 3 Recommendations

1. **HTTP/2 Support** - Table stakes for production APIs. Competitors have it.
   - Reach: 9, Impact: 2, Confidence: 1.0, Effort: M(2) → **Score: 9.0**

2. **Database Migrations** - Every serious framework has this. Major DX gap.
   - Reach: 8, Impact: 2, Confidence: 0.9, Effort: M(2) → **Score: 7.2**

3. **Admin Dashboard Generator** - Massive time saver, differentiator vs Actix/Axum.
   - Reach: 6, Impact: 3, Confidence: 0.8, Effort: L(4) → **Score: 3.6**

---

## Competitive Analysis Summary

| Feature | Armature | Actix | Axum | NestJS |
|---------|----------|-------|------|--------|
| HTTP/2 | ✅ | ✅ | ✅ | ✅ |
| GraphQL | ✅ | ✅ | ✅ | ✅ |
| WebSocket | ✅ | ✅ | ✅ | ✅ |
| Built-in DI | ✅ | ❌ | ❌ | ✅ |
| Decorator Syntax | ✅ | ❌ | ❌ | ✅ |
| Database Migrations | ❌ | ❌ | ❌ | ✅ |
| Admin Generator | ❌ | ❌ | ❌ | 🔶 |
| OpenAPI | ✅ | 🔶 | 🔶 | ✅ |
| CLI Tooling | ✅ | ❌ | ❌ | ✅ |

✅ = Built-in | 🔶 = Via plugin | ❌ = Not available

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
