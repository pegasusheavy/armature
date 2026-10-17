//! JWT Authentication API Example
//!
//! This example demonstrates how to build a secure API with Armature including:
//! - User registration and login
//! - JWT token generation and validation
//! - Protected routes with authentication guards
//! - Role-based access control
//!
//! Run with: `cargo run --example auth_api --features "auth jwt"`

#![allow(dead_code)]

use armature::prelude::*;
use armature_auth::{PasswordHasher, PasswordVerifier};
use armature_jwt::{JwtConfig, JwtManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// =============================================================================
// Domain Models
// =============================================================================

/// A registered user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
}

/// User roles for RBAC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
}

/// Registration request.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

/// Login request.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Authentication response with tokens.
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
    pub user: UserProfile,
}

/// Public user profile (safe to expose).
#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: u64,
    pub email: String,
    pub role: UserRole,
}

/// JWT Claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

impl User {
    pub fn to_profile(&self) -> UserProfile {
        UserProfile {
            id: self.id,
            email: self.email.clone(),
            role: self.role,
        }
    }
}

// =============================================================================
// Repository
// =============================================================================

/// In-memory user repository.
#[derive(Clone, Default)]
pub struct UserRepository {
    users: Arc<RwLock<HashMap<u64, User>>>,
    email_index: Arc<RwLock<HashMap<String, u64>>>,
    next_id: Arc<RwLock<u64>>,
}

impl UserRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_by_email(&self, email: &str) -> Option<User> {
        let email_index = self.email_index.read().unwrap();
        let users = self.users.read().unwrap();
        email_index.get(email).and_then(|id| users.get(id).cloned())
    }

    pub fn find_by_id(&self, id: u64) -> Option<User> {
        self.users.read().unwrap().get(&id).cloned()
    }

    pub fn create(&self, email: String, password_hash: String, role: UserRole) -> User {
        let mut next_id = self.next_id.write().unwrap();
        let id = *next_id;
        *next_id += 1;

        let user = User {
            id,
            email: email.clone(),
            password_hash,
            role,
        };

        self.users.write().unwrap().insert(id, user.clone());
        self.email_index.write().unwrap().insert(email, id);
        user
    }

    pub fn email_exists(&self, email: &str) -> bool {
        self.email_index.read().unwrap().contains_key(email)
    }
}

// =============================================================================
// Authentication Service
// =============================================================================

/// Authentication service handling registration, login, and tokens.
#[derive(Clone)]
pub struct AuthenticationService {
    repository: UserRepository,
    jwt_manager: JwtManager,
    password_hasher: PasswordHasher,
    /// Hash verified when the email is unknown, so a login for a missing
    /// account costs the same as one with a wrong password (no user
    /// enumeration by timing).
    dummy_hash: String,
    token_expiry_secs: u64,
}

impl AuthenticationService {
    pub fn new(
        repository: UserRepository,
        jwt_manager: JwtManager,
        token_expiry_secs: u64,
    ) -> Self {
        let password_hasher = PasswordHasher::default();
        let dummy_hash = password_hasher
            .hash(&random_hex(16))
            .expect("hashing a random string cannot fail");
        Self {
            repository,
            jwt_manager,
            password_hasher,
            dummy_hash,
            token_expiry_secs,
        }
    }

    /// Register a new user.
    pub fn register(&self, req: RegisterRequest) -> std::result::Result<User, String> {
        if !req.email.contains('@') {
            return Err("Invalid email format".to_string());
        }

        if self.repository.email_exists(&req.email) {
            return Err("Email already registered".to_string());
        }

        if req.password.len() < 8 {
            return Err("Password must be at least 8 characters".to_string());
        }

        let password_hash = self
            .password_hasher
            .hash(&req.password)
            .map_err(|e| e.to_string())?;

        let user = self
            .repository
            .create(req.email, password_hash, UserRole::User);
        Ok(user)
    }

    /// Authenticate a user and generate tokens.
    pub fn login(&self, req: LoginRequest) -> std::result::Result<AuthResponse, String> {
        let Some(user) = self.repository.find_by_email(&req.email) else {
            let _ = self.password_hasher.verify(&req.password, &self.dummy_hash);
            return Err("Invalid credentials".to_string());
        };

        let valid = self
            .password_hasher
            .verify(&req.password, &user.password_hash)
            .map_err(|e| e.to_string())?;

        if !valid {
            return Err("Invalid credentials".to_string());
        }

        let now = chrono::Utc::now().timestamp();
        let claims = UserClaims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: format!("{:?}", user.role).to_lowercase(),
            exp: now + self.token_expiry_secs as i64,
            iat: now,
        };

        let access_token = self.jwt_manager.sign(&claims).map_err(|e| e.to_string())?;

        Ok(AuthResponse {
            access_token,
            token_type: "Bearer",
            expires_in: self.token_expiry_secs,
            user: user.to_profile(),
        })
    }

    /// Validate a JWT token and return the user.
    pub fn validate_token(&self, token: &str) -> std::result::Result<(User, UserClaims), String> {
        let claims: UserClaims = self.jwt_manager.verify(token).map_err(|e| e.to_string())?;

        let user_id: u64 = claims
            .sub
            .parse()
            .map_err(|_| "Invalid token subject".to_string())?;

        let user = self
            .repository
            .find_by_id(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        Ok((user, claims))
    }
}

// =============================================================================
// Global state for sharing auth service
// =============================================================================

static AUTH_SERVICE: std::sync::OnceLock<AuthenticationService> = std::sync::OnceLock::new();

fn get_auth_service() -> &'static AuthenticationService {
    AUTH_SERVICE
        .get()
        .expect("AuthenticationService not initialized")
}

// =============================================================================
// Controllers
// =============================================================================

/// Public authentication endpoints.
#[controller("/api/auth")]
#[derive(Default, Clone)]
struct AuthController;

#[routes]
impl AuthController {
    #[post("/register")]
    async fn register(req: HttpRequest) -> Result<HttpResponse, Error> {
        let body: RegisterRequest = req
            .json()
            .map_err(|e| Error::bad_request(format!("Invalid JSON: {}", e)))?;

        match get_auth_service().register(body) {
            Ok(user) => {
                let profile = user.to_profile();
                let mut response = HttpResponse::created();
                response = response.with_json(&profile)?;
                Ok(response)
            }
            Err(e) => Err(Error::validation(e)),
        }
    }

    #[post("/login")]
    async fn login(req: HttpRequest) -> Result<HttpResponse, Error> {
        let body: LoginRequest = req
            .json()
            .map_err(|e| Error::bad_request(format!("Invalid JSON: {}", e)))?;

        match get_auth_service().login(body) {
            Ok(response) => HttpResponse::json(&response),
            Err(e) => Err(Error::unauthorized(e)),
        }
    }
}

/// Protected user endpoints.
#[controller("/api/users")]
#[derive(Default, Clone)]
struct UserController;

#[routes]
impl UserController {
    #[get("/me")]
    async fn get_me(req: HttpRequest) -> Result<HttpResponse, Error> {
        let token = Self::extract_token(&req)?;
        let (user, _claims) = get_auth_service()
            .validate_token(token)
            .map_err(Error::unauthorized)?;

        HttpResponse::json(&user.to_profile())
    }

    fn extract_token(req: &HttpRequest) -> Result<&str, Error> {
        let header = req
            .headers
            .get("Authorization")
            .or_else(|| req.headers.get("authorization"))
            .ok_or_else(|| Error::unauthorized("Missing Authorization header"))?;

        header
            .strip_prefix("Bearer ")
            .ok_or_else(|| Error::unauthorized("Invalid Authorization header format"))
    }
}

/// Admin-only endpoints.
#[controller("/api/admin")]
#[derive(Default, Clone)]
struct AdminController;

#[routes]
impl AdminController {
    #[get("/stats")]
    async fn get_stats(req: HttpRequest) -> Result<HttpResponse, Error> {
        let token = UserController::extract_token(&req)?;
        let (user, _claims) = get_auth_service()
            .validate_token(token)
            .map_err(Error::unauthorized)?;

        if user.role != UserRole::Admin {
            return Err(Error::forbidden("Admin access required"));
        }

        HttpResponse::json(&serde_json::json!({
            "total_users": 42,
            "active_sessions": 7,
            "requests_today": 1234,
        }))
    }
}

// =============================================================================
// Module
// =============================================================================

#[module(
    controllers: [AuthController, UserController, AdminController]
)]
#[derive(Default)]
struct AppModule;

// =============================================================================
// Main
// =============================================================================

/// `n` random bytes, hex-encoded.
fn random_hex(n: usize) -> String {
    (0..n)
        .map(|_| format!("{:02x}", rand::random::<u8>()))
        .collect()
}

/// The value of `name`, or — for this demo only — `n` random bytes,
/// hex-encoded and printed so the operator can use them.
fn env_or_random(name: &str, n: usize) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        let value = random_hex(n);
        println!("{name} is not set; generated a random demo value: {value}");
        value
    })
}

#[tokio::main]
async fn main() {
    println!("Starting Auth API example");

    // Configure JWT. The signing secret comes from the environment; this demo
    // generates a random one when it is missing (tokens then stop verifying
    // when the process restarts). Never hardcode it.
    let jwt_secret = env_or_random("JWT_SECRET", 32);
    let jwt_config = match JwtConfig::new(jwt_secret) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("JWT_SECRET is unusable: {e}. Use at least 32 random bytes,");
            eprintln!("e.g. `export JWT_SECRET=$(openssl rand -base64 48)`.");
            std::process::exit(1);
        }
    };
    let jwt_manager = JwtManager::new(jwt_config).expect("Failed to create JWT manager");

    // Create repository and seed the admin user. Its password comes from
    // ADMIN_PASSWORD, or is generated at random and printed once.
    let admin_password = env_or_random("ADMIN_PASSWORD", 16);
    let repository = UserRepository::new();
    {
        let hasher = PasswordHasher::default();
        let hash = hasher
            .hash(&admin_password)
            .expect("Failed to hash admin password");
        repository.create("admin@example.com".to_string(), hash, UserRole::Admin);
    }

    // Initialize global auth service
    let auth_service = AuthenticationService::new(repository, jwt_manager, 3600);
    if AUTH_SERVICE.set(auth_service).is_err() {
        panic!("Failed to set auth service");
    }

    println!("Server running at http://127.0.0.1:3000");
    println!();
    println!("Test the API:");
    println!();
    println!("  1. Register a new user:");
    println!(r#"     curl -X POST http://localhost:3000/api/auth/register \"#);
    println!(r#"       -H "Content-Type: application/json" \"#);
    println!(r#"       -d '{{"email":"user@example.com","password":"<choose a password>"}}'"#);
    println!();
    println!("  2. Login to get a token:");
    println!(r#"     curl -X POST http://localhost:3000/api/auth/login \"#);
    println!(r#"       -H "Content-Type: application/json" \"#);
    println!(r#"       -d '{{"email":"user@example.com","password":"<choose a password>"}}'"#);
    println!();
    println!("  3. Access protected endpoint:");
    println!(r#"     curl http://localhost:3000/api/users/me \"#);
    println!(r#"       -H "Authorization: Bearer YOUR_TOKEN""#);
    println!();
    println!("  Admin account: admin@example.com (password from ADMIN_PASSWORD)");

    let app = Application::create::<AppModule>().await;
    app.listen(3000).await.unwrap();
}
