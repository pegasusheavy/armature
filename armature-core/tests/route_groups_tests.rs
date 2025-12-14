//! Integration tests for Route Groups

use armature_core::*;
use std::sync::Arc;

#[test]
fn test_route_group_prefix() {
    let group = RouteGroup::new().prefix("/api/v1");

    assert_eq!(group.get_prefix(), "/api/v1");
    assert_eq!(group.apply_prefix("/users"), "/api/v1/users");
    assert_eq!(group.apply_prefix("users"), "/api/v1/users");
    assert_eq!(group.apply_prefix(""), "/api/v1");
}

#[test]
fn test_route_group_prefix_normalization() {
    // No leading slash
    let group1 = RouteGroup::new().prefix("api/v1");
    assert_eq!(group1.get_prefix(), "/api/v1");

    // Trailing slash
    let group2 = RouteGroup::new().prefix("/api/v1/");
    assert_eq!(group2.get_prefix(), "/api/v1");

    // Both
    let group3 = RouteGroup::new().prefix("api/v1/");
    assert_eq!(group3.get_prefix(), "/api/v1");
}

#[test]
fn test_route_group_empty_prefix() {
    let group = RouteGroup::new();

    assert_eq!(group.get_prefix(), "");
    assert_eq!(group.apply_prefix("/users"), "/users");
    assert_eq!(group.apply_prefix("users"), "users");
}

#[test]
fn test_route_group_with_middleware() {
    let group = RouteGroup::new()
        .middleware(Arc::new(TestMiddleware));

    assert_eq!(group.get_middleware().len(), 1);
}

#[test]
fn test_route_group_with_multiple_middleware() {
    let middleware_stack = vec![
        Arc::new(TestMiddleware) as Arc<dyn Middleware>,
        Arc::new(TestMiddleware) as Arc<dyn Middleware>,
    ];

    let group = RouteGroup::new()
        .with_middleware(middleware_stack);

    assert_eq!(group.get_middleware().len(), 2);
}

#[test]
fn test_route_group_with_guards() {
    let group = RouteGroup::new()
        .guard(Box::new(TestGuard));

    assert_eq!(group.get_guards().len(), 1);
}

#[test]
fn test_route_group_with_multiple_guards() {
    let guards = vec![
        Box::new(TestGuard) as Box<dyn Guard>,
        Box::new(TestGuard) as Box<dyn Guard>,
    ];

    let group = RouteGroup::new()
        .with_guards(guards);

    assert_eq!(group.get_guards().len(), 2);
}

#[test]
fn test_route_group_with_parent() {
    let parent = RouteGroup::new()
        .prefix("/api")
        .middleware(Arc::new(TestMiddleware));

    let child = RouteGroup::new()
        .prefix("/v1")
        .with_parent(&parent);

    // Prefix should be combined
    assert_eq!(child.get_prefix(), "/api/v1");

    // Middleware should be inherited
    assert_eq!(child.get_middleware().len(), 1);
}

#[test]
fn test_route_group_nested_parents() {
    let root = RouteGroup::new()
        .prefix("/api");

    let v1 = RouteGroup::new()
        .prefix("/v1")
        .with_parent(&root);

    let admin = RouteGroup::new()
        .prefix("/admin")
        .with_parent(&v1);

    assert_eq!(root.get_prefix(), "/api");
    assert_eq!(v1.get_prefix(), "/api/v1");
    assert_eq!(admin.get_prefix(), "/api/v1/admin");
}

#[test]
fn test_route_group_apply_prefix_variations() {
    let group = RouteGroup::new().prefix("/api/v1");

    // With leading slash
    assert_eq!(group.apply_prefix("/users"), "/api/v1/users");

    // Without leading slash
    assert_eq!(group.apply_prefix("users"), "/api/v1/users");

    // Empty path
    assert_eq!(group.apply_prefix(""), "/api/v1");

    // Just slash
    assert_eq!(group.apply_prefix("/"), "/api/v1");

    // Nested path
    assert_eq!(group.apply_prefix("/users/123/posts"), "/api/v1/users/123/posts");
}

#[test]
fn test_route_group_middleware_chaining() {
    let group = RouteGroup::new()
        .middleware(Arc::new(TestMiddleware))
        .middleware(Arc::new(TestMiddleware))
        .middleware(Arc::new(TestMiddleware));

    assert_eq!(group.get_middleware().len(), 3);
}

#[test]
fn test_route_group_guard_chaining() {
    let group = RouteGroup::new()
        .guard(Box::new(TestGuard))
        .guard(Box::new(TestGuard))
        .guard(Box::new(TestGuard));

    assert_eq!(group.get_guards().len(), 3);
}

#[test]
fn test_route_group_complete_configuration() {
    let group = RouteGroup::new()
        .prefix("/api/v1")
        .middleware(Arc::new(TestMiddleware))
        .middleware(Arc::new(TestMiddleware))
        .guard(Box::new(TestGuard));

    assert_eq!(group.get_prefix(), "/api/v1");
    assert_eq!(group.get_middleware().len(), 2);
    assert_eq!(group.get_guards().len(), 1);
}

#[test]
fn test_route_group_parent_inherits_middleware() {
    let parent = RouteGroup::new()
        .middleware(Arc::new(TestMiddleware))
        .middleware(Arc::new(TestMiddleware));

    let child = RouteGroup::new()
        .middleware(Arc::new(TestMiddleware))
        .with_parent(&parent);

    // Child should have parent middleware + its own
    assert_eq!(child.get_middleware().len(), 3);
}

#[test]
fn test_route_group_clone() {
    let group1 = RouteGroup::new()
        .prefix("/api/v1")
        .middleware(Arc::new(TestMiddleware));

    let group2 = group1.clone();

    assert_eq!(group1.get_prefix(), group2.get_prefix());
    assert_eq!(group1.get_middleware().len(), group2.get_middleware().len());
}

// Test helpers
struct TestMiddleware;

#[async_trait::async_trait]
impl Middleware for TestMiddleware {
    async fn handle(
        &self,
        request: HttpRequest,
        next: middleware::Next,
    ) -> Result<HttpResponse, Error> {
        next(request).await
    }
}

struct TestGuard;

#[async_trait::async_trait]
impl Guard for TestGuard {
    async fn can_activate(&self, _context: &GuardContext) -> Result<bool, Error> {
        Ok(true)
    }
}

