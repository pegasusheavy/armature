#![allow(
    dead_code,
    unused_imports,
    clippy::default_constructed_unit_structs,
    clippy::needless_borrow,
    clippy::unnecessary_lazy_evaluations
)]
// Guards and Interceptors example
//
// This example wires up the framework's REAL guard system --
// `armature_core::guard::{AuthenticationGuard, RolesGuard}` applied via the
// `#[guard(...)]` attribute macro -- rather than simulating guard behavior
// with inline `if let` header checks. `/api/protected` and `/api/admin`
// below are genuinely rejected by these guards before the handler body ever
// runs; see the doc comments on each endpoint for exactly what each guard
// checks (and does not check).

use armature::prelude::*;
use armature::{guard, middleware};
use armature_core::guard::{AuthenticationGuard, RequestRoles, RolesGuard, VerifiedIdentity};
use armature_core::middleware::{Middleware, Next};
use serde::{Deserialize, Serialize};

// ========== DTOs ==========

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    content: String,
}

// ========== Services ==========

#[injectable]
#[derive(Clone, Default)]
struct DataService;

impl DataService {
    fn get_public_data(&self) -> Message {
        Message {
            content: "This is public data".to_string(),
        }
    }

    fn get_protected_data(&self) -> Message {
        Message {
            content: "This is protected data - you're authenticated!".to_string(),
        }
    }

    fn get_admin_data(&self) -> Message {
        Message {
            content: "This is admin-only data - you have admin role!".to_string(),
        }
    }
}

// ========== Authentication (stand-in for real JWT verification) ==========

/// Both guards used below fail closed: `AuthenticationGuard` admits a request
/// only when an authentication layer has attached a [`VerifiedIdentity`],
/// and `RolesGuard` only when it has attached verified [`RequestRoles`]. A
/// bare `Authorization: Bearer ...` header satisfies neither.
///
/// In a real deployment that layer is `armature_auth::JwtAuthMiddleware`,
/// which verifies a JWT's signature, expiry, issuer and audience before
/// attaching the caller's identity and roles. This example has no JWT
/// infrastructure, so `DemoAuthMiddleware` stands in for it: it accepts two
/// opaque demo tokens (from `DEMO_USER_TOKEN` / `DEMO_ADMIN_TOKEN`, or
/// random ones generated at startup and printed), compared in constant time.
/// Any other token, or no token, leaves the request unauthenticated.
///
/// It is registered on each guarded handler via `#[middleware(...)]` listed
/// *below* `#[guard(...)]`: at the function level, attribute macros are
/// expanded top-down and each wraps the *next* one's expansion, so the
/// bottom-most attribute (middleware) ends up as the outermost runtime
/// layer and runs before the guard it's stacked under.
struct DemoAuthMiddleware;

/// `name` from the environment, or a random 32-byte hex token.
fn token_from_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        let bytes: [u8; 32] = rand::random();
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    })
}

fn user_token() -> &'static str {
    static TOKEN: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    TOKEN.get_or_init(|| token_from_env("DEMO_USER_TOKEN"))
}

fn admin_token() -> &'static str {
    static TOKEN: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    TOKEN.get_or_init(|| token_from_env("DEMO_ADMIN_TOKEN"))
}

/// Constant-time comparison, so timing does not reveal matching prefixes.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[async_trait::async_trait]
impl Middleware for DemoAuthMiddleware {
    async fn handle(&self, mut req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        let presented = req
            .headers
            .get("authorization")
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::to_owned);
        if let Some(token) = presented {
            if constant_time_eq(token.as_bytes(), admin_token().as_bytes()) {
                req.extensions.insert(VerifiedIdentity::new("demo-admin"));
                req.extensions.insert(RequestRoles::new(["admin", "user"]));
            } else if constant_time_eq(token.as_bytes(), user_token().as_bytes()) {
                req.extensions.insert(VerifiedIdentity::new("demo-user"));
                req.extensions.insert(RequestRoles::new(["user"]));
            }
        }
        next(req).await
    }
}

// ========== Controllers ==========

#[controller("/api")]
#[derive(Default, Clone)]
struct DataController;

#[routes]
impl DataController {
    #[get("/public")]
    async fn public_endpoint() -> Result<HttpResponse, Error> {
        let service = DataService::default();
        HttpResponse::json(&service.get_public_data())
    }

    /// Protected by the real `AuthenticationGuard`: it admits only requests
    /// that `DemoAuthMiddleware` authenticated (a [`VerifiedIdentity`] is
    /// attached). A missing header, or a Bearer token that does not verify,
    /// never reaches this body; the guard rejects it with 401 first.
    #[get("/protected")]
    #[guard(AuthenticationGuard)]
    #[middleware(DemoAuthMiddleware)]
    // `_req` must stay an explicit parameter here (rather than being
    // dropped like the zero-arg `public_endpoint` above): `#[routes]`
    // decides the call-site argument count from this handler's signature
    // *before* `#[guard(...)]` expands, and `#[guard(...)]` injects an
    // implicit request parameter into guarded handlers that have none --
    // dropping this parameter would make the two disagree on arity.
    async fn protected_endpoint(_req: HttpRequest) -> Result<HttpResponse, Error> {
        let service = DataService::default();
        HttpResponse::json(&service.get_protected_data())
    }

    /// Protected by the real `RolesGuard`, requiring the `admin` role. See
    /// `DemoAuthMiddleware` above for how the verified roles get attached to
    /// the request before this guard evaluates them.
    #[get("/admin")]
    #[guard(RolesGuard::new(vec!["admin".to_string()]))]
    #[middleware(DemoAuthMiddleware)]
    // See `protected_endpoint` above for why `_req` stays an explicit
    // parameter on guarded handlers.
    async fn admin_endpoint(_req: HttpRequest) -> Result<HttpResponse, Error> {
        let service = DataService::default();
        HttpResponse::json(&service.get_admin_data())
    }
}

// ========== Module ==========

#[module(
    providers: [DataService],
    controllers: [DataController]
)]
#[derive(Default, Clone)]
struct AppModule;

#[tokio::main]
async fn main() {
    println!("🛡️  Armature Guards & Interceptors Example");
    println!("==========================================\n");

    if std::env::var("DEMO_USER_TOKEN").is_err() || std::env::var("DEMO_ADMIN_TOKEN").is_err() {
        println!("Demo tokens (set DEMO_USER_TOKEN / DEMO_ADMIN_TOKEN to choose them):");
        println!("  export USER_TOKEN={}", user_token());
        println!("  export ADMIN_TOKEN={}", admin_token());
        println!();
    }

    println!("Server running on http://localhost:3012");
    println!();
    println!("API Endpoints:");
    println!("  GET /api/public     - Public (no auth required)");
    println!("  GET /api/protected  - Protected (requires a verified token)");
    println!("  GET /api/admin      - Admin only (requires the admin token)");
    println!();
    println!("Example usage:");
    println!();
    println!("1. Public endpoint (no auth):");
    println!("   curl http://localhost:3012/api/public");
    println!();
    println!("2. Protected endpoint (requires auth):");
    println!("   curl http://localhost:3012/api/protected \\");
    println!("     -H \"Authorization: Bearer $USER_TOKEN\"");
    println!(
        "   (no header, or a Bearer token that does not verify, gets a 401 from AuthenticationGuard)"
    );
    println!();
    println!("3. Admin endpoint:");
    println!("   curl http://localhost:3012/api/admin \\");
    println!("     -H \"Authorization: Bearer $ADMIN_TOKEN\"");
    println!("   (the user token gets a 403 from RolesGuard: authenticated, but not admin)");
    println!();
    println!("Guards (armature_core::guard, applied via #[guard(...)]):");
    println!("  ✓ AuthenticationGuard - Requires a verified identity (/api/protected)");
    println!("  ✓ RolesGuard - Requires verified roles (/api/admin, requires \"admin\")");
    println!();

    let app = Application::create::<AppModule>().await;

    if let Err(e) = app.listen(3012).await {
        eprintln!("Server error: {}", e);
    }
}

// ========== Tests ==========
//
// Verifies -- in-process, without a human running curl -- the claim in this
// file's top doc comment: `/api/protected` and `/api/admin` are genuinely
// rejected by the real `armature_core::guard` guards before their handler
// bodies ever run.
//
// `armature_testing::TestAppBuilder::add_module` wires `AppModule`'s
// providers/controllers through the same `ProviderRegistration` /
// `ControllerRegistration` machinery `Application` itself uses, so the
// controller instances it builds are the real, `#[guard(...)]`/
// `#[middleware(...)]`-wrapped handlers -- those attributes bake the guard
// checks and middleware chain directly into the generated handler function
// body (see `armature-proc-macro/src/guard_attr.rs` and
// `middleware_attr.rs`), so they run under `TestClient`/`Router::route`
// exactly as they would under a real `Application`. (`TestAppBuilder`'s own
// doc comment notes that *struct-level* `#[guard(...)]`/`#[middleware(...)]`
// on a controller are not enforced this way; this file only uses the
// function-level form, so that caveat does not apply here.)
//
// `armature_testing::TestClient`'s `get`/`post`/etc. convenience methods
// don't expose custom headers, so requests are built directly with
// `HttpRequest::from_parts` and routed via `Router::route` to set the
// `Authorization` header these guards inspect.
#[cfg(test)]
mod tests {
    use super::*;
    use armature_testing::TestAppBuilder;
    use std::collections::HashMap;

    fn build_app() -> armature_testing::TestApp {
        TestAppBuilder::new().add_module(AppModule).build()
    }

    fn get_request(path: &str, headers: HashMap<String, String>) -> HttpRequest {
        HttpRequest::from_parts(
            "GET",
            path.to_string(),
            headers,
            Vec::new(),
            HashMap::new(),
            HashMap::new(),
        )
    }

    fn bearer(token: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), format!("Bearer {token}"));
        headers
    }

    #[tokio::test]
    async fn public_endpoint_requires_no_authentication() {
        let app = build_app();
        let response = app
            .app
            .router
            .route(get_request("/api/public", HashMap::new()))
            .await
            .expect("/api/public has no guard and must succeed");
        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn protected_endpoint_rejects_missing_authorization_header() {
        let app = build_app();
        let err = app
            .app
            .router
            .route(get_request("/api/protected", HashMap::new()))
            .await
            .expect_err(
                "AuthenticationGuard must reject a request with no Authorization header \
                 before protected_endpoint's body runs",
            );
        assert_eq!(err.status_code(), 401);
    }

    #[tokio::test]
    async fn protected_endpoint_rejects_an_unverified_bearer_token() {
        let app = build_app();
        let err = app
            .app
            .router
            .route(get_request("/api/protected", bearer("token123")))
            .await
            .expect_err("a bare Bearer header is not a verified identity");
        assert_eq!(err.status_code(), 401);
    }

    #[tokio::test]
    async fn protected_endpoint_accepts_a_verified_token() {
        let app = build_app();
        let response = app
            .app
            .router
            .route(get_request("/api/protected", bearer(user_token())))
            .await
            .expect("DemoAuthMiddleware verifies the user token, so the guard admits it");
        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn admin_endpoint_rejects_missing_authorization_header() {
        let app = build_app();
        let err = app
            .app
            .router
            .route(get_request("/api/admin", HashMap::new()))
            .await
            .expect_err(
                "RolesGuard must reject a request with no verified roles attached \
                 before admin_endpoint's body runs",
            );
        assert_eq!(err.status_code(), 403);
    }

    #[tokio::test]
    async fn admin_endpoint_rejects_authenticated_non_admin_token() {
        let app = build_app();
        let err = app
            .app
            .router
            .route(get_request("/api/admin", bearer(user_token())))
            .await
            .expect_err("the user token carries no `admin` role, so RolesGuard must reject it");
        assert_eq!(err.status_code(), 403);
    }

    #[tokio::test]
    async fn admin_endpoint_accepts_admin_token() {
        let app = build_app();
        let response = app
            .app
            .router
            .route(get_request("/api/admin", bearer(admin_token())))
            .await
            .expect("the admin token gets the verified `admin` role, which RolesGuard accepts");
        assert_eq!(response.status, 200);
    }
}
