//! The attribute macros documented as `use armature::{...}` resolve through
//! the facade crate, not only through `armature_proc_macro`.

use armature::{
    AuthenticationGuard, Error, HttpRequest, HttpResponse, VerifiedIdentity, guard, middleware,
    use_guard, use_middleware,
};
use armature_core::middleware::{Middleware, Next};

#[derive(Default)]
struct Passthrough;

#[async_trait::async_trait]
impl Middleware for Passthrough {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        next(req).await
    }
}

#[use_guard(AuthenticationGuard)]
async fn by_type(req: HttpRequest) -> Result<HttpResponse, Error> {
    let _ = &req;
    Ok(HttpResponse::ok())
}

#[guard(AuthenticationGuard)]
async fn by_instance(req: HttpRequest) -> Result<HttpResponse, Error> {
    let _ = &req;
    Ok(HttpResponse::ok())
}

#[use_middleware(Passthrough)]
async fn with_middleware_type(req: HttpRequest) -> Result<HttpResponse, Error> {
    let _ = &req;
    Ok(HttpResponse::ok())
}

#[middleware(Passthrough)]
async fn with_middleware_instance(req: HttpRequest) -> Result<HttpResponse, Error> {
    let _ = &req;
    Ok(HttpResponse::ok())
}

fn verified() -> HttpRequest {
    let mut req = HttpRequest::new("GET", "/me".to_string());
    req.extensions.insert(VerifiedIdentity::new("user-1"));
    req
}

#[tokio::test]
async fn guard_macros_from_the_facade_fail_closed() {
    assert!(
        by_type(HttpRequest::new("GET", "/me".to_string()))
            .await
            .is_err()
    );
    assert!(
        by_instance(HttpRequest::new("GET", "/me".to_string()))
            .await
            .is_err()
    );
    assert_eq!(by_type(verified()).await.unwrap().status, 200);
    assert_eq!(by_instance(verified()).await.unwrap().status, 200);
}

#[tokio::test]
async fn middleware_macros_from_the_facade_run_the_handler() {
    let req = || HttpRequest::new("GET", "/".to_string());
    assert_eq!(with_middleware_type(req()).await.unwrap().status, 200);
    assert_eq!(with_middleware_instance(req()).await.unwrap().status, 200);
}
