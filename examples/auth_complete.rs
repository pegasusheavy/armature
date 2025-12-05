// Complete authentication example with guards and password hashing

use armature_auth::{
    AuthService, AuthUser, PasswordHasher, PasswordVerifier, RoleGuard, UserContext,
};
use armature_jwt::{Claims, JwtConfig, JwtManager};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// User model
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    email: String,
    password_hash: String,
    roles: Vec<String>,
    active: bool,
}

impl AuthUser for User {
    fn user_id(&self) -> String {
        self.id.clone()
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    fn has_permission(&self, _permission: &str) -> bool {
        // Implement based on your permission system
        true
    }
}

fn main() {
    println!("🔐 Armature Complete Authentication Example");
    println!("==========================================\n");

    // 1. Setup Authentication Service
    println!("1. Setting up authentication service...");
    let jwt_config = JwtConfig::new("your-secret-key-change-in-production".to_string())
        .with_expiration(Duration::from_secs(3600))
        .with_issuer("armature-app".to_string());

    let jwt_manager = JwtManager::new(jwt_config).expect("Failed to create JWT manager");
    let auth_service = AuthService::with_jwt(jwt_manager);
    println!("   ✓ Auth service created\n");

    // 2. Create a User with Hashed Password
    println!("2. Creating user with hashed password...");
    let password = "SuperSecret123!";
    let password_hash = auth_service
        .hash_password(password)
        .expect("Failed to hash password");

    let user = User {
        id: "user123".to_string(),
        email: "admin@example.com".to_string(),
        password_hash: password_hash.clone(),
        roles: vec!["admin".to_string(), "user".to_string()],
        active: true,
    };

    println!("   User ID: {}", user.id);
    println!("   Email: {}", user.email);
    println!("   Roles: {:?}", user.roles);
    println!(
        "   Password Hash (first 50 chars): {}...",
        &password_hash[..50.min(password_hash.len())]
    );
    println!();

    // 3. Verify Password
    println!("3. Verifying password...");
    let valid = auth_service
        .verify_password(password, &user.password_hash)
        .expect("Failed to verify");
    println!(
        "   ✓ Password verification: {}",
        if valid { "SUCCESS" } else { "FAILED" }
    );

    let invalid = auth_service
        .verify_password("wrong-password", &user.password_hash)
        .expect("Failed to verify");
    println!(
        "   ✗ Wrong password verification: {}",
        if invalid {
            "SUCCESS"
        } else {
            "FAILED (expected)"
        }
    );
    println!();

    // 4. Validate User
    println!("4. Validating user...");
    match auth_service.validate(&user) {
        Ok(_) => println!("   ✓ User validation passed"),
        Err(e) => println!("   ✗ User validation failed: {}", e),
    }
    println!();

    // 5. Generate JWT Token
    println!("5. Generating JWT token for user...");
    if let Some(jwt_manager) = auth_service.jwt_manager() {
        let user_claims = UserContext::new(user.id.clone())
            .with_email(user.email.clone())
            .with_roles(user.roles.clone());

        let claims = Claims::new(user_claims)
            .with_subject(user.email.clone())
            .with_expiration(3600);

        let token = jwt_manager.sign(&claims).expect("Failed to sign token");
        println!(
            "   Token (first 50 chars): {}...",
            &token[..50.min(token.len())]
        );

        // Verify the token
        let verified: Claims<UserContext> = jwt_manager.verify(&token).expect("Failed to verify");
        println!("   ✓ Token verified - User ID: {}", verified.custom.user_id);
        println!();

        // 6. Role-Based Access Control
        println!("6. Testing role-based access control...");
        let admin_guard = RoleGuard::any(vec!["admin".to_string()]);
        let guest_guard = RoleGuard::any(vec!["guest".to_string()]);
        let multi_role_guard = RoleGuard::all(vec!["admin".to_string(), "user".to_string()]);

        println!(
            "   Admin role check: {}",
            admin_guard.check_roles(&verified.custom)
        );
        println!(
            "   Guest role check: {}",
            guest_guard.check_roles(&verified.custom)
        );
        println!(
            "   Admin AND User check: {}",
            multi_role_guard.check_roles(&verified.custom)
        );
        println!();
    }

    // 7. Different Password Algorithms
    println!("7. Testing different password hashing algorithms...");

    println!("   Bcrypt:");
    let bcrypt_hasher = PasswordHasher::new(armature_auth::password::HashAlgorithm::Bcrypt);
    let bcrypt_hash = bcrypt_hasher.hash("test123").expect("Failed to hash");
    println!(
        "     Hash: {}...",
        &bcrypt_hash[..30.min(bcrypt_hash.len())]
    );
    println!(
        "     Verified: {}",
        bcrypt_hasher.verify("test123", &bcrypt_hash).unwrap()
    );

    println!("   Argon2:");
    let argon2_hasher = PasswordHasher::new(armature_auth::password::HashAlgorithm::Argon2);
    let argon2_hash = argon2_hasher.hash("test123").expect("Failed to hash");
    println!(
        "     Hash: {}...",
        &argon2_hash[..30.min(argon2_hash.len())]
    );
    println!(
        "     Verified: {}",
        argon2_hasher.verify("test123", &argon2_hash).unwrap()
    );
    println!();

    // 8. Summary
    println!("✅ Authentication System Summary:");
    println!("   ✓ Password hashing (Bcrypt & Argon2)");
    println!("   ✓ Password verification");
    println!("   ✓ JWT token generation");
    println!("   ✓ Token verification");
    println!("   ✓ User validation");
    println!("   ✓ Role-based access control");
    println!("   ✓ Guard system");
    println!();
    println!("Features implemented:");
    println!("   ✓ Multiple hash algorithms (Bcrypt, Argon2)");
    println!("   ✓ JWT integration");
    println!("   ✓ Role guards (ANY/ALL)");
    println!("   ✓ Permission guards");
    println!("   ✓ User context");
    println!("   ✓ Authentication strategies");
    println!();
    println!("For HTTP integration examples, see:");
    println!("   - docs/AUTH_GUIDE.md");
    println!("   - examples/jwt_auth.rs");
}
