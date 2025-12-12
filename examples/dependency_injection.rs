// Example demonstrating full dependency injection in Armature

use armature::prelude::*;
use serde::{Deserialize, Serialize};

// ========== Domain Models ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    id: u32,
    name: String,
    price: f64,
}

#[derive(Debug, Deserialize)]
struct CreateProductDto {
    name: String,
    price: f64,
}

// ========== Services (Injectable) ==========

/// Database service - simulates database operations
#[injectable]
#[derive(Default, Clone)]
struct DatabaseService;

impl DatabaseService {
    fn connect_info(&self) -> String {
        "Connected to database".to_string()
    }
}

/// Product service - handles business logic for products
#[injectable]
#[derive(Default, Clone)]
struct ProductService {
    database: DatabaseService,
}

impl ProductService {
    fn get_all_products(&self) -> Vec<Product> {
        println!("ProductService using: {}", self.database.connect_info());
        vec![
            Product {
                id: 1,
                name: "Laptop".to_string(),
                price: 999.99,
            },
            Product {
                id: 2,
                name: "Mouse".to_string(),
                price: 29.99,
            },
            Product {
                id: 3,
                name: "Keyboard".to_string(),
                price: 79.99,
            },
        ]
    }

    fn get_product_by_id(&self, id: u32) -> Option<Product> {
        self.get_all_products().into_iter().find(|p| p.id == id)
    }

    fn create_product(&self, name: String, price: f64) -> Product {
        println!(
            "ProductService creating product via: {}",
            self.database.connect_info()
        );
        Product {
            id: 4, // In real app, would be auto-generated
            name,
            price,
        }
    }
}

// ========== Controllers with DI ==========

/// Product controller - has ProductService automatically injected
#[controller("/products")]
#[derive(Default, Clone)]
struct ProductController {
    product_service: ProductService,
}

impl ProductController {
    #[get("")]
    async fn list_products() -> Result<Json<Vec<Product>>, Error> {
        let service = ProductService::default();
        let products = service.get_all_products();
        Ok(Json(products))
    }

    #[get("/:id")]
    async fn get_product(req: HttpRequest) -> Result<Json<Product>, Error> {
        let id_str = req
            .param("id")
            .ok_or_else(|| Error::Validation("Missing id".to_string()))?;
        let id: u32 = id_str
            .parse()
            .map_err(|_| Error::Validation("Invalid id".to_string()))?;

        let service = ProductService::default();
        match service.get_product_by_id(id) {
            Some(product) => Ok(Json(product)),
            None => Err(Error::RouteNotFound(format!("Product {} not found", id))),
        }
    }

    #[post("")]
    async fn create_product(req: HttpRequest) -> Result<Json<Product>, Error> {
        let dto: CreateProductDto = req.json()?;
        let service = ProductService::default();
        let product = service.create_product(dto.name, dto.price);
        Ok(Json(product))
    }
}

/// Health controller - has no dependencies
#[controller("/health")]
#[derive(Default, Clone)]
struct HealthController;

impl HealthController {
    #[get("")]
    async fn check() -> Result<Json<serde_json::Value>, Error> {
        Ok(Json(serde_json::json!({
            "status": "healthy",
            "message": "DI example is running"
        })))
    }
}

// ========== Module Configuration ==========

#[module(
    providers: [DatabaseService, ProductService],
    controllers: [ProductController, HealthController]
)]
#[derive(Default)]
struct AppModule;

// ========== Main Application ==========

#[tokio::main]
async fn main() {
    println!("🦾 Armature Dependency Injection Example");
    println!("=========================================\n");

    println!("Setting up dependency injection:");
    println!("  1. DatabaseService (no dependencies)");
    println!("  2. ProductService (depends on DatabaseService)");
    println!("  3. ProductController (depends on ProductService)\n");

    println!("📚 Available routes:");
    println!("  GET    /health              - Health check");
    println!("  GET    /products            - List all products");
    println!("  GET    /products/:id        - Get product by ID");
    println!("  POST   /products            - Create new product");

    println!("\n💡 Try:");
    println!("  curl http://localhost:3003/health");
    println!("  curl http://localhost:3003/products");
    println!("  curl http://localhost:3003/products/1");
    println!("  curl -X POST http://localhost:3003/products \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!("    -d '{{\"name\":\"Monitor\",\"price\":299.99}}'");
    println!();

    let app = Application::create::<AppModule>().await;

    if let Err(e) = app.listen(3003).await {
        eprintln!("❌ Server error: {}", e);
    }
}
