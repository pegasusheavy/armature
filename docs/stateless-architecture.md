# Stateless Architecture

Armature is designed as a **completely stateless** framework, following RESTful and cloud-native best practices.

## Core Principles

### 1. No Server-Side Sessions

Armature's core keeps no per-user state in process memory:
- No in-memory session management
- No server-side user state between requests
- No sticky-session requirement

When an application does need server-side sessions (for example to hold an
OAuth2 `PendingAuth` between redirect and callback, or for cookie-based browser
logins), the optional `armature-session` crate stores them in an **external**
store (Redis, memcached, CouchDB), so every instance stays interchangeable. See
the [Session guide](session-guide.md).

### 2. Stateless Authentication

All authentication is token-based and stateless:

```rust,ignore
// JWT tokens carry all user information
let token = jwt_manager.sign(&user_claims)?;

// Each request is independent: verify the token on every request.
// `verify_access` rejects refresh tokens presented as access tokens.
let claims: UserClaims = jwt_manager.verify_access(&token)?;
```

**Key Points:**
- User identity is embedded in tokens (JWT)
- No server-side session lookup required
- Each request contains all necessary auth information
- Tokens can be validated without database lookups

### 3. Request Context Only

User information exists only for the duration of a single request:

```rust,ignore
use armature_auth::UserContext;

impl UserController {
    async fn get_profile(&self, req: HttpRequest) -> Result<Json<User>, Error> {
        // JwtAuthMiddleware verified the bearer token and attached the
        // identity to this request only.
        let user = req
            .extension::<UserContext>()
            .ok_or_else(|| Error::Unauthorized("not authenticated".into()))?;

        // No session storage
        Ok(Json(get_user_from_db(&user.user_id).await?))
    }
}
```

## Authentication Patterns

### JWT-Based Authentication

```rust,ignore
use armature_auth::JwtAuthMiddleware;
use armature_jwt::{JwtConfig, JwtManager};

// At least 32 random bytes; JwtConfig::new rejects short, placeholder or
// low-entropy secrets and returns a Result.
let secret = std::env::var("JWT_SECRET")?;
let jwt_manager = JwtManager::new(JwtConfig::new(secret)?)?;

// Create token at login
let token = jwt_manager.sign(&serde_json::json!({
    "sub": user.id,
    "roles": user.roles,
    "exp": chrono::Utc::now().timestamp() + 900,
}))?;

// Client sends the token with each request:
// Authorization: Bearer <token>

// Server verifies it on each request: register the middleware globally so it
// runs before every guard.
let app = Application::create::<AppModule>()
    .await
    .use_middleware(JwtAuthMiddleware::new(jwt_manager));
```

### OAuth2/OIDC Integration

The OAuth2 redirect flow is the one place a short-lived server-side record is
required. `authorization_url` returns the URL together with a `PendingAuth`
(PKCE verifier, `state`, OIDC `nonce`) that must be kept **server-side** until
the callback and is valid for 10 minutes. Keep it in an external store keyed by
`pending.state()`, never in a client-readable cookie or form field.

```rust,ignore
use armature_auth::OAuth2Provider;

// 1. Start the flow
let (auth_url, pending) = provider.authorization_url()?;
pending_store.put(pending.state(), &pending).await?; // e.g. armature-session / Redis

// 2. Callback: look up the pending flow by the returned state
let pending = pending_store.take(&returned_state).await?;
let token = provider.exchange_code(code, &returned_state, pending).await?;

// 3. Issue your own JWT from the verified user info
let user_info = provider.get_user_info(&token).await?;
let jwt = jwt_manager.sign(&claims_for(&user_info))?;
```

`state` is checked in constant time and the OIDC `id_token` is verified before
its claims are exposed. See the [OAuth2 providers guide](oauth2-providers-guide.md).

### SAML 2.0

SP-initiated SAML also needs a short-lived server-side record: the
AuthnRequest ID, which the IdP echoes as `InResponseTo`. Validation requires it,
and accepted assertion IDs go into a replay cache. Use a shared
`AssertionReplayCache` (`SamlServiceProvider::with_replay_cache`) when you run
several instances; the default `InMemoryReplayCache` is per process.

```rust,ignore
use armature_auth::saml::SamlProvider;

// 1. Generate the AuthnRequest and remember its ID server-side
let request = saml_provider.create_auth_request()?;
pending_store.put(&login_id, &request.request_id).await?;

// 2. On the ACS callback, validate against that ID
let request_id = pending_store.take(&login_id).await?;
let assertion = saml_provider
    .validate_response(&saml_response, &request_id, None, None)
    .await?;

// 3. Create a JWT from the assertion
let jwt = jwt_manager.sign(&serde_json::json!({ "sub": assertion.name_id }))?;
```

## Why Stateless?

### Benefits

1. **Horizontal Scalability**
   - Any server can handle any request
   - No session affinity (sticky sessions) needed
   - Load balancing is trivial

2. **Cloud-Native**
   - Works seamlessly with containers
   - No shared state between instances
   - Perfect for Kubernetes, serverless, etc.

3. **Reliability**
   - No session store to fail
   - No session synchronization issues
   - Server restarts don't affect users

4. **Performance**
   - No session lookups
   - No database queries for auth
   - Token validation is cryptographic (fast)

5. **Security**
   - No server-side session to fixate or hijack
   - Token expiration is built-in
   - Tokens can still be stolen: keep access tokens short-lived

### Trade-offs

1. **Token Size**
   - JWTs can be larger than session IDs
   - Include only necessary claims

2. **Token Revocation**
   - Tokens are valid until expiry
   - Use short expiration times (15-60 minutes)
   - Implement token refresh flow
   - For immediate revocation, maintain token blacklist (separate concern)

3. **Client Responsibility**
   - Client must store and manage tokens
   - Client must handle token refresh

## Anti-Patterns to Avoid

### ❌ Don't Create Session Storage

```rust
// BAD - Don't do this
lazy_static! {
    static ref SESSIONS: Arc<Mutex<HashMap<String, UserSession>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

// This breaks stateless architecture
fn store_session(session_id: String, user: User) {
    let mut sessions = SESSIONS.lock().unwrap();
    sessions.insert(session_id, UserSession { user });
}
```

### ❌ Don't Cache User Data Server-Side

```rust
// BAD - Don't do this
lazy_static! {
    static ref USER_CACHE: Arc<Mutex<HashMap<String, User>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

// This creates state
fn cache_user(user_id: String, user: User) {
    let mut cache = USER_CACHE.lock().unwrap();
    cache.insert(user_id, user);
}
```

### ❌ Don't Store Request-Specific State

```rust
// BAD - Don't do this
static mut CURRENT_USER: Option<User> = None;

// This breaks concurrent requests
fn set_current_user(user: User) {
    unsafe {
        CURRENT_USER = Some(user);
    }
}
```

## Correct Patterns

### ✅ Extract User from Token Each Request

```rust,ignore
// GOOD - Stateless: JwtAuthMiddleware verified the token for this request
async fn get_user(req: HttpRequest) -> Result<User, Error> {
    let identity = req
        .extension::<UserContext>()
        .ok_or_else(|| Error::Unauthorized("not authenticated".into()))?;

    // Optionally fetch from database
    // (database is external state, not server state)
    let user = database.find_user(&identity.user_id).await?;

    Ok(user)
}
```

### ✅ Use Guards for Auth

Guards authorize; they do not parse credentials. The built-in guards fail
closed: `armature_core::AuthenticationGuard` admits a request only when an
authentication layer has attached a verified identity (`VerifiedIdentity` or
`RequestRoles`), and `armature_auth::{AuthGuard, RoleGuard, PermissionGuard}`
require the verified `UserContext`. `JwtAuthMiddleware` attaches both.

```rust,ignore
use armature_auth::{JwtAuthMiddleware, RoleGuard};

// Global middleware runs before every guard (middleware -> guards -> handler).
let app = Application::create::<AppModule>()
    .await
    .use_middleware(JwtAuthMiddleware::new(jwt_manager));

// On a controller or route:
// #[guard(RoleGuard::any(vec!["admin".to_string()]))]
```

A bare `Authorization: Bearer ...` header no longer satisfies any built-in
guard.

### ✅ Use Middleware for Request Context

Use `JwtAuthMiddleware` (or `JwtAuthMiddleware::optional` for routes that also
serve anonymous callers). It verifies the token with `verify_access`, then
attaches `armature_auth::UserContext` and `armature_core::VerifiedIdentity` as
request **extensions**, which exist only for that request.

If you write your own authentication middleware, attach identity with
`req.insert_extension(VerifiedIdentity::new(subject))`. Never copy it into a
request header such as `x-user-id`: headers are client-controlled, so a client
could send that header itself.

## Token Refresh Pattern

For long-lived sessions without server-side state:

```rust,ignore
use armature_jwt::TokenPair;

// Login: return both tokens. The access token carries `typ: at+jwt`, the
// refresh token `typ: refresh+jwt` and `token_use: refresh`.
let pair: TokenPair = jwt_manager.generate_token_pair(&user_claims)?;

// /auth/refresh: only a refresh token is accepted here...
let new_pair = jwt_manager.refresh_token::<UserClaims>(&pair.refresh_token)?;

// ...and never as a bearer token: JwtAuthMiddleware and
// JwtManager::verify_access reject refresh tokens.
```

## Rate Limiting (Stateless)

Even rate limiting can be stateless using distributed stores:

```rust
// Use external store (Redis, etc.) not in-memory
use redis::AsyncCommands;

async fn check_rate_limit(ip: &str) -> Result<bool, Error> {
    let mut conn = redis_client.get_async_connection().await?;
    let key = format!("rate:{}",  ip);

    let count: u32 = conn.get(&key).await.unwrap_or(0);

    if count > 100 {
        return Err(Error::TooManyRequests("Rate limit exceeded".into()));
    }

    conn.incr(&key, 1).await?;
    conn.expire(&key, 60).await?; // 60 seconds

    Ok(true)
}
```

**Note:** This uses external state (Redis), not server state. Any server instance can check the rate limit.

## WebSockets and SSE

Even real-time features remain stateless:

```rust
// WebSocket connections are ephemeral
// No user state persists beyond connection lifetime

async fn handle_websocket(req: HttpRequest) -> Result<(), Error> {
    // Authenticate via token in initial request
    let token = req.query_params.get("token")
        .ok_or_else(|| Error::Forbidden("Missing token".into()))?;

    let claims: UserClaims = jwt_manager.verify_access(token)?;

    // Connection is scoped to this handler
    // When connection closes, all state is gone

    websocket_upgrade(req, move |socket| async move {
        // Handle messages
        // User info from claims (not stored globally)
    }).await
}
```

## Deployment Considerations

### Multiple Instances

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│ Server 1 │     │ Server 2 │     │ Server 3 │
│ (stateless)    │ (stateless)    │ (stateless)
└────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │
     └────────────────┴────────────────┘
                      │
              ┌───────▼────────┐
              │  Load Balancer │
              └───────┬────────┘
                      │
              ┌───────▼────────┐
              │     Client      │
              │  (stores JWT)   │
              └─────────────────┘
```

Each server instance:
- Can handle any request
- Validates JWT independently
- No shared state needed
- No session synchronization

### Container/Kubernetes Friendly

```yaml
# Perfect for Kubernetes
apiVersion: apps/v1
kind: Deployment
metadata:
  name: armature-app
spec:
  replicas: 10  # Scale freely
  template:
    spec:
      containers:
      - name: app
        image: myapp:latest
        # No volume mounts for sessions
        # No sticky sessions needed
```

## Summary

Armature enforces stateless architecture by:

1. **No in-process sessions** - Optional sessions (`armature-session`) live in an external store
2. **JWT-based auth** - All user context in verified tokens
3. **Request-scoped data** - No persistence between requests
4. **External stores only** - Database, cache (Redis) are external, not in-server memory
5. **Cloud-native design** - Horizontal scaling without shared state

This design ensures your Armature application can:
- Scale horizontally with ease
- Deploy anywhere (containers, serverless, VMs)
- Handle millions of requests across multiple instances
- Recover from failures without user impact
- Keep authentication independent of which instance serves a request

**Remember:** If you need to "remember" something about a user, put it in the JWT or look it up from a database on each request. Never store it in server memory.

