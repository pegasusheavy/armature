//! Advanced Security Example
//!
//! Demonstrates advanced security features:
//! - Granular CORS configuration
//! - Content Security Policy
//! - HSTS (HTTP Strict Transport Security)
//! - Request signing with HMAC
//!
//! Run with:
//! ```bash
//! cargo run --example security_advanced
//! ```
//!
//! Test with:
//! ```bash
//! # Test CORS preflight
//! curl -X OPTIONS http://localhost:3000/api/users \
//!   -H "Origin: https://app.example.com" \
//!   -H "Access-Control-Request-Method: POST"
//!
//! # Test CORS with credentials
//! curl http://localhost:3000/api/users \
//!   -H "Origin: https://app.example.com"
//!
//! # Test signed request
//! # (requires signature generation - see code below)
//! ```

use armature_core::*;
use armature_security::content_security_policy::CspConfig;
use armature_security::cors::CorsConfig;
use armature_security::hsts::HstsConfig;
use armature_security::request_signing::RequestSigner;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let _guard = LogConfig::default().init();

    info!("Advanced Security Example");
    info!("=========================");

    // Configure CORS with granular control
    info!("\nConfiguring CORS...");
    let cors = CorsConfig::new()
        // Allow specific origins
        .allow_origin("https://app.example.com")
        .allow_origin("https://admin.example.com")
        // Allow regex pattern for subdomains
        .allow_origin_regex(r"https://.*\.mydomain\.com")
        .unwrap()
        // Allowed methods
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
        // Allowed headers
        .allow_headers(vec![
            "Content-Type",
            "Authorization",
            "X-Requested-With",
            "X-Custom-Header",
        ])
        // Exposed headers (visible to browser)
        .expose_headers(vec!["X-Total-Count", "X-Page-Number", "X-Request-Id"])
        // Allow credentials (cookies, auth headers)
        .allow_credentials(true)
        // Preflight cache for 2 hours
        .max_age(7200);

    info!("✓ CORS configured:");
    info!("  - Origins: app.example.com, admin.example.com, *.mydomain.com");
    info!("  - Methods: GET, POST, PUT, DELETE, PATCH");
    info!("  - Credentials: Allowed");
    info!("  - Max age: 2 hours");

    // Configure Content Security Policy
    info!("\nConfiguring CSP...");
    let _csp = CspConfig::new()
        .default_src(vec!["'self'".to_string()])
        .script_src(vec![
            "'self'".to_string(),
            "https://cdn.example.com".to_string(),
        ])
        .style_src(vec![
            "'self'".to_string(),
            "'unsafe-inline'".to_string(), // For inline styles (use nonce in production)
            "https://fonts.googleapis.com".to_string(),
        ])
        .img_src(vec![
            "'self'".to_string(),
            "data:".to_string(),
            "https:".to_string(),
        ])
        .font_src(vec![
            "'self'".to_string(),
            "https://fonts.gstatic.com".to_string(),
        ])
        .connect_src(vec![
            "'self'".to_string(),
            "https://api.example.com".to_string(),
        ]);

    info!("✓ CSP configured with secure defaults");

    // Configure HSTS
    info!("\nConfiguring HSTS...");
    let _hsts = HstsConfig::new(31536000) // 1 year
        .include_subdomains(true)
        .preload(true);

    info!("✓ HSTS configured:");
    info!("  - Max age: 1 year");
    info!("  - Include subdomains: Yes");
    info!("  - Preload: Yes");

    // Configure request signing
    info!("\nConfiguring request signing...");
    let signing_secret = "super-secret-key-change-in-production";

    info!("✓ Request signing configured:");
    info!("  - Algorithm: HMAC-SHA256");
    info!("  - Max age: 5 minutes");
    info!("  - Skipped paths: /health, /metrics");

    // Create router
    let mut router = Router::new();

    // OPTIONS handler for CORS preflight
    let cors_clone = cors.clone();
    router.add_route(Route {
        method: HttpMethod::OPTIONS,
        path: "/api/users".to_string(),
        handler: Arc::new(move |req| {
            let cors = cors_clone.clone();
            Box::pin(async move { cors.handle_preflight(&req) })
        }),
        constraints: None,
    });

    // API endpoint with CORS
    let cors_clone2 = cors.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/users".to_string(),
        handler: Arc::new(move |req| {
            let cors = cors_clone2.clone();
            Box::pin(async move {
                let response = HttpResponse::ok().with_json(&serde_json::json!({
                    "users": [
                        {"id": 1, "name": "Alice"},
                        {"id": 2, "name": "Bob"}
                    ]
                }))?;

                // Apply CORS headers
                Ok(cors.apply(&req, response))
            })
        }),
        constraints: None,
    });

    // Signed request endpoint
    router.add_route(Route {
        method: HttpMethod::POST,
        path: "/api/secure".to_string(),
        handler: Arc::new(|_req| {
            Box::pin(async move {
                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "message": "Securely accessed with valid signature"
                }))?)
            })
        }),
        constraints: None,
    });

    // Generate signature helper endpoint
    let signer = RequestSigner::new(signing_secret);
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/generate-signature".to_string(),
        handler: Arc::new(move |_req| {
            let signer = signer.clone();
            Box::pin(async move {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let signature = signer.sign("POST", "/api/secure", "{\"test\":\"data\"}", timestamp);

                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "message": "Use these headers for signed request",
                    "headers": {
                        "X-Signature": signature,
                        "X-Timestamp": timestamp
                    },
                    "example_curl": format!(
                        "curl -X POST http://localhost:3000/api/secure -H 'X-Signature: {}' -H 'X-Timestamp: {}' -d '{{\"test\":\"data\"}}'",
                        signature, timestamp
                    )
                }))?)
            })
        }),
        constraints: None,
    });

    // Home endpoint with all security info
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/".to_string(),
        handler: Arc::new(|_req| {
            Box::pin(async move {
                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "message": "Advanced Security Example",
                    "features": {
                        "cors": "Granular cross-origin control",
                        "csp": "Content Security Policy",
                        "hsts": "HTTP Strict Transport Security",
                        "request_signing": "HMAC-SHA256 verification"
                    },
                    "endpoints": {
                        "OPTIONS /api/users": "CORS preflight",
                        "GET /api/users": "CORS-enabled API",
                        "POST /api/secure": "Signed requests only",
                        "GET /generate-signature": "Get signature for testing"
                    },
                    "cors_config": {
                        "allowed_origins": [
                            "https://app.example.com",
                            "https://admin.example.com",
                            "https://*.mydomain.com (regex)"
                        ],
                        "allowed_methods": ["GET", "POST", "PUT", "DELETE", "PATCH"],
                        "credentials": true
                    },
                    "security_headers": {
                        "Content-Security-Policy": "Enabled",
                        "Strict-Transport-Security": "max-age=31536000; includeSubDomains; preload",
                        "X-Frame-Options": "DENY",
                        "X-Content-Type-Options": "nosniff"
                    }
                }))?)
            })
        }),
        constraints: None,
    });

    // Build application
    let container = Container::new();
    let app = Application::new(container, router);

    info!("\n✓ Server started on http://localhost:3000");
    info!("\n🔒 Security Features Enabled:");
    info!("  ✓ Granular CORS with origin patterns");
    info!("  ✓ Content Security Policy");
    info!("  ✓ HSTS with preload");
    info!("  ✓ Request signing (HMAC-SHA256)");
    info!("\n🧪 Test Commands:");
    info!("  # Test CORS preflight");
    info!("  curl -X OPTIONS http://localhost:3000/api/users \\");
    info!("    -H 'Origin: https://app.example.com' \\");
    info!("    -H 'Access-Control-Request-Method: POST' -v");
    info!("\n  # Get signature for signed request");
    info!("  curl http://localhost:3000/generate-signature");
    info!("\nPress Ctrl+C to stop\n");

    app.listen(3000).await?;

    Ok(())
}
