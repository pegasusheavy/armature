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
    use std::collections::HashMap;

    #[test]
    fn test_logging_interceptor_creation() {
        let _interceptor = LoggingInterceptor;
    }

    #[test]
    fn test_cache_interceptor_creation() {
        let interceptor = CacheInterceptor::new(60);
        assert_eq!(interceptor.ttl_seconds, 60);
    }

    #[test]
    fn test_cache_interceptor_different_ttls() {
        let i1 = CacheInterceptor::new(30);
        let i2 = CacheInterceptor::new(120);
        let i3 = CacheInterceptor::new(3600);

        assert_eq!(i1.ttl_seconds, 30);
        assert_eq!(i2.ttl_seconds, 120);
        assert_eq!(i3.ttl_seconds, 3600);
    }

    #[test]
    fn test_transform_interceptor_creation() {
        let _interceptor = TransformInterceptor::new(|res| res);
    }

    #[test]
    fn test_execution_context_creation() {
        let request = crate::HttpRequest::new("GET".to_string(), "/test".to_string());

        let context = ExecutionContext::new(request.clone());
        assert_eq!(context.request.method, "GET");
        assert_eq!(context.request.path, "/test");
    }

    #[test]
    fn test_execution_context_with_metadata() {
        let mut request = crate::HttpRequest::new("POST".to_string(), "/api/users".to_string());
        request.body = vec![1, 2, 3];

        let context = ExecutionContext::new(request.clone());
        assert_eq!(context.request.body.len(), 3);
    }

    #[test]
    fn test_cache_interceptor_zero_ttl() {
        let interceptor = CacheInterceptor::new(0);
        assert_eq!(interceptor.ttl_seconds, 0);
    }

    #[test]
    fn test_cache_interceptor_long_ttl() {
        let one_day = 86400;
        let interceptor = CacheInterceptor::new(one_day);
        assert_eq!(interceptor.ttl_seconds, one_day);
    }
}
