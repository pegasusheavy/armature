# Security Middleware

Comprehensive security middleware for Armature - inspired by Helmet for Express.js.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Security Headers](#security-headers)
- [Best Practices](#best-practices)
- [API Reference](#api-reference)

## Overview

The Armature Security middleware provides a collection of security headers and protections that help secure your web applications against common vulnerabilities. It's inspired by [Helmet](https://helmetjs.github.io/) for Express.js and provides similar functionality for Rust/Armature applications.

## Features

✅ **Content Security Policy (CSP)** - Prevent XSS attacks  
✅ **HTTP Strict Transport Security (HSTS)** - Force HTTPS  
✅ **X-Frame-Options** - Prevent clickjacking  
✅ **X-Content-Type-Options** - Prevent MIME sniffing  
✅ **X-XSS-Protection** - Enable browser XSS filters  
✅ **Referrer Policy** - Control referrer information  
✅ **DNS Prefetch Control** - Control DNS prefetching  
✅ **Expect-CT** - Certificate Transparency  
✅ **X-Download-Options** - Prevent IE download execution  
✅ **X-Permitted-Cross-Domain-Policies** - Control Flash/PDF policies  
✅ **Hide X-Powered-By** - Remove server fingerprinting  

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
armature = { version = "0.1", features = ["security"] }
```

## Quick Start

### Default Configuration (Recommended)

```rust
use armature_security::SecurityMiddleware;
use armature_core::HttpResponse;

// Use default settings - all protections enabled
let security = SecurityMiddleware::default();

// Apply to response
let response = HttpResponse::ok();
let secured_response = security.apply(response);
```

### Custom Configuration

```rust
use armature_security::{
    SecurityMiddleware,
    content_security_policy::CspConfig,
    hsts::HstsConfig,
    frame_guard::FrameGuard,
    referrer_policy::ReferrerPolicy,
};

let security = SecurityMiddleware::new()
    .with_csp(CspConfig::default())
    .with_hsts(HstsConfig::new(31536000)) // 1 year
    .with_frame_guard(FrameGuard::Deny)
    .with_referrer_policy(ReferrerPolicy::NoReferrer)
    .hide_powered_by(true);
```

## Configuration

### Enable All Features (Recommended)

```rust
// All security features with 1-year HSTS
let security = SecurityMiddleware::enable_all(31536000);
```

### Start from Scratch

```rust
// No protections - configure manually
let security = SecurityMiddleware::new()
    .with_frame_guard(FrameGuard::SameOrigin)
    .hide_powered_by(true);
```

## Security Headers

### Content Security Policy (CSP)

Prevents XSS attacks by declaring which dynamic resources are allowed to load.

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
        "'unsafe-inline'".to_string()
    ])
    .img_src(vec![
        "'self'".to_string(),
        "data:".to_string(),
        "https:".to_string()
    ]);

let security = SecurityMiddleware::new().with_csp(csp);
```

**Output Header:**
```
Content-Security-Policy: default-src 'self'; script-src 'self' https://cdn.example.com; ...
```

### HTTP Strict Transport Security (HSTS)

Forces browsers to use HTTPS.

```rust
use armature_security::hsts::HstsConfig;

let hsts = HstsConfig::new(31536000) // 1 year in seconds
    .include_subdomains(true)
    .preload(true);

let security = SecurityMiddleware::new().with_hsts(hsts);
```

**Output Header:**
```
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
```

### X-Frame-Options

Prevents clickjacking by controlling if your site can be framed.

```rust
use armature_security::frame_guard::FrameGuard;

// Deny all framing
let security = SecurityMiddleware::new()
    .with_frame_guard(FrameGuard::Deny);

// Allow same origin
let security = SecurityMiddleware::new()
    .with_frame_guard(FrameGuard::SameOrigin);

// Allow specific origin
let security = SecurityMiddleware::new()
    .with_frame_guard(FrameGuard::AllowFrom("https://example.com".to_string()));
```

**Output Header:**
```
X-Frame-Options: DENY
X-Frame-Options: SAMEORIGIN
X-Frame-Options: ALLOW-FROM https://example.com
```

### Referrer Policy

Controls how much referrer information is sent with requests.

```rust
use armature_security::referrer_policy::ReferrerPolicy;

let security = SecurityMiddleware::new()
    .with_referrer_policy(ReferrerPolicy::NoReferrer);

// Other options:
// - NoReferrer
// - NoReferrerWhenDowngrade
// - Origin
// - OriginWhenCrossOrigin
// - SameOrigin
// - StrictOrigin
// - StrictOriginWhenCrossOrigin
// - UnsafeUrl
```

**Output Header:**
```
Referrer-Policy: no-referrer
```

### X-XSS-Protection

Enables the browser's XSS filtering.

```rust
use armature_security::xss_filter::XssFilter;

// Enable with blocking
let security = SecurityMiddleware::new()
    .with_xss_filter(XssFilter::EnabledBlock);

// Just enable (don't block)
let security = SecurityMiddleware::new()
    .with_xss_filter(XssFilter::Enabled);

// Disable
let security = SecurityMiddleware::new()
    .with_xss_filter(XssFilter::Disabled);
```

**Output Header:**
```
X-XSS-Protection: 1; mode=block
```

### DNS Prefetch Control

Controls browser DNS prefetching.

```rust
use armature_security::dns_prefetch_control::DnsPrefetchControl;

// Disable DNS prefetching (more privacy)
let security = SecurityMiddleware::new()
    .with_dns_prefetch_control(DnsPrefetchControl::Off);

// Enable DNS prefetching (better performance)
let security = SecurityMiddleware::new()
    .with_dns_prefetch_control(DnsPrefetchControl::On);
```

**Output Header:**
```
X-DNS-Prefetch-Control: off
```

### Expect-CT

Helps detect misissued certificates.

```rust
use armature_security::expect_ct::ExpectCtConfig;

let expect_ct = ExpectCtConfig::new(86400) // 1 day
    .enforce(true)
    .report_uri("https://example.com/report".to_string());

let security = SecurityMiddleware::new().with_expect_ct(expect_ct);
```

**Output Header:**
```
Expect-CT: max-age=86400, enforce, report-uri="https://example.com/report"
```

### Hide X-Powered-By

Removes the `X-Powered-By` header to prevent server fingerprinting.

```rust
let security = SecurityMiddleware::new()
    .hide_powered_by(true);
```

**Result:** `X-Powered-By` header is removed from responses.

## Best Practices

### 1. Use Default Settings for Production

```rust
// Recommended for most applications
let security = SecurityMiddleware::default();
```

### 2. Customize for Your Needs

```rust
// Example: API server with custom CSP
let security = SecurityMiddleware::new()
    .with_csp(
        CspConfig::new()
            .default_src(vec!["'self'".to_string()])
            .connect_src(vec!["'self'".to_string(), "https://api.example.com".to_string()])
    )
    .with_hsts(HstsConfig::new(31536000))
    .hide_powered_by(true);
```

### 3. Test in Development

Use browser developer tools to verify headers are applied:

```bash
# Chrome/Firefox: Network tab → Select request → Headers section
```

### 4. HSTS Considerations

- Start with a shorter `max-age` (e.g., 300 seconds) in testing
- Gradually increase to 1 year (31536000 seconds) in production
- Only enable `preload` after testing thoroughly

### 5. CSP Development

- Start with `report-only` mode:
  ```rust
  let csp = CspConfig::default().report_only(true);
  ```
- Monitor violations before enforcing
- Gradually tighten policies

## API Reference

### `SecurityMiddleware`

Main security middleware struct.

#### Methods

- `new()` - Create with no protections
- `default()` - Create with recommended settings
- `enable_all(max_age: u64)` - Enable all protections
- `with_csp(config: CspConfig)` - Add CSP
- `with_hsts(config: HstsConfig)` - Add HSTS
- `with_frame_guard(guard: FrameGuard)` - Set frame options
- `with_referrer_policy(policy: ReferrerPolicy)` - Set referrer policy
- `with_xss_filter(filter: XssFilter)` - Set XSS filter
- `with_dns_prefetch_control(control: DnsPrefetchControl)` - Control DNS prefetch
- `with_expect_ct(config: ExpectCtConfig)` - Add Expect-CT
- `hide_powered_by(hide: bool)` - Hide X-Powered-By header
- `apply(response: HttpResponse) -> HttpResponse` - Apply headers to response

### Submodules

- `content_security_policy` - CSP configuration
- `hsts` - HSTS configuration
- `frame_guard` - X-Frame-Options
- `referrer_policy` - Referrer-Policy
- `xss_filter` - X-XSS-Protection
- `dns_prefetch_control` - X-DNS-Prefetch-Control
- `expect_ct` - Expect-CT
- `content_type_options` - X-Content-Type-Options
- `download_options` - X-Download-Options
- `permitted_cross_domain_policies` - X-Permitted-Cross-Domain-Policies

## Examples

### Complete Application

```rust
use armature::prelude::*;
use armature_security::SecurityMiddleware;

#[controller("/")]
struct HomeController;

impl HomeController {
    #[get("/")]
    fn index(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        let security = SecurityMiddleware::default();
        let response = HttpResponse::ok()
            .with_body(b"Hello, secure world!".to_vec());
        
        Ok(security.apply(response))
    }
}
```

### API with Custom Security

```rust
use armature_security::{SecurityMiddleware, frame_guard::FrameGuard};

let security = SecurityMiddleware::new()
    .with_frame_guard(FrameGuard::SameOrigin)
    .hide_powered_by(true);

// Apply to all API responses
let secured = security.apply(api_response);
```

## Summary

The Armature Security middleware provides comprehensive protection against common web vulnerabilities:

- ✅ Easy to use - `SecurityMiddleware::default()` for most cases
- ✅ Highly customizable - Configure each feature individually
- ✅ Production-ready - Based on industry best practices
- ✅ Well-tested - 21+ unit tests covering all features
- ✅ Type-safe - Full Rust type safety

For more examples, see `examples/security_example.rs` in the repository.

