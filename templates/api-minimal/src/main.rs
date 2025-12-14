//! Armature API - Minimal Template
//!
//! A simple REST API demonstrating basic Armature features.
//!
//! Run with: cargo run
//! Test with: curl http://localhost:3000/health

use armature::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// =============================================================================
// Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

// =============================================================================
// Services
// =============================================================================

pub struct UserService {
    users: std::sync::RwLock<Vec<User>>,
    next_id: std::sync::atomic::AtomicU64,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            users: std::sync::RwLock::new(vec![
                User {
                    id: 1,
                    name: "Alice".to_string(),
                    email: "alice@example.com".to_string(),
                },
                User {
                    id: 2,
                    name: "Bob".to_string(),
                    email: "bob@example.com".to_string(),
                },
            ]),
            next_id: std::sync::atomic::AtomicU64::new(3),
        }
    }

    pub fn get_all(&self) -> Vec<User> {
        self.users.read().unwrap().clone()
    }

    pub fn get_by_id(&self, id: u64) -> Option<User> {
        self.users.read().unwrap().iter().find(|u| u.id == id).cloned()
    }

    pub fn create(&self, create: CreateUser) -> User {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let user = User {
            id,
            name: create.name,
            email: create.email,
        };
        self.users.write().unwrap().push(user.clone());
        user
    }

    pub fn delete(&self, id: u64) -> bool {
        let mut users = self.users.write().unwrap();
        let initial_len = users.len();
        users.retain(|u| u.id != id);
        users.len() != initial_len
    }
}

// Provider is automatically implemented via blanket impl

// =============================================================================
// Controllers
// =============================================================================

pub struct HealthController;

impl Controller for HealthController {
    fn routes(&self) -> Vec<Route> {
        vec![Route::new(HttpMethod::GET, "/health", "health")]
    }

    fn handle(&self, _route_name: &str, _request: &HttpRequest) -> HttpResponse {
        HttpResponse::json(serde_json::json!({
            "status": "healthy",
            "timestamp": chrono_lite_now(),
        }))
    }
}

fn chrono_lite_now() -> String {
    // Simple timestamp without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}", duration.as_secs())
}

pub struct UserController {
    service: Arc<UserService>,
}

impl UserController {
    pub fn new(service: Arc<UserService>) -> Self {
        Self { service }
    }
}

impl Controller for UserController {
    fn routes(&self) -> Vec<Route> {
        vec![
            Route::new(HttpMethod::GET, "/api/users", "list"),
            Route::new(HttpMethod::GET, "/api/users/:id", "get"),
            Route::new(HttpMethod::POST, "/api/users", "create"),
            Route::new(HttpMethod::DELETE, "/api/users/:id", "delete"),
        ]
    }

    fn handle(&self, route_name: &str, request: &HttpRequest) -> HttpResponse {
        match route_name {
            "list" => {
                let users = self.service.get_all();
                HttpResponse::json(ApiResponse::success(users))
            }
            "get" => {
                let id = request
                    .path_params
                    .get("id")
                    .and_then(|s| s.parse::<u64>().ok());

                match id {
                    Some(id) => match self.service.get_by_id(id) {
                        Some(user) => HttpResponse::json(ApiResponse::success(user)),
                        None => HttpResponse::not_found()
                            .json(ApiResponse::<()>::error("User not found")),
                    },
                    None => {
                        HttpResponse::bad_request().json(ApiResponse::<()>::error("Invalid user ID"))
                    }
                }
            }
            "create" => {
                let body = match request.json::<CreateUser>() {
                    Ok(b) => b,
                    Err(_) => {
                        return HttpResponse::bad_request()
                            .json(ApiResponse::<()>::error("Invalid request body"));
                    }
                };

                let user = self.service.create(body);
                HttpResponse::created().json(ApiResponse::success(user))
            }
            "delete" => {
                let id = request
                    .path_params
                    .get("id")
                    .and_then(|s| s.parse::<u64>().ok());

                match id {
                    Some(id) => {
                        if self.service.delete(id) {
                            HttpResponse::no_content()
                        } else {
                            HttpResponse::not_found()
                                .json(ApiResponse::<()>::error("User not found"))
                        }
                    }
                    None => {
                        HttpResponse::bad_request().json(ApiResponse::<()>::error("Invalid user ID"))
                    }
                }
            }
            _ => HttpResponse::not_found(),
        }
    }
}

// =============================================================================
// Module
// =============================================================================

pub struct AppModule {
    user_service: Arc<UserService>,
}

impl AppModule {
    pub fn new() -> Self {
        Self {
            user_service: Arc::new(UserService::new()),
        }
    }
}

impl Module for AppModule {
    fn name(&self) -> &'static str {
        "AppModule"
    }

    fn providers(&self) -> Vec<Arc<dyn Provider>> {
        vec![self.user_service.clone()]
    }

    fn controllers(&self) -> Vec<Box<dyn Controller>> {
        vec![
            Box::new(HealthController),
            Box::new(UserController::new(self.user_service.clone())),
        ]
    }
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Armature API (Minimal Template)");

    let module = AppModule::new();
    let app = Application::create(Box::new(module));

    println!("📡 Server running at http://localhost:3000");
    println!("");
    println!("Available endpoints:");
    println!("  GET  /health         - Health check");
    println!("  GET  /api/users      - List all users");
    println!("  GET  /api/users/:id  - Get user by ID");
    println!("  POST /api/users      - Create user");
    println!("  DELETE /api/users/:id - Delete user");
    println!("");
    println!("Try: curl http://localhost:3000/api/users");

    app.listen("0.0.0.0:3000").await?;

    Ok(())
}

