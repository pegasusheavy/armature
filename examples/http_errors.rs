#![allow(
    dead_code,
    unused_imports,
    clippy::default_constructed_unit_structs,
    clippy::needless_borrow,
    clippy::unnecessary_lazy_evaluations
)]
// HTTP Status and Error Handling Example

use armature::prelude::*;
use armature::Error;
use serde::{Deserialize, Serialize};

// ========== DTOs ==========

#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    status: u16,
    error: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    id: u32,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StatusInfo {
    code: u16,
    name: String,
    category: String,
}

// ========== Services ==========

#[injectable]
#[derive(Clone, Default)]
struct ItemService;

impl ItemService {
    fn get_item(&self, id: u32) -> Result<Item, Error> {
        match id {
            1 => Ok(Item {
                id: 1,
                name: "Valid Item".to_string(),
            }),
            404 => Err(Error::NotFound("Item not found".to_string())),
            400 => Err(Error::BadRequest("Invalid item ID format".to_string())),
            401 => Err(Error::Unauthorized("Authentication required".to_string())),
            403 => Err(Error::Forbidden("Access denied".to_string())),
            409 => Err(Error::Conflict("Item already exists".to_string())),
            418 => Err(Error::ImATeapot("I'm a teapot!".to_string())),
            422 => Err(Error::UnprocessableEntity("Invalid data".to_string())),
            429 => Err(Error::TooManyRequests("Rate limit exceeded".to_string())),
            500 => Err(Error::Internal("Internal server error".to_string())),
            501 => Err(Error::NotImplemented("Feature not implemented".to_string())),
            502 => Err(Error::BadGateway("Bad gateway".to_string())),
            503 => Err(Error::ServiceUnavailable("Service unavailable".to_string())),
            _ => Err(Error::NotFound("Unknown error code".to_string())),
        }
    }

    fn list_statuses(&self) -> Vec<StatusInfo> {
        vec![
            StatusInfo {
                code: 200,
                name: "OK".to_string(),
                category: "Success".to_string(),
            },
            StatusInfo {
                code: 201,
                name: "Created".to_string(),
                category: "Success".to_string(),
            },
            StatusInfo {
                code: 400,
                name: "Bad Request".to_string(),
                category: "Client Error".to_string(),
            },
            StatusInfo {
                code: 401,
                name: "Unauthorized".to_string(),
                category: "Client Error".to_string(),
            },
            StatusInfo {
                code: 403,
                name: "Forbidden".to_string(),
                category: "Client Error".to_string(),
            },
            StatusInfo {
                code: 404,
                name: "Not Found".to_string(),
                category: "Client Error".to_string(),
            },
            StatusInfo {
                code: 429,
                name: "Too Many Requests".to_string(),
                category: "Client Error".to_string(),
            },
            StatusInfo {
                code: 500,
                name: "Internal Server Error".to_string(),
                category: "Server Error".to_string(),
            },
            StatusInfo {
                code: 503,
                name: "Service Unavailable".to_string(),
                category: "Server Error".to_string(),
            },
        ]
    }
}

// ========== Controllers ==========

#[controller("/api")]
#[derive(Default, Clone)]
struct ErrorController;

#[routes]
impl ErrorController {
    #[get("/statuses")]
    async fn list_statuses() -> Result<HttpResponse, Error> {
        let service = ItemService::default();
        HttpResponse::json(&service.list_statuses())
    }

    #[get("/items/:id")]
    async fn get_item(req: HttpRequest) -> Result<HttpResponse, Error> {
        let id_str = req
            .param("id")
            .ok_or_else(|| Error::BadRequest("Missing ID".to_string()))?;
        let id: u32 = id_str
            .parse()
            .map_err(|_| Error::BadRequest("Invalid ID format".to_string()))?;

        let service = ItemService::default();
        let item = service.get_item(id)?;
        HttpResponse::json(&item)
    }
}

// ========== Module ==========

#[module(
    providers: [ItemService],
    controllers: [ErrorController]
)]
#[derive(Default, Clone)]
struct AppModule;

#[tokio::main]
async fn main() {
    println!("⚠️  Armature HTTP Status & Error Example");
    println!("=========================================\n");

    println!("Server running on http://localhost:3013");
    println!();
    println!("API Endpoints:");
    println!("  GET /api/statuses           - List all HTTP statuses");
    println!("  GET /api/items/:id          - Get item by ID (use status code as ID)");
    println!();
    println!("Example usage:");
    println!();
    println!("1. List all statuses:");
    println!("   curl http://localhost:3013/api/statuses");
    println!();
    println!("2. Success (200):");
    println!("   curl http://localhost:3013/api/items/1");
    println!();
    println!("3. Not Found (404):");
    println!("   curl http://localhost:3013/api/items/404");
    println!();
    println!("4. Bad Request (400):");
    println!("   curl http://localhost:3013/api/items/400");
    println!();
    println!("5. Unauthorized (401):");
    println!("   curl http://localhost:3013/api/items/401");
    println!();
    println!("6. Forbidden (403):");
    println!("   curl http://localhost:3013/api/items/403");
    println!();
    println!("7. I'm a teapot (418):");
    println!("   curl http://localhost:3013/api/items/418");
    println!();
    println!("8. Internal Server Error (500):");
    println!("   curl http://localhost:3013/api/items/500");
    println!();

    let app = Application::create::<AppModule>().await;

    if let Err(e) = app.listen(3013).await {
        eprintln!("Server error: {}", e);
    }
}
