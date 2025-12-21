# armature-security

Security utilities for the Armature framework.

## Features

- **CSRF Protection** - Cross-site request forgery prevention
- **XSS Prevention** - Content sanitization
- **CORS** - Cross-origin resource sharing
- **Security Headers** - Best practice headers
- **Input Sanitization** - Clean user input

## Installation

```toml
[dependencies]
armature-security = "0.1"
```

## Quick Start

```rust
use armature_security::{CsrfMiddleware, CorsMiddleware, SecurityHeaders};

let app = Application::new()
    .with_middleware(SecurityHeaders::default())
    .with_middleware(CorsMiddleware::new().allow_origin("https://example.com"))
    .with_middleware(CsrfMiddleware::new("secret"));
```

## CSRF Protection

```rust
let csrf = CsrfMiddleware::new("secret")
    .token_header("X-CSRF-Token")
    .cookie_name("csrf_token");
```

## CORS

```rust
let cors = CorsMiddleware::new()
    .allow_origin("https://example.com")
    .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
    .allow_headers(vec!["Content-Type", "Authorization"])
    .max_age(3600);
```

## Security Headers

```rust
let headers = SecurityHeaders::new()
    .content_security_policy("default-src 'self'")
    .strict_transport_security(31536000)
    .x_frame_options("DENY")
    .x_content_type_options("nosniff");
```

## License

MIT OR Apache-2.0

