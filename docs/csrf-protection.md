# CSRF Protection

Complete guide to Cross-Site Request Forgery (CSRF) protection in Armature.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Usage](#usage)
- [Best Practices](#best-practices)
- [Advanced Topics](#advanced-topics)
- [API Reference](#api-reference)

## Overview

CSRF (Cross-Site Request Forgery) is an attack that tricks users into performing unwanted actions on a web application where they're authenticated. The `armature-csrf` module provides comprehensive protection against CSRF attacks using the **Synchronizer Token Pattern**.

### How It Works

1. **Token Generation**: Server generates a unique, signed token for each session/request
2. **Token Distribution**: Token is sent to client via cookie
3. **Token Submission**: Client must include token in requests (header or form field)
4. **Token Validation**: Server validates token before processing state-changing requests

## Features

- ✅ **Token-Based Protection** - Synchronizer token pattern
- ✅ **Signed Tokens** - HMAC-SHA256 cryptographic signatures
- ✅ **Flexible Delivery** - Cookie, header, or form field
- ✅ **Configurable TTL** - Token expiration control
- ✅ **Session Binding** - Optional session-specific tokens
- ✅ **Path Exclusion** - Exclude specific endpoints
- ✅ **Safe Methods** - Automatic bypass for GET/HEAD/OPTIONS

## Quick Start

### 1. Add Dependency

```toml
[dependencies]
armature = { version = "0.1", features = ["csrf"] }
```

### 2. Basic Usage

```rust
use armature::prelude::*;
use armature_csrf::{CsrfConfig, CsrfMiddleware};

#[injectable]
struct MyService {
    csrf: CsrfMiddleware,
}

impl MyService {
    fn new() -> Self {
        let config = CsrfConfig::default();
        Self {
            csrf: CsrfMiddleware::new(config),
        }
    }
}

#[controller("/api")]
struct ApiController {
    service: MyService,
}

impl ApiController {
    #[get("/form")]
    async fn get_form(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        // Generate token
        let token = self.service.csrf.generate_token().unwrap();

        // Create response
        let mut response = HttpResponse::ok();

        // Add token as cookie
        response = self.service.csrf.add_token_cookie(response, &token).unwrap();

        Ok(response)
    }

    #[post("/submit")]
    async fn submit(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // Validate CSRF token
        self.service.csrf.validate_request(&req)?;

        // Process request
        Ok(HttpResponse::ok())
    }
}
```

## Configuration

### CsrfConfig Options

```rust
let config = CsrfConfig::new(secret_key)?
    .with_token_ttl(3600)              // 1 hour (default)
    .with_cookie_name("_csrf")          // Cookie name
    .with_header_name("X-CSRF-Token")   // Header name
    .with_field_name("csrf_token")      // Form field name
    .with_cookie_secure(true)           // HTTPS only
    .with_cookie_http_only(true)        // No JavaScript access
    .with_cookie_same_site(SameSite::Strict)
    .with_safe_methods(vec!["GET".to_string(), "HEAD".to_string()])
    .with_exclude_paths(vec!["/api/webhook".to_string()]);
```

### Secret Key

```rust
// Generate a new secret
let secret = CsrfConfig::generate_secret();

// Or use existing secret (min 32 bytes)
let config = CsrfConfig::new(my_secret_key)?;
```

### Cookie Configuration

```rust
use armature_csrf::SameSite;

let config = CsrfConfig::default()
    .with_cookie_name("_csrf_token")
    .with_cookie_path("/")
    .with_cookie_domain(Some("example.com".to_string()))
    .with_cookie_secure(true)           // HTTPS only
    .with_cookie_http_only(true)        // Prevent JavaScript access
    .with_cookie_same_site(SameSite::Strict);
```

#### SameSite Options

- **Strict**: Token only sent for same-site requests (most secure)
- **Lax**: Token sent for top-level navigation (balanced)
- **None**: Token sent for all requests (requires Secure flag)

## Usage

### HTML Forms

```html
<form method="POST" action="/api/submit">
    <input type="hidden" name="csrf_token" value="{{csrf_token}}" />
    <!-- Other form fields -->
    <button type="submit">Submit</button>
</form>
```

### AJAX/Fetch Requests

```javascript
// Get token from cookie
const csrfToken = getCookie('_csrf');

// Send in header
fetch('/api/submit', {
    method: 'POST',
    headers: {
        'X-CSRF-Token': csrfToken,
        'Content-Type': 'application/json'
    },
    body: JSON.stringify({ data: 'value' })
});

// Or in body
fetch('/api/submit', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json'
    },
    body: JSON.stringify({
        csrf_token: csrfToken,
        data: 'value'
    })
});
```

### Session Binding

```rust
// Generate session-specific token
let token = CsrfToken::generate_with_session(3600, session_id.to_string());

// Token is now bound to this session
// Additional validation can check session_id matches
```

## Best Practices

### 1. Use HTTPS

Always use HTTPS in production and set `cookie_secure` to `true`:

```rust
let config = CsrfConfig::default()
    .with_cookie_secure(true);
```

### 2. Set Appropriate TTL

Balance security and usability:

```rust
let config = CsrfConfig::default()
    .with_token_ttl(3600);  // 1 hour - adjust based on your needs
```

### 3. Use SameSite Cookies

Enable `SameSite=Strict` or `SameSite=Lax`:

```rust
use armature_csrf::SameSite;

let config = CsrfConfig::default()
    .with_cookie_same_site(SameSite::Strict);
```

### 4. Exclude Only Safe Endpoints

Carefully exclude paths from CSRF protection:

```rust
let config = CsrfConfig::default()
    .with_exclude_paths(vec![
        "/api/webhook".to_string(),  // External webhooks
        "/api/public".to_string(),   // Public read-only APIs
    ]);
```

### 5. Rotate Tokens

Generate new tokens for each form or after sensitive actions:

```rust
// After login
let new_token = csrf.generate_token().unwrap();

// After password change
let new_token = csrf.generate_token().unwrap();
```

### 6. Validate on All State-Changing Operations

Protect all POST, PUT, PATCH, DELETE requests:

```rust
impl ApiController {
    #[post("/create")]
    async fn create(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        self.csrf.validate_request(&req)?;
        // ...
    }

    #[put("/update")]
    async fn update(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        self.csrf.validate_request(&req)?;
        // ...
    }

    #[delete("/delete")]
    async fn delete(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        self.csrf.validate_request(&req)?;
        // ...
    }
}
```

## Advanced Topics

### Custom Token Storage

```rust
// Token can be stored in various places:
// 1. Cookie (default, recommended)
// 2. HTTP header
// 3. Form field
// 4. Request body (JSON)
```

### Token Regeneration

```rust
// Regenerate token after authentication
async fn login(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
    // Authenticate user...

    // Generate new CSRF token
    let token = self.csrf.generate_token().unwrap();
    let mut response = HttpResponse::ok();
    response = self.csrf.add_token_cookie(response, &token).unwrap();

    Ok(response)
}
```

### Double Submit Cookie Pattern

The default implementation uses a single cookie containing the signed token. For double-submit pattern:

```rust
// 1. Set token cookie
response = csrf.add_token_cookie(response, &token).unwrap();

// 2. Require token in header or body
// Validation automatically checks both cookie and submitted value
```

## API Reference

### CsrfConfig

```rust
pub struct CsrfConfig {
    pub secret: Vec<u8>,
    pub token_ttl: i64,
    pub cookie_name: String,
    pub header_name: String,
    pub field_name: String,
    pub cookie_domain: Option<String>,
    pub cookie_path: String,
    pub cookie_secure: bool,
    pub cookie_http_only: bool,
    pub cookie_same_site: SameSite,
    pub safe_methods: Vec<String>,
    pub exclude_paths: Vec<String>,
}
```

### CsrfMiddleware

```rust
impl CsrfMiddleware {
    pub fn new(config: CsrfConfig) -> Self;
    pub fn needs_protection(&self, request: &HttpRequest) -> bool;
    pub fn generate_token(&self) -> Result<CsrfToken, CsrfError>;
    pub fn add_token_cookie(&self, response: HttpResponse, token: &CsrfToken)
        -> Result<HttpResponse, CsrfError>;
    pub fn validate_request(&self, request: &HttpRequest)
        -> Result<(), ArmatureError>;
}
```

### CsrfToken

```rust
impl CsrfToken {
    pub fn generate(ttl_seconds: i64) -> Self;
    pub fn generate_with_session(ttl_seconds: i64, session_id: String) -> Self;
    pub fn is_expired(&self) -> bool;
    pub fn validate(&self) -> Result<()>;
    pub fn encode(&self, secret: &[u8]) -> Result<String>;
    pub fn decode(encoded: &str, secret: &[u8]) -> Result<Self>;
}
```

## Common Pitfalls

### ❌ Don't: Use CSRF for Authentication

CSRF tokens are not authentication tokens. They prevent forged requests, not unauthorized access.

### ❌ Don't: Expose Secrets

Never commit secret keys to version control:

```rust
// ❌ BAD
let secret = b"hardcoded_secret_key_12345678901";

// ✅ GOOD
let secret = env::var("CSRF_SECRET")
    .expect("CSRF_SECRET must be set")
    .into_bytes();
```

### ❌ Don't: Skip Validation for "Safe" Operations

Always validate state-changing operations, even if they seem safe:

```rust
// ❌ BAD - No CSRF check
#[post("/logout")]
async fn logout(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
    // Logout user
}

// ✅ GOOD - CSRF protected
#[post("/logout")]
async fn logout(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
    self.csrf.validate_request(&req)?;
    // Logout user
}
```

## Summary

**Key Takeaways:**

1. **Always protect state-changing operations** (POST, PUT, DELETE)
2. **Use HTTPS** and secure cookie flags
3. **Set appropriate token TTL** (balance security and UX)
4. **Don't exclude paths unnecessarily**
5. **Regenerate tokens** after authentication
6. **Use SameSite cookies** when possible
7. **Never expose secret keys**

For more examples, see `examples/csrf_protection.rs`.

