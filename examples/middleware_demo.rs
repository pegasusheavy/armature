#![allow(
    dead_code,
    unused_imports,
    clippy::default_constructed_unit_structs,
    clippy::needless_borrow,
    clippy::unnecessary_lazy_evaluations
)]
// Comprehensive Middleware System Example
//
// Note: This example demonstrates custom middleware implementations.
// The middleware is applied at the application level, not per-route,
// which requires a hybrid approach.

use armature::prelude::*;
use armature::{
    BodySizeLimitMiddleware, CompressionMiddleware, CorsMiddleware, Error, HttpResponse,
    LoggerMiddleware, Middleware, MiddlewareChain, RequestIdMiddleware, SecurityHeadersMiddleware,
    TimeoutMiddleware,
};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

// ========== Custom Middleware ==========

/// API Key validation middleware
struct ApiKeyMiddleware {
    valid_keys: Vec<String>,
}

impl ApiKeyMiddleware {
    fn new(keys: Vec<String>) -> Self {
        Self { valid_keys: keys }
    }
}

#[async_trait::async_trait]
impl Middleware for ApiKeyMiddleware {
    async fn handle(
        &self,
        req: HttpRequest,
        next: Box<
            dyn FnOnce(
                    HttpRequest,
                )
                    -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>
                + Send,
        >,
    ) -> Result<HttpResponse, Error> {
        if let Some(api_key) = req.headers.get("x-api-key") {
            if self.valid_keys.contains(api_key) {
                println!("✓ Valid API key: {}", api_key);
                return next(req).await;
            }
        }
        Err(Error::Unauthorized(
            "Invalid or missing API key".to_string(),
        ))
    }
}

/// Rate limiting middleware (simple in-memory version)
struct RateLimitMiddleware {
    requests_per_minute: u32,
}

impl RateLimitMiddleware {
    fn new(limit: u32) -> Self {
        Self {
            requests_per_minute: limit,
        }
    }
}

#[async_trait::async_trait]
impl Middleware for RateLimitMiddleware {
    async fn handle(
        &self,
        req: HttpRequest,
        next: Box<
            dyn FnOnce(
                    HttpRequest,
                )
                    -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>
                + Send,
        >,
    ) -> Result<HttpResponse, Error> {
        let client_ip = req
            .headers
            .get("x-forwarded-for")
            .or_else(|| req.headers.get("x-real-ip"))
            .map(|s| s.as_str())
            .unwrap_or("unknown");

        println!(
            "Rate limit check for IP: {} (limit: {}/min)",
            client_ip, self.requests_per_minute
        );
        next(req).await
    }
}

/// Request timing middleware
struct TimingMiddleware;

#[async_trait::async_trait]
impl Middleware for TimingMiddleware {
    async fn handle(
        &self,
        req: HttpRequest,
        next: Box<
            dyn FnOnce(
                    HttpRequest,
                )
                    -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>
                + Send,
        >,
    ) -> Result<HttpResponse, Error> {
        let start = std::time::Instant::now();
        let result = next(req).await;
        let duration = start.elapsed();
        println!("⏱  Request completed in {:?}", duration);
        result
    }
}

// ========== DTOs ==========

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    message: String,
    data: Option<serde_json::Value>,
}

// ========== Services ==========

#[injectable]
#[derive(Clone, Default)]
struct DataService;

impl DataService {
    fn get_data(&self) -> ApiResponse {
        ApiResponse {
            message: "Data retrieved successfully".to_string(),
            data: Some(serde_json::json!({
                "items": ["item1", "item2", "item3"]
            })),
        }
    }
}

// ========== Controllers ==========

#[controller("/api")]
#[derive(Default, Clone)]
struct ApiController;

impl ApiController {
    #[get("/public")]
    async fn get_public_data() -> Result<Json<ApiResponse>, Error> {
        Ok(Json(ApiResponse {
            message: "Public data - no middleware required".to_string(),
            data: None,
        }))
    }

    #[get("/protected")]
    async fn get_protected_data() -> Result<Json<ApiResponse>, Error> {
        let service = DataService::default();
        Ok(Json(service.get_data()))
    }

    #[get("/slow")]
    async fn get_slow_data() -> Result<HttpResponse, Error> {
        // Simulate slow operation
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        Ok(HttpResponse::ok().with_body(b"This should timeout".to_vec()))
    }

    #[post("/upload")]
    async fn upload(req: HttpRequest) -> Result<HttpResponse, Error> {
        let size = req.body.len();
        Ok(HttpResponse::ok().with_body(format!("Uploaded {} bytes", size).into_bytes()))
    }
}

// ========== Module ==========

#[module(
    providers: [DataService],
    controllers: [ApiController]
)]
#[derive(Default)]
struct AppModule;

#[tokio::main]
async fn main() {
    println!("🔧 Armature Middleware System Demo");
    println!("===================================\n");

    // Display middleware chain info
    println!("Middleware Chain:");
    println!("  1. Request ID - Assigns unique ID to each request");
    println!("  2. Logger - Logs request/response details");
    println!("  3. CORS - Handles cross-origin requests");
    println!("  4. Security Headers - Adds security headers");
    println!("  5. Compression - Compresses large responses");
    println!("  6. Timeout - Enforces request timeout (5s)");
    println!("  7. Body Size Limit - Max 1MB");
    println!("  8. API Key - Validates API key");
    println!("  9. Rate Limit - 60 requests/minute");
    println!(" 10. Timing - Measures request duration");
    println!();

    // Create middleware chain (for demonstration)
    let mut _middleware_chain = MiddlewareChain::new();
    _middleware_chain.use_middleware(RequestIdMiddleware);
    _middleware_chain.use_middleware(LoggerMiddleware::new());
    _middleware_chain.use_middleware(
        CorsMiddleware::new()
            .allow_origin("*")
            .allow_credentials(false),
    );
    _middleware_chain.use_middleware(SecurityHeadersMiddleware::new());
    _middleware_chain.use_middleware(CompressionMiddleware::new());
    _middleware_chain.use_middleware(TimeoutMiddleware::new(5));
    _middleware_chain.use_middleware(BodySizeLimitMiddleware::new(1024 * 1024));
    _middleware_chain.use_middleware(ApiKeyMiddleware::new(vec![
        "secret-key-123".to_string(),
        "admin-key-456".to_string(),
    ]));
    _middleware_chain.use_middleware(RateLimitMiddleware::new(60));
    _middleware_chain.use_middleware(TimingMiddleware);

    println!("Server running on http://localhost:3014");
    println!();
    println!("API Endpoints:");
    println!();
    println!("1. Public (no middleware):");
    println!("   curl http://localhost:3014/api/public");
    println!();
    println!("2. Protected (all middleware):");
    println!("   curl http://localhost:3014/api/protected \\");
    println!("     -H \"x-api-key: secret-key-123\"");
    println!();
    println!("3. CORS preflight:");
    println!("   curl -X OPTIONS http://localhost:3014/api/protected");
    println!();
    println!("4. Test timeout (will timeout after 5s):");
    println!("   curl http://localhost:3014/api/slow");
    println!();
    println!("5. Test body size limit:");
    println!("   curl -X POST http://localhost:3014/api/upload \\");
    println!("     -d \"$(head -c 2000 </dev/urandom | base64)\"");
    println!();

    let app = Application::create::<AppModule>().await;

    if let Err(e) = app.listen(3014).await {
        eprintln!("Server error: {}", e);
    }
}
