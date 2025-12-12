//! Integration tests for armature-core

use armature_core::*;

#[test]
fn test_http_request_creation() {
    let req = HttpRequest::new("GET".to_string(), "/test".to_string());
    assert_eq!(req.method, "GET");
    assert_eq!(req.path, "/test");
    assert!(req.headers.is_empty());
    assert!(req.body.is_empty());
}

#[test]
fn test_http_response_creation() {
    let res = HttpResponse::ok();
    assert_eq!(res.status, 200);

    let res = HttpResponse::created();
    assert_eq!(res.status, 201);

    let res = HttpResponse::bad_request();
    assert_eq!(res.status, 400);

    let res = HttpResponse::not_found();
    assert_eq!(res.status, 404);

    let res = HttpResponse::internal_server_error();
    assert_eq!(res.status, 500);
}

#[test]
fn test_http_response_with_json() {
    use serde_json::json;

    let data = json!({"message": "Hello"});
    let res = HttpResponse::ok().with_json(&data).unwrap();

    assert_eq!(res.status, 200);
    assert!(res.headers.contains_key("Content-Type"));
    assert_eq!(res.headers.get("Content-Type").unwrap(), "application/json");
}

#[test]
fn test_http_status_codes() {
    assert_eq!(HttpStatus::Ok.code(), 200);
    assert_eq!(HttpStatus::Created.code(), 201);
    assert_eq!(HttpStatus::BadRequest.code(), 400);
    assert_eq!(HttpStatus::Unauthorized.code(), 401);
    assert_eq!(HttpStatus::Forbidden.code(), 403);
    assert_eq!(HttpStatus::NotFound.code(), 404);
    assert_eq!(HttpStatus::InternalServerError.code(), 500);
}

#[test]
fn test_error_conversion() {
    let err = Error::NotFound("Resource not found".to_string());
    assert_eq!(err.status_code(), 404);
    assert!(err.is_client_error());
    assert!(!err.is_server_error());

    let err = Error::InternalServerError("Server error".to_string());
    assert_eq!(err.status_code(), 500);
    assert!(!err.is_client_error());
    assert!(err.is_server_error());
}

#[test]
fn test_middleware_chain() {
    let mut chain = MiddlewareChain::new();

    // Add middlewares
    chain.add(Box::new(LoggerMiddleware::new()));
    chain.add(Box::new(CorsMiddleware::new()));

    // Chain should be created successfully
    assert_eq!(chain.len(), 2);
}

#[test]
fn test_rate_limiter_creation() {
    use std::sync::Arc;

    let config = RateLimitConfig::default();
    let store = Arc::new(InMemoryStore::new());
    let limiter = RateLimiter::new(config, store);

    // Limiter should be created
    assert!(format!("{:?}", limiter).contains("RateLimiter"));
}

#[tokio::test]
async fn test_tls_config_creation() {
    let tls = TlsConfig::self_signed().unwrap();

    // Should have cert and key
    assert!(!tls.cert.is_empty());
    assert!(tls.key.is_some());
}

#[test]
fn test_https_config() {
    let tls = TlsConfig::self_signed().unwrap();
    let https_config = HttpsConfig::new("0.0.0.0:443", tls);

    assert_eq!(https_config.addr, "0.0.0.0:443");
    assert!(https_config.http_redirect_addr.is_none());
}

#[test]
fn test_https_config_with_redirect() {
    let tls = TlsConfig::self_signed().unwrap();
    let https_config = HttpsConfig::new("0.0.0.0:443", tls).with_http_redirect("0.0.0.0:80");

    assert_eq!(
        https_config.http_redirect_addr,
        Some("0.0.0.0:80".to_string())
    );
}
