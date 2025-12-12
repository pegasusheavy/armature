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

// Injectable service
#[injectable]
#[derive(Default)]
struct UserService {
    // In a real app, this would connect to a database
}

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
#[derive(Default)]
struct UserController {
    // In a full implementation, services would be injected here
}

impl UserController {
    #[get("")]
    async fn get_users() -> Result<Json<Vec<User>>, Error> {
        // Manually create service for now (in full impl, would be injected)
        let service = UserService::default();
        let users = service.get_all_users();
        Ok(Json(users))
    }

    #[get("/:id")]
    async fn get_user(req: HttpRequest) -> Result<Json<User>, Error> {
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

        Ok(Json(user))
    }

    #[post("")]
    async fn create_user(req: HttpRequest) -> Result<Json<User>, Error> {
        #[derive(Deserialize)]
        struct CreateUserDto {
            name: String,
            email: String,
        }

        let dto: CreateUserDto = req.json()?;
        let service = UserService::default();
        let user = service.create_user(dto.name, dto.email);

        Ok(Json(user))
    }
}

// Health check controller
#[controller("/health")]
#[derive(Default)]
struct HealthController;

impl HealthController {
    #[get("")]
    async fn health_check() -> Result<Json<serde_json::Value>, Error> {
        Ok(Json(serde_json::json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
        })))
    }
}

// Root module
#[module(
    providers: [UserService],
    controllers: [UserController, HealthController]
)]
#[derive(Default)]
struct AppModule;

#[tokio::main]
async fn main() {
    println!("🦾 Starting Armature HTTP Framework Example");
    println!("==========================================");

    // Create application - note: this is a simplified bootstrap
    // In a real implementation, the module system would fully integrate
    let app = create_simple_app();

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

    if let Err(e) = app.listen(3000).await {
        eprintln!("❌ Server error: {}", e);
    }
}

// Simplified application creation
// In a full implementation, this would use Application::create::<AppModule>()
fn create_simple_app() -> Application {
    use std::sync::Arc;

    let container = Container::new();
    let mut router = Router::new();

    // Manually register routes for demonstration
    // In a full implementation, the macros would generate registration code

    // Health check route
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/health".to_string(),
        handler: Arc::new(|_req| {
            Box::pin(async {
                let response = serde_json::json!({
                    "status": "healthy",
                    "version": env!("CARGO_PKG_VERSION"),
                });
                HttpResponse::ok().with_json(&response)
            })
        }),
    });

    // Get all users route
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/users".to_string(),
        handler: Arc::new(|_req| {
            Box::pin(async {
                let service = UserService::default();
                let users = service.get_all_users();
                HttpResponse::ok().with_json(&users)
            })
        }),
    });

    // Get user by ID route
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/users/:id".to_string(),
        handler: Arc::new(|req| {
            Box::pin(async move {
                let id_str = match req.param("id") {
                    Some(id) => id,
                    None => {
                        return HttpResponse::bad_request().with_json(&serde_json::json!({
                            "error": "Missing id parameter"
                        }));
                    }
                };

                let id: u32 = match id_str.parse() {
                    Ok(id) => id,
                    Err(_) => {
                        return HttpResponse::bad_request().with_json(&serde_json::json!({
                            "error": "Invalid id format"
                        }));
                    }
                };

                let service = UserService::default();
                match service.get_user_by_id(id) {
                    Some(user) => HttpResponse::ok().with_json(&user),
                    None => HttpResponse::not_found().with_json(&serde_json::json!({
                        "error": format!("User {} not found", id)
                    })),
                }
            })
        }),
    });

    // Create user route
    router.add_route(Route {
        method: HttpMethod::POST,
        path: "/users".to_string(),
        handler: Arc::new(|req| {
            Box::pin(async move {
                #[derive(Deserialize)]
                struct CreateUserDto {
                    name: String,
                    email: String,
                }

                let dto: CreateUserDto = match req.json() {
                    Ok(dto) => dto,
                    Err(e) => {
                        return HttpResponse::bad_request().with_json(&serde_json::json!({
                            "error": format!("Invalid request body: {}", e)
                        }));
                    }
                };

                let service = UserService::default();
                let CreateUserDto { name, email } = dto;
                let user = service.create_user(name, email);
                HttpResponse::created().with_json(&user)
            })
        }),
    });

    Application::new(container, router)
}
