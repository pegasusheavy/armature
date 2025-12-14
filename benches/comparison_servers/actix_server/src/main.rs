//! Actix-web Benchmark Server
//! Port: 3001

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct JsonResponse {
    message: &'static str,
}

#[derive(Serialize)]
struct UserResponse {
    id: String,
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    #[serde(default)]
    email: Option<String>,
}

#[derive(Serialize)]
struct CreateUserResponse {
    id: u64,
    name: String,
    email: String,
    created: bool,
}

#[get("/")]
async fn plaintext() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/plain")
        .body("Hello, World!")
}

#[get("/json")]
async fn json() -> impl Responder {
    HttpResponse::Ok().json(JsonResponse {
        message: "Hello, World!",
    })
}

#[get("/users/{id}")]
async fn get_user(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(UserResponse {
        id: id.clone(),
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

#[post("/api/users")]
async fn create_user(body: web::Json<CreateUserRequest>) -> impl Responder {
    HttpResponse::Created().json(CreateUserResponse {
        id: 12345,
        name: body.name.clone(),
        email: body.email.clone().unwrap_or_else(|| "default@example.com".to_string()),
        created: true,
    })
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "framework": "actix-web"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Actix-web Benchmark Server on http://localhost:3001");

    HttpServer::new(|| {
        App::new()
            .service(plaintext)
            .service(json)
            .service(get_user)
            .service(create_user)
            .service(health)
    })
    .bind("127.0.0.1:3001")?
    .run()
    .await
}

