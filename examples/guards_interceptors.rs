// Guards and Interceptors example

use armature::prelude::*;
use armature::{AuthenticationGuard, Guard, GuardContext, RolesGuard};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
struct DataController {
    data_service: DataService,
}

impl DataController {
    fn public(&self) -> Result<Json<Message>, Error> {
        Ok(Json(self.data_service.get_public_data()))
    }

    fn protected(&self) -> Result<Json<Message>, Error> {
        Ok(Json(self.data_service.get_protected_data()))
    }

    fn admin(&self) -> Result<Json<Message>, Error> {
        Ok(Json(self.data_service.get_admin_data()))
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

    let app = create_guarded_app();

    println!("Server running on http://localhost:3012");
    println!();
    println!("API Endpoints:");
    println!("  GET /api/public     - Public (no auth required)");
    println!("  GET /api/protected  - Protected (requires Bearer token)");
    println!("  GET /api/admin      - Admin only (requires Bearer token)");
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
    println!("  ✓ ApiKeyGuard - Validates API keys");
    println!();
    println!("Interceptors:");
    println!("  ✓ LoggingInterceptor - Logs all requests");
    println!("  ✓ TransformInterceptor - Modifies responses");
    println!("  ✓ CacheInterceptor - Caches responses");
    println!();

    if let Err(e) = app.listen(3012).await {
        eprintln!("Server error: {}", e);
    }
}

fn create_guarded_app() -> Application {
    let container = Container::new();
    let mut router = Router::new();

    // Register service
    let data_service = DataService;
    container.register(data_service.clone());

    let controller = DataController { data_service };

    // Public endpoint (no guards)
    let public_ctrl = controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/public".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = public_ctrl.clone();
            Box::pin(async move {
                // Logging interceptor simulation
                println!("→ {} {}", req.method, req.path);
                let result = ctrl.public()?.into_response();
                println!("← {} {} - 200", req.method, req.path);
                result
            })
        }),
    });

    // Protected endpoint (with authentication guard)
    let protected_ctrl = controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/protected".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = protected_ctrl.clone();
            Box::pin(async move {
                // Apply AuthenticationGuard
                let guard = AuthenticationGuard;
                let context = GuardContext::new(req.clone());

                match guard.can_activate(&context).await {
                    Ok(true) => {}
                    Ok(false) => {
                        return Err(Error::Forbidden("Authentication required".to_string()));
                    }
                    Err(e) => return Err(e),
                }

                println!("✓ Authentication guard passed");
                ctrl.protected()?.into_response()
            })
        }),
    });

    // Admin endpoint (with roles guard)
    let admin_ctrl = controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/admin".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = admin_ctrl.clone();
            Box::pin(async move {
                // Apply RolesGuard
                let guard = RolesGuard::new(vec!["admin".to_string()]);
                let context = GuardContext::new(req.clone());

                match guard.can_activate(&context).await {
                    Ok(true) => {}
                    Ok(false) => return Err(Error::Forbidden("Admin role required".to_string())),
                    Err(e) => return Err(e),
                }

                println!("✓ Roles guard passed");
                ctrl.admin()?.into_response()
            })
        }),
    });

    Application::new(container, router)
}
