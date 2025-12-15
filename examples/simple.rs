#![allow(dead_code)]
// Simple example demonstrating basic routing and JSON responses

use armature::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Message {
    text: String,
}

#[controller("/api")]
#[derive(Default, Clone)]
struct ApiController;

#[routes]
impl ApiController {
    #[get("/hello")]
    async fn hello() -> Result<HttpResponse, Error> {
        HttpResponse::json(&Message {
            text: "Hello from Armature!".to_string(),
        })
    }

    #[post("/echo")]
    async fn echo(req: HttpRequest) -> Result<HttpResponse, Error> {
        let msg: Message = req.json()?;
        HttpResponse::json(&msg)
    }
}

#[module(
    controllers: [ApiController]
)]
#[derive(Default)]
struct AppModule;

#[tokio::main]
async fn main() {
    println!("Starting simple example on http://localhost:3001");
    println!("Try: curl http://localhost:3001/api/hello");

    let app = Application::create::<AppModule>().await;
    app.listen(3001).await.unwrap();
}
