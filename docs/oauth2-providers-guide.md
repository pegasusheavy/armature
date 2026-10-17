# OAuth2 Providers Guide

Complete guide to using OAuth2/OIDC providers with Armature authentication.

## Table of Contents

1. [Overview](#overview)
2. [Supported Providers](#supported-providers)
3. [Google OAuth2](#google-oauth2)
4. [Microsoft Entra](#microsoft-entra-azure-ad)
5. [AWS Cognito](#aws-cognito)
6. [Okta](#okta)
7. [Auth0](#auth0)
8. [OAuth2 Flow](#oauth2-flow)
9. [Complete Example](#complete-example)
10. [Best Practices](#best-practices)

## Overview

`armature-auth` provides built-in support for major enterprise OAuth2 and OIDC identity providers. This allows you to integrate with existing authentication systems without building your own user management.

### Benefits

- **Single Sign-On (SSO)**: Users authenticate with their existing accounts
- **Enterprise Ready**: Integrates with corporate identity providers
- **Security**: PKCE (S256) on every flow, library-verified `state`, and for
  OIDC providers a verified `id_token` with `nonce`
- **No User Management**: Delegate user storage to providers
- **Multi-Provider**: Support multiple providers simultaneously

## Supported Providers

| Provider | Type | Use Case |
|----------|------|----------|
| **Google** | Consumer | Gmail users, Google Workspace |
| **Microsoft Entra** | Enterprise | Azure AD, Office 365, Enterprise SSO |
| **AWS Cognito** | Cloud | AWS-native applications |
| **Okta** | Enterprise | Enterprise SSO, SAML bridge |
| **Auth0** | Universal | Multi-provider, custom branding |

Google, Microsoft Entra, Okta, Auth0 and Cognito (with a user pool ID) are
configured for OpenID Connect: their `id_token` is verified against the
provider's JWKS (asymmetric algorithms only; `iss`, `aud`, `azp`, `exp` and
`nonce` checked) before it is exposed. Provider domains (Okta, Auth0, the
Cognito user-pool domain, the Microsoft tenant) must be bare hostnames: a
scheme, path, port, `@` or whitespace is rejected by the constructor.

All provider HTTP calls (token, userinfo, JWKS) use a rustls client that does
not follow redirects, so a 307/308 from a token endpoint cannot re-send the
client secret and code elsewhere. Client secrets and tokens are redacted from
`Debug` output.

## Google OAuth2

### Setup

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing
3. Navigate to "APIs & Services" > "Credentials"
4. Create OAuth 2.0 Client ID
5. Add authorized redirect URI: `http://localhost:3000/auth/google/callback`

### Configuration

```rust
use armature_auth::providers::{GoogleConfig, GoogleProvider};

let config = GoogleConfig::new(
    "your-client-id.apps.googleusercontent.com".to_string(),
    "your-client-secret".to_string(),
    "http://localhost:3000/auth/google/callback".to_string(),
);

// Optional: Custom scopes
let config = config.with_scopes(vec![
    "openid".to_string(),
    "email".to_string(),
    "profile".to_string(),
    "https://www.googleapis.com/auth/userinfo.profile".to_string(),
]);

let provider = GoogleProvider::new(config)?;
```

### User Info

User info comes from Google's OIDC userinfo endpoint and must name the same
`sub` as the verified `id_token`:
- `sub`: Google user ID (key accounts on this)
- `email`: User's Gmail address
- `name`: Full name
- `picture`: Profile picture URL
- `email_verified`: Email verification status

## Microsoft Entra (Azure AD)

### Setup

1. Go to [Azure Portal](https://portal.azure.com/)
2. Navigate to "Azure Active Directory"
3. Select "App registrations" > "New registration"
4. Add redirect URI: `http://localhost:3000/auth/microsoft/callback`
5. Create a client secret in "Certificates & secrets"

### Configuration

```rust
use armature_auth::providers::{MicrosoftEntraConfig, MicrosoftEntraProvider};

// Option 1: Common tenant (any Azure AD account)
let config = MicrosoftEntraConfig::common(
    "your-application-id".to_string(),
    "your-client-secret".to_string(),
    "http://localhost:3000/auth/microsoft/callback".to_string(),
);

// Option 2: Organizations only (work/school accounts)
let config = MicrosoftEntraConfig::organizations(
    "your-application-id".to_string(),
    "your-client-secret".to_string(),
    "http://localhost:3000/auth/microsoft/callback".to_string(),
);

// Option 3: Consumers only (personal Microsoft accounts)
let config = MicrosoftEntraConfig::consumers(
    "your-application-id".to_string(),
    "your-client-secret".to_string(),
    "http://localhost:3000/auth/microsoft/callback".to_string(),
);

// Option 4: Specific tenant
let config = MicrosoftEntraConfig::new(
    "your-application-id".to_string(),
    "your-client-secret".to_string(),
    "http://localhost:3000/auth/microsoft/callback".to_string(),
    "your-tenant-id".to_string(),
);

let provider = MicrosoftEntraProvider::new(config)?;
```

### Tenant Types

- **common**: Any Azure AD account (personal or work/school)
- **organizations**: Work or school accounts only
- **consumers**: Personal Microsoft accounts only
- **{tenant-id}**: Specific Azure AD tenant (a GUID is matched exactly against the token issuer)

### User Info

`get_user_info` requires the verified `id_token` (request the `openid`
scope) and keys the user on its immutable tenant and object IDs:

- `sub`: `"{tid}:{oid}"` from the verified `id_token`. `oid` is only unique
  within a tenant, so always key accounts on this combined value.
- `email`: Graph `mail` (or `userPrincipalName`), always reported with
  `email_verified: Some(false)`. It is a mutable, tenant-administered
  attribute; never use it to find or link accounts.
- `name`: Graph `displayName`
- `additional`: includes `tid`, `oid` and `graph_id`

## AWS Cognito

### Setup

1. Go to [AWS Console](https://console.aws.amazon.com/cognito/)
2. Create a User Pool
3. Configure "App integration" > "App client"
4. Enable "Hosted UI"
5. Add callback URL: `http://localhost:3000/auth/cognito/callback`
6. Note your domain: `your-app.auth.{region}.amazoncognito.com`

### Configuration

```rust
use armature_auth::providers::{AwsCognitoConfig, AwsCognitoProvider};

let config = AwsCognitoConfig::new(
    "your-client-id".to_string(),
    "your-client-secret".to_string(),
    "http://localhost:3000/auth/cognito/callback".to_string(),
    "your-app.auth.us-east-1.amazoncognito.com".to_string(),
    "us-east-1".to_string(),
)
// Required for OIDC: enables id_token verification. Without it no id_token
// is exposed.
.with_user_pool_id("us-east-1_AbCdEf123".to_string());

// Optional: Custom scopes
let config = config.with_scopes(vec![
    "openid".to_string(),
    "email".to_string(),
    "profile".to_string(),
    "aws.cognito.signin.user.admin".to_string(),
]);

let provider = AwsCognitoProvider::new(config)?;
```

### User Info

Cognito returns:
- `sub`: Cognito user UUID
- `email`: User's email
- `email_verified`: Email verification status
- Custom attributes you've configured

## Okta

### Setup

1. Go to [Okta Admin Console](https://your-domain.okta.com/admin)
2. Navigate to "Applications" > "Create App Integration"
3. Select "OIDC - OpenID Connect"
4. Choose "Web Application"
5. Add redirect URI: `http://localhost:3000/auth/okta/callback`
6. Note your Okta domain: `dev-12345.okta.com`

### Configuration

```rust
use armature_auth::providers::{OktaConfig, OktaProvider};

let config = OktaConfig::new(
    "your-client-id".to_string(),
    "your-client-secret".to_string(),
    "http://localhost:3000/auth/okta/callback".to_string(),
    "dev-12345.okta.com".to_string(),
);

// Optional: Additional scopes
let config = config.with_scopes(vec![
    "openid".to_string(),
    "email".to_string(),
    "profile".to_string(),
    "groups".to_string(), // Include group membership
]);

let provider = OktaProvider::new(config)?;
```

### User Info

Okta returns:
- `sub`: Okta user ID
- `email`: User's email
- `name`: Full name
- `preferred_username`: Username
- `groups`: Group membership (if requested)

## Auth0

### Setup

1. Go to [Auth0 Dashboard](https://manage.auth0.com/)
2. Navigate to "Applications" > "Create Application"
3. Select "Regular Web Applications"
4. Add callback URL: `http://localhost:3000/auth/auth0/callback`
5. Note your domain: `your-tenant.us.auth0.com`

### Configuration

```rust
use armature_auth::providers::{Auth0Config, Auth0Provider};

let config = Auth0Config::new(
    "your-client-id".to_string(),
    "your-client-secret".to_string(),
    "http://localhost:3000/auth/auth0/callback".to_string(),
    "your-tenant.us.auth0.com".to_string(),
);

// Optional: API audience for access tokens
let config = config.with_audience("https://api.example.com".to_string());

// Optional: Custom scopes
let config = config.with_scopes(vec![
    "openid".to_string(),
    "email".to_string(),
    "profile".to_string(),
    "offline_access".to_string(), // For refresh tokens
]);

let provider = Auth0Provider::new(config)?;
```

### User Info

Auth0 returns:
- `sub`: Auth0 user ID (format: `provider|id`)
- `email`: User's email
- `name`: Full name
- `picture`: Profile picture
- `email_verified`: Email verification
- Custom user metadata

## OAuth2 Flow

The library generates and checks every piece of per-flow state itself.
`authorization_url()` returns the URL **and** a `PendingAuth` holding the
`state`, the PKCE verifier and (for OIDC) the `nonce`. Keep the `PendingAuth`
**server-side**, bound to the user's session, until the callback; never put it
in a client-readable cookie or form field (the verifier and nonce are secrets).
It is `Serialize`/`Deserialize` only so it can live in a session store, and it
expires after 10 minutes (`PENDING_AUTH_MAX_AGE`).

### Step 1: Generate Authorization URL

```rust
use armature_auth::{OAuth2Provider, PendingAuth};
use armature_session::SessionStore;

let (auth_url, pending) = provider.authorization_url()?;

// Store the pending flow in the user's server-side session
// (`session_id` comes from the session cookie).
sessions.set_value(&session_id, "oauth_pending", &pending).await?;

// Redirect user to auth_url
let response = HttpResponse::redirect(auth_url.as_str());
```

### Step 2: Handle Callback

```rust
// Extract code and state from query parameters
let code = request.query_param("code").ok_or_else(|| Error::BadRequest("missing code".into()))?;
let state = request.query_param("state").ok_or_else(|| Error::BadRequest("missing state".into()))?;

// Take the pending flow out of the session (single use)
let pending: PendingAuth = sessions
    .get_value(&session_id, "oauth_pending")
    .await?
    .ok_or_else(|| Error::BadRequest("no login in progress".into()))?;
sessions.remove_value(&session_id, "oauth_pending").await?;

// Compares `state` in constant time, checks the 10-minute expiry, sends the
// PKCE verifier and, for OIDC providers, verifies the id_token and nonce.
let token = provider.exchange_code(code.to_string(), state, pending).await?;

// Verified id_token claims (OIDC providers only)
if let Some(claims) = &token.id_token_claims {
    let subject = claims["sub"].as_str();
}
```

### Step 3: Get User Info

```rust
// Fetch user information. For OIDC providers the userinfo `sub` must match
// the verified id_token's, or this fails.
let user_info = provider.get_user_info(&token).await?;

// Rotate the session ID now that the user is logged in (session fixation)
let session = sessions.regenerate(&session_id).await?;

// Or issue your own JWT for a stateless API
let claims = Claims::new(serde_json::json!({ "roles": ["user"] }))
    .with_subject(user_info.sub.clone())
    .with_expiration(900);
let jwt = jwt_manager.sign(&claims)?;
```

### Step 4: Token Refresh

```rust
// When access token expires
if let Some(refresh_token) = token.refresh_token.clone() {
    let new_token = provider.refresh_token(refresh_token).await?;
    // Any id_token in a refresh response is not surfaced (it cannot be
    // bound to a nonce).
}
```

## Complete Example

```rust
use armature_framework::prelude::*;
use armature_auth::providers::{GoogleConfig, GoogleProvider};
use armature_auth::{OAuth2Provider, PendingAuth, UserContext};
use armature_session::{RedisSessionStore, SessionStore};

#[controller("/auth")]
struct AuthController {
    google_provider: Arc<GoogleProvider>,
    sessions: Arc<RedisSessionStore>,
}

impl AuthController {
    #[get("/google")]
    async fn google_login(&self, request: HttpRequest) -> Result<HttpResponse> {
        let session_id = session_id_from_cookie(&request)?; // your session cookie

        // Generate authorization URL (PKCE + state + nonce)
        let (auth_url, pending) = self.google_provider
            .authorization_url()
            .map_err(|e| Error::Internal(e.to_string()))?;

        // Keep the pending flow server-side, bound to this session
        self.sessions
            .set_value(&session_id, "oauth_pending", &pending)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;

        // Redirect to Google
        Ok(HttpResponse::redirect(auth_url.as_str()))
    }

    #[get("/google/callback")]
    async fn google_callback(&self, request: HttpRequest) -> Result<HttpResponse> {
        let session_id = session_id_from_cookie(&request)?;

        // Extract authorization code and state
        let code = request.query_param("code")
            .ok_or_else(|| Error::BadRequest("Missing code".into()))?;
        let state = request.query_param("state")
            .ok_or_else(|| Error::BadRequest("Missing state".into()))?;

        let pending: PendingAuth = self.sessions
            .get_value(&session_id, "oauth_pending")
            .await
            .map_err(|e| Error::Internal(e.to_string()))?
            .ok_or_else(|| Error::BadRequest("No login in progress".into()))?;
        self.sessions
            .remove_value(&session_id, "oauth_pending")
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;

        // Exchange code for token: state, PKCE and id_token are all verified here
        let token = self.google_provider
            .exchange_code(code.to_string(), state, pending)
            .await
            .map_err(|_| Error::Unauthorized("Login failed".into()))?;

        // Get user info
        let user_info = self.google_provider
            .get_user_info(&token)
            .await
            .map_err(|_| Error::Unauthorized("Login failed".into()))?;

        // Rotate the session ID on login, then store the user in it
        let session = self.sessions
            .regenerate(&session_id)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;

        let user = UserContext::new(user_info.sub)
            .with_email(user_info.email.unwrap_or_default());
        self.sessions
            .set_value(&session.id, "user", &user)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;

        // Set the new session cookie (see the Session Guide) and redirect.
        // Never return provider tokens to the browser.
        Ok(HttpResponse::redirect("/"))
    }
}
```

## Best Practices

### 1. State, PKCE and Nonce

You no longer generate or compare `state` yourself. `authorization_url()`
creates the `state`, a PKCE (S256) verifier and, for OIDC, a `nonce`, and
returns them in a `PendingAuth`; `exchange_code(code, returned_state, pending)`
compares the state in constant time, rejects flows older than 10 minutes,
sends the verifier, and verifies the `id_token` and its `nonce`.

Your responsibilities:

- Store `PendingAuth` server-side, bound to the user's session (not in a
  client-readable cookie), and remove it when the callback arrives so it is
  used once.
- Pass the `state` query parameter the provider returned, unchanged.
- Treat any `exchange_code` error as a failed login.

### 2. Secure Redirect URIs

- Use HTTPS in production
- Whitelist exact URIs (no wildcards)
- Use separate URIs per environment

```rust
let redirect_url = if cfg!(debug_assertions) {
    "http://localhost:3000/callback"
} else {
    "https://app.example.com/callback"
};
```

### 3. Token Storage

```rust
// ✓ Store tokens securely
let encrypted_token = encrypt(token.access_token)?;
db.save_user_token(user_id, encrypted_token)?;

// ✓ Use refresh tokens
if token_expired && refresh_token.is_some() {
    let new_token = provider.refresh_token(refresh_token).await?;
}

// ✗ Don't expose tokens to client
// ✗ Don't log tokens (OAuth2Token and PendingAuth already redact them from Debug)
```

### 4. Error Handling

```rust
match provider.exchange_code(code, state, pending).await {
    Ok(token) => { /* Success */ },
    Err(e) => {
        log::error!("OAuth2 error: {}", e);
        // Show user-friendly message
        return Err(Error::Unauthorized("Login failed".into()));
    }
}
```

### 5. Multi-Provider Support

```rust
enum Provider {
    Google(GoogleProvider),
    Microsoft(MicrosoftEntraProvider),
    Okta(OktaProvider),
}

impl Provider {
    fn as_dyn(&self) -> &dyn OAuth2Provider {
        match self {
            Provider::Google(p) => p,
            Provider::Microsoft(p) => p,
            Provider::Okta(p) => p,
        }
    }

    async fn authenticate(
        &self,
        code: String,
        state: &str,
        pending: PendingAuth,
    ) -> Result<OAuth2Token> {
        self.as_dyn().exchange_code(code, state, pending).await
    }
}
```

### 6. User Linking

```rust
async fn link_oauth_user(
    oauth_sub: String,
    provider: String,
    user_info: OAuth2UserInfo,
) -> Result<User> {
    // Check if OAuth account already linked
    if let Some(user) = db.find_by_oauth(provider, oauth_sub).await? {
        return Ok(user);
    }

    // Only link by e-mail when the provider says it is verified.
    // Microsoft Entra always reports `email_verified: Some(false)`: its
    // `mail` attribute is tenant-administered and mutable.
    if let (Some(email), Some(true)) = (user_info.email.clone(), user_info.email_verified) {
        if let Some(user) = db.find_by_email(&email).await? {
            // Prefer asking the user to log in to the existing account
            // and confirm the link before doing this.
            db.link_oauth(user.id, provider, oauth_sub).await?;
            return Ok(user);
        }
    }

    // Create new user
    let user = User::new(user_info);
    db.save_user(&user).await?;
    db.link_oauth(user.id, provider, oauth_sub).await?;

    Ok(user)
}
```

## Summary

The `armature-auth` OAuth2 provider system offers:

- ✅ **5 major providers** out of the box
- ✅ **Consistent API** across all providers
- ✅ **Type-safe** configuration
- ✅ **Async-first** design
- ✅ **PKCE, state and OIDC nonce** handled by the library
- ✅ **Extensible** for custom providers

For more information:
- [Auth Guide](auth-guide.md) - Complete authentication documentation
- [JWT Guide](jwt-guide.md) - JWT token management (coming soon)
- [Examples](../examples/) - Working code examples

## See Also

- [Google OAuth2 Documentation](https://developers.google.com/identity/protocols/oauth2)
- [Microsoft Identity Platform](https://docs.microsoft.com/en-us/azure/active-directory/develop/)
- [AWS Cognito Documentation](https://docs.aws.amazon.com/cognito/)
- [Okta Developer Documentation](https://developer.okta.com/)
- [Auth0 Documentation](https://auth0.com/docs)

