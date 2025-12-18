#![allow(clippy::needless_question_mark)]
//! Route Groups Example
//!
//! This example demonstrates how to use Route Groups to organize routes
//! with shared configuration including prefixes, middleware, and guards.
//!
//! Run with:
//! ```bash
//! cargo run --example route_groups_example
//! ```
//!
//! Test with:
//! ```bash
//! # Public API (no auth required)
//! curl http://localhost:3000/api/public/health
//!
//! # Authenticated API (requires auth)
//! curl http://localhost:3000/api/v1/users -H "Authorization: Bearer token123"
//!
//! # Admin API (requires auth + admin role)
//! curl http://localhost:3000/api/v1/admin/users -H "Authorization: Bearer token123"
//! ```

use armature_core::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let _guard = LogConfig::default().init();

    info!("Route Groups Example");
    info!("====================");

    // Create base API group with logging middleware
    let api = RouteGroup::new()
        .prefix("/api")
        .middleware(Arc::new(LoggerMiddleware));

    info!("Created base /api group with logging");

    // Public API group - no authentication required
    let public = RouteGroup::new().prefix("/public").with_parent(&api);

    info!("Created /api/public group (no auth)");

    // Authenticated API group (v1)
    let v1 = RouteGroup::new().prefix("/v1").with_parent(&api);

    info!("Created /api/v1 group");

    // Admin API group - requires authentication + admin role
    let admin = RouteGroup::new().prefix("/admin").with_parent(&v1);

    info!("Created /api/v1/admin group");

    // Demonstrate route prefix application
    info!("\nRoute Prefix Examples:");
    info!("----------------------");

    let public_health = public.apply_prefix("/health");
    info!("Public health: {}", public_health);

    let v1_users = v1.apply_prefix("/users");
    info!("V1 users: {}", v1_users);

    let admin_users = admin.apply_prefix("/users");
    info!("Admin users: {}", admin_users);

    // Show middleware inheritance
    info!("\nMiddleware Configuration:");
    info!("------------------------");
    info!(
        "Public group middleware count: {}",
        public.get_middleware().len()
    );
    info!("V1 group middleware count: {}", v1.get_middleware().len());
    info!(
        "Admin group middleware count: {}",
        admin.get_middleware().len()
    );

    // Create application with route groups
    info!("\nStarting server on http://localhost:3000");
    info!("Try these endpoints:");
    info!("  GET /api/public/health");
    info!("  GET /api/v1/users");
    info!("  GET /api/v1/admin/users");

    // Build router with grouped routes
    let mut router = Router::new();

    // Public routes
    router.add_route(Route {
        method: HttpMethod::GET,
        path: public.apply_prefix("/health"),
        handler: Arc::new(|_req| {
            Box::pin(async move {
                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "status": "healthy",
                    "group": "public"
                }))?)
            })
        }),
        constraints: None,
    });

    // V1 routes
    router.add_route(Route {
        method: HttpMethod::GET,
        path: v1.apply_prefix("/users"),
        handler: Arc::new(|_req| {
            Box::pin(async move {
                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "users": ["alice", "bob", "charlie"],
                    "version": "v1"
                }))?)
            })
        }),
        constraints: None,
    });

    router.add_route(Route {
        method: HttpMethod::GET,
        path: v1.apply_prefix("/posts"),
        handler: Arc::new(|_req| {
            Box::pin(async move {
                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "posts": [
                        {"id": 1, "title": "First Post"},
                        {"id": 2, "title": "Second Post"}
                    ],
                    "version": "v1"
                }))?)
            })
        }),
        constraints: None,
    });

    // Admin routes
    router.add_route(Route {
        method: HttpMethod::GET,
        path: admin.apply_prefix("/users"),
        handler: Arc::new(|_req| {
            Box::pin(async move {
                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "users": [
                        {"id": 1, "username": "alice", "role": "admin"},
                        {"id": 2, "username": "bob", "role": "user"},
                        {"id": 3, "username": "charlie", "role": "user"}
                    ],
                    "group": "admin",
                    "version": "v1"
                }))?)
            })
        }),
        constraints: None,
    });

    router.add_route(Route {
        method: HttpMethod::GET,
        path: admin.apply_prefix("/stats"),
        handler: Arc::new(|_req| {
            Box::pin(async move {
                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "total_users": 3,
                    "total_posts": 2,
                    "total_requests": 42,
                    "group": "admin"
                }))?)
            })
        }),
        constraints: None,
    });

    // Start server
    let container = Container::new();
    let app = Application::new(container, router);

    info!("\n✓ Server started successfully");
    info!("Press Ctrl+C to stop");

    app.listen(3000).await?;

    Ok(())
}

/// Logger middleware for demonstration
struct LoggerMiddleware;

#[async_trait::async_trait]
impl Middleware for LoggerMiddleware {
    async fn handle(
        &self,
        request: HttpRequest,
        next: middleware::Next,
    ) -> Result<HttpResponse, Error> {
        info!("→ {} {}", request.method, request.path);
        let response = next(request).await?;
        info!("← Status: {}", response.status);
        Ok(response)
    }
}
