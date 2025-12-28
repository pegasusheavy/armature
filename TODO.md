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
| **Node.js FFI Bindings** | 7.5 | Expose Armature to TypeScript/Node.js via NAPI-RS | XL | ⏳ |
| **ML Model Serving** | 3.0 | ONNX/TensorFlow Lite inference endpoints | L | ⏳ |

---

## Node.js FFI Roadmap

Expose Armature's high-performance Rust core to TypeScript/Node.js developers via native bindings.

### Value Proposition

- **10-100x faster** than Express/Fastify for CPU-bound operations
- **NestJS-familiar API** for easy adoption
- **Type-safe** with auto-generated TypeScript definitions
- **Zero-copy** where possible for maximum performance

### Technology Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| FFI Layer | **NAPI-RS** | Best Node.js binding library, async support, N-API stability |
| Package | `@armature/core` | Scoped npm package |
| TypeScript | Auto-generated `.d.ts` | From Rust types via `ts-rs` or NAPI-RS |
| Runtime | Node.js 18+ | N-API v8, stable async support |

### Phase 1: Core Bindings (Effort: L)

| Task | Description | Status |
|------|-------------|--------|
| **1.1 Project Setup** | Create `armature-node` crate with NAPI-RS | ⏳ |
| **1.2 HttpRequest Binding** | Expose request object with headers, body, params | ⏳ |
| **1.3 HttpResponse Binding** | Response builder with status, headers, body | ⏳ |
| **1.4 Router Binding** | Route registration and matching | ⏳ |
| **1.5 Async Handler Support** | JS Promise → Rust Future bridging | ⏳ |

```typescript
// Target API (Phase 1)
import { Router, HttpRequest, HttpResponse } from '@armature/core';

const router = new Router();

router.get('/users/:id', async (req: HttpRequest): Promise<HttpResponse> => {
  const id = req.param('id');
  return HttpResponse.json({ id, name: 'Alice' });
});

await router.listen(3000);
```

### Phase 2: Micro-Framework API (Effort: M)

| Task | Description | Status |
|------|-------------|--------|
| **2.1 App Builder** | `App.new()` fluent builder in JS | ⏳ |
| **2.2 Route Helpers** | `get()`, `post()`, etc. as JS functions | ⏳ |
| **2.3 Middleware System** | `wrap()` with JS middleware functions | ⏳ |
| **2.4 Scope/Service** | Route grouping and nested scopes | ⏳ |
| **2.5 Data/State** | Shared state via `app.data()` | ⏳ |

```typescript
// Target API (Phase 2)
import { App, get, post, scope, Logger, Cors } from '@armature/core';

const app = App.new()
  .wrap(Logger.default())
  .wrap(Cors.permissive())
  .route('/', get(async () => HttpResponse.ok()))
  .service(
    scope('/api/v1')
      .route('/users', get(listUsers).post(createUser))
      .route('/users/:id', get(getUser))
  );

await app.run('0.0.0.0:8080');
```

### Phase 3: Advanced Features (Effort: L)

| Task | Description | Status |
|------|-------------|--------|
| **3.1 WebSocket Support** | Real-time with `@armature/websocket` | ⏳ |
| **3.2 Validation** | Schema validation via `@armature/validation` | ⏳ |
| **3.3 OpenAPI Generation** | Auto-generate OpenAPI from routes | ⏳ |
| **3.4 GraphQL** | GraphQL server via `@armature/graphql` | ⏳ |
| **3.5 Caching** | Redis/in-memory cache bindings | ⏳ |

### Phase 4: DX & Ecosystem (Effort: M)

| Task | Description | Status |
|------|-------------|--------|
| **4.1 CLI Tool** | `npx @armature/cli new my-app` | ⏳ |
| **4.2 TypeScript Plugin** | IDE support, route hints | ⏳ |
| **4.3 ESBuild Plugin** | Bundle optimization | ⏳ |
| **4.4 Vitest Integration** | Testing utilities | ⏳ |
| **4.5 npm Publishing** | CI/CD for multi-platform binaries | ⏳ |

### Technical Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    TypeScript/JavaScript                     │
│  import { App, get } from '@armature/core'                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      NAPI-RS Bridge                          │
│  - JsFunction → Rust closure conversion                     │
│  - Promise ↔ Future bridging                                │
│  - Zero-copy Buffer handling                                │
│  - ThreadsafeFunction for callbacks                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    armature-node crate                       │
│  - Thin wrapper over armature-core                          │
│  - JS-friendly error handling                               │
│  - Async runtime integration (tokio)                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      armature-core                           │
│  - Router, HttpRequest, HttpResponse                        │
│  - Middleware, State, Scopes                                │
│  - All existing Rust optimizations                          │
└─────────────────────────────────────────────────────────────┘
```

### Key Implementation Details

#### Async Handler Bridging

```rust
// armature-node/src/handler.rs
use napi::{JsFunction, Env, Result, threadsafe_function::*};
use napi_derive::napi;

#[napi]
pub struct JsHandler {
    callback: ThreadsafeFunction<HttpRequest, Promise<HttpResponse>>,
}

impl JsHandler {
    pub async fn call(&self, req: HttpRequest) -> Result<HttpResponse> {
        self.callback.call_async(req).await
    }
}
```

#### Zero-Copy Request Body

```rust
// Expose request body as Node.js Buffer without copying
#[napi]
impl HttpRequest {
    #[napi]
    pub fn body_buffer(&self, env: Env) -> Result<JsBuffer> {
        // Create Buffer view over Rust Vec<u8>
        env.create_buffer_with_borrowed_data(
            self.body.as_slice(),
            self.body.len(),
            self.body.clone(), // prevent deallocation
            |_, _| {}
        )
    }
}
```

#### Multi-Platform Binary Distribution

```yaml
# .github/workflows/node-publish.yml
strategy:
  matrix:
    include:
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
      - os: ubuntu-latest
        target: aarch64-unknown-linux-gnu
      - os: macos-latest
        target: x86_64-apple-darwin
      - os: macos-latest
        target: aarch64-apple-darwin
      - os: windows-latest
        target: x86_64-pc-windows-msvc
```

### Performance Targets

| Benchmark | Express | Fastify | Armature-Node | Goal |
|-----------|---------|---------|---------------|------|
| Hello World (req/s) | 15k | 45k | 120k+ | 3x Fastify |
| JSON serialize | 10µs | 5µs | 0.5µs | 10x faster |
| Route matching | 2µs | 0.8µs | 0.05µs | 16x faster |
| Memory per request | 50KB | 20KB | 5KB | 4x less |

### npm Package Structure

```
@armature/
├── core/           # Main package (router, app, middleware)
├── websocket/      # WebSocket support
├── graphql/        # GraphQL server
├── validation/     # Schema validation
├── cache/          # Caching (Redis, memory)
├── queue/          # Background jobs
├── cli/            # CLI tool
└── create-app/     # Project scaffolding
```

### RICE Score Calculation

- **Reach:** 9 (massive Node.js ecosystem)
- **Impact:** 3 (game-changing performance for Node devs)
- **Confidence:** 0.8 (NAPI-RS is proven, but XL effort)
- **Effort:** XL (8 person-weeks)

**Score:** (9 × 3 × 0.8) / 8 = **2.7** (but strategic value much higher)

### Dependencies

| Crate | Purpose |
|-------|---------|
| `napi` | N-API bindings |
| `napi-derive` | Proc macros for `#[napi]` |
| `napi-build` | Build script for native module |
| `tokio` | Async runtime |
| `ts-rs` | TypeScript type generation (optional) |

### Milestones

| Milestone | Target | Deliverable |
|-----------|--------|-------------|
| M1: Alpha | +4 weeks | Basic router, handlers, `npm install` works |
| M2: Beta | +8 weeks | Full micro-framework API, middleware |
| M3: RC | +12 weeks | WebSocket, validation, OpenAPI |
| M4: 1.0 | +16 weeks | Production-ready, docs, examples |

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
| Node.js Bindings | 🔶 | ❌ | ❌ | N/A |

✅ = Built-in | 🔶 = Planned/Via plugin | ❌ = Not available

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
