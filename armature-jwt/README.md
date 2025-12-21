# armature-jwt

JWT authentication and authorization for the Armature framework.

## Features

- **Token Generation** - Create signed JWTs with custom claims
- **Token Verification** - Validate signatures and expiration
- **Multiple Algorithms** - HS256, HS384, HS512, RS256, RS384, RS512, ES256, ES384
- **Refresh Tokens** - Built-in token refresh flow
- **Custom Claims** - Extend with your own claim types

## Installation

```toml
[dependencies]
armature-jwt = "0.1"
```

## Quick Start

```rust
use armature_jwt::{JwtManager, JwtConfig, Claims};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create JWT manager
    let config = JwtConfig::new("your-secret-key")
        .expiration(Duration::from_secs(3600));
    let jwt = JwtManager::new(config);

    // Create a token
    let claims = Claims::new()
        .subject("user123")
        .claim("role", "admin");
    let token = jwt.sign(&claims)?;

    // Verify a token
    let verified = jwt.verify(&token)?;
    println!("User: {}", verified.sub.unwrap());

    Ok(())
}
```

## Token Refresh

```rust
// Generate token pair (access + refresh)
let (access, refresh) = jwt.generate_pair(&claims)?;

// Refresh the access token
let new_access = jwt.refresh(&refresh)?;
```

## License

MIT OR Apache-2.0

