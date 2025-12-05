// Middleware system for request/response processing

use crate::{Error, HttpRequest, HttpResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Type alias for the next handler in the middleware chain
pub type Next = Box<
    dyn FnOnce(HttpRequest) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>
        + Send,
>;

/// Type alias for handler functions
pub type HandlerFn = Arc<
    dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>
        + Send
        + Sync,
>;

/// Middleware trait for processing requests before they reach the handler
#[async_trait]
pub trait Middleware: Send + Sync {
    /// Process the request and optionally pass to next middleware
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error>;
}

/// Middleware chain executor
#[derive(Clone)]
pub struct MiddlewareChain {
    middlewares: Arc<Vec<Arc<dyn Middleware>>>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self {
            middlewares: Arc::new(Vec::new()),
        }
    }

    /// Add a middleware to the chain
    pub fn use_middleware<M: Middleware + 'static>(&mut self, middleware: M) {
        let mut mws = (*self.middlewares).clone();
        mws.push(Arc::new(middleware));
        self.middlewares = Arc::new(mws);
    }

    /// Execute the middleware chain with a handler
    pub async fn apply(&self, req: HttpRequest, handler: HandlerFn) -> Result<HttpResponse, Error> {
        self.execute_from(0, req, handler).await
    }

    fn execute_from(
        &self,
        index: usize,
        req: HttpRequest,
        handler: HandlerFn,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>> {
        if index >= self.middlewares.len() {
            // No more middleware, call the handler
            handler(req)
        } else {
            let middleware = self.middlewares[index].clone();
            let chain = self.clone();
            let handler_clone = handler.clone();

            Box::pin(async move {
                middleware
                    .handle(
                        req,
                        Box::new(move |req| chain.execute_from(index + 1, req, handler_clone)),
                    )
                    .await
            })
        }
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}

// ========== Built-in Middleware ==========

/// CORS (Cross-Origin Resource Sharing) middleware
pub struct CorsMiddleware {
    pub allow_origin: String,
    pub allow_methods: String,
    pub allow_headers: String,
    pub allow_credentials: bool,
    pub max_age: u32,
}

impl CorsMiddleware {
    pub fn new() -> Self {
        Self {
            allow_origin: "*".to_string(),
            allow_methods: "GET, POST, PUT, DELETE, OPTIONS, PATCH".to_string(),
            allow_headers: "Content-Type, Authorization, Accept".to_string(),
            allow_credentials: false,
            max_age: 86400, // 24 hours
        }
    }

    pub fn allow_origin(mut self, origin: &str) -> Self {
        self.allow_origin = origin.to_string();
        self
    }

    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for CorsMiddleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        // Handle preflight requests
        if req.method == "OPTIONS" {
            let mut headers = HashMap::new();
            headers.insert(
                "Access-Control-Allow-Origin".to_string(),
                self.allow_origin.clone(),
            );
            headers.insert(
                "Access-Control-Allow-Methods".to_string(),
                self.allow_methods.clone(),
            );
            headers.insert(
                "Access-Control-Allow-Headers".to_string(),
                self.allow_headers.clone(),
            );
            headers.insert(
                "Access-Control-Max-Age".to_string(),
                self.max_age.to_string(),
            );

            if self.allow_credentials {
                headers.insert(
                    "Access-Control-Allow-Credentials".to_string(),
                    "true".to_string(),
                );
            }

            return Ok(HttpResponse {
                status: 204,
                headers,
                body: Vec::new(),
            });
        }

        // Process request and add CORS headers to response
        let mut response = next(req).await?;

        response.headers.insert(
            "Access-Control-Allow-Origin".to_string(),
            self.allow_origin.clone(),
        );
        if self.allow_credentials {
            response.headers.insert(
                "Access-Control-Allow-Credentials".to_string(),
                "true".to_string(),
            );
        }

        Ok(response)
    }
}

/// Logging middleware
pub struct LoggerMiddleware {
    pub log_body: bool,
}

impl LoggerMiddleware {
    pub fn new() -> Self {
        Self { log_body: false }
    }

    pub fn with_body(mut self) -> Self {
        self.log_body = true;
        self
    }
}

impl Default for LoggerMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for LoggerMiddleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        let start = std::time::Instant::now();
        let method = req.method.clone();
        let path = req.path.clone();

        if self.log_body && !req.body.is_empty() {
            println!("→ {} {} (body: {} bytes)", method, path, req.body.len());
        } else {
            println!("→ {} {}", method, path);
        }

        let result = next(req).await;
        let duration = start.elapsed();

        match &result {
            Ok(response) => {
                println!(
                    "← {} {} - {} ({:?})",
                    method, path, response.status, duration
                );
            }
            Err(e) => {
                println!("← {} {} - Error: {} ({:?})", method, path, e, duration);
            }
        }

        result
    }
}

/// Request ID middleware
pub struct RequestIdMiddleware;

#[async_trait]
impl Middleware for RequestIdMiddleware {
    async fn handle(&self, mut req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        // Generate or use existing request ID
        let request_id = req
            .headers
            .get("x-request-id")
            .cloned()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        req.headers
            .insert("x-request-id".to_string(), request_id.clone());

        let mut response = next(req).await?;
        response
            .headers
            .insert("x-request-id".to_string(), request_id);

        Ok(response)
    }
}

/// Body size limit middleware
pub struct BodySizeLimitMiddleware {
    max_size: usize,
}

impl BodySizeLimitMiddleware {
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }
}

#[async_trait]
impl Middleware for BodySizeLimitMiddleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        if req.body.len() > self.max_size {
            return Err(Error::PayloadTooLarge(format!(
                "Request body exceeds maximum size of {} bytes",
                self.max_size
            )));
        }

        next(req).await
    }
}

/// Timeout middleware
pub struct TimeoutMiddleware {
    duration: std::time::Duration,
}

impl TimeoutMiddleware {
    pub fn new(seconds: u64) -> Self {
        Self {
            duration: std::time::Duration::from_secs(seconds),
        }
    }
}

#[async_trait]
impl Middleware for TimeoutMiddleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        match tokio::time::timeout(self.duration, next(req)).await {
            Ok(result) => result,
            Err(_) => Err(Error::RequestTimeout(format!(
                "Request exceeded timeout of {:?}",
                self.duration
            ))),
        }
    }
}

/// Security headers middleware
pub struct SecurityHeadersMiddleware {
    hsts_enabled: bool,
    nosniff_enabled: bool,
    xss_protection_enabled: bool,
    frame_options: Option<String>,
}

impl SecurityHeadersMiddleware {
    pub fn new() -> Self {
        Self {
            hsts_enabled: true,
            nosniff_enabled: true,
            xss_protection_enabled: true,
            frame_options: Some("DENY".to_string()),
        }
    }

    pub fn with_hsts(mut self, enabled: bool) -> Self {
        self.hsts_enabled = enabled;
        self
    }

    pub fn with_frame_options(mut self, value: &str) -> Self {
        self.frame_options = Some(value.to_string());
        self
    }
}

impl Default for SecurityHeadersMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for SecurityHeadersMiddleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        let mut response = next(req).await?;

        if self.hsts_enabled {
            response.headers.insert(
                "Strict-Transport-Security".to_string(),
                "max-age=31536000; includeSubDomains".to_string(),
            );
        }

        if self.nosniff_enabled {
            response
                .headers
                .insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        }

        if self.xss_protection_enabled {
            response
                .headers
                .insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());
        }

        if let Some(frame_opts) = &self.frame_options {
            response
                .headers
                .insert("X-Frame-Options".to_string(), frame_opts.clone());
        }

        Ok(response)
    }
}

/// Compression middleware (stub - would need compression crate)
pub struct CompressionMiddleware {
    min_size: usize,
}

impl CompressionMiddleware {
    pub fn new() -> Self {
        Self { min_size: 1024 } // Only compress responses > 1KB
    }

    pub fn with_min_size(mut self, size: usize) -> Self {
        self.min_size = size;
        self
    }
}

impl Default for CompressionMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for CompressionMiddleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        let mut response = next(req).await?;

        // Check if response is large enough to compress
        if response.body.len() > self.min_size {
            response
                .headers
                .insert("X-Compression-Eligible".to_string(), "true".to_string());
            // In production: compress response.body here
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_middleware_chain() {
        let mut chain = MiddlewareChain::new();
        chain.use_middleware(LoggerMiddleware::new());

        let req = HttpRequest::new("GET".to_string(), "/test".to_string());

        let handler = Arc::new(|_req: HttpRequest| {
            Box::pin(async {
                Ok(HttpResponse {
                    status: 200,
                    headers: HashMap::new(),
                    body: Vec::new(),
                })
            }) as Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>
        });

        let result = chain.apply(req, handler).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cors_middleware() {
        let cors = CorsMiddleware::new().allow_origin("https://example.com");
        let req = HttpRequest::new("GET".to_string(), "/api".to_string());

        let result = cors
            .handle(
                req,
                Box::new(|_req| {
                    Box::pin(async {
                        Ok(HttpResponse {
                            status: 200,
                            headers: HashMap::new(),
                            body: Vec::new(),
                        })
                    })
                }),
            )
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(
            response.headers.get("Access-Control-Allow-Origin"),
            Some(&"https://example.com".to_string())
        );
    }

    #[tokio::test]
    async fn test_body_size_limit() {
        let middleware = BodySizeLimitMiddleware::new(10);
        let mut req = HttpRequest::new("POST".to_string(), "/api".to_string());
        req.body = vec![0; 20]; // 20 bytes, exceeds limit

        let result = middleware
            .handle(
                req,
                Box::new(|_req| {
                    Box::pin(async {
                        Ok(HttpResponse {
                            status: 200,
                            headers: HashMap::new(),
                            body: Vec::new(),
                        })
                    })
                }),
            )
            .await;

        assert!(result.is_err());
    }
}
