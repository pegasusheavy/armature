// Interceptors for transforming requests and responses

use crate::{Error, HttpRequest, HttpResponse};
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

/// Execution context passed to interceptors
pub struct ExecutionContext {
    pub request: HttpRequest,
}

impl ExecutionContext {
    pub fn new(request: HttpRequest) -> Self {
        Self { request }
    }
}

/// Interceptor trait for request/response transformation
#[async_trait]
pub trait Interceptor: Send + Sync {
    /// Intercept the request before/after handler execution
    async fn intercept(
        &self,
        context: ExecutionContext,
        next: Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>,
    ) -> Result<HttpResponse, Error>;
}

/// Logging interceptor
pub struct LoggingInterceptor;

#[async_trait]
impl Interceptor for LoggingInterceptor {
    async fn intercept(
        &self,
        context: ExecutionContext,
        next: Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>,
    ) -> Result<HttpResponse, Error> {
        let start = std::time::Instant::now();
        let method = context.request.method.clone();
        let path = context.request.path.clone();

        println!("→ {} {}", method, path);

        let result = next.await;

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

/// Transform interceptor for modifying responses
pub struct TransformInterceptor<F>
where
    F: Fn(HttpResponse) -> HttpResponse + Send + Sync,
{
    transform: F,
}

impl<F> TransformInterceptor<F>
where
    F: Fn(HttpResponse) -> HttpResponse + Send + Sync,
{
    pub fn new(transform: F) -> Self {
        Self { transform }
    }
}

#[async_trait]
impl<F> Interceptor for TransformInterceptor<F>
where
    F: Fn(HttpResponse) -> HttpResponse + Send + Sync,
{
    async fn intercept(
        &self,
        _context: ExecutionContext,
        next: Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>,
    ) -> Result<HttpResponse, Error> {
        let response = next.await?;
        Ok((self.transform)(response))
    }
}

/// Cache interceptor
pub struct CacheInterceptor {
    pub ttl_seconds: u64,
}

impl CacheInterceptor {
    pub fn new(ttl_seconds: u64) -> Self {
        Self { ttl_seconds }
    }
}

#[async_trait]
impl Interceptor for CacheInterceptor {
    async fn intercept(
        &self,
        context: ExecutionContext,
        next: Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>,
    ) -> Result<HttpResponse, Error> {
        // Simple cache implementation (in production, use a proper cache)
        let _cache_key = format!("{}:{}", context.request.method, context.request.path);

        // For now, just pass through
        // In production, check cache here
        let response = next.await?;

        // Store in cache with TTL
        // cache.set(cache_key, response.clone(), self.ttl_seconds);

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_interceptor_creation() {
        let _interceptor = LoggingInterceptor;
    }

    #[test]
    fn test_cache_interceptor_creation() {
        let interceptor = CacheInterceptor::new(60);
        assert_eq!(interceptor.ttl_seconds, 60);
    }
}
