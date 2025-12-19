//! Actix-web benchmark server
//! Run with: cargo run --release --bin actix_bench

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct Message {
    message: &'static str,
}

#[get("/json")]
async fn json() -> impl Responder {
    web::Json(Message { message: "Hello, World!" })
}

#[get("/plaintext")]
async fn plaintext() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/plain")
        .body("Hello, World!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8082".to_string())
        .parse()
        .unwrap();

    println!("Actix-web benchmark server starting on port {}", port);

    HttpServer::new(|| {
        App::new()
            .service(json)
            .service(plaintext)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

