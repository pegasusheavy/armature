# Armature Framework - TODO

## Status

**113 optimizations implemented** | Axum/Actix-competitive performance achieved

---

## Open Issues

### Micro-Framework Performance Optimizations

Benchmark results show the micro-framework has **1.5-3x overhead** vs direct Router usage.

| Benchmark | Direct Router | Micro App | Overhead |
|-----------|---------------|-----------|----------|
| Static route | ~510ns | ~1.7µs | **3.3x** |
| Route with param | ~1.1µs | ~5.6µs | **5x** |
| JSON handler | - | ~3.7µs | - |

#### Issues to Fix

| Issue | Impact | Effort | Status |
|-------|--------|--------|--------|
| **Middleware chain rebuilt every request** | High | S | ⏳ |
| `BuiltApp::handle()` creates closures per request | | | |
| **`any()` clones handler 7 times** | Medium | S | ⏳ |
| Should take `Arc<H>` or use single BoxedHandler | | | |
| **Route registration allocates per-route** | Medium | M | ⏳ |
| Consider arena allocation for route strings | | | |
| **AppState type lookup via HashMap** | Low | S | ⏳ |
| Could use type ID directly without hashing | | | |

#### Recommended Fixes

1. **Pre-build middleware chain** - Build once in `App::build()`, not per-request
   ```rust
   // Current: Builds closure chain in handle()
   // Fix: Store pre-composed middleware in BuiltApp
   struct BuiltApp {
       middleware_chain: Arc<dyn Fn(HttpRequest) -> ...>,
   }
   ```

2. **Optimize `any()` helper** - Single clone instead of 7
   ```rust
   pub fn any<H>(handler: H) -> RouteBuilder {
       let boxed = Arc::new(BoxedHandler::new(handler.into_handler()));
       RouteBuilder::new()
           .with_shared_handler(HttpMethod::GET, boxed.clone())
           // ... etc
   }
   ```

3. **Use `SmallVec` for routes** - Avoid heap for small apps
   ```rust
   routes: SmallVec<[Route; 16]>,  // Inline up to 16 routes
   ```

---

## Feature Roadmap (Product Manager Analysis)

### P0: Critical Gaps (vs Competitors)

| Feature | RICE Score | Description | Effort | Status |
|---------|------------|-------------|--------|--------|
| **HTTP/2 Support** | 8.0 | Actix/Axum support HTTP/2; required for modern deployments | M | ✅ Done |
| **Database Migrations** | 7.5 | CLI-driven migrations like `armature migrate` (NestJS, Rails pattern) | M | ⏳ |
| **OpenAPI Client Gen** | 6.0 | Generate TypeScript/Rust clients from OpenAPI spec | S | ✅ Done |

### P1: High-Value Enterprise Features

| Feature | RICE Score | Description | Effort | Status |
|---------|------------|-------------|--------|--------|
| **Admin Dashboard Generator** | 7.2 | Auto-generate CRUD admin UI from models (like Django Admin) | L | ✅ Done |
| **GraphQL Federation** | 6.8 | Apollo Federation for microservices architecture | M | ✅ Done |
| **API Analytics Module** | 6.5 | Built-in usage tracking, rate limit insights, error rates | M | ✅ Done |
| **Payment Processing** | 6.0 | Stripe, PayPal, Braintree integration module | M | ✅ Done |

### P2: Developer Experience

| Feature | RICE Score | Description | Effort | Status |
|---------|------------|-------------|--------|--------|
| **Mock Server Mode** | 5.5 | `armature mock` to run API with fake data for frontend dev | S | ✅ Done |
| **Database Seeding** | 5.0 | `armature db:seed` with factories and fixtures | S | ⏳ |
| **VS Code Extension** | 4.8 | Syntax highlighting, snippets, route navigation | M | ⏳ |
| **Interactive Docs** | 4.5 | Embedded try-it-out in generated OpenAPI docs | S | ⏳ |

### P3: Advanced Capabilities

| Feature | RICE Score | Description | Effort | Status |
|---------|------------|-------------|--------|--------|
| **HTTP/3 (QUIC)** | 4.0 | Next-gen HTTP protocol support | L | ✅ Done |
| **File Processing Pipeline** | 3.8 | Image resize, PDF gen, format conversion | M | ✅ Done |
| **Real-time Collaboration** | 3.5 | CRDTs/OT for collaborative features | L | ✅ Done |
| **ML Model Serving** | 3.0 | ONNX/TensorFlow Lite inference endpoints | L | ⏳ |

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
| HTTP/3 | ✅ | ❌ | ❌ | ❌ |
| GraphQL | ✅ | ✅ | ✅ | ✅ |
| WebSocket | ✅ | ✅ | ✅ | ✅ |
| Built-in DI | ✅ | ❌ | ❌ | ✅ |
| Decorator Syntax | ✅ | ❌ | ❌ | ✅ |
| Micro-framework Mode | ✅ | ✅ | ✅ | ❌ |
| Database Migrations | ❌ | ❌ | ❌ | ✅ |
| Admin Generator | ✅ | ❌ | ❌ | 🔶 |
| OpenAPI | ✅ | 🔶 | 🔶 | ✅ |
| CLI Tooling | ✅ | ❌ | ❌ | ✅ |
| Payment Processing | ✅ | ❌ | ❌ | 🔶 |

✅ = Built-in | 🔶 = Via plugin | ❌ = Not available

---

## Benchmark Reference (December 2025)

### Core Framework

| Benchmark | Time |
|-----------|------|
| Health check | 386ns |
| GET with param | 692ns |
| POST with body | 778ns |
| Route first match | 51ns |
| JSON serialize (small) | 17ns |

### Micro-Framework (`armature_core::micro`)

| Benchmark | Time |
|-----------|------|
| Empty app creation | 25ns |
| App with 5 routes | 1.9-4.7µs |
| App with scope | 1.5µs |
| App with middleware | 857ns |
| Route (no middleware) | 875ns |
| Route (1 middleware) | 607ns |
| Route (3 middleware) | 1.9µs |
| Data creation | 30ns |
| Data access | <1ns |
| Data clone | 10ns |
| JSON handler | 3.7µs |
| Single route builder | 97ns |
| Multi-method builder | 525ns |
| Scope with routes | 448ns |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
