#![allow(dead_code)]
// RESTful API example with CRUD operations

use armature::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct Task {
    id: u32,
    title: String,
    completed: bool,
}

#[injectable]
#[derive(Default, Clone)]
struct TaskService;

impl TaskService {
    fn get_all(&self) -> Vec<Task> {
        vec![
            Task {
                id: 1,
                title: "Learn Rust".to_string(),
                completed: true,
            },
            Task {
                id: 2,
                title: "Build with Armature".to_string(),
                completed: false,
            },
        ]
    }
}

#[controller("/tasks")]
#[derive(Default)]
struct TaskController;

impl TaskController {
    #[get("")]
    async fn list() -> Result<Json<Vec<Task>>, Error> {
        let service = TaskService;
        Ok(Json(service.get_all()))
    }

    #[get("/:id")]
    async fn get(req: HttpRequest) -> Result<Json<Task>, Error> {
        let id_str = req
            .param("id")
            .ok_or_else(|| Error::Validation("Missing id".to_string()))?;
        let id: u32 = id_str
            .parse()
            .map_err(|_| Error::Validation("Invalid id".to_string()))?;

        let service = TaskService;
        let task = service
            .get_all()
            .into_iter()
            .find(|t| t.id == id)
            .ok_or_else(|| Error::RouteNotFound("Task not found".to_string()))?;

        Ok(Json(task))
    }

    #[post("")]
    async fn create(req: HttpRequest) -> Result<Json<Task>, Error> {
        let mut task: Task = req.json()?;
        task.id = 3; // In real app, would be auto-generated
        Ok(Json(task))
    }

    #[put("/:id")]
    async fn update(req: HttpRequest) -> Result<Json<Task>, Error> {
        let _id = req
            .param("id")
            .ok_or_else(|| Error::Validation("Missing id".to_string()))?;
        let task: Task = req.json()?;
        Ok(Json(task))
    }

    #[delete("/:id")]
    async fn delete(req: HttpRequest) -> Result<HttpResponse, Error> {
        let _id = req
            .param("id")
            .ok_or_else(|| Error::Validation("Missing id".to_string()))?;
        Ok(HttpResponse::no_content())
    }
}

#[module(
    providers: [TaskService],
    controllers: [TaskController]
)]
#[derive(Default)]
struct AppModule;

#[tokio::main]
async fn main() {
    println!("Starting REST API example on http://localhost:3002");
    println!("\nEndpoints:");
    println!("  GET    /tasks      - List all tasks");
    println!("  GET    /tasks/:id  - Get task by ID");
    println!("  POST   /tasks      - Create new task");
    println!("  PUT    /tasks/:id  - Update task");
    println!("  DELETE /tasks/:id  - Delete task");

    let app = Application::create::<AppModule>().await;
    app.listen(3002).await.unwrap();
}
