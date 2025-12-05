// Comprehensive Middleware System Example

use armature::prelude::*;
use armature::{
    BodySizeLimitMiddleware, CompressionMiddleware, CorsMiddleware, Error, HttpResponse,
    LoggerMiddleware, Middleware, MiddlewareChain, RequestIdMiddleware, SecurityHeadersMiddleware,
    TimeoutMiddleware,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

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
        // Check for API key in header
        if let Some(api_key) = req.headers.get("x-api-key") {
            if self.valid_keys.contains(api_key) {
                println!("✓ Valid API key: {}", api_key);
                return next(req).await;
            }
        }

        // Return 401 Unauthorized
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
        // In production: check rate limit against Redis/in-memory store
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
struct ApiController {
    data_service: DataService,
}

impl ApiController {
    fn get_public_data(&self) -> Result<Json<ApiResponse>, Error> {
        Ok(Json(ApiResponse {
            message: "Public data - no middleware required".to_string(),
            data: None,
        }))
    }

    fn get_protected_data(&self) -> Result<Json<ApiResponse>, Error> {
        Ok(Json(self.data_service.get_data()))
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

    let app = create_app_with_middleware();

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

    if let Err(e) = app.listen(3014).await {
        eprintln!("Server error: {}", e);
    }
}

fn create_app_with_middleware() -> Application {
    let container = Container::new();
    let mut router = Router::new();

    // Register services
    let data_service = DataService::default();
    container.register(data_service.clone());

    let controller = ApiController { data_service };

    // Create middleware chain
    let mut middleware_chain = MiddlewareChain::new();

    // Add middleware in order
    middleware_chain.use_middleware(RequestIdMiddleware);
    middleware_chain.use_middleware(LoggerMiddleware::new());
    middleware_chain.use_middleware(
        CorsMiddleware::new()
            .allow_origin("*")
            .allow_credentials(false),
    );
    middleware_chain.use_middleware(SecurityHeadersMiddleware::new());
    middleware_chain.use_middleware(CompressionMiddleware::new());
    middleware_chain.use_middleware(TimeoutMiddleware::new(5)); // 5 second timeout
    middleware_chain.use_middleware(BodySizeLimitMiddleware::new(1024 * 1024)); // 1MB limit
    middleware_chain.use_middleware(ApiKeyMiddleware::new(vec![
        "secret-key-123".to_string(),
        "admin-key-456".to_string(),
    ]));
    middleware_chain.use_middleware(RateLimitMiddleware::new(60));
    middleware_chain.use_middleware(TimingMiddleware);

    let middleware = Arc::new(middleware_chain);

    // Public endpoint (no middleware)
    let public_ctrl = controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/public".to_string(),
        handler: Arc::new(move |_req| {
            let ctrl = public_ctrl.clone();
            Box::pin(async move { ctrl.get_public_data()?.into_response() })
        }),
    });

    // Protected endpoint (with middleware)
    let protected_ctrl = controller.clone();
    let protected_middleware = middleware.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/protected".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = protected_ctrl.clone();
            let mw = protected_middleware.clone();
            let handler = Arc::new(move |_req: HttpRequest| {
                let ctrl = ctrl.clone();
                Box::pin(async move { ctrl.get_protected_data()?.into_response() })
                    as Pin<
                        Box<dyn std::future::Future<Output = Result<HttpResponse, Error>> + Send>,
                    >
            });
            Box::pin(async move { mw.apply(req, handler).await })
        }),
    });

    // Slow endpoint for testing timeout
    let slow_middleware = middleware.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/slow".to_string(),
        handler: Arc::new(move |req| {
            let mw = slow_middleware.clone();
            let handler = Arc::new(|_req: HttpRequest| {
                Box::pin(async move {
                    // Simulate slow operation
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    Ok(HttpResponse {
                        status: 200,
                        headers: HashMap::new(),
                        body: b"This should timeout".to_vec(),
                    })
                })
                    as Pin<
                        Box<dyn std::future::Future<Output = Result<HttpResponse, Error>> + Send>,
                    >
            });
            Box::pin(async move { mw.apply(req, handler).await })
        }),
    });

    // Upload endpoint for testing body size limit
    let upload_middleware = middleware.clone();
    router.add_route(Route {
        method: HttpMethod::POST,
        path: "/api/upload".to_string(),
        handler: Arc::new(move |req| {
            let mw = upload_middleware.clone();
            let handler = Arc::new(|req: HttpRequest| {
                Box::pin(async move {
                    let size = req.body.len();
                    Ok(HttpResponse {
                        status: 200,
                        headers: HashMap::new(),
                        body: format!("Uploaded {} bytes", size).into_bytes(),
                    })
                })
                    as Pin<
                        Box<dyn std::future::Future<Output = Result<HttpResponse, Error>> + Send>,
                    >
            });
            Box::pin(async move { mw.apply(req, handler).await })
        }),
    });

    Application::new(container, router)
}
