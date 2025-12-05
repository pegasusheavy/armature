// JWT Authentication example

use armature::armature_jwt::{Claims, JwtConfig, JwtManager, StandardClaims};
use armature::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

// ========== Custom Claims ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserClaims {
    user_id: String,
    email: String,
    role: String,
}

// ========== DTOs ==========

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: i64,
}

// ========== Services ==========

#[injectable]
#[derive(Clone)]
struct AuthService {
    jwt_manager: JwtManager,
}

impl Default for AuthService {
    fn default() -> Self {
        // In production, load secret from environment
        let config = JwtConfig::new("your-secret-key-change-in-production".to_string())
            .with_expiration(Duration::from_secs(3600)) // 1 hour
            .with_refresh_expiration(Duration::from_secs(604800)) // 7 days
            .with_issuer("armature-app".to_string());

        Self {
            jwt_manager: JwtManager::new(config).unwrap(),
        }
    }
}

impl AuthService {
    fn login(&self, email: String, password: String) -> Result<LoginResponse, Error> {
        // In production, verify password against database
        if password != "password123" {
            return Err(Error::Unauthorized("Invalid credentials".to_string()));
        }

        // Create custom claims
        let user_claims = UserClaims {
            user_id: "user123".to_string(),
            email: email.clone(),
            role: "user".to_string(),
        };

        // Generate standard claims with custom data
        let claims = Claims::new(user_claims)
            .with_subject(email)
            .with_expiration(3600); // 1 hour

        // Generate token pair
        let token_pair = self
            .jwt_manager
            .generate_token_pair(&claims)
            .map_err(|e| Error::Internal(e.to_string()))?;

        Ok(LoginResponse {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            token_type: token_pair.token_type,
            expires_in: token_pair.expires_in,
        })
    }

    fn verify_token(&self, token: &str) -> Result<UserClaims, Error> {
        let claims: Claims<UserClaims> = self
            .jwt_manager
            .verify(token)
            .map_err(|e| Error::Unauthorized(e.to_string()))?;

        Ok(claims.custom)
    }

    fn refresh(&self, refresh_token: &str) -> Result<LoginResponse, Error> {
        let token_pair = self
            .jwt_manager
            .refresh_token::<Claims<UserClaims>>(refresh_token)
            .map_err(|e| Error::Unauthorized(e.to_string()))?;

        Ok(LoginResponse {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            token_type: token_pair.token_type,
            expires_in: token_pair.expires_in,
        })
    }
}

// ========== Controllers ==========

#[controller("/auth")]
#[derive(Default, Clone)]
struct AuthController {
    auth_service: AuthService,
}

impl AuthController {
    fn login(&self, req: Json<LoginRequest>) -> Result<Json<LoginResponse>, Error> {
        let response = self
            .auth_service
            .login(req.email.clone(), req.password.clone())?;
        Ok(Json(response))
    }

    fn refresh(&self, token: String) -> Result<Json<LoginResponse>, Error> {
        let response = self.auth_service.refresh(&token)?;
        Ok(Json(response))
    }

    fn verify(&self, token: String) -> Result<Json<UserClaims>, Error> {
        let claims = self.auth_service.verify_token(&token)?;
        Ok(Json(claims))
    }

    fn profile(&self, token: String) -> Result<Json<serde_json::Value>, Error> {
        let claims = self.auth_service.verify_token(&token)?;
        Ok(Json(serde_json::json!({
            "user_id": claims.user_id,
            "email": claims.email,
            "role": claims.role
        })))
    }
}

// ========== Module ==========

#[module(
    providers: [AuthService],
    controllers: [AuthController]
)]
#[derive(Default)]
struct AuthModule;

// ========== Main ==========

#[tokio::main]
async fn main() {
    println!("🔐 Armature JWT Authentication Example");
    println!("======================================\n");

    let app = create_auth_app();

    println!("Server running on http://localhost:3011");
    println!();
    println!("API Endpoints:");
    println!("  POST /auth/login       - Login and get tokens");
    println!("  POST /auth/refresh     - Refresh access token");
    println!("  GET  /auth/profile     - Get user profile (requires token)");
    println!();
    println!("Example usage:");
    println!();
    println!("1. Login:");
    println!("   curl -X POST http://localhost:3011/auth/login \\");
    println!("     -H \"Content-Type: application/json\" \\");
    println!("     -d '{{\"email\":\"user@example.com\",\"password\":\"password123\"}}'");
    println!();
    println!("2. Access protected resource:");
    println!("   curl http://localhost:3011/auth/profile \\");
    println!("     -H \"Authorization: Bearer YOUR_ACCESS_TOKEN\"");
    println!();
    println!("3. Refresh token:");
    println!("   curl -X POST http://localhost:3011/auth/refresh \\");
    println!("     -H \"Authorization: Bearer YOUR_REFRESH_TOKEN\"");
    println!();

    if let Err(e) = app.listen(3011).await {
        eprintln!("Server error: {}", e);
    }
}

fn create_auth_app() -> Application {
    let container = Container::new();
    let mut router = Router::new();

    // Register auth service
    let auth_service = AuthService::default();
    container.register(auth_service.clone());

    let controller = AuthController { auth_service };

    // Login endpoint
    let login_ctrl = controller.clone();
    router.add_route(Route {
        method: HttpMethod::POST,
        path: "/auth/login".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = login_ctrl.clone();
            Box::pin(async move {
                let body: LoginRequest = req.json()?;
                ctrl.login(Json(body))?.into_response()
            })
        }),
    });

    // Refresh endpoint
    let refresh_ctrl = controller.clone();
    router.add_route(Route {
        method: HttpMethod::POST,
        path: "/auth/refresh".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = refresh_ctrl.clone();
            Box::pin(async move {
                // Extract token from Authorization header
                let auth_header = req.headers.get("authorization").ok_or_else(|| {
                    Error::Unauthorized("Missing authorization header".to_string())
                })?;

                let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
                    Error::Unauthorized("Invalid authorization header".to_string())
                })?;

                ctrl.refresh(token.to_string())?.into_response()
            })
        }),
    });

    // Profile endpoint (protected)
    let profile_ctrl = controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/auth/profile".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = profile_ctrl.clone();
            Box::pin(async move {
                // Extract and verify token
                let auth_header = req.headers.get("authorization").ok_or_else(|| {
                    Error::Unauthorized("Missing authorization header".to_string())
                })?;

                let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
                    Error::Unauthorized("Invalid authorization header".to_string())
                })?;

                ctrl.profile(token.to_string())?.into_response()
            })
        }),
    });

    Application {
        container,
        router: Arc::new(router),
    }
}
