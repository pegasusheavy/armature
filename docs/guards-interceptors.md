# Guards and Interceptors

Armature provides a powerful middleware system through **Guards** and **Interceptors**, inspired by Angular and NestJS.

## Table of Contents

- [Overview](#overview)
- [Guards](#guards)
  - [Request Pipeline Order](#request-pipeline-order)
  - [Guards Fail Closed](#guards-fail-closed)
  - [Built-in Guards](#built-in-guards)
  - [Custom Guards](#custom-guards)
  - [Usage Examples](#guard-usage-examples)
- [Interceptors](#interceptors)
  - [Built-in Interceptors](#built-in-interceptors)
  - [Custom Interceptors](#custom-interceptors)
  - [Usage Examples](#interceptor-usage-examples)
- [Complete Example](#complete-example)

## Overview

### Guards

Guards are used to **protect routes** by determining whether a request should be allowed to proceed. They run **after** middleware and **before** the route handler, and can:

- Authenticate users
- Check permissions/roles
- Validate API keys
- Implement custom authorization logic

Guards return `Result<bool, Error>`:
- `Ok(true)` - Allow request to proceed
- `Ok(false)` - Block request (should return error instead)
- `Err(e)` - Block request with specific error

### Interceptors

Interceptors are used to **transform requests and responses**. They can:

- Log requests and responses
- Modify response data
- Cache responses
- Add custom headers
- Measure performance
- Handle errors

Interceptors wrap the handler execution and can modify both input and output.

## Guards

### Request Pipeline Order

Armature follows the NestJS order. For every request:

1. The request path is canonicalized once (`//` collapses, percent-encoded unreserved
   characters decode). Paths containing `.`/`..` segments, `%2F`, `%5C`, `%00`, `\` or a
   malformed escape are refused with `400` before any middleware runs.
2. Global middleware (`Application::use_middleware`), in the order added.
3. Global guards (`Application::with_guard`).
4. Route-scoped middleware, then route-scoped guards: module guards, controller
   `#[middleware]` then controller `#[guard]`, and `RouteGroup` layers.
5. The handler.

Because middleware runs before guards, an authentication middleware can attach a verified
identity that the guards then inspect. Middleware also sees (and can post-process) a guard's
`401`/`403`.

### Guards Fail Closed

The built-in authentication and role guards never trust the shape of an `Authorization`
header. They admit a request only when an authentication layer has **verified** a credential
and attached one of these request extensions:

| Extension | Inserted by | Read by |
|-----------|-------------|---------|
| `armature_core::VerifiedIdentity` | `armature_auth::JwtAuthMiddleware` (or your own verified auth layer) | `AuthenticationGuard`, empty-role `RolesGuard` |
| `armature_core::RequestRoles` | `JwtAuthMiddleware` | `RolesGuard` |
| `armature_auth::UserContext` | `JwtAuthMiddleware` | `armature_auth::{AuthGuard, RoleGuard, PermissionGuard}` |

Without such a layer, every guarded route answers `401`. Never insert these extensions from
anything the client controls (a header such as `x-user-id`/`x-user-role`, a cookie, or an
unverified token claim).

### Built-in Guards

#### JwtAuthMiddleware (armature-auth)

Verifies a JWT bearer token and attaches the caller's identity so the guards can authorize.
Register it globally with `Application::use_middleware` so it runs before every guard:

```rust
use armature::prelude::*;
use armature_auth::JwtAuthMiddleware;
use armature_jwt::{JwtConfig, JwtManager};

// `JwtConfig::new` returns a Result: the secret must be at least 32 bytes and not a
// known placeholder. Load it from a secret store, e.g. `openssl rand -base64 48`.
let secret = std::env::var("JWT_SECRET")?;
let jwt_manager = JwtManager::new(JwtConfig::new(secret)?)?;

let app = Application::new(container, router)
    // Required mode: a missing/invalid/expired token is rejected with 401.
    .use_middleware(JwtAuthMiddleware::new(jwt_manager));

// Optional mode: attaches identity when a valid token is present, otherwise passes
// through (with any rejected `Authorization` header stripped) so a downstream guard
// makes the decision.
// .use_middleware(JwtAuthMiddleware::optional(jwt_manager));
```

On a valid **access** token it inserts a `UserContext` (from the `sub`/`roles`/`permissions`
claims, configurable with `with_subject_claim`/`with_roles_claim`/`with_permissions_claim`),
an `armature_core::RequestRoles` and an `armature_core::VerifiedIdentity` carrying the
subject. Refresh tokens, tokens without a subject, and requests with more than one
`Authorization` header are rejected.

#### AuthenticationGuard

Admits only requests carrying a verified identity (`VerifiedIdentity` or `RequestRoles`).
A bare `Authorization: Bearer …` header is **not** consulted; without a verified identity
the guard returns `Error::Unauthorized` (`401`).

```rust
use armature::{AuthenticationGuard, Guard, GuardContext, HttpRequest, VerifiedIdentity};

// A forged header alone is rejected.
let mut forged = HttpRequest::new("GET", "/me");
forged.headers.insert("authorization", "Bearer anything");
assert!(AuthenticationGuard.can_activate(&GuardContext::new(forged)).await.is_err());

// The extension an authentication middleware inserts after verifying a credential.
let mut verified = HttpRequest::new("GET", "/me");
verified.extensions.insert(VerifiedIdentity::new("user-1"));
assert!(AuthenticationGuard.can_activate(&GuardContext::new(verified)).await?);
```

#### RolesGuard

Requires a verified `RequestRoles` extension containing at least one of the listed roles.
Missing roles or no matching role is `Error::Forbidden` (`403`).

```rust
use armature::{Guard, GuardContext, RolesGuard};

let guard = RolesGuard::new(vec!["admin".to_string(), "moderator".to_string()]);
let context = GuardContext::new(request);

match guard.can_activate(&context).await {
    Ok(true) => println!("Has required role!"),
    Ok(false) | Err(_) => println!("Insufficient permissions"),
}
```

An **empty** role list (`RolesGuard::new(vec![])`) means "any authenticated caller": the
request must still carry a verified identity. It never means "allow everyone". The same
holds for `armature_auth`'s `RoleGuard::any(vec![])` and `PermissionGuard::any(vec![])`.

#### ApiKeyGuard

Validates the `x-api-key` header against a list of keys. Every configured key is compared in
constant time, so response timing does not reveal which key a guess shares a prefix with.
Load keys from configuration, and use long random values.

```rust
use armature::{ApiKeyGuard, Guard, GuardContext};

let valid_keys: Vec<String> = std::env::var("API_KEYS")?
    .split(',')
    .map(str::to_string)
    .collect();

let guard = ApiKeyGuard::new(valid_keys);
let context = GuardContext::new(request);

match guard.can_activate(&context).await {
    Ok(true) => println!("Valid API key"),
    Ok(false) | Err(_) => println!("Invalid API key"),
}
```

**Usage:**
```bash
curl http://localhost:3000/api/data \
  -H "x-api-key: $API_KEY"
```

### Custom Guards

Create custom guards by implementing the `Guard` trait:

```rust
use armature::{Guard, GuardContext, Error};
use async_trait::async_trait;
use std::net::IpAddr;

pub struct IpAllowlistGuard {
    allowed_ips: Vec<IpAddr>,
    /// Number of reverse proxies in front of the app whose `X-Forwarded-For`
    /// entries you trust. `0` uses the socket peer and ignores the header.
    trusted_proxy_depth: usize,
}

impl IpAllowlistGuard {
    pub fn new(ips: Vec<IpAddr>, trusted_proxy_depth: usize) -> Self {
        Self { allowed_ips: ips, trusted_proxy_depth }
    }
}

#[async_trait]
impl Guard for IpAllowlistGuard {
    async fn can_activate(&self, context: &GuardContext) -> Result<bool, Error> {
        // Never read the leftmost `X-Forwarded-For` hop or `X-Real-IP` directly:
        // clients can set both. `client_address` walks the chain from the right.
        let client_ip = context
            .request
            .client_address(self.trusted_proxy_depth)
            .ok_or_else(|| Error::Forbidden("Cannot determine client IP".to_string()))?;

        if self.allowed_ips.contains(&client_ip) {
            Ok(true)
        } else {
            Err(Error::Forbidden("Client address not allowed".to_string()))
        }
    }
}
```

#### Function-based Custom Guard

For simpler cases, use `CustomGuard`:

```rust
use armature::{CustomGuard, GuardContext, Error};
use chrono::Datelike;

let guard = CustomGuard::new(|_context: &GuardContext| {
    // Only allow requests on weekdays
    let weekday = chrono::Local::now().weekday();
    if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
        Err(Error::Forbidden("Service only available on weekdays".to_string()))
    } else {
        Ok(true)
    }
});
```

### Guard Usage Examples

#### Protecting Routes with a RouteGroup

Register routes **through** a `RouteGroup` with `RouteGroup::route` (or `add_route`). The
group's prefix, middleware and guards are bound to each route, so they run however the
client spells the path (`/api/protected`, `//api/protected`, ...):

```rust
use armature::prelude::*;
use armature::{AuthenticationGuard, RouteGroup, Router};

async fn protected(_req: HttpRequest) -> Result<HttpResponse, Error> {
    HttpResponse::ok().with_json(&serde_json::json!({ "data": "protected data" }))
}

let mut router = Router::new();

RouteGroup::new()
    .prefix("/api")
    .guard(Box::new(AuthenticationGuard))
    .route(&mut router, HttpMethod::GET, "/protected", protected);
```

#### Chaining Multiple Guards

All guards in a group must pass; they run in the order added:

```rust
use armature::{AuthenticationGuard, RolesGuard, RouteGroup};

RouteGroup::new()
    .prefix("/api/admin")
    .guard(Box::new(AuthenticationGuard))
    .guard(Box::new(RolesGuard::new(vec!["admin".to_string()])))
    .route(&mut router, HttpMethod::DELETE, "/users/:id", delete_user);
```

#### Global Guards

`Application::with_guard` adds a guard that runs for every request, after global
middleware:

```rust
use std::sync::Arc;

let app = Application::new(container, router)
    .use_middleware(JwtAuthMiddleware::new(jwt_manager))
    .with_guard(Arc::new(AuthenticationGuard));
```

#### Module Guards

Guards declared by a `#[module]` are bound to the routes that module's controllers
register; they do not apply to other routes that happen to share a URL prefix. If a module
guard cannot be constructed, `Application::create` panics rather than serve the module's
routes unguarded; use `Application::try_create` to handle the error:

```rust
let app = Application::try_create::<AppModule>().await?;
```

## Interceptors

### Built-in Interceptors

#### LoggingInterceptor

Logs all incoming requests and outgoing responses with timing.

```rust
use armature::{Interceptor, LoggingInterceptor, ExecutionContext};

let interceptor = LoggingInterceptor;

// Output:
// → GET /api/users
// ← GET /api/users - 200 (12.3ms)
```

#### TransformInterceptor

Transforms responses using a custom function.

```rust
use armature::{Interceptor, TransformInterceptor, HttpResponse};

let interceptor = TransformInterceptor::new(|mut response| {
    // Add custom header to all responses
    response.headers.insert(
        "X-Powered-By".to_string(),
        "Armature".to_string(),
    );
    response
});
```

#### CacheInterceptor

Caches responses for a specified TTL (Time To Live).

```rust
use armature::{Interceptor, CacheInterceptor};

let interceptor = CacheInterceptor::new(60); // Cache for 60 seconds
```

### Custom Interceptors

Create custom interceptors by implementing the `Interceptor` trait:

```rust
use armature::{Interceptor, ExecutionContext, HttpResponse, Error};
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

pub struct CompressionInterceptor {
    min_size: usize,
}

impl CompressionInterceptor {
    pub fn new(min_size: usize) -> Self {
        Self { min_size }
    }
}

#[async_trait]
impl Interceptor for CompressionInterceptor {
    async fn intercept(
        &self,
        context: ExecutionContext,
        next: Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>,
    ) -> Result<HttpResponse, Error> {
        let mut response = next.await?;

        // Compress if response is large enough
        if response.body.len() > self.min_size {
            // Compress the body (pseudo-code)
            // response.body = compress(response.body);
            response.headers.insert(
                "Content-Encoding".to_string(),
                "gzip".to_string(),
            );
        }

        Ok(response)
    }
}
```

#### Metrics Interceptor Example

```rust
use armature::{Interceptor, ExecutionContext, HttpResponse, Error};
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct MetricsInterceptor {
    request_count: Arc<AtomicU64>,
    total_duration_ms: Arc<AtomicU64>,
}

impl MetricsInterceptor {
    pub fn new() -> Self {
        Self {
            request_count: Arc::new(AtomicU64::new(0)),
            total_duration_ms: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn get_metrics(&self) -> (u64, u64) {
        (
            self.request_count.load(Ordering::Relaxed),
            self.total_duration_ms.load(Ordering::Relaxed),
        )
    }
}

#[async_trait]
impl Interceptor for MetricsInterceptor {
    async fn intercept(
        &self,
        context: ExecutionContext,
        next: Pin<Box<dyn Future<Output = Result<HttpResponse, Error>> + Send>>,
    ) -> Result<HttpResponse, Error> {
        let start = std::time::Instant::now();

        let result = next.await;

        let duration = start.elapsed().as_millis() as u64;
        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.total_duration_ms.fetch_add(duration, Ordering::Relaxed);

        result
    }
}
```

### Interceptor Usage Examples

#### Applying Interceptors to Routes

Currently, interceptors need to be manually applied within route handlers. A more integrated approach (similar to NestJS's `@UseInterceptors()`) can be added to the macro system in the future.

```rust
use armature::prelude::*;

async fn get_data(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Simulate interceptor logging
    let start = std::time::Instant::now();
    println!("→ {} {}", req.method, req.path);

    // Handler logic
    let response = HttpResponse::ok().with_json(&serde_json::json!({ "data": "value" }))?;

    // Log completion
    let duration = start.elapsed();
    println!("← {} {} - {} ({:?})", req.method, req.path, response.status, duration);

    Ok(response)
}

router.get("/api/data", get_data);
```

## Complete Example

Here's a complete example: a JWT authentication middleware registered globally, a public
route, an authenticated route, and an admin-only route.

```rust
use armature::prelude::*;
use armature::{AuthenticationGuard, RolesGuard, RouteGroup, Router};
use armature_auth::JwtAuthMiddleware;
use armature_jwt::{JwtConfig, JwtManager};
use serde::Serialize;

#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

fn message(text: &str) -> Result<HttpResponse, Error> {
    HttpResponse::ok().with_json(&ApiResponse { message: text.to_string() })
}

async fn public_data(_req: HttpRequest) -> Result<HttpResponse, Error> {
    message("Public data")
}

async fn protected_data(_req: HttpRequest) -> Result<HttpResponse, Error> {
    message("Protected data")
}

async fn admin_delete(_req: HttpRequest) -> Result<HttpResponse, Error> {
    message("Admin action completed")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A >= 32-byte random secret from the environment / a secret store.
    let jwt_manager = JwtManager::new(JwtConfig::new(std::env::var("JWT_SECRET")?)?)?;

    let mut router = Router::new();

    // Public endpoint - no guards
    router.get("/public", public_data);

    // Protected endpoint - verified identity required
    RouteGroup::new()
        .guard(Box::new(AuthenticationGuard))
        .route(&mut router, HttpMethod::GET, "/protected", protected_data);

    // Admin endpoint - verified identity with the `admin` role
    RouteGroup::new()
        .prefix("/admin")
        .guard(Box::new(RolesGuard::new(vec!["admin".to_string()])))
        .route(&mut router, HttpMethod::DELETE, "/delete", admin_delete);

    // Optional mode, so `/public` stays reachable without a token while the
    // guards still require the identity the middleware attaches.
    let app = Application::new(Container::new(), router)
        .use_middleware(JwtAuthMiddleware::optional(jwt_manager));

    println!("Server running on http://localhost:3000");
    app.listen(3000).await?;
    Ok(())
}
```

### Testing the Example

```bash
# 1. Public endpoint (no auth required)
curl http://localhost:3000/public
# → 200 OK

# 2. Protected endpoint without a token
curl http://localhost:3000/protected
# → 401 Unauthorized

# 3. Protected endpoint with a token that does not verify
curl http://localhost:3000/protected \
  -H "Authorization: Bearer not-a-real-jwt"
# → 401 Unauthorized

# 4. Admin endpoint with a valid token lacking the `admin` role
curl -X DELETE http://localhost:3000/admin/delete \
  -H "Authorization: Bearer $USER_JWT"
# → 403 Forbidden

# 5. Admin endpoint with a valid access token carrying `"roles": ["admin"]`
curl -X DELETE http://localhost:3000/admin/delete \
  -H "Authorization: Bearer $ADMIN_JWT"
# → 200 OK
```

## Best Practices

1. **Authenticate in middleware, authorize in guards**: register an authentication
   middleware such as `JwtAuthMiddleware` with `Application::use_middleware` (or as
   controller `#[middleware]`); guards then only check the verified identity it attaches.

2. **Guard Order**: Apply guards in order of cost - fast checks first (API key) before expensive ones (database lookups)

3. **Error Messages**: Be specific but don't leak security information:
   ```rust
   // Good
   Err(Error::Forbidden("Invalid credentials".to_string()))

   // Bad - leaks information
   Err(Error::Forbidden("User john@example.com not found".to_string()))
   ```

4. **Guard Composition**: Create reusable guard combinations:
   ```rust
   async fn apply_admin_guards(context: &GuardContext) -> Result<(), Error> {
       // Missing identity -> 401 Unauthorized
       AuthenticationGuard.can_activate(context).await?;

       // Missing or wrong role -> 403 Forbidden
       RolesGuard::new(vec!["admin".to_string()])
           .can_activate(context)
           .await?;
       Ok(())
   }
   ```

5. **Interceptor Performance**: Keep interceptors lightweight - avoid heavy computation or I/O

6. **Logging**: Use structured logging in interceptors:
   ```rust
   println!(
       "request_id={} method={} path={} duration_ms={} status={}",
       request_id, method, path, duration, status
   );
   ```

## Decorator Syntax

Guards and middleware can also be applied with decorators. See the
[Use Guard Guide](use-guard-guide.md) and [Use Middleware Guide](use-middleware-guide.md):

```rust
#[controller("/api")]
#[middleware(LoggerMiddleware::new())] // controller middleware runs first...
#[guard(AuthGuard)]                    // ...then controller guards
struct ApiController;
```

## Future Enhancements

The following features are planned for future releases:

1. **Interceptor Chaining**: Multiple interceptors with defined order
   ```rust
   #[use_interceptors(LoggingInterceptor, CacheInterceptor)]
   ```

2. **Exception Filters**: Dedicated error handling interceptors
   ```rust
   #[use_filters(HttpExceptionFilter)]
   ```

## See Also

- [Authentication Guide](auth-guide.md) - JWT and OAuth2 integration
- [Dependency Injection](di-guide.md) - Injecting guards and interceptors
- [Middleware](use-middleware-guide.md) - Alternative middleware patterns

