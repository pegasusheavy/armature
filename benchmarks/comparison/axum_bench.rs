//! Axum benchmark server
//! Run with: cargo run --release --bin axum_bench

use axum::{routing::get, Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;

#[derive(Serialize)]
struct Message {
    message: &'static str,
}

async fn json_handler() -> Json<Message> {
    Json(Message { message: "Hello, World!" })
}

async fn plaintext_handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8083".to_string())
        .parse()
        .unwrap();

    println!("Axum benchmark server starting on port {}", port);

    let app = Router::new()
        .route("/json", get(json_handler))
        .route("/plaintext", get(plaintext_handler));

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

