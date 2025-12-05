// Angular SSR example with Armature

use armature::prelude::*;
use armature_angular::{AngularConfig, AngularService, RenderOptions};
use std::path::PathBuf;
use std::sync::Arc;

// ========== Services ==========

#[injectable]
#[derive(Clone, Default)]
struct ApiService;

impl ApiService {
    fn get_data(&self) -> serde_json::Value {
        serde_json::json!({
            "message": "Hello from Armature API!",
            "timestamp": "2024-01-01T00:00:00Z"
        })
    }
}

// ========== Controllers ==========

#[controller("/api")]
#[derive(Default, Clone)]
struct ApiController {
    api_service: ApiService,
}

impl ApiController {
    fn get_data(&self) -> Result<Json<serde_json::Value>, Error> {
        Ok(Json(self.api_service.get_data()))
    }
}

// ========== Module ==========

#[module(
    providers: [ApiService, AngularService],
    controllers: [ApiController]
)]
#[derive(Default)]
struct AppModule;

// ========== Main ==========

#[tokio::main]
async fn main() {
    println!("🅰️  Armature Angular SSR Example");
    println!("=================================\n");

    // Note: This example assumes you have an Angular application built with SSR
    // Run: ng build && ng run your-app:server

    let app = create_angular_app();

    println!("Server running on http://localhost:3010");
    println!();
    println!("Features:");
    println!("  ✅ Angular Universal SSR");
    println!("  ✅ Static file serving");
    println!("  ✅ API routes (/api/*)");
    println!("  ✅ Client-side hydration");
    println!();
    println!("Setup Angular:");
    println!("  1. Create Angular app: ng new my-app");
    println!("  2. Add SSR: ng add @angular/ssr");
    println!("  3. Build: ng build && ng run my-app:server");
    println!("  4. Configure paths in AngularConfig");
    println!();

    if let Err(e) = app.listen(3010).await {
        eprintln!("Server error: {}", e);
    }
}

fn create_angular_app() -> Application {
    let container = Container::new();
    let mut router = Router::new();

    // Configure Angular SSR
    let angular_config = AngularConfig::new()
        .with_node_path(PathBuf::from("node"))
        .with_server_bundle(PathBuf::from("dist/my-app/server/main.js"))
        .with_browser_dist(PathBuf::from("dist/my-app/browser"))
        .exclude_route("/api".to_string())
        .with_cache(true, 300); // Cache for 5 minutes

    // Create Angular service (with fallback to demo mode)
    let angular_service = match AngularService::new(angular_config.clone()) {
        Ok(service) => service,
        Err(e) => {
            eprintln!("⚠️  Warning: Angular SSR not configured: {}", e);
            eprintln!("   Running in demo mode - build your Angular app first!");
            // Return a demo service
            AngularService::new(AngularConfig::default()).unwrap_or_default()
        }
    };

    container.register(angular_service.clone());

    // Register API service
    let api_service = ApiService;
    container.register(api_service.clone());

    // API Controller routes
    let api_controller = ApiController { api_service };

    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/data".to_string(),
        handler: Arc::new(move |_req| {
            let ctrl = api_controller.clone();
            Box::pin(async move { ctrl.get_data()?.into_response() })
        }),
    });

    // Static files handler (CSS, JS, images, etc.)
    let static_angular = angular_service.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/assets/*".to_string(),
        handler: Arc::new(move |req| {
            let service = static_angular.clone();
            Box::pin(async move {
                let path = &req.path;
                match service.serve_static(path).await {
                    Ok(content) => {
                        let content_type = service
                            .config()
                            .browser_dist_path
                            .join(path.trim_start_matches('/'))
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| match ext {
                                "js" => "application/javascript",
                                "css" => "text/css",
                                "png" => "image/png",
                                "jpg" | "jpeg" => "image/jpeg",
                                "svg" => "image/svg+xml",
                                "ico" => "image/x-icon",
                                _ => "application/octet-stream",
                            })
                            .unwrap_or("application/octet-stream");

                        Ok(HttpResponse::ok()
                            .with_header("Content-Type".to_string(), content_type.to_string())
                            .with_body(content))
                    }
                    Err(_) => Ok(HttpResponse::not_found().with_body(b"File not found".to_vec())),
                }
            })
        }),
    });

    // Main bundle files (*.js, *.css)
    let bundle_angular = angular_service.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/*.js".to_string(),
        handler: Arc::new(move |req| {
            let service = bundle_angular.clone();
            Box::pin(async move {
                let path = &req.path;
                match service.serve_static(path).await {
                    Ok(content) => Ok(HttpResponse::ok()
                        .with_header(
                            "Content-Type".to_string(),
                            "application/javascript".to_string(),
                        )
                        .with_body(content)),
                    Err(_) => Ok(HttpResponse::not_found().with_body(b"File not found".to_vec())),
                }
            })
        }),
    });

    // SSR handler for all other routes (Angular pages)
    let ssr_angular = angular_service.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/*".to_string(),
        handler: Arc::new(move |req| {
            let service = ssr_angular.clone();
            Box::pin(async move {
                let path = &req.path;

                // Check if this route should be server-rendered
                if !service.should_render(path) {
                    // Serve as static file
                    match service.serve_static(path).await {
                        Ok(content) => {
                            return Ok(HttpResponse::ok().with_body(content));
                        }
                        Err(_) => {}
                    }
                }

                // Perform SSR
                let render_options = RenderOptions::new(path.to_string());

                match service.render(path, render_options).await {
                    Ok(html) => Ok(HttpResponse::ok()
                        .with_header(
                            "Content-Type".to_string(),
                            "text/html; charset=utf-8".to_string(),
                        )
                        .with_body(html.into_bytes())),
                    Err(e) => {
                        eprintln!("SSR Error: {}", e);
                        // Fallback to index.html for client-side rendering
                        match service.serve_static("/index.html").await {
                            Ok(content) => Ok(HttpResponse::ok()
                                .with_header("Content-Type".to_string(), "text/html".to_string())
                                .with_body(content)),
                            Err(_) => Ok(HttpResponse::internal_server_error()
                                .with_body(b"Server error".to_vec())),
                        }
                    }
                }
            })
        }),
    });

    Application::new(container, router)
}
