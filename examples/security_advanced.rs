#![allow(clippy::needless_question_mark)]
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
//! # Confirm CSP / HSTS / X-Frame-Options are genuinely attached to responses
//! curl -i http://localhost:3000/
//!
//! # Test the signed-request endpoint WITHOUT a signature -> rejected (401)
//! curl -i -X POST http://localhost:3000/api/secure -d '{"test":"data"}'
//!
//! # Test the signed-request endpoint WITH a valid signature -> succeeds.
//! # The server reads SIGNING_SECRET (at least 32 bytes) from the environment;
//! # without it, it generates a random one at startup and prints it. A client
//! # computes the signature itself with the shared secret -- this example does
//! # that in its `sign` mode, which prints ready-to-use headers:
//! export SIGNING_SECRET=...   # the value the server printed or was given
//! cargo run --example security_advanced -- sign POST /api/secure '{"test":"data"}'
//! curl -i -X POST http://localhost:3000/api/secure \
//!   -H "X-Signature: <printed signature>" \
//!   -H "X-Timestamp: <printed timestamp>" \
//!   -d '{"test":"data"}'
//! ```
//!
//! Note: this example predates the controller-macro style used elsewhere in
//! this codebase; see `examples/security_example.rs` for the idiomatic
//! `#[middleware(...)]` form built on `#[controller]` / `#[module]`.

use armature_core::handler::handler;
use armature_core::*;
use armature_proc_macro::use_middleware;
use armature_security::SecurityMiddleware;
use armature_security::content_security_policy::CspConfig;
use armature_security::cors::CorsConfig;
use armature_security::frame_guard::FrameGuard;
use armature_security::hsts::HstsConfig;
use armature_security::request_signing::{RequestSigner, RequestVerifier, SigningError};
use std::time::{SystemTime, UNIX_EPOCH};

/// Shared secret used to sign and verify requests, loaded once at startup
/// (see [`load_signing_secret`]). Never hardcode it.
static SIGNING_SECRET: std::sync::OnceLock<String> = std::sync::OnceLock::new();

fn signing_secret() -> &'static str {
    SIGNING_SECRET
        .get()
        .expect("signing secret is initialized in main")
}

/// `SIGNING_SECRET` from the environment, or — for this demo only — a fresh
/// random 32-byte secret, printed so a client can be pointed at it.
fn load_signing_secret() -> String {
    match std::env::var("SIGNING_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            let bytes: [u8; 32] = rand::random();
            let secret: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            println!("SIGNING_SECRET is not set; generated a random demo secret:");
            println!("  export SIGNING_SECRET={secret}");
            secret
        }
    }
}

/// CORS configuration shared by the preflight (`OPTIONS /api/users`) and
/// data (`GET /api/users`) routes below.
fn cors_config() -> CorsConfig {
    CorsConfig::new()
        // Allow specific origins
        .allow_origin("https://app.example.com")
        .allow_origin("https://admin.example.com")
        // Allow regex pattern for subdomains
        // Allow one subdomain label (the pattern is anchored; never use `.*`)
        .allow_origin_regex(r"https://[a-z0-9-]+\.mydomain\.com")
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
        .max_age(7200)
}

/// Combined CSP + HSTS + Frame Guard security middleware.
///
/// Wired onto every route below via `#[use_middleware(security_config())]`,
/// so the "security_headers" claims advertised by the home endpoint's JSON
/// (CSP / HSTS / X-Frame-Options "Enabled") are actually true, not just
/// printed to the console.
fn security_config() -> SecurityMiddleware {
    let csp = CspConfig::new()
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

    let hsts = HstsConfig::new(31536000) // 1 year
        .include_subdomains(true)
        .preload(true);

    SecurityMiddleware::new()
        .with_csp(csp)
        .expect("the CSP above is valid")
        .with_hsts(hsts)
        .with_frame_guard(FrameGuard::Deny)
        .hide_powered_by(true)
}

/// Verifier used to *actually* check signatures on `POST /api/secure`.
fn verifier() -> RequestVerifier {
    RequestVerifier::new(signing_secret()).expect("secret is validated at startup")
}

/// OPTIONS handler for CORS preflight.
#[use_middleware(security_config())]
async fn cors_preflight(req: HttpRequest) -> Result<HttpResponse, Error> {
    cors_config().handle_preflight(&req)
}

/// API endpoint with CORS.
#[use_middleware(security_config())]
async fn list_users(req: HttpRequest) -> Result<HttpResponse, Error> {
    let response = HttpResponse::ok().with_json(&serde_json::json!({
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]
    }))?;

    // Apply CORS headers; security headers are applied afterward by the
    // `security_config()` middleware wrapping this handler.
    Ok(cors_config().apply(&req, response))
}

/// Signed request endpoint.
///
/// This is the endpoint the home route (below) advertises as "Signed
/// requests only". It must actually verify the HMAC-SHA256 signature via
/// `RequestVerifier::verify_request` and reject anything that doesn't check
/// out — an unsigned, malformed, or replayed request must NEVER reach the
/// "Securely accessed" success response.
#[use_middleware(security_config())]
async fn secure_endpoint(req: HttpRequest) -> Result<HttpResponse, Error> {
    match verifier().verify_request(&req) {
        Ok(true) => Ok(HttpResponse::ok().with_json(&serde_json::json!({
            "message": "Securely accessed with valid signature"
        }))?),
        Ok(false) => Err(Error::Unauthorized("Invalid request signature".to_string())),
        Err(SigningError::RequestExpired) => Err(Error::Unauthorized(
            "Request expired (replay protection). Sign the request again with a \
             fresh timestamp."
                .to_string(),
        )),
        Err(SigningError::MissingSignature) => Err(Error::Unauthorized(
            "Missing X-Signature header".to_string(),
        )),
        Err(SigningError::MissingTimestamp) => Err(Error::Unauthorized(
            "Missing X-Timestamp header".to_string(),
        )),
        Err(e) => Err(Error::Unauthorized(format!(
            "Signature verification failed: {}",
            e
        ))),
    }
}

/// Home endpoint with all security info.
#[use_middleware(security_config())]
async fn home(_req: HttpRequest) -> Result<HttpResponse, Error> {
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
            "POST /api/secure": "Signed requests only"
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // `sign METHOD PATH BODY`: act as a client and print signed headers.
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("sign") {
        let [_, method, path, body] = args.as_slice() else {
            return Err("usage: security_advanced sign METHOD PATH BODY".into());
        };
        let secret = std::env::var("SIGNING_SECRET")
            .map_err(|_| "set SIGNING_SECRET to the server's signing secret")?;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let signature = RequestSigner::new(secret)?.sign(method, path, body.as_bytes(), timestamp);
        println!("X-Signature: {signature}");
        println!("X-Timestamp: {timestamp}");
        return Ok(());
    }

    // Load and validate the signing secret before serving anything.
    let secret = load_signing_secret();
    RequestVerifier::new(&secret)
        .map_err(|e| format!("SIGNING_SECRET is unusable ({e}); use at least 32 bytes"))?;
    SIGNING_SECRET.set(secret).expect("set once");

    // Initialize logging
    let _guard = LogConfig::default().init();

    info!("Advanced Security Example");
    info!("=========================");

    info!("\nConfiguring CORS...");
    info!("✓ CORS configured:");
    info!("  - Origins: app.example.com, admin.example.com, *.mydomain.com");
    info!("  - Methods: GET, POST, PUT, DELETE, PATCH");
    info!("  - Credentials: Allowed");
    info!("  - Max age: 2 hours");

    info!("\nConfiguring CSP...");
    info!("✓ CSP configured with secure defaults");

    info!("\nConfiguring HSTS...");
    info!("✓ HSTS configured:");
    info!("  - Max age: 1 year");
    info!("  - Include subdomains: Yes");
    info!("  - Preload: Yes");

    info!("\nConfiguring request signing...");
    info!("✓ Request signing configured:");
    info!("  - Algorithm: HMAC-SHA256");
    info!("  - Max age: 5 minutes");
    info!("  - Skipped paths: /health, /metrics");

    // Create router
    let mut router = Router::new();

    router.add_route(Route {
        method: HttpMethod::OPTIONS,
        path: "/api/users".to_string(),
        handler: handler(cors_preflight),
        constraints: None,
    });

    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/users".to_string(),
        handler: handler(list_users),
        constraints: None,
    });

    router.add_route(Route {
        method: HttpMethod::POST,
        path: "/api/secure".to_string(),
        handler: handler(secure_endpoint),
        constraints: None,
    });

    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/".to_string(),
        handler: handler(home),
        constraints: None,
    });

    // Build application
    let container = Container::new();
    let app = Application::new(container, router);

    info!("\n✓ Server started on http://localhost:3000");
    info!("\n🔒 Security Features Enabled:");
    info!("  ✓ Granular CORS with origin patterns");
    info!("  ✓ Content Security Policy (attached to every response)");
    info!("  ✓ HSTS with preload (attached to every response)");
    info!("  ✓ Request signing (HMAC-SHA256, actually verified on POST /api/secure)");
    info!("\n🧪 Test Commands:");
    info!("  # Test CORS preflight");
    info!("  curl -X OPTIONS http://localhost:3000/api/users \\");
    info!("    -H 'Origin: https://app.example.com' \\");
    info!("    -H 'Access-Control-Request-Method: POST' -v");
    info!("\n  # Confirm CSP / HSTS / X-Frame-Options headers are genuinely present");
    info!("  curl -i http://localhost:3000/");
    info!("\n  # Unsigned POST to /api/secure -> rejected with 401");
    info!("  curl -i -X POST http://localhost:3000/api/secure -d '{{\"test\":\"data\"}}'");
    info!("\n  # Sign a request as a client (same SIGNING_SECRET), then POST it");
    info!(
        "  cargo run --example security_advanced -- sign POST /api/secure '{{\"test\":\"data\"}}'"
    );
    info!("  # ...POST with the printed X-Signature/X-Timestamp headers -> 200 OK");
    info!("\nPress Ctrl+C to stop\n");

    app.listen(3000).await?;

    Ok(())
}
