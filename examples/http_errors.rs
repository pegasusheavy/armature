// HTTP Status and Error Handling Example

use armature::Error;
use armature::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
}

// ========== Controllers ==========

#[controller("/api")]
#[derive(Default, Clone)]
struct ErrorController {
    item_service: ItemService,
}

impl ErrorController {
    fn get_item(&self, id: u32) -> Result<Json<Item>, Error> {
        self.item_service.get_item(id).map(Json)
    }

    fn list_statuses(&self) -> Result<Json<Vec<StatusInfo>>, Error> {
        let statuses = vec![
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
        ];
        Ok(Json(statuses))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StatusInfo {
    code: u16,
    name: String,
    category: String,
}

// ========== Module ==========

#[module(
    providers: [ItemService],
    controllers: [ErrorController]
)]
#[derive(Default)]
struct AppModule;

#[tokio::main]
async fn main() {
    println!("⚠️  Armature HTTP Status & Error Example");
    println!("=========================================\n");

    let app = create_app();

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
    println!("7. Conflict (409):");
    println!("   curl http://localhost:3013/api/items/409");
    println!();
    println!("8. I'm a teapot (418):");
    println!("   curl http://localhost:3013/api/items/418");
    println!();
    println!("9. Too Many Requests (429):");
    println!("   curl http://localhost:3013/api/items/429");
    println!();
    println!("10. Internal Server Error (500):");
    println!("    curl http://localhost:3013/api/items/500");
    println!();
    println!("11. Service Unavailable (503):");
    println!("    curl http://localhost:3013/api/items/503");
    println!();

    if let Err(e) = app.listen(3013).await {
        eprintln!("Server error: {}", e);
    }
}

fn create_app() -> Application {
    let container = Container::new();
    let mut router = Router::new();

    // Register service
    let item_service = ItemService::default();
    container.register(item_service.clone());

    let controller = ErrorController { item_service };

    // List all statuses endpoint
    let list_ctrl = controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/statuses".to_string(),
        handler: Arc::new(move |_req| {
            let ctrl = list_ctrl.clone();
            Box::pin(async move { ctrl.list_statuses()?.into_response() })
        }),
    });

    // Get item by ID endpoint (demonstrates various errors)
    let get_ctrl = controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/items/:id".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = get_ctrl.clone();
            Box::pin(async move {
                let id_str = req
                    .param("id")
                    .ok_or_else(|| Error::BadRequest("Missing ID".to_string()))?;
                let id: u32 = id_str
                    .parse()
                    .map_err(|_| Error::BadRequest("Invalid ID format".to_string()))?;

                match ctrl.get_item(id) {
                    Ok(item) => item.into_response(),
                    Err(e) => {
                        // Create custom error response
                        let status = e.status_code();
                        let error_response = ErrorResponse {
                            status,
                            error: e.http_status().reason().to_string(),
                            message: e.to_string(),
                        };

                        println!("❌ Error {} - {}", status, e);

                        Ok(HttpResponse {
                            status,
                            headers: std::collections::HashMap::new(),
                            body: serde_json::to_vec(&error_response)
                                .map_err(|e| Error::Serialization(e.to_string()))?,
                        })
                    }
                }
            })
        }),
    });

    Application::new(container, router)
}
