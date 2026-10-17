# Advanced Security Guide

Guide to the advanced security features in Armature: CORS, CSP and cross-origin isolation headers, HSTS, CSRF protection and request signing.

## Table of Contents

- [Overview](#overview)
- [CORS (Cross-Origin Resource Sharing)](#cors)
- [Content Security Policy (CSP)](#csp)
- [HSTS (HTTP Strict Transport Security)](#hsts)
- [CSRF Protection](#csrf-protection)
- [Request Signing with HMAC](#request-signing)
- [Best Practices](#best-practices)
- [Security Checklist](#security-checklist)

## Overview

Armature provides security features that protect applications from common web vulnerabilities and attacks. The defaults are deliberately strict: loosen them only where a feature needs it.

### Security Features

- ✅ **Granular CORS Control** - Origin patterns, method restrictions, credential handling
- ✅ **Content Security Policy** - Prevent XSS attacks with validated CSP directives
- ✅ **Cross-origin isolation** - `Cross-Origin-Opener-Policy`, `Cross-Origin-Resource-Policy` and `Permissions-Policy` on by default
- ✅ **HSTS** - Force HTTPS with preload support
- ✅ **CSRF Protection** - Signed double-submit cookie (`CsrfMiddleware`)
- ✅ **Request Signing** - HMAC-SHA256 over an unambiguous canonical form, with a freshness window
- ✅ **Security Headers** - Applied without overwriting headers your handlers set
- ✅ **Rate Limiting** - Token bucket and sliding window algorithms

## CORS

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

Allow multiple subdomains with regex patterns. `allow_origin_regex` returns
`Result<Self, regex::Error>` and compiles the pattern **anchored** as
`^(?:pattern)$`, so it must match the *whole* origin, scheme included:

```rust
let cors = CorsConfig::new()
    // One label below example.com: admits https://app.example.com,
    // refuses https://app.example.com.attacker.net
    .allow_origin_regex(r"https://[a-z0-9-]+\.example\.com")?
    // Several TLDs
    .allow_origin_regex(r"https://app\.(com|net|org)")?;
```

Prefer a single-label class such as `[a-z0-9-]+` over `.*` in a host pattern: `.*`
admits any depth of subdomain, including ones you do not control.
Patterns that relied on the old substring match
(for example one without a scheme) must be rewritten to match the full origin.
The opaque origin `null` is never allowed by an allowlist or a pattern.

### Development vs Production

```rust
// ❌ Development only - allows all origins (never combine with credentials)
let cors = CorsConfig::permissive();

// ✅ Production - specific origins only
let cors = CorsConfig::new()
    .allow_origin("https://app.example.com")
    .allow_credentials(true);
```

## CSP

### Default Policy

`CspConfig::default()` (used by `SecurityMiddleware::default()` and
`enable_all`) is a strict baseline:

```text
base-uri 'none'; connect-src 'self'; default-src 'self'; font-src 'self';
form-action 'self'; frame-ancestors 'none'; img-src 'self' data:;
object-src 'none'; script-src 'self'; style-src 'self'
```

Inline styles (`'unsafe-inline'`) and arbitrary `https:` images are **not**
allowed; add them explicitly if the application needs them. Directives are
emitted in name order, so the header is deterministic.

### Custom Configuration

```rust
use armature_security::SecurityMiddleware;
use armature_security::content_security_policy::{CspConfig, CspError};

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
    .img_src(vec!["'self'".to_string(), "data:".to_string()])
    .object_src(vec!["'none'".to_string()])
    .frame_ancestors(vec!["'none'".to_string()])
    .base_uri(vec!["'none'".to_string()])
    .form_action(vec!["'self'".to_string()]);

// Rendering validates the policy: `;`, `,`, whitespace or control
// characters (CR/LF) in a directive name or value are a `CspError`.
let header: String = csp.to_header_value()?;

// `with_csp` validates too, so it returns `Result<SecurityMiddleware, CspError>`.
let security = SecurityMiddleware::new().with_csp(csp)?;
# Ok::<(), CspError>(())
```

A policy written directly into the public `csp` field that fails validation
is replaced by `default-src 'none'; frame-ancestors 'none'` rather than
emitted.

### CSP Directives

Common directives:

- `default-src` - Fallback for all directives
- `script-src` - JavaScript sources
- `style-src` - CSS sources
- `img-src` - Image sources
- `font-src` - Font sources
- `connect-src` - AJAX, WebSocket, EventSource
- `frame-src` - iframe sources
- `object-src` - plugins (`'none'` by default)
- `frame-ancestors` - who may frame this page (`'none'` by default)
- `base-uri` - allowed `<base>` URLs (`'none'` by default)
- `form-action` - where forms may submit (`'self'` by default)

### Cross-Origin Isolation and Permissions-Policy

`SecurityMiddleware::new()`, `default()` and `enable_all()` also send
`Cross-Origin-Opener-Policy: same-origin`, `Cross-Origin-Resource-Policy:
same-origin` and a restrictive `Permissions-Policy` (camera, microphone,
geolocation, payment, USB, ... disabled). Relax them per deployment; pass
`None` to stop sending a header:

```rust
use armature_security::SecurityMiddleware;
use armature_security::cross_origin::{CrossOriginOpenerPolicy, CrossOriginResourcePolicy};
use armature_security::permissions_policy::PermissionsPolicy;

let security = SecurityMiddleware::default()
    // OAuth / payment popups that need `window.opener`
    .with_coop(Some(CrossOriginOpenerPolicy::SameOriginAllowPopups))
    // Public assets embedded by other origins
    .with_corp(Some(CrossOriginResourcePolicy::CrossOrigin))
    // Keep the restrictive policy but allow fullscreen for this origin
    .with_permissions_policy(Some(PermissionsPolicy::restrictive().allow_self("fullscreen")));
```

`SecurityMiddleware::apply` only adds a security header the response does not
already carry (compared case-insensitively), so a handler that sets its own
route-specific policy keeps it.

## HSTS

### Basic Configuration

```rust
use armature_security::hsts::HstsConfig;

let hsts = HstsConfig::new(31536000) // 1 year in seconds
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

## CSRF Protection

Browser applications that authenticate with cookies need CSRF protection on
every state-changing route. `armature_security::csrf` implements the *signed
double-submit cookie* pattern:

- A random 256-bit nonce lives in a `__Host-armature-csrf` cookie
  (`Secure`, `HttpOnly`, `SameSite=Strict`, `Path=/`).
- The token is `HMAC-SHA256(secret, nonce ‖ binding)`, optionally bound to a
  session or principal with `CsrfConfig::bind_to`.
- Safe methods (`GET`, `HEAD`, `OPTIONS`, `TRACE`) pass and receive a
  `CsrfToken` request extension; every other method must send the token in
  the `X-CSRF-Token` header or the `_csrf` form field, compared in constant
  time, or is rejected with `403`.

```rust
use armature_core::{Application, VerifiedIdentity};
use armature_security::csrf::{CsrfConfig, CsrfError, CsrfMiddleware};

// The secret must be at least 32 bytes (`CsrfError::WeakSecret` otherwise).
let secret = std::env::var("CSRF_SECRET").expect("CSRF_SECRET");

let csrf = CsrfMiddleware::new(
    CsrfConfig::new(secret)?
        // Bind tokens to the authenticated principal, so a token leaked
        // from one account is useless for another.
        .bind_to(|req| req.extensions.get::<VerifiedIdentity>().map(|id| id.subject.clone()))
        // Machine-to-machine endpoints authenticated by other means.
        .exempt_path("/webhooks"),
);

let app = Application::create::<AppModule>().await
    // Authenticate first, so the binding sees the verified identity
    .use_middleware(armature_auth::JwtAuthMiddleware::optional(jwt_manager))
    .use_middleware(csrf);
```

Handlers render the token for the current request into their forms:

```rust
use armature_security::csrf::CsrfToken;

let token = req.extensions.get::<CsrfToken>().map(|t| t.as_str().to_owned());
// <input type="hidden" name="_csrf" value="{token}">
```

For plain-HTTP local development, use a cookie name without the `__Host-`
prefix and `.secure(false)` (both return `Result`, since `__Host-` requires
`Secure`). Exempt paths match whole segments, and a path containing `.`/`..`
segments is never exempt.

## Request Signing

### Basic Setup

Secrets must be at least 32 bytes (`MIN_SECRET_LEN`); the constructors
return `Result<_, SigningError>` and reject shorter ones with
`SigningError::WeakSecret`.

```rust
use armature_security::request_signing::{RequestSigner, RequestVerifier, SigningError};

let secret = std::env::var("API_SIGNING_SECRET").expect("API_SIGNING_SECRET");

// Server-side: Verify incoming requests
let verifier = RequestVerifier::new(&secret)?
    .with_max_age(300); // 5 minutes

// Client-side: Sign outgoing requests
let signer = RequestSigner::new(&secret)?;
let signature = signer.sign("POST", "/api/orders?id=7", body.as_bytes(), timestamp);
# Ok::<(), SigningError>(())
```

### Canonical Form

The signature is hex `HMAC-SHA256` over this string, with variable-length
fields length-prefixed so no two requests share a canonical form, and the
body entering as the SHA-256 of its exact bytes:

```text
ARMATURE-HMAC-SHA256-V1\n
<len(method)>:<method>\n
<len(path)>:<path>\n
<hex(sha256(body))>\n
<timestamp>
```

`path` is the raw request target, query string included. Clients send the
signature in `X-Signature` and the Unix timestamp in `X-Timestamp`. Signatures
from the earlier `method:path:body:timestamp` format no longer verify, so
non-Rust clients must re-implement signing to this form.

### Verifying Requests

```rust
// body is &[u8]
match verifier.verify(method, path, body, timestamp, &signature) {
    Ok(true) => println!("Valid signature"),
    Ok(false) => println!("Invalid signature"),
    Err(e) => println!("Verification error: {}", e), // expired, future timestamp, ...
}
```

The timestamp check is a *freshness* window (`with_max_age`, plus 60 s of
tolerated clock skew), not full replay protection: a captured request can be
replayed until the window closes. Pair it with a nonce or idempotency store if
you need exactly-once semantics.

### Middleware

Automatically verify all requests. Nothing is skipped by default (earlier
versions silently exempted `/health/*` and `/metrics/*`); exempt paths
explicitly:

```rust
use armature_security::request_signing::RequestSigningMiddleware;

let signing = RequestSigningMiddleware::new(&secret)?
    .with_max_age(300)
    .skip_path("/health")
    .skip_path("/metrics");

let app = Application::create::<AppModule>().await
    .use_middleware(signing);
```

## Best Practices

### Security Checklist

- [ ] Use HTTPS in production (required for HSTS)
- [ ] Configure strict CORS (no wildcards in production)
- [ ] Enable HSTS with `includeSubDomains` and `preload`
- [ ] Implement Content Security Policy
- [ ] Protect cookie-authenticated, state-changing routes with `CsrfMiddleware`
- [ ] Use request signing for API authentication
- [ ] Enable all security headers
- [ ] Set up rate limiting
- [ ] Keep secrets secure (use environment variables) and at least 32 random bytes
- [ ] Rotate secrets regularly
- [ ] Monitor CSP reports

### Production Security Stack

```rust
use std::sync::Arc;
use std::time::Duration;
use armature_core::{Application, CorsMiddleware};
use armature_ratelimit::{Algorithm, RateLimitMiddleware, RateLimiter};
use armature_security::SecurityMiddleware;
use armature_security::request_signing::RequestSigningMiddleware;

// 1. Security Headers (CSP, HSTS, COOP/CORP, Permissions-Policy, ...)
let security = SecurityMiddleware::default();

// 2. CORS (specific origins only when credentials are allowed)
let cors = CorsMiddleware::new()
    .allow_origin("https://app.example.com")
    .allow_methods("GET, POST, PUT, DELETE")
    .allow_credentials(true);

// 3. Rate Limiting: 100 requests per minute, keyed on the socket peer.
//    Set `with_trusted_proxy_depth(n)` only when n trusted proxies append
//    to X-Forwarded-For.
let limiter = RateLimiter::builder()
    .algorithm(Algorithm::SlidingWindowLog {
        max_requests: 100,
        window: Duration::from_secs(60),
    })
    .build()
    .await?;
let rate_limit = RateLimitMiddleware::new(Arc::new(limiter));

// 4. Request Signing (secret of at least 32 bytes)
let signing = RequestSigningMiddleware::new(std::env::var("API_SECRET")?)?
    .skip_path("/health");

// Global middleware runs in registration order, before any guard.
let app = Application::create::<AppModule>().await
    .use_middleware(security)
    .use_middleware(cors)
    .use_middleware(rate_limit)
    .use_middleware(signing);
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
- CSRF protection for cookie-authenticated forms
- HSTS with subdomains
- Rate limiting

**Advanced**:
- Request signing
- CSP with nonces
- HSTS preload
- Origin patterns
- Comprehensive monitoring

---

**Security is not optional!** Start with defaults and customize as needed. 🔒

