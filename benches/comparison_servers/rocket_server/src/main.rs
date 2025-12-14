//! Rocket Benchmark Server
//! Port: 3004

#[macro_use]
extern crate rocket;

use rocket::serde::json::Json;
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

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    framework: &'static str,
}

#[get("/")]
fn plaintext() -> &'static str {
    "Hello, World!"
}

#[get("/json")]
fn json() -> Json<JsonResponse> {
    Json(JsonResponse {
        message: "Hello, World!",
    })
}

#[get("/users/<id>")]
fn get_user(id: String) -> Json<UserResponse> {
    Json(UserResponse {
        id: id.clone(),
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

#[post("/api/users", data = "<body>")]
fn create_user(body: Json<CreateUserRequest>) -> Json<CreateUserResponse> {
    Json(CreateUserResponse {
        id: 12345,
        name: body.name.clone(),
        email: body
            .email
            .clone()
            .unwrap_or_else(|| "default@example.com".to_string()),
        created: true,
    })
}

#[get("/health")]
fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        framework: "rocket",
    })
}

#[launch]
fn rocket() -> _ {
    println!("🚀 Rocket Benchmark Server on http://localhost:3004");

    rocket::build()
        .configure(rocket::Config {
            port: 3004,
            address: std::net::Ipv4Addr::new(127, 0, 0, 1).into(),
            ..rocket::Config::default()
        })
        .mount(
            "/",
            routes![plaintext, json, get_user, create_user, health],
        )
}

