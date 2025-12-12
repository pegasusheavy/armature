//! Authentication controller

use crate::models::{ApiResponse, AuthResponse, LoginRequest, RegisterRequest, UserResponse, ValidationError};
use crate::services::{AuthService, UserService};
use armature::prelude::*;
use std::sync::Arc;

pub struct AuthController {
    auth_service: Arc<AuthService>,
    user_service: Arc<UserService>,
}

impl AuthController {
    pub fn new(auth_service: Arc<AuthService>, user_service: Arc<UserService>) -> Self {
        Self {
            auth_service,
            user_service,
        }
    }

    fn handle_login(&self, request: &HttpRequest) -> HttpResponse {
        let body: LoginRequest = match request.json() {
            Ok(b) => b,
            Err(_) => {
                return HttpResponse::bad_request()
                    .json(ApiResponse::<()>::error("BAD_REQUEST", "Invalid request body"));
            }
        };

        // Validate input
        let mut errors = Vec::new();
        if body.email.is_empty() {
            errors.push(ValidationError {
                field: "email".to_string(),
                message: "Email is required".to_string(),
            });
        }
        if body.password.is_empty() {
            errors.push(ValidationError {
                field: "password".to_string(),
                message: "Password is required".to_string(),
            });
        }
        if !errors.is_empty() {
            return HttpResponse::bad_request().json(ApiResponse::<()>::validation_error(errors));
        }

        // Find user
        let user = match self.user_service.find_by_email(&body.email) {
            Some(u) => u,
            None => {
                return HttpResponse::unauthorized()
                    .json(ApiResponse::<()>::error("INVALID_CREDENTIALS", "Invalid email or password"));
            }
        };

        // Verify password
        if !self.auth_service.verify_password(&body.password, &user.password_hash) {
            return HttpResponse::unauthorized()
                .json(ApiResponse::<()>::error("INVALID_CREDENTIALS", "Invalid email or password"));
        }

        // Generate token
        let token = match self.auth_service.generate_token(&user) {
            Ok(t) => t,
            Err(_) => {
                return HttpResponse::internal_server_error()
                    .json(ApiResponse::<()>::error("TOKEN_ERROR", "Failed to generate token"));
            }
        };

        HttpResponse::json(ApiResponse::success(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: self.auth_service.token_expiry_seconds(),
            user: UserResponse::from(&user),
        }))
    }

    fn handle_register(&self, request: &HttpRequest) -> HttpResponse {
        let body: RegisterRequest = match request.json() {
            Ok(b) => b,
            Err(_) => {
                return HttpResponse::bad_request()
                    .json(ApiResponse::<()>::error("BAD_REQUEST", "Invalid request body"));
            }
        };

        // Validate input
        let mut errors = Vec::new();
        if body.email.is_empty() {
            errors.push(ValidationError {
                field: "email".to_string(),
                message: "Email is required".to_string(),
            });
        } else if !body.email.contains('@') {
            errors.push(ValidationError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
            });
        }
        if body.password.is_empty() {
            errors.push(ValidationError {
                field: "password".to_string(),
                message: "Password is required".to_string(),
            });
        } else if body.password.len() < 8 {
            errors.push(ValidationError {
                field: "password".to_string(),
                message: "Password must be at least 8 characters".to_string(),
            });
        }
        if body.name.is_empty() {
            errors.push(ValidationError {
                field: "name".to_string(),
                message: "Name is required".to_string(),
            });
        }
        if !errors.is_empty() {
            return HttpResponse::bad_request().json(ApiResponse::<()>::validation_error(errors));
        }

        // Check if email exists
        if self.user_service.email_exists(&body.email) {
            return HttpResponse::conflict()
                .json(ApiResponse::<()>::error("EMAIL_EXISTS", "Email already registered"));
        }

        // Create user
        let password_hash = self.auth_service.hash_password(&body.password);
        let user = self.user_service.create(body.email, password_hash, body.name);

        // Generate token
        let token = match self.auth_service.generate_token(&user) {
            Ok(t) => t,
            Err(_) => {
                return HttpResponse::internal_server_error()
                    .json(ApiResponse::<()>::error("TOKEN_ERROR", "Failed to generate token"));
            }
        };

        HttpResponse::created().json(ApiResponse::success(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: self.auth_service.token_expiry_seconds(),
            user: UserResponse::from(&user),
        }))
    }
}

impl Controller for AuthController {
    fn routes(&self) -> Vec<Route> {
        vec![
            Route::new(HttpMethod::POST, "/api/auth/login", "login"),
            Route::new(HttpMethod::POST, "/api/auth/register", "register"),
        ]
    }

    fn handle(&self, route_name: &str, request: &HttpRequest) -> HttpResponse {
        match route_name {
            "login" => self.handle_login(request),
            "register" => self.handle_register(request),
            _ => HttpResponse::not_found(),
        }
    }
}

