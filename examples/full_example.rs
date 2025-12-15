#![allow(
    dead_code,
    unused_imports,
    clippy::default_constructed_unit_structs,
    clippy::needless_borrow,
    clippy::unnecessary_lazy_evaluations
)]
// Example application demonstrating the Armature framework

use armature::prelude::*;
use serde::{Deserialize, Serialize};

// Domain model
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct CreateUserDto {
    name: String,
    email: String,
}

// Injectable service
#[injectable]
#[derive(Default, Clone)]
struct UserService;

impl UserService {
    fn get_all_users(&self) -> Vec<User> {
        vec![
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
        ]
    }

    fn get_user_by_id(&self, id: u32) -> Option<User> {
        self.get_all_users().into_iter().find(|u| u.id == id)
    }

    fn create_user(&self, name: String, email: String) -> User {
        User {
            id: 3, // In a real app, this would be auto-generated
            name,
            email,
        }
    }
}

// Controller with dependency injection
#[controller("/users")]
#[derive(Default, Clone)]
struct UserController;

#[routes]
impl UserController {
    #[get("")]
    async fn get_users() -> Result<HttpResponse, Error> {
        let service = UserService::default();
        let users = service.get_all_users();
        HttpResponse::json(&users)
    }

    #[get("/:id")]
    async fn get_user(req: HttpRequest) -> Result<HttpResponse, Error> {
        let id_str = req
            .param("id")
            .ok_or_else(|| Error::Validation("Missing id parameter".to_string()))?;
        let id: u32 = id_str
            .parse()
            .map_err(|_| Error::Validation("Invalid id format".to_string()))?;

        let service = UserService::default();
        let user = service
            .get_user_by_id(id)
            .ok_or_else(|| Error::RouteNotFound(format!("User {} not found", id)))?;

        HttpResponse::json(&user)
    }

    #[post("")]
    async fn create_user(req: HttpRequest) -> Result<HttpResponse, Error> {
        let dto: CreateUserDto = req.json()?;
        let service = UserService::default();
        let user = service.create_user(dto.name, dto.email);
        HttpResponse::json(&user)
    }
}

// Health check controller
#[controller("/health")]
#[derive(Default, Clone)]
struct HealthController;

#[routes]
impl HealthController {
    #[get("")]
    async fn health_check() -> Result<HttpResponse, Error> {
        HttpResponse::json(&serde_json::json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
        }))
    }
}

// Root module
#[module(
    providers: [UserService],
    controllers: [UserController, HealthController]
)]
#[derive(Default, Clone)]
struct AppModule;

#[tokio::main]
async fn main() {
    println!("🦾 Starting Armature HTTP Framework Example");
    println!("==========================================");

    println!("\n📚 Available routes:");
    println!("  GET    /health           - Health check");
    println!("  GET    /users            - Get all users");
    println!("  GET    /users/:id        - Get user by ID");
    println!("  POST   /users            - Create new user");
    println!("\n💡 Try:");
    println!("  curl http://localhost:3000/health");
    println!("  curl http://localhost:3000/users");
    println!("  curl http://localhost:3000/users/1");
    println!(
        "  curl -X POST http://localhost:3000/users -H 'Content-Type: application/json' -d '{{\"name\":\"Charlie\",\"email\":\"charlie@example.com\"}}'"
    );
    println!();

    let app = Application::create::<AppModule>().await;

    if let Err(e) = app.listen(3000).await {
        eprintln!("❌ Server error: {}", e);
    }
}
