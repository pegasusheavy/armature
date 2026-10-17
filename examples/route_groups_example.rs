#![allow(clippy::needless_question_mark)]
//! Route Groups Example
//!
//! This example demonstrates `RouteGroup` for organizing related routes under
//! shared path prefixes, middleware and guards. A group's middleware and
//! guards are **enforced** on every route registered through it
//! (`RouteGroup::route` / `RouteGroup::add_route`): each such route runs the
//! group's middleware (parents first), then its guards, then the handler.
//! They are bound to the route itself, so no spelling of the path
//! (`//api/v1/admin/users`, `%61pi/...`) reaches the handler around them.
//!
//! Groups compose with `with_parent`: `/api/v1/admin` inherits the
//! authentication middleware and guard of `/api/v1` and adds a role check.
//!
//! Run with:
//! ```bash
//! cargo run --example route_groups_example
//! ```
//!
//! The demo tokens come from `DEMO_USER_TOKEN` / `DEMO_ADMIN_TOKEN`, or are
//! generated at random and printed at startup. Test with:
//! ```bash
//! # Public API (no auth required)
//! curl http://localhost:3000/api/public/health
//!
//! # Authenticated API: the token must verify; a bare Bearer header does not.
//! curl http://localhost:3000/api/v1/users -H "Authorization: Bearer $USER_TOKEN"
//! # Missing or unverified token -> 401:
//! curl -i http://localhost:3000/api/v1/users -H "Authorization: Bearer token123"
//!
//! # Admin API: requires authentication AND the verified admin role.
//! curl http://localhost:3000/api/v1/admin/users -H "Authorization: Bearer $ADMIN_TOKEN"
//! # Authenticated but non-admin token -> 403:
//! curl -i http://localhost:3000/api/v1/admin/users -H "Authorization: Bearer $USER_TOKEN"
//! ```

use armature_core::*;
use std::sync::{Arc, OnceLock};

// ===== Authentication (stand-in for real JWT verification) =====

/// Attaches a verified identity and roles when the request presents one of
/// the demo tokens; anything else stays unauthenticated.
///
/// In a real application this is `armature_auth::JwtAuthMiddleware`, which
/// verifies a JWT before attaching the caller's `UserContext`/roles. The
/// guards below fail closed without it: a bare `Authorization: Bearer ...`
/// header never satisfies `AuthenticationGuard` or `RolesGuard`.
struct DemoAuthMiddleware;

/// `name` from the environment, or a random 32-byte hex token.
fn token_from_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        let bytes: [u8; 32] = rand::random();
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    })
}

fn user_token() -> &'static str {
    static TOKEN: OnceLock<String> = OnceLock::new();
    TOKEN.get_or_init(|| token_from_env("DEMO_USER_TOKEN"))
}

fn admin_token() -> &'static str {
    static TOKEN: OnceLock<String> = OnceLock::new();
    TOKEN.get_or_init(|| token_from_env("DEMO_ADMIN_TOKEN"))
}

/// Constant-time comparison, so timing does not reveal matching prefixes.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[async_trait::async_trait]
impl Middleware for DemoAuthMiddleware {
    async fn handle(
        &self,
        mut request: HttpRequest,
        next: middleware::Next,
    ) -> Result<HttpResponse, Error> {
        let presented = request
            .headers
            .get("authorization")
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::to_owned);
        if let Some(token) = presented {
            if constant_time_eq(token.as_bytes(), admin_token().as_bytes()) {
                request
                    .extensions
                    .insert(VerifiedIdentity::new("demo-admin"));
                request
                    .extensions
                    .insert(RequestRoles::new(["admin", "user"]));
            } else if constant_time_eq(token.as_bytes(), user_token().as_bytes()) {
                request
                    .extensions
                    .insert(VerifiedIdentity::new("demo-user"));
                request.extensions.insert(RequestRoles::new(["user"]));
            }
        }
        next(request).await
    }
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

// ===== Handlers =====

async fn health(_req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok().with_json(&serde_json::json!({
        "status": "healthy",
        "group": "public"
    }))?)
}

async fn users(_req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok().with_json(&serde_json::json!({
        "users": ["alice", "bob", "charlie"],
        "version": "v1"
    }))?)
}

async fn posts(_req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok().with_json(&serde_json::json!({
        "posts": [
            {"id": 1, "title": "First Post"},
            {"id": 2, "title": "Second Post"}
        ],
        "version": "v1"
    }))?)
}

async fn admin_users(_req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok().with_json(&serde_json::json!({
        "users": [
            {"id": 1, "username": "alice", "role": "admin"},
            {"id": 2, "username": "bob", "role": "user"},
            {"id": 3, "username": "charlie", "role": "user"}
        ],
        "group": "admin",
        "version": "v1"
    }))?)
}

async fn admin_stats(_req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::ok().with_json(&serde_json::json!({
        "total_users": 3,
        "total_posts": 2,
        "total_requests": 42,
        "group": "admin"
    }))?)
}

/// Build the router: every route is registered through its group, so the
/// group's middleware and guards run on it.
fn build_router() -> Router {
    let mut router = Router::new();

    let api = RouteGroup::new()
        .prefix("/api")
        .middleware(Arc::new(LoggerMiddleware));

    // Public: logging only.
    let public = RouteGroup::new().prefix("/public").with_parent(&api);
    public.route(&mut router, HttpMethod::GET, "/health", health);

    // V1: authentication middleware, then a guard that requires a verified
    // identity.
    let v1 = RouteGroup::new()
        .prefix("/v1")
        .middleware(Arc::new(DemoAuthMiddleware))
        .guard(Box::new(AuthenticationGuard))
        .with_parent(&api);
    v1.route(&mut router, HttpMethod::GET, "/users", users)
        .route(&mut router, HttpMethod::GET, "/posts", posts);

    // Admin: inherits v1's middleware and guard, and requires the admin role.
    let admin = RouteGroup::new()
        .prefix("/admin")
        .guard(Box::new(RolesGuard::new(vec!["admin".to_string()])))
        .with_parent(&v1);
    admin
        .route(&mut router, HttpMethod::GET, "/users", admin_users)
        .route(&mut router, HttpMethod::GET, "/stats", admin_stats);

    router
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let _guard = LogConfig::default().init();

    info!("Route Groups Example");
    info!("====================");

    if std::env::var("DEMO_USER_TOKEN").is_err() || std::env::var("DEMO_ADMIN_TOKEN").is_err() {
        println!("Demo tokens (set DEMO_USER_TOKEN / DEMO_ADMIN_TOKEN to choose them):");
        println!("  export USER_TOKEN={}", user_token());
        println!("  export ADMIN_TOKEN={}", admin_token());
    }

    let router = build_router();

    info!("\nStarting server on http://localhost:3000");
    info!("Try these endpoints:");
    info!("  GET /api/public/health          (no auth)");
    info!("  GET /api/v1/users               (requires Authorization: Bearer $USER_TOKEN)");
    info!("  GET /api/v1/posts               (requires Authorization: Bearer $USER_TOKEN)");
    info!("  GET /api/v1/admin/users         (requires Authorization: Bearer $ADMIN_TOKEN)");
    info!("  GET /api/v1/admin/stats         (requires Authorization: Bearer $ADMIN_TOKEN)");

    let app = Application::new(Container::new(), router);

    info!("\n✓ Server started successfully");
    info!("Press Ctrl+C to stop");

    app.listen(3000).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(path: &str, token: Option<&str>) -> HttpRequest {
        let mut req = HttpRequest::new("GET".to_string(), path.to_string());
        if let Some(token) = token {
            req.headers
                .insert("authorization", format!("Bearer {token}"));
        }
        req
    }

    async fn status(router: &Router, req: HttpRequest) -> u16 {
        match router.route(req).await {
            Ok(response) => response.status,
            Err(e) => e.status_code(),
        }
    }

    #[tokio::test]
    async fn group_guards_are_enforced() {
        let router = build_router();

        assert_eq!(status(&router, get("/api/public/health", None)).await, 200);

        assert_eq!(status(&router, get("/api/v1/users", None)).await, 401);
        assert_eq!(
            status(&router, get("/api/v1/users", Some("token123"))).await,
            401,
            "a bare Bearer header is not a verified identity"
        );
        assert_eq!(
            status(&router, get("/api/v1/users", Some(user_token()))).await,
            200
        );

        assert_eq!(
            status(&router, get("/api/v1/admin/users", Some(user_token()))).await,
            403
        );
        assert_eq!(
            status(&router, get("/api/v1/admin/stats", Some(admin_token()))).await,
            200
        );
    }
}
