//! Framework Comparison Benchmarks
//!
//! This benchmark suite measures Armature's performance characteristics
//! against patterns commonly benchmarked in other Rust web frameworks.
//!
//! ## Comparison Methodology
//!
//! These benchmarks measure:
//! 1. Request/Response object creation overhead
//! 2. JSON serialization/deserialization performance
//! 3. Routing performance with varying route counts
//! 4. Middleware chain processing overhead
//! 5. Dependency injection resolution speed
//! 6. Handler invocation latency
//!
//! ## Running Comparisons
//!
//! ```bash
//! cargo bench --bench framework_comparison
//! ```
//!
//! For real HTTP benchmarks, use the comparison runner:
//! ```bash
//! cargo run --bin benchmark-runner
//! ```

use armature_core::*;
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

// =============================================================================
// Test Data Structures
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SmallPayload {
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MediumPayload {
    id: u64,
    name: String,
    email: String,
    active: bool,
    created_at: String,
    tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LargePayload {
    users: Vec<UserRecord>,
    metadata: PayloadMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserRecord {
    id: u64,
    name: String,
    email: String,
    role: String,
    permissions: Vec<String>,
    profile: UserProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserProfile {
    avatar: String,
    bio: String,
    location: String,
    website: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PayloadMetadata {
    total: u64,
    page: u32,
    per_page: u32,
    pages: u32,
}

fn create_small_payload() -> SmallPayload {
    SmallPayload {
        message: "Hello, World!".to_string(),
    }
}

fn create_medium_payload() -> MediumPayload {
    MediumPayload {
        id: 12345,
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        active: true,
        created_at: "2024-01-15T10:30:00Z".to_string(),
        tags: vec![
            "developer".to_string(),
            "rust".to_string(),
            "web".to_string(),
        ],
    }
}

fn create_large_payload() -> LargePayload {
    let users: Vec<UserRecord> = (0..100)
        .map(|i| UserRecord {
            id: i,
            name: format!("User {}", i),
            email: format!("user{}@example.com", i),
            role: if i % 10 == 0 { "admin" } else { "user" }.to_string(),
            permissions: vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
            ],
            profile: UserProfile {
                avatar: format!("https://example.com/avatars/{}.png", i),
                bio: "A passionate developer working on exciting projects.".to_string(),
                location: "San Francisco, CA".to_string(),
                website: format!("https://user{}.dev", i),
            },
        })
        .collect();

    LargePayload {
        users,
        metadata: PayloadMetadata {
            total: 1000,
            page: 1,
            per_page: 100,
            pages: 10,
        },
    }
}

// =============================================================================
// Request/Response Creation Benchmarks
// =============================================================================

fn bench_request_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_creation");
    group.throughput(Throughput::Elements(1));

    // Minimal request creation
    group.bench_function("minimal", |b| {
        b.iter(|| HttpRequest::new(black_box("GET".to_string()), black_box("/".to_string())))
    });

    // Request with path parameters
    group.bench_function("with_path_params", |b| {
        b.iter(|| {
            let mut req = HttpRequest::new(
                black_box("GET".to_string()),
                black_box("/api/users/123".to_string()),
            );
            req.path_params.insert("id".to_string(), "123".to_string());
            req
        })
    });

    // Request with query parameters
    group.bench_function("with_query_params", |b| {
        b.iter(|| {
            let mut req = HttpRequest::new(
                black_box("GET".to_string()),
                black_box("/api/users".to_string()),
            );
            req.query_params.insert("page".to_string(), "1".to_string());
            req.query_params
                .insert("limit".to_string(), "10".to_string());
            req.query_params
                .insert("sort".to_string(), "-created_at".to_string());
            req
        })
    });

    // Request with headers
    group.bench_function("with_headers", |b| {
        b.iter(|| {
            let mut req = HttpRequest::new(
                black_box("POST".to_string()),
                black_box("/api/data".to_string()),
            );
            req.headers
                .insert("Content-Type".to_string(), "application/json".to_string());
            req.headers
                .insert("Authorization".to_string(), "Bearer token123".to_string());
            req.headers
                .insert("X-Request-ID".to_string(), "abc-123".to_string());
            req
        })
    });

    // Full request with body
    group.bench_function("with_json_body", |b| {
        let body = serde_json::to_vec(&create_medium_payload()).unwrap();
        b.iter(|| {
            let mut req = HttpRequest::new(
                black_box("POST".to_string()),
                black_box("/api/users".to_string()),
            );
            req.headers
                .insert("Content-Type".to_string(), "application/json".to_string());
            req.body = black_box(body.clone());
            req
        })
    });

    group.finish();
}

fn bench_response_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_creation");
    group.throughput(Throughput::Elements(1));

    // Empty OK response
    group.bench_function("empty_ok", |b| b.iter(HttpResponse::ok));

    // Response with status
    group.bench_function("status_codes", |b| {
        b.iter(|| {
            let _ok = HttpResponse::ok();
            let _created = HttpResponse::created();
            let _bad = HttpResponse::bad_request();
            let _not_found = HttpResponse::not_found();
            let _internal = HttpResponse::internal_server_error();
        })
    });

    // Response with JSON body
    let small = create_small_payload();
    group.bench_function("with_json_small", |b| {
        b.iter(|| HttpResponse::ok().with_json(&small))
    });

    let medium = create_medium_payload();
    group.bench_function("with_json_medium", |b| {
        b.iter(|| HttpResponse::ok().with_json(&medium))
    });

    let large = create_large_payload();
    group.bench_function("with_json_large", |b| {
        b.iter(|| HttpResponse::ok().with_json(&large))
    });

    // Response with headers
    group.bench_function("with_headers", |b| {
        b.iter(|| {
            HttpResponse::ok()
                .with_header("X-Request-ID".to_string(), "abc-123".to_string())
                .with_header("Cache-Control".to_string(), "no-cache".to_string())
                .with_header("X-Response-Time".to_string(), "50ms".to_string())
        })
    });

    group.finish();
}

// =============================================================================
// JSON Serialization/Deserialization Benchmarks
// =============================================================================

fn bench_json_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_operations");

    // Small payload serialization
    let small = create_small_payload();
    group.bench_function("serialize_small", |b| {
        b.iter(|| serde_json::to_vec(black_box(&small)))
    });

    // Medium payload serialization
    let medium = create_medium_payload();
    group.bench_function("serialize_medium", |b| {
        b.iter(|| serde_json::to_vec(black_box(&medium)))
    });

    // Large payload serialization
    let large = create_large_payload();
    group.bench_function("serialize_large", |b| {
        b.iter(|| serde_json::to_vec(black_box(&large)))
    });

    // Small payload deserialization
    let small_bytes = serde_json::to_vec(&small).unwrap();
    group.bench_function("deserialize_small", |b| {
        b.iter(|| serde_json::from_slice::<SmallPayload>(black_box(&small_bytes)))
    });

    // Medium payload deserialization
    let medium_bytes = serde_json::to_vec(&medium).unwrap();
    group.bench_function("deserialize_medium", |b| {
        b.iter(|| serde_json::from_slice::<MediumPayload>(black_box(&medium_bytes)))
    });

    // Large payload deserialization
    let large_bytes = serde_json::to_vec(&large).unwrap();
    group.bench_function("deserialize_large", |b| {
        b.iter(|| serde_json::from_slice::<LargePayload>(black_box(&large_bytes)))
    });

    group.finish();
}

// =============================================================================
// Routing Benchmarks
// =============================================================================

fn generate_routes(count: usize) -> Vec<(String, String)> {
    let methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];
    let resources = [
        "users",
        "posts",
        "comments",
        "articles",
        "products",
        "orders",
        "customers",
        "invoices",
    ];

    let mut routes = Vec::with_capacity(count);

    for i in 0..count {
        let method = methods[i % methods.len()].to_string();
        let resource = resources[i % resources.len()];
        let path = match i % 4 {
            0 => format!("/api/{}", resource),
            1 => format!("/api/{}/:id", resource),
            2 => format!("/api/v1/{}/:id/related", resource),
            3 => format!("/api/v2/{}/nested/:id/deep", resource),
            _ => unreachable!(),
        };
        routes.push((method, path));
    }

    routes
}

fn bench_routing_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing");

    for route_count in [10, 50, 100, 500].iter() {
        let routes = generate_routes(*route_count);

        // Build router
        let mut router = Router::new();
        for (method, path) in &routes {
            let method = match method.as_str() {
                "GET" => HttpMethod::GET,
                "POST" => HttpMethod::POST,
                "PUT" => HttpMethod::PUT,
                "DELETE" => HttpMethod::DELETE,
                "PATCH" => HttpMethod::PATCH,
                _ => HttpMethod::GET,
            };
            let handler: HandlerFn = Arc::new(|_req| Box::pin(async { Ok(HttpResponse::ok()) }));
            router.add_route(Route {
                method,
                path: path.clone(),
                handler,
                constraints: None,
            });
        }

        // Benchmark route matching - first route (best case)
        let first_path = &routes[0].1.replace(":id", "123");
        group.bench_with_input(
            BenchmarkId::new("match_first", route_count),
            route_count,
            |b, _| {
                b.iter(|| {
                    router.match_route(black_box("GET"), black_box(first_path));
                })
            },
        );

        // Benchmark route matching - middle route
        let middle_idx = routes.len() / 2;
        let middle_path = &routes[middle_idx].1.replace(":id", "456");
        let middle_method = &routes[middle_idx].0;
        group.bench_with_input(
            BenchmarkId::new("match_middle", route_count),
            route_count,
            |b, _| {
                b.iter(|| {
                    router.match_route(black_box(middle_method), black_box(middle_path));
                })
            },
        );

        // Benchmark route matching - last route (worst case)
        let last_idx = routes.len() - 1;
        let last_path = &routes[last_idx].1.replace(":id", "789");
        let last_method = &routes[last_idx].0;
        group.bench_with_input(
            BenchmarkId::new("match_last", route_count),
            route_count,
            |b, _| {
                b.iter(|| {
                    router.match_route(black_box(last_method), black_box(last_path));
                })
            },
        );

        // Benchmark route not found
        group.bench_with_input(
            BenchmarkId::new("match_not_found", route_count),
            route_count,
            |b, _| {
                b.iter(|| {
                    router.match_route(black_box("GET"), black_box("/nonexistent/path"));
                })
            },
        );
    }

    group.finish();
}

// =============================================================================
// Middleware Benchmarks
// =============================================================================

fn bench_middleware_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("middleware_creation");

    group.bench_function("logger", |b| b.iter(LoggerMiddleware::new));

    group.bench_function("cors", |b| b.iter(CorsMiddleware::new));

    group.bench_function("request_id", |b| {
        b.iter(|| {
            let id = Uuid::new_v4().to_string();
            black_box(id);
        })
    });

    group.finish();
}

// =============================================================================
// Dependency Injection Benchmarks
// =============================================================================

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct SimpleService {
    name: String,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct ComplexService {
    id: u64,
    name: String,
    config: HashMap<String, String>,
}

fn bench_dependency_injection(c: &mut Criterion) {
    let mut group = c.benchmark_group("dependency_injection");

    // Container creation
    group.bench_function("container_new", |b| b.iter(Container::new));

    // Simple service registration
    group.bench_function("register_simple", |b| {
        b.iter(|| {
            let container = Container::new();
            container.singleton(SimpleService {
                name: "test".to_string(),
            });
            black_box(container);
        })
    });

    // Complex service registration
    group.bench_function("register_complex", |b| {
        b.iter(|| {
            let container = Container::new();
            let mut config = HashMap::new();
            config.insert("key1".to_string(), "value1".to_string());
            config.insert("key2".to_string(), "value2".to_string());

            container.singleton(ComplexService {
                id: 1,
                name: "complex".to_string(),
                config,
            });
            black_box(container);
        })
    });

    // Service resolution
    let container = Container::new();
    container.singleton(SimpleService {
        name: "test".to_string(),
    });

    group.bench_function("resolve_simple", |b| {
        b.iter(|| {
            let service = container.get::<SimpleService>();
            let _ = black_box(service);
        })
    });

    // Multiple services resolution
    let multi_container = Container::new();
    multi_container.singleton(SimpleService {
        name: "simple".to_string(),
    });

    let mut config = HashMap::new();
    config.insert("key".to_string(), "value".to_string());
    multi_container.singleton(ComplexService {
        id: 1,
        name: "complex".to_string(),
        config,
    });

    group.bench_function("resolve_multiple", |b| {
        b.iter(|| {
            let _s1 = multi_container.get::<SimpleService>();
            let _s2 = multi_container.get::<ComplexService>();
        })
    });

    // Scoped container
    group.bench_function("scoped_resolution", |b| {
        b.iter(|| {
            let scoped = multi_container.scope();
            let _service = scoped.get::<SimpleService>();
            black_box(scoped);
        })
    });

    group.finish();
}

// =============================================================================
// Handler Invocation Benchmarks
// =============================================================================

fn bench_handler_invocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("handler_invocation");
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Sync-like handler (minimal async)
    let simple_handler: HandlerFn = Arc::new(|_req| Box::pin(async { Ok(HttpResponse::ok()) }));

    group.bench_function("simple_handler", |b| {
        let handler = simple_handler.clone();
        b.iter(|| {
            let h = handler.clone();
            runtime.block_on(async move {
                let req = HttpRequest::new("GET".to_string(), "/".to_string());
                let _ = h(black_box(req)).await;
            })
        })
    });

    // Handler with JSON response
    let json_handler: HandlerFn = Arc::new(|_req| {
        Box::pin(async {
            let data = create_small_payload();
            Ok(HttpResponse::ok().with_json(&data)?)
        })
    });

    group.bench_function("json_handler", |b| {
        let handler = json_handler.clone();
        b.iter(|| {
            let h = handler.clone();
            runtime.block_on(async move {
                let req = HttpRequest::new("GET".to_string(), "/".to_string());
                let _ = h(black_box(req)).await;
            })
        })
    });

    // Handler with path parameter extraction
    let param_handler: HandlerFn = Arc::new(|req| {
        Box::pin(async move {
            let id = req.path_params.get("id").cloned().unwrap_or_default();
            Ok(HttpResponse::ok().with_json(&serde_json::json!({ "id": id }))?)
        })
    });

    group.bench_function("param_handler", |b| {
        let handler = param_handler.clone();
        b.iter(|| {
            let h = handler.clone();
            runtime.block_on(async move {
                let mut req = HttpRequest::new("GET".to_string(), "/users/123".to_string());
                req.path_params.insert("id".to_string(), "123".to_string());
                let _ = h(black_box(req)).await;
            })
        })
    });

    // Handler with JSON body parsing
    let body_handler: HandlerFn = Arc::new(|req| {
        Box::pin(async move {
            let _payload: MediumPayload = req.json()?;
            Ok(HttpResponse::ok().with_json(&serde_json::json!({ "status": "received" }))?)
        })
    });

    let body = serde_json::to_vec(&create_medium_payload()).unwrap();
    group.bench_function("body_handler", |b| {
        let handler = body_handler.clone();
        let body = body.clone();
        b.iter(|| {
            let h = handler.clone();
            let b = body.clone();
            runtime.block_on(async move {
                let mut req = HttpRequest::new("POST".to_string(), "/api/data".to_string());
                req.body = b;
                let _ = h(black_box(req)).await;
            })
        })
    });

    group.finish();
}

// =============================================================================
// Complete Request/Response Cycle Benchmarks
// =============================================================================

fn bench_full_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_cycle");
    group.measurement_time(Duration::from_secs(10));

    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Setup router
    let mut router = Router::new();

    // Simple endpoint
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/health".to_string(),
        handler: Arc::new(|_req| {
            Box::pin(async {
                Ok(HttpResponse::ok().with_json(&serde_json::json!({"status": "ok"}))?)
            })
        }),
        constraints: None,
    });

    // JSON endpoint
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/users/:id".to_string(),
        handler: Arc::new(|req| {
            Box::pin(async move {
                let id = req.path_params.get("id").cloned().unwrap_or_default();
                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "id": id,
                    "name": "John Doe",
                    "email": "john@example.com"
                }))?)
            })
        }),
        constraints: None,
    });

    // POST endpoint
    router.add_route(Route {
        method: HttpMethod::POST,
        path: "/api/users".to_string(),
        handler: Arc::new(|req| {
            Box::pin(async move {
                let _payload: MediumPayload = req.json()?;
                Ok(HttpResponse::created().with_json(&serde_json::json!({
                    "status": "created",
                    "id": 12345
                }))?)
            })
        }),
        constraints: None,
    });

    // Health check
    group.bench_function("health_check", |b| {
        let router = router.clone();
        b.iter(|| {
            let r = router.clone();
            runtime.block_on(async move {
                let req = HttpRequest::new("GET".to_string(), "/health".to_string());
                let _ = r.route(black_box(req)).await;
            })
        })
    });

    // GET with path param
    group.bench_function("get_with_param", |b| {
        let router = router.clone();
        b.iter(|| {
            let r = router.clone();
            runtime.block_on(async move {
                let req = HttpRequest::new("GET".to_string(), "/api/users/123".to_string());
                let _ = r.route(black_box(req)).await;
            })
        })
    });

    // POST with body
    let body = serde_json::to_vec(&create_medium_payload()).unwrap();
    group.bench_function("post_with_body", |b| {
        let router = router.clone();
        let body = body.clone();
        b.iter(|| {
            let r = router.clone();
            let b = body.clone();
            runtime.block_on(async move {
                let mut req = HttpRequest::new("POST".to_string(), "/api/users".to_string());
                req.headers
                    .insert("Content-Type".to_string(), "application/json".to_string());
                req.body = b;
                let _ = r.route(black_box(req)).await;
            })
        })
    });

    group.finish();
}

// =============================================================================
// Memory Allocation Benchmarks
// =============================================================================

fn bench_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocations");

    // String allocations
    group.bench_function("string_small", |b| {
        b.iter(|| {
            let s: String = black_box("hello").to_string();
            black_box(s);
        })
    });

    group.bench_function("string_medium", |b| {
        b.iter(|| {
            let s: String =
                black_box("This is a medium length string for testing allocations").to_string();
            black_box(s);
        })
    });

    // Vec allocations
    group.bench_function("vec_small", |b| {
        b.iter(|| {
            let v: Vec<u8> = black_box(vec![1, 2, 3, 4, 5]);
            black_box(v);
        })
    });

    group.bench_function("vec_large", |b| {
        b.iter(|| {
            let v: Vec<u8> = black_box((0..1000).map(|i| i as u8).collect());
            black_box(v);
        })
    });

    // HashMap allocations
    group.bench_function("hashmap_small", |b| {
        b.iter(|| {
            let mut m = HashMap::new();
            m.insert(black_box("key1"), black_box("value1"));
            m.insert(black_box("key2"), black_box("value2"));
            black_box(m);
        })
    });

    group.bench_function("hashmap_large", |b| {
        b.iter(|| {
            let mut m = HashMap::new();
            for i in 0..100 {
                m.insert(format!("key{}", i), format!("value{}", i));
            }
            black_box(m);
        })
    });

    group.finish();
}

// =============================================================================
// Criterion Configuration
// =============================================================================

criterion_group! {
    name = framework_comparison;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(1));
    targets =
        bench_request_creation,
        bench_response_creation,
        bench_json_operations,
        bench_routing_performance,
        bench_middleware_creation,
        bench_dependency_injection,
        bench_handler_invocation,
        bench_full_cycle,
        bench_allocations
}

criterion_main!(framework_comparison);
