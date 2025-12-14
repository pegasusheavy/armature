//! Warp Benchmark Server
//! Port: 3003

use serde::{Deserialize, Serialize};
use warp::Filter;

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

#[tokio::main]
async fn main() {
    println!("🚀 Warp Benchmark Server on http://localhost:3003");

    // GET /
    let plaintext = warp::path::end()
        .and(warp::get())
        .map(|| "Hello, World!");

    // GET /json
    let json = warp::path("json")
        .and(warp::get())
        .map(|| warp::reply::json(&JsonResponse { message: "Hello, World!" }));

    // GET /users/:id
    let get_user = warp::path!("users" / String)
        .and(warp::get())
        .map(|id: String| {
            warp::reply::json(&UserResponse {
                id: id.clone(),
                name: format!("User {}", id),
                email: format!("user{}@example.com", id),
            })
        });

    // POST /api/users
    let create_user = warp::path!("api" / "users")
        .and(warp::post())
        .and(warp::body::json())
        .map(|body: CreateUserRequest| {
            warp::reply::with_status(
                warp::reply::json(&CreateUserResponse {
                    id: 12345,
                    name: body.name,
                    email: body.email.unwrap_or_else(|| "default@example.com".to_string()),
                    created: true,
                }),
                warp::http::StatusCode::CREATED,
            )
        });

    // GET /health
    let health = warp::path("health")
        .and(warp::get())
        .map(|| {
            warp::reply::json(&HealthResponse {
                status: "healthy",
                framework: "warp",
            })
        });

    let routes = plaintext
        .or(json)
        .or(get_user)
        .or(create_user)
        .or(health);

    warp::serve(routes).run(([127, 0, 0, 1], 3003)).await;
}

