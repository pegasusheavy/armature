//! Axum Benchmark Server
//! Port: 3002

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
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

async fn plaintext() -> &'static str {
    "Hello, World!"
}

async fn json() -> Json<JsonResponse> {
    Json(JsonResponse {
        message: "Hello, World!",
    })
}

async fn get_user(Path(id): Path<String>) -> Json<UserResponse> {
    Json(UserResponse {
        id: id.clone(),
        name: format!("User {}", id),
        email: format!("user{}@example.com", id),
    })
}

async fn create_user(Json(payload): Json<CreateUserRequest>) -> Response {
    let response = CreateUserResponse {
        id: 12345,
        name: payload.name,
        email: payload.email.unwrap_or_else(|| "default@example.com".to_string()),
        created: true,
    };
    (StatusCode::CREATED, Json(response)).into_response()
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "framework": "axum"
    }))
}

#[tokio::main]
async fn main() {
    println!("🚀 Axum Benchmark Server on http://localhost:3002");

    let app = Router::new()
        .route("/", get(plaintext))
        .route("/json", get(json))
        .route("/users/:id", get(get_user))
        .route("/api/users", post(create_user))
        .route("/health", get(health));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3002")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

