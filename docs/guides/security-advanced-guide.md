# Advanced Security Guide

Comprehensive guide to advanced security features in Armature including CORS, CSP, HSTS, and request signing.

## Table of Contents

- [Overview](#overview)
- [CORS (Cross-Origin Resource Sharing)](#cors)
- [Content Security Policy (CSP)](#csp)
- [HSTS (HTTP Strict Transport Security)](#hsts)
- [Request Signing with HMAC](#request-signing)
- [Best Practices](#best-practices)
- [Security Checklist](#security-checklist)

## Overview

Armature provides enterprise-grade security features to protect your applications from common web vulnerabilities and attacks.

### Security Features

- ✅ **Granular CORS Control** - Origin patterns, method restrictions, credential handling
- ✅ **Content Security Policy** - Prevent XSS attacks with CSP directives
- ✅ **HSTS** - Force HTTPS with preload support
- ✅ **Request Signing** - HMAC-SHA256 verification with replay protection
- ✅ **Security Headers** - 11+ security headers automatically applied
- ✅ **Rate Limiting** - Token bucket and sliding window algorithms

## CORS

### Overview

CORS (Cross-Origin Resource Sharing) controls which origins can access your API. Armature provides granular CORS configuration with:

- Specific origin allowlisting
- Regex pattern matching for origins
- Method and header restrictions
- Credentials support
- Preflight caching

### Basic Configuration

```rust
use armature_security::cors::CorsConfig;

// Strict production CORS
let cors = CorsConfig::new()
    .allow_origin("https://app.example.com")
    .allow_origin("https://admin.example.com")
    .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
    .allow_headers(vec!["Content-Type", "Authorization"])
    .allow_credentials(true)
    .max_age(3600); // 1 hour preflight cache
```

### Origin Patterns (Regex)

Allow multiple subdomains with regex patterns:

```rust
use armature_security::cors::CorsConfig;

let cors = CorsConfig::new()
    // Allow all subdomains of example.com
    .allow_origin_regex(r"https://.*\.example\.com").unwrap()
    // Allow multiple TLDs
    .allow_origin_regex(r"https://app\.(com|net|org)").unwrap();
```

### Development vs Production

```rust
// ❌ Development only - allows all origins
let cors = CorsConfig::permissive();

// ✅ Production - specific origins only
let cors = CorsConfig::new()
    .allow_origin("https://app.example.com")
    .allow_credentials(true);
```

### Credentials and Wildcards

**Important**: You cannot use `allow_any_origin()` with `allow_credentials(true)`. This is a security restriction.

```rust
// ❌ INVALID - will fail
let cors = CorsConfig::new()
    .allow_any_origin()
    .allow_credentials(true); // Error!

// ✅ VALID - specific origins with credentials
let cors = CorsConfig::new()
    .allow_origin("https://app.example.com")
    .allow_credentials(true);
```

### Exposed Headers

Control which response headers are accessible to JavaScript:

```rust
let cors = CorsConfig::new()
    .allow_origin("https://app.example.com")
    .expose_headers(vec![
        "X-Total-Count",
        "X-Page-Number",
        "X-Request-Id"
    ]);
```

### Preflight Requests

Handle OPTIONS preflight requests:

```rust
use armature_core::*;

// In your router
router.add_route(Route {
    method: HttpMethod::OPTIONS,
    path: "/api/users".to_string(),
    handler: Arc::new(move |req| {
        let cors = cors_config.clone();
        Box::pin(async move {
            cors.handle_preflight(&req)
        })
    }),
    constraints: None,
});
```

### Apply CORS to Response

```rust
// Apply CORS headers to a response
let cors = CorsConfig::new().allow_origin("https://app.example.com");

router.add_route(Route {
    method: HttpMethod::GET,
    path: "/api/data".to_string(),
    handler: Arc::new(move |req| {
        let cors = cors.clone();
        Box::pin(async move {
            let response = HttpResponse::ok().with_json(&data)?;
            Ok(cors.apply(&req, response))
        })
    }),
    constraints: None,
});
```

## CSP

### Overview

Content Security Policy (CSP) prevents XSS attacks by controlling which resources can be loaded.

### Basic Configuration

```rust
use armature_security::content_security_policy::CspConfig;

let csp = CspConfig::new()
    .default_src(vec!["'self'".to_string()])
    .script_src(vec![
        "'self'".to_string(),
        "https://cdn.example.com".to_string()
    ])
    .style_src(vec![
        "'self'".to_string(),
        "https://fonts.googleapis.com".to_string()
    ])
    .img_src(vec![
        "'self'".to_string(),
        "data:".to_string(),
        "https:".to_string()
    ]);
```

### CSP Directives

Common directives:

- `default-src` - Fallback for all directives
- `script-src` - JavaScript sources
- `style-src` - CSS sources
- `img-src` - Image sources
- `font-src` - Font sources
- `connect-src` - AJAX, WebSocket, EventSource
- `frame-src` - iframe sources
- `object-src` - `<object>`, `<embed>`, `<applet>`

### Strict CSP

```rust
let csp = CspConfig::new()
    .default_src(vec!["'none'".to_string()])
    .script_src(vec!["'self'".to_string()])
    .style_src(vec!["'self'".to_string()])
    .img_src(vec!["'self'".to_string()])
    .font_src(vec!["'self'".to_string()])
    .connect_src(vec!["'self'".to_string()])
    .frame_ancestors(vec!["'none'".to_string()]);
```

### CSP with Nonces

For inline scripts (recommended over `'unsafe-inline'`):

```rust
// Generate nonce per request
let nonce = generate_nonce();

let csp = CspConfig::new()
    .script_src(vec![
        "'self'".to_string(),
        format!("'nonce-{}'", nonce)
    ]);

// In HTML
// <script nonce="{{nonce}}">...</script>
```

### Report-Only Mode

Test CSP without blocking:

```rust
let csp = CspConfig::new()
    .default_src(vec!["'self'".to_string()])
    .report_only(true)
    .report_uri("https://example.com/csp-reports");
```

## HSTS

### Overview

HTTP Strict Transport Security (HSTS) forces browsers to only use HTTPS connections.

### Basic Configuration

```rust
use armature_security::hsts::HstsConfig;

let hsts = HstsConfig::new(31536000) // 1 year in seconds
    .include_subdomains(true)
    .preload(true);
```

### HSTS Preload

Submit your site to the [HSTS preload list](https://hstspreload.org/):

```rust
// Requirements for preload:
// 1. Valid HTTPS certificate
// 2. Redirect HTTP to HTTPS
// 3. Serve HSTS header on base domain
// 4. max-age >= 31536000 (1 year)
// 5. includeSubDomains directive
// 6. preload directive

let hsts = HstsConfig::new(31536000)
    .include_subdomains(true)
    .preload(true);
```

### Gradual Rollout

Start with a shorter max-age and increase:

```rust
// Week 1: 1 week
let hsts = HstsConfig::new(604800);

// Week 2: 1 month
let hsts = HstsConfig::new(2592000);

// Week 3+: 1 year
let hsts = HstsConfig::new(31536000);
```

## Request Signing

### Overview

Request signing with HMAC-SHA256 ensures requests haven't been tampered with and protects against replay attacks.

### Basic Setup

```rust
use armature_security::request_signing::{RequestSigner, RequestVerifier};

// Server-side: Verify incoming requests
let verifier = RequestVerifier::new("shared-secret")
    .with_max_age(300); // 5 minutes

// Client-side: Sign outgoing requests
let signer = RequestSigner::new("shared-secret");
```

### Signing Requests (Client)

```rust
use armature_security::request_signing::RequestSigner;

let signer = RequestSigner::new("shared-secret");

let method = "POST";
let path = "/api/users";
let body = r#"{"name":"Alice"}"#;
let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs();

let signature = signer.sign(method, path, body, timestamp);

// Add headers to request:
// X-Signature: <signature>
// X-Timestamp: <timestamp>
```

### Verifying Requests (Server)

```rust
use armature_security::request_signing::RequestVerifier;

let verifier = RequestVerifier::new("shared-secret")
    .with_max_age(300); // 5 minutes

match verifier.verify(method, path, body, timestamp, signature) {
    Ok(true) => println!("Valid signature"),
    Ok(false) => println!("Invalid signature"),
    Err(e) => println!("Verification error: {}", e),
}
```

### Middleware

Automatically verify all requests:

```rust
use armature_security::request_signing::RequestSigningMiddleware;

let signing = RequestSigningMiddleware::new("shared-secret")
    .with_max_age(300)
    .skip_path("/health")
    .skip_path("/metrics");

let app = Application::new()
    .middleware(Arc::new(signing))
    .build();
```

### Replay Protection

Signatures include timestamps and expire after `max_age`:

```rust
let verifier = RequestVerifier::new("secret")
    .with_max_age(300); // 5 minutes

// Requests older than 5 minutes will be rejected
```

### HMAC Algorithm

Armature uses HMAC-SHA256:

```
signature = HMAC-SHA256(secret, method:path:body:timestamp)
```

### Constant-Time Comparison

Signature verification uses constant-time comparison to prevent timing attacks.

## Best Practices

### Security Checklist

- [ ] Use HTTPS in production (required for HSTS)
- [ ] Configure strict CORS (no wildcards in production)
- [ ] Enable HSTS with `includeSubDomains` and `preload`
- [ ] Implement Content Security Policy
- [ ] Use request signing for API authentication
- [ ] Enable all security headers (use `SecurityMiddleware::default()`)
- [ ] Set up rate limiting
- [ ] Keep secrets secure (use environment variables)
- [ ] Rotate secrets regularly
- [ ] Monitor CSP reports
- [ ] Test security configuration before deploying

### Production Security Stack

```rust
use armature_security::*;
use armature_ratelimit::*;

// 1. Security Headers
let security = SecurityMiddleware::default(); // Includes CSP, HSTS, etc.

// 2. CORS
let cors = CorsConfig::new()
    .allow_origin("https://app.example.com")
    .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
    .allow_credentials(true);

// 3. Rate Limiting
let rate_limit = RateLimitMiddleware::new(100, 60); // 100 req/min

// 4. Request Signing
let signing = RequestSigningMiddleware::new(std::env::var("API_SECRET")?);

let app = Application::new()
    .middleware(Arc::new(security))
    .middleware(Arc::new(rate_limit))
    .middleware(Arc::new(signing))
    .build();
```

### Environment Variables

```bash
# .env
API_SECRET=your-256-bit-secret-here
CORS_ORIGINS=https://app.example.com,https://admin.example.com
HSTS_MAX_AGE=31536000
CSP_REPORT_URI=https://example.com/csp-reports
```

### Testing Security

```bash
# Test CORS
curl -H "Origin: https://app.example.com" http://localhost:3000/api

# Test CSP
curl -I http://localhost:3000 | grep Content-Security-Policy

# Test HSTS
curl -I https://localhost:3000 | grep Strict-Transport-Security

# Test signed request
curl -X POST http://localhost:3000/api/secure \
  -H "X-Signature: abc123..." \
  -H "X-Timestamp: 1702468800" \
  -d '{"data":"test"}'
```

## Summary

### Key Takeaways

1. **CORS**: Use specific origins in production, never wildcards with credentials
2. **CSP**: Start with report-only mode, gradually tighten
3. **HSTS**: Start with short max-age, increase over time
4. **Request Signing**: Use unique secrets, rotate regularly
5. **Defense in Depth**: Combine multiple security layers

### Security Levels

**Basic** (Minimum):
- Security headers (default middleware)
- HTTPS in production
- Basic CORS

**Intermediate**:
- Custom CSP policy
- HSTS with subdomains
- Rate limiting

**Advanced**:
- Request signing
- CSP with nonces
- HSTS preload
- Origin patterns
- Comprehensive monitoring

### Next Steps

1. Review your current security configuration
2. Implement missing security features
3. Test with security scanning tools
4. Monitor CSP reports
5. Set up automated security checks in CI/CD

---

**Security is not optional!** Start with defaults and customize as needed. 🔒

