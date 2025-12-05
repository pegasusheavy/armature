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

// ========== Services (Injectable) ==========

/// Database service - simulates database operations
#[injectable]
#[derive(Default, Clone)]
struct DatabaseService {
    // In a real app, this would hold a connection pool
}

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
    product_service: ProductService, // This will be auto-injected!
}

impl ProductController {
    fn list_products(&self) -> Result<Json<Vec<Product>>, Error> {
        let products = self.product_service.get_all_products();
        Ok(Json(products))
    }

    fn get_product(&self, id: u32) -> Result<Json<Product>, Error> {
        match self.product_service.get_product_by_id(id) {
            Some(product) => Ok(Json(product)),
            None => Err(Error::RouteNotFound(format!("Product {} not found", id))),
        }
    }

    fn create_product(&self, name: String, price: f64) -> Result<Json<Product>, Error> {
        let product = self.product_service.create_product(name, price);
        Ok(Json(product))
    }
}

/// Health controller - has no dependencies
#[controller("/health")]
#[derive(Default, Clone)]
struct HealthController;

impl HealthController {
    fn check(&self) -> Result<Json<serde_json::Value>, Error> {
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

    // Create application with full DI
    let app = create_app_with_di();

    println!("\n📚 Available routes:");
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

    if let Err(e) = app.listen(3003).await {
        eprintln!("❌ Server error: {}", e);
    }
}

/// Create application with full DI integration
fn create_app_with_di() -> Application {
    let container = Container::new();
    let mut router = Router::new();

    // In a full implementation, this would use Application::create::<AppModule>()
    // For now, manually set up DI to demonstrate the pattern

    // Step 1: Register services in dependency order
    println!("  ✓ Registering DatabaseService");
    let db_service = DatabaseService::default();
    container.register(db_service.clone());

    println!("  ✓ Registering ProductService (with DatabaseService injected)");
    let product_service = ProductService {
        database: db_service,
    };
    container.register(product_service.clone());

    // Step 2: Create controllers with injected dependencies
    println!("  ✓ Creating ProductController (with ProductService injected)");
    let product_controller = ProductController {
        product_service: product_service.clone(),
    };

    let health_controller = HealthController;

    // Step 3: Register routes with controller instances
    println!("  ✓ Registering routes");

    // Health routes
    let health_ctrl_clone = health_controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/health".to_string(),
        handler: std::sync::Arc::new(move |_req| {
            let ctrl = health_ctrl_clone.clone();
            Box::pin(async move { ctrl.check().and_then(|j| j.into_response()) })
        }),
    });

    // Product routes
    let product_ctrl_clone1 = product_controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/products".to_string(),
        handler: std::sync::Arc::new(move |_req| {
            let ctrl = product_ctrl_clone1.clone();
            Box::pin(async move { ctrl.list_products().and_then(|j| j.into_response()) })
        }),
    });

    let product_ctrl_clone2 = product_controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/products/:id".to_string(),
        handler: std::sync::Arc::new(move |req| {
            let ctrl = product_ctrl_clone2.clone();
            Box::pin(async move {
                let id_str = req
                    .param("id")
                    .ok_or_else(|| Error::Validation("Missing id".to_string()))?;
                let id: u32 = id_str
                    .parse()
                    .map_err(|_| Error::Validation("Invalid id".to_string()))?;
                ctrl.get_product(id).and_then(|j| j.into_response())
            })
        }),
    });

    let product_ctrl_clone3 = product_controller.clone();
    router.add_route(Route {
        method: HttpMethod::POST,
        path: "/products".to_string(),
        handler: std::sync::Arc::new(move |req| {
            let ctrl = product_ctrl_clone3.clone();
            Box::pin(async move {
                #[derive(Deserialize)]
                struct CreateProductDto {
                    name: String,
                    price: f64,
                }

                let dto: CreateProductDto = req.json()?;
                ctrl.create_product(dto.name, dto.price)
                    .and_then(|j| j.into_response())
            })
        }),
    });

    Application {
        container,
        router: std::sync::Arc::new(router),
    }
}
