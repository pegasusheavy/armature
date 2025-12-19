//! Armature benchmark server - simple JSON endpoint
//! Run with: cargo run --release --bin armature_bench

use armature::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct Message {
    message: &'static str,
}

#[controller("/")]
#[derive(Default, Clone, Debug)]
struct BenchController;

#[routes]
impl BenchController {
    #[get("/json")]
    async fn json() -> Result<HttpResponse, Error> {
        let msg = Message { message: "Hello, World!" };
        HttpResponse::ok().with_json(&msg)
    }

    #[get("/plaintext")]
    async fn plaintext() -> Result<HttpResponse, Error> {
        Ok(HttpResponse::ok()
            .with_header("Content-Type".to_string(), "text/plain".to_string())
            .with_body(b"Hello, World!".to_vec()))
    }
}

#[module(controllers: [BenchController])]
#[derive(Default)]
struct BenchModule;

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port: u16 = port.parse().unwrap();

    println!("Armature benchmark server starting on port {}", port);

    let app = Application::create::<BenchModule>().await;
    app.listen(port).await.unwrap();
}
