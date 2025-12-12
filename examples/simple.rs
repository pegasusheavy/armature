// Simple example demonstrating basic routing and JSON responses

use armature::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Message {
    text: String,
}

#[controller("/api")]
#[derive(Default)]
struct ApiController;

impl ApiController {
    #[get("/hello")]
    async fn hello() -> Result<Json<Message>, Error> {
        Ok(Json(Message {
            text: "Hello from Armature!".to_string(),
        }))
    }

    #[post("/echo")]
    async fn echo(req: HttpRequest) -> Result<Json<Message>, Error> {
        let msg: Message = req.json()?;
        Ok(Json(msg))
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

    // In a full implementation, this would work:
    // let app = Application::create::<AppModule>();
    // app.listen(3001).await.unwrap();

    // For now, use simplified bootstrap
    let container = Container::new();
    let router = Router::new();
    let app = Application::new(container, router);

    app.listen(3001).await.unwrap();
}
