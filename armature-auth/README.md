# armature-auth

Authentication and authorization for the Armature framework.

## Features

- **Password Hashing** - bcrypt and Argon2 support
- **OAuth2/OIDC** - Google, GitHub, Microsoft, custom providers
- **JWT Integration** - Works with `armature-jwt`
- **Role-Based Access** - Guards and middleware for authorization
- **WebAuthn/FIDO2** - Passwordless authentication (optional)
- **SAML 2.0** - Enterprise SSO support (optional)

## Installation

```toml
[dependencies]
armature-auth = "0.1"
```

## Quick Start

```rust
use armature_auth::{AuthService, PasswordHasher};

// Hash a password
let hasher = PasswordHasher::argon2();
let hash = hasher.hash("my_password")?;

// Verify a password
assert!(hasher.verify("my_password", &hash)?);

// OAuth2 flow
let oauth = OAuth2Config::google()
    .client_id("your-client-id")
    .client_secret("your-secret")
    .redirect_uri("http://localhost:3000/callback");

let auth_url = oauth.authorization_url();
```

## Features Flags

- `oauth2` - OAuth2/OIDC support (default)
- `webauthn` - WebAuthn/FIDO2 passwordless auth
- `saml` - SAML 2.0 enterprise SSO

## License

MIT OR Apache-2.0

