#![allow(
    dead_code,
    unused_imports,
    clippy::default_constructed_unit_structs,
    clippy::needless_borrow,
    clippy::unnecessary_lazy_evaluations
)]
// Guards and Interceptors example

use armature::prelude::*;
use serde::{Deserialize, Serialize};

// ========== DTOs ==========

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    content: String,
}

// ========== Services ==========

#[injectable]
#[derive(Clone, Default)]
struct DataService;

impl DataService {
    fn get_public_data(&self) -> Message {
        Message {
            content: "This is public data".to_string(),
        }
    }

    fn get_protected_data(&self) -> Message {
        Message {
            content: "This is protected data - you're authenticated!".to_string(),
        }
    }

    fn get_admin_data(&self) -> Message {
        Message {
            content: "This is admin-only data - you have admin role!".to_string(),
        }
    }
}

// ========== Controllers ==========

#[controller("/api")]
#[derive(Default, Clone)]
struct DataController;

impl DataController {
    #[get("/public")]
    async fn public_endpoint() -> Result<Json<Message>, Error> {
        let service = DataService::default();
        Ok(Json(service.get_public_data()))
    }

    #[get("/protected")]
    async fn protected_endpoint(req: HttpRequest) -> Result<Json<Message>, Error> {
        // Check for Bearer token (simple guard simulation)
        if let Some(auth) = req.headers.get("authorization") {
            if auth.starts_with("Bearer ") {
                let service = DataService::default();
                return Ok(Json(service.get_protected_data()));
            }
        }
        Err(Error::Unauthorized("Authentication required".to_string()))
    }

    #[get("/admin")]
    async fn admin_endpoint(req: HttpRequest) -> Result<Json<Message>, Error> {
        // Check for admin token (simple roles guard simulation)
        if let Some(auth) = req.headers.get("authorization") {
            if auth == "Bearer admin-token" {
                let service = DataService::default();
                return Ok(Json(service.get_admin_data()));
            }
        }
        Err(Error::Forbidden("Admin role required".to_string()))
    }
}

// ========== Module ==========

#[module(
    providers: [DataService],
    controllers: [DataController]
)]
#[derive(Default)]
struct AppModule;

#[tokio::main]
async fn main() {
    println!("🛡️  Armature Guards & Interceptors Example");
    println!("==========================================\n");

    println!("Server running on http://localhost:3012");
    println!();
    println!("API Endpoints:");
    println!("  GET /api/public     - Public (no auth required)");
    println!("  GET /api/protected  - Protected (requires Bearer token)");
    println!("  GET /api/admin      - Admin only (requires Bearer admin-token)");
    println!();
    println!("Example usage:");
    println!();
    println!("1. Public endpoint (no auth):");
    println!("   curl http://localhost:3012/api/public");
    println!();
    println!("2. Protected endpoint (requires auth):");
    println!("   curl http://localhost:3012/api/protected \\");
    println!("     -H \"Authorization: Bearer token123\"");
    println!();
    println!("3. Admin endpoint:");
    println!("   curl http://localhost:3012/api/admin \\");
    println!("     -H \"Authorization: Bearer admin-token\"");
    println!();
    println!("Guards:");
    println!("  ✓ AuthenticationGuard - Checks for Bearer token");
    println!("  ✓ RolesGuard - Checks user roles");
    println!();

    let app = Application::create::<AppModule>().await;

    if let Err(e) = app.listen(3012).await {
        eprintln!("Server error: {}", e);
    }
}
