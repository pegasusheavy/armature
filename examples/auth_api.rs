//! JWT Authentication API Example
//!
//! This example demonstrates how to build a secure API with Armature including:
//! - User registration and login
//! - JWT token generation and validation
//! - Protected routes with authentication guards
//! - Role-based access control
//!
//! Run with: `cargo run --example auth_api --features "auth jwt"`

use armature::prelude::*;
use armature_auth::prelude::*;
use armature_jwt::prelude::*;
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
    pub sub: String,        // user id
    pub email: String,
    pub role: String,
    pub exp: i64,           // expiration timestamp
    pub iat: i64,           // issued at
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
    token_expiry_secs: u64,
}

impl AuthenticationService {
    pub fn new(
        repository: UserRepository,
        jwt_manager: JwtManager,
        token_expiry_secs: u64,
    ) -> Self {
        Self {
            repository,
            jwt_manager,
            password_hasher: PasswordHasher::default(),
            token_expiry_secs,
        }
    }

    /// Register a new user.
    pub fn register(&self, req: RegisterRequest) -> Result<User, String> {
        // Validate email
        if !req.email.contains('@') {
            return Err("Invalid email format".to_string());
        }

        // Check if email exists
        if self.repository.email_exists(&req.email) {
            return Err("Email already registered".to_string());
        }

        // Validate password
        if req.password.len() < 8 {
            return Err("Password must be at least 8 characters".to_string());
        }

        // Hash password
        let password_hash = self
            .password_hasher
            .hash(&req.password)
            .map_err(|e| format!("Failed to hash password: {}", e))?;

        // Create user
        let user = self.repository.create(req.email, password_hash, UserRole::User);
        Ok(user)
    }

    /// Authenticate a user and generate tokens.
    pub fn login(&self, req: LoginRequest) -> Result<AuthResponse, String> {
        // Find user
        let user = self
            .repository
            .find_by_email(&req.email)
            .ok_or_else(|| "Invalid email or password".to_string())?;

        // Verify password
        let valid = self
            .password_hasher
            .verify(&req.password, &user.password_hash)
            .map_err(|e| format!("Password verification failed: {}", e))?;

        if !valid {
            return Err("Invalid email or password".to_string());
        }

        // Generate JWT
        let now = chrono::Utc::now().timestamp();
        let claims = UserClaims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: format!("{:?}", user.role).to_lowercase(),
            exp: now + self.token_expiry_secs as i64,
            iat: now,
        };

        let access_token = self
            .jwt_manager
            .sign(&claims)
            .map_err(|e| format!("Failed to generate token: {}", e))?;

        Ok(AuthResponse {
            access_token,
            token_type: "Bearer",
            expires_in: self.token_expiry_secs,
            user: user.to_profile(),
        })
    }

    /// Validate a JWT token and return the user.
    pub fn validate_token(&self, token: &str) -> Result<(User, UserClaims), String> {
        let claims: UserClaims = self
            .jwt_manager
            .verify(token)
            .map_err(|e| format!("Invalid token: {}", e))?;

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
// Controllers
// =============================================================================

/// Public authentication endpoints.
#[controller("/api/auth")]
struct AuthController;

#[controller_impl]
impl AuthController {
    /// POST /api/auth/register - Register a new user
    #[post("/register")]
    async fn register(
        &self,
        #[inject] auth: Arc<AuthenticationService>,
        req: HttpRequest,
    ) -> Result<HttpResponse, Error> {
        let body: RegisterRequest = req
            .json()
            .map_err(|e| Error::bad_request(format!("Invalid JSON: {}", e)))?;

        match auth.register(body) {
            Ok(user) => {
                let profile = user.to_profile();
                let mut response = HttpResponse::created();
                response = response.with_json(&profile)?;
                Ok(response)
            }
            Err(msg) => Err(Error::validation(msg)),
        }
    }

    /// POST /api/auth/login - Authenticate and get tokens
    #[post("/login")]
    async fn login(
        &self,
        #[inject] auth: Arc<AuthenticationService>,
        req: HttpRequest,
    ) -> Result<HttpResponse, Error> {
        let body: LoginRequest = req
            .json()
            .map_err(|e| Error::bad_request(format!("Invalid JSON: {}", e)))?;

        match auth.login(body) {
            Ok(response) => HttpResponse::json(&response),
            Err(msg) => Err(Error::unauthorized(msg)),
        }
    }
}

/// Protected user endpoints.
#[controller("/api/users")]
struct UserController;

#[controller_impl]
impl UserController {
    /// GET /api/users/me - Get current user profile
    #[get("/me")]
    async fn get_me(
        &self,
        #[inject] auth: Arc<AuthenticationService>,
        req: HttpRequest,
    ) -> Result<HttpResponse, Error> {
        // Extract token from Authorization header
        let token = Self::extract_token(&req)?;
        let (user, _claims) = auth
            .validate_token(token)
            .map_err(|e| Error::unauthorized(e))?;

        HttpResponse::json(&user.to_profile())
    }

    /// Helper to extract bearer token from Authorization header
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
struct AdminController;

#[controller_impl]
impl AdminController {
    /// GET /api/admin/stats - Get admin statistics (admin only)
    #[get("/stats")]
    async fn get_stats(
        &self,
        #[inject] auth: Arc<AuthenticationService>,
        req: HttpRequest,
    ) -> Result<HttpResponse, Error> {
        // Extract and validate token
        let token = UserController::extract_token(&req)?;
        let (user, _claims) = auth
            .validate_token(token)
            .map_err(|e| Error::unauthorized(e))?;

        // Check admin role
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

#[module]
struct AuthModule;

#[module_impl]
impl AuthModule {
    fn providers(&self) -> Vec<ProviderRegistration> {
        vec![]
    }

    fn controllers(&self) -> Vec<ControllerRegistration> {
        vec![]
    }

    fn imports(&self) -> Vec<Box<dyn Module>> {
        vec![]
    }

    fn exports(&self) -> Vec<std::any::TypeId> {
        vec![]
    }
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() {
    // Initialize logging
    let _guard = LogConfig::new()
        .level(LogLevel::Info)
        .format(LogFormat::Pretty)
        .init();

    info!("Starting Auth API example");

    // Configure JWT
    let jwt_config = JwtConfig::new("your-super-secret-key-change-in-production".to_string());
    let jwt_manager = JwtManager::new(jwt_config).expect("Failed to create JWT manager");

    // Create services
    let repository = UserRepository::new();

    // Create an admin user for testing
    {
        let hasher = PasswordHasher::default();
        let hash = hasher.hash("admin123").unwrap();
        repository.create("admin@example.com".to_string(), hash, UserRole::Admin);
    }

    let auth_service = AuthenticationService::new(repository, jwt_manager, 3600);

    // Create the DI container
    let container = Container::new();
    container.register(auth_service);

    // Build the application
    let app = Application::builder()
        .container(container)
        .build()
        .expect("Failed to build application");

    // Start the server
    info!("Server running at http://127.0.0.1:3000");
    info!("");
    info!("Test the API:");
    info!("");
    info!("  1. Register a new user:");
    info!(r#"     curl -X POST http://localhost:3000/api/auth/register \"#);
    info!(r#"       -H "Content-Type: application/json" \"#);
    info!(r#"       -d '{{"email":"user@example.com","password":"password123"}}'"#);
    info!("");
    info!("  2. Login to get a token:");
    info!(r#"     curl -X POST http://localhost:3000/api/auth/login \"#);
    info!(r#"       -H "Content-Type: application/json" \"#);
    info!(r#"       -d '{{"email":"user@example.com","password":"password123"}}'"#);
    info!("");
    info!("  3. Access protected endpoint:");
    info!(r#"     curl http://localhost:3000/api/users/me \"#);
    info!(r#"       -H "Authorization: Bearer YOUR_TOKEN""#);
    info!("");
    info!("  Admin credentials: admin@example.com / admin123");

    app.listen("127.0.0.1:3000").await.expect("Server failed");
}

