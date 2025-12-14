#![allow(dead_code)]
// Simple JWT example demonstrating token operations

use armature_jwt::{Claims, JwtConfig, JwtManager, StandardClaims};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Custom user claims
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct UserClaims {
    user_id: String,
    email: String,
    role: String,
}

fn main() {
    println!("🔐 Armature JWT Example");
    println!("========================\n");

    // 1. Create JWT configuration
    println!("1. Creating JWT configuration...");
    let config = JwtConfig::new("your-secret-key-change-in-production".to_string())
        .with_expiration(Duration::from_secs(3600)) // 1 hour
        .with_refresh_expiration(Duration::from_secs(604800)) // 7 days
        .with_issuer("armature-app".to_string())
        .with_audience(vec!["web-app".to_string()]);

    println!("   Algorithm: {:?}", config.algorithm);
    println!("   Expires in: {} seconds", config.expires_in.as_secs());
    println!();

    // 2. Create JWT manager
    println!("2. Creating JWT manager...");
    let jwt_manager = JwtManager::new(config).expect("Failed to create JWT manager");
    println!("   ✓ JWT manager created\n");

    // 3. Create custom claims
    println!("3. Creating custom claims...");
    let user_claims = UserClaims {
        user_id: "user123".to_string(),
        email: "user@example.com".to_string(),
        role: "admin".to_string(),
    };

    let claims = Claims::new(user_claims.clone())
        .with_subject("user@example.com".to_string())
        .with_expiration(3600);

    println!("   User ID: {}", user_claims.user_id);
    println!("   Email: {}", user_claims.email);
    println!("   Role: {}", user_claims.role);
    println!();

    // 4. Sign the token
    println!("4. Signing JWT token...");
    let token = jwt_manager.sign(&claims).expect("Failed to sign token");
    println!(
        "   Token (first 50 chars): {}...",
        &token[..50.min(token.len())]
    );
    println!("   Token length: {} bytes\n", token.len());

    // 5. Verify the token
    println!("5. Verifying token...");
    let verified_claims: Claims<UserClaims> =
        jwt_manager.verify(&token).expect("Failed to verify token");

    println!("   ✓ Token verified successfully");
    println!("   Decoded user_id: {}", verified_claims.custom.user_id);
    println!("   Decoded email: {}", verified_claims.custom.email);
    println!("   Decoded role: {}", verified_claims.custom.role);
    println!();

    // 6. Generate token pair
    println!("6. Generating access + refresh token pair...");
    let token_pair = jwt_manager
        .generate_token_pair(&claims)
        .expect("Failed to generate token pair");

    println!(
        "   Access Token (first 50 chars): {}...",
        &token_pair.access_token[..50.min(token_pair.access_token.len())]
    );
    println!(
        "   Refresh Token (first 50 chars): {}...",
        &token_pair.refresh_token[..50.min(token_pair.refresh_token.len())]
    );
    println!("   Token Type: {}", token_pair.token_type);
    println!("   Access Expires In: {} seconds", token_pair.expires_in);
    println!(
        "   Refresh Expires In: {} seconds",
        token_pair.refresh_expires_in
    );
    println!();

    // 7. Refresh the token
    println!("7. Refreshing access token...");
    let refreshed_pair = jwt_manager
        .refresh_token::<Claims<UserClaims>>(&token_pair.refresh_token)
        .expect("Failed to refresh token");

    println!("   ✓ Token refreshed successfully");
    println!(
        "   New Access Token (first 50 chars): {}...",
        &refreshed_pair.access_token[..50.min(refreshed_pair.access_token.len())]
    );
    println!();

    // 8. Decode without verification (inspect expired tokens)
    println!("8. Decoding token without verification...");
    let decoded: Claims<UserClaims> = jwt_manager
        .decode_unverified(&token)
        .expect("Failed to decode token");

    println!("   ✓ Token decoded (unverified)");
    println!("   User ID: {}", decoded.custom.user_id);
    println!();

    // 9. Demonstrate standard claims
    println!("9. Working with standard claims...");
    let standard = StandardClaims::new()
        .with_subject("user456".to_string())
        .with_issuer("my-app".to_string())
        .with_expiration(7200);

    let _standard_token = jwt_manager.sign(&standard).expect("Failed to sign");
    println!("   ✓ Standard claims token created");
    println!("   Subject: {}", standard.sub.unwrap());
    println!();

    // 10. Summary
    println!("✅ Summary:");
    println!("   - JWT tokens generated and verified successfully");
    println!("   - Token pairs (access + refresh) working");
    println!("   - Custom claims supported");
    println!("   - Standard RFC 7519 claims supported");
    println!();
    println!("Features demonstrated:");
    println!("   ✓ Token generation");
    println!("   ✓ Token verification");
    println!("   ✓ Token refresh");
    println!("   ✓ Custom claims");
    println!("   ✓ Standard claims");
    println!("   ✓ Multiple algorithms (HS256, RS256, etc.)");
    println!();
    println!("For a full authentication example, see:");
    println!("   - docs/jwt-guide.md");
    println!("   - examples/jwt_auth.rs");
}
