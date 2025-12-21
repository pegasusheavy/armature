//! Tenant Middleware
//!
//! Automatic tenant resolution middleware.

use crate::resolver::TenantResolver;
use crate::tenant::TenantContext;
use armature_core::{Error, HttpRequest, HttpResponse, Middleware};
use async_trait::async_trait;
use std::sync::Arc;

/// Tenant middleware
///
/// Automatically resolves tenant from request and stores in context.
pub struct TenantMiddleware {
    resolver: Arc<dyn TenantResolver>,
    optional: bool,
}

impl TenantMiddleware {
    /// Create new tenant middleware
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use armature_tenancy::{TenantMiddleware, HeaderTenantResolver};
    /// use std::sync::Arc;
    ///
    /// let resolver = Arc::new(HeaderTenantResolver::new(store, "X-Tenant-ID"));
    /// let middleware = TenantMiddleware::new(resolver);
    /// ```
    pub fn new(resolver: Arc<dyn TenantResolver>) -> Self {
        Self {
            resolver,
            optional: false,
        }
    }

    /// Make tenant resolution optional
    ///
    /// If true, requests without valid tenant will proceed.
    /// If false, requests without valid tenant will return 401.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let middleware = TenantMiddleware::new(resolver)
    ///     .with_optional(true);
    /// ```
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }
}

#[async_trait]
impl Middleware for TenantMiddleware {
    async fn handle(
        &self,
        mut request: HttpRequest,
        next: armature_core::middleware::Next,
    ) -> Result<HttpResponse, Error> {
        // Resolve tenant
        match self.resolver.resolve(&request).await {
            Ok(tenant) => {
                // Store tenant in request context
                // In a real implementation, this would use request-local storage
                // For now, we'll use a simple approach

                // Create tenant context
                let _context = TenantContext::with_tenant(tenant.clone());

                // Store in request headers (temporary approach)
                // In production, use proper request-local storage
                request
                    .headers
                    .insert("__tenant_id".to_string(), tenant.id.clone());
                request
                    .headers
                    .insert("__tenant_name".to_string(), tenant.name.clone());

                // Continue with request
                next(request).await
            }
            Err(e) => {
                if self.optional {
                    // Continue without tenant
                    next(request).await
                } else {
                    // Return error
                    Err(Error::Unauthorized(format!(
                        "Tenant resolution failed: {}",
                        e
                    )))
                }
            }
        }
    }
}

/// Helper to extract tenant from request
///
/// Extracts tenant information stored by TenantMiddleware.
pub fn get_tenant_id(request: &HttpRequest) -> Option<String> {
    request.headers.get("__tenant_id").cloned()
}

/// Helper to extract tenant name from request
pub fn get_tenant_name(request: &HttpRequest) -> Option<String> {
    request.headers.get("__tenant_name").cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TenantError;
    use crate::tenant::Tenant;
    

    struct MockResolver {
        tenant: Option<Tenant>,
    }

    #[async_trait]
    impl TenantResolver for MockResolver {
        async fn resolve(&self, _request: &HttpRequest) -> Result<Tenant, TenantError> {
            self.tenant
                .clone()
                .ok_or_else(|| TenantError::NotFound("No tenant".to_string()))
        }
    }

    fn create_request() -> HttpRequest {
        HttpRequest::new("GET".to_string(), "/api/users".to_string())
    }

    #[tokio::test]
    async fn test_middleware_with_tenant() {
        let tenant = Tenant::new("tenant-1", "acme");
        let resolver = Arc::new(MockResolver {
            tenant: Some(tenant.clone()),
        });
        let middleware = TenantMiddleware::new(resolver);

        let request = create_request();

        let result = middleware
            .handle(
                request,
                Box::new(|req| {
                    Box::pin(async move {
                        // Check tenant was stored
                        assert_eq!(get_tenant_id(&req), Some("tenant-1".to_string()));
                        assert_eq!(get_tenant_name(&req), Some("acme".to_string()));
                        Ok(HttpResponse::ok())
                    })
                }),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_middleware_without_tenant_required() {
        let resolver = Arc::new(MockResolver { tenant: None });
        let middleware = TenantMiddleware::new(resolver).with_optional(false);

        let request = create_request();

        let result = middleware
            .handle(
                request,
                Box::new(|_req| Box::pin(async move { Ok(HttpResponse::ok()) })),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_middleware_without_tenant_optional() {
        let resolver = Arc::new(MockResolver { tenant: None });
        let middleware = TenantMiddleware::new(resolver).with_optional(true);

        let request = create_request();

        let result = middleware
            .handle(
                request,
                Box::new(|req| {
                    Box::pin(async move {
                        // No tenant should be stored
                        assert_eq!(get_tenant_id(&req), None);
                        Ok(HttpResponse::ok())
                    })
                }),
            )
            .await;

        assert!(result.is_ok());
    }
}
