// React SSR Example

use armature::armature_react::{ReactConfig, ReactService};
use armature::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

// ========== DTOs ==========

#[derive(Debug, Serialize, Deserialize)]
struct ApiData {
    message: String,
    items: Vec<String>,
}

// ========== Services ==========

#[injectable]
#[derive(Clone, Default)]
struct ApiService;

impl ApiService {
    fn get_data(&self) -> ApiData {
        ApiData {
            message: "Data from API".to_string(),
            items: vec![
                "Item 1".to_string(),
                "Item 2".to_string(),
                "Item 3".to_string(),
            ],
        }
    }
}

// ========== Controllers ==========

#[controller("/api")]
#[derive(Default, Clone)]
struct ApiController {
    api_service: ApiService,
}

impl ApiController {
    fn get_data(&self) -> Result<Json<ApiData>, Error> {
        Ok(Json(self.api_service.get_data()))
    }
}

// ========== Module ==========

#[module(
    providers: [ApiService],
    controllers: [ApiController]
)]
#[derive(Default)]
struct AppModule;

#[tokio::main]
async fn main() {
    println!("⚛️  Armature React SSR Example");
    println!("==============================\n");

    let app = create_app();

    println!("Server running on http://localhost:3015");
    println!();
    println!("Setup Instructions:");
    println!();
    println!("1. Create a React app with SSR:");
    println!("   npx create-react-app my-app");
    println!("   cd my-app");
    println!();
    println!("2. Add React SSR support:");
    println!("   npm install express");
    println!();
    println!("3. Create server/index.js:");
    println!("   ```javascript");
    println!("   const express = require('express');");
    println!("   const React = require('react');");
    println!("   const ReactDOMServer = require('react-dom/server');");
    println!("   const App = require('../src/App').default;");
    println!("   ");
    println!("   // Read from stdin");
    println!("   let input = '';");
    println!("   process.stdin.on('data', (chunk) => {{");
    println!("     input += chunk;");
    println!("   }});");
    println!("   ");
    println!("   process.stdin.on('end', () => {{");
    println!("     const {{ url, props }} = JSON.parse(input);");
    println!("     const html = ReactDOMServer.renderToString(");
    println!("       React.createElement(App, props)");
    println!("     );");
    println!("     console.log(html);");
    println!("   }});");
    println!("   ```");
    println!();
    println!("4. Build your React app:");
    println!("   npm run build");
    println!();
    println!("Endpoints:");
    println!("  GET /                   - React SSR home page");
    println!("  GET /about              - React SSR about page");
    println!("  GET /static/*           - Static files (JS, CSS, images)");
    println!("  GET /api/data           - JSON API endpoint");
    println!();
    println!("Features:");
    println!("  ✓ Server-Side Rendering with React");
    println!("  ✓ Static file serving");
    println!("  ✓ SEO-friendly pages");
    println!("  ✓ Fast initial page load");
    println!("  ✓ Hydration on client-side");
    println!("  ✓ API integration");
    println!();

    if let Err(e) = app.listen(3015).await {
        eprintln!("Server error: {}", e);
    }
}

fn create_app() -> Application {
    let container = Container::new();
    let mut router = Router::new();

    // Register services
    let api_service = ApiService::default();
    container.register(api_service.clone());

    let api_controller = ApiController { api_service };

    // Configure React SSR
    let react_config = ReactConfig::new(PathBuf::from("build"))
        .with_static_dir(PathBuf::from("build/static"))
        .with_server_entry("server/index.js".to_string())
        .with_cache(false)
        .with_compression(true);

    let react_service = ReactService::new(react_config);

    // API routes
    let data_ctrl = api_controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/data".to_string(),
        handler: Arc::new(move |_req| {
            let ctrl = data_ctrl.clone();
            Box::pin(async move { ctrl.get_data()?.into_response() })
        }),
    });

    // Static files route
    let static_service = react_service.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/static".to_string(),
        handler: Arc::new(move |req| {
            let service = static_service.clone();
            Box::pin(async move {
                let path = req.path.trim_start_matches("/static");
                service.serve_static(path).await
            })
        }),
    });

    // React SSR routes (catch-all for app routes)
    let ssr_service = react_service.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/".to_string(),
        handler: Arc::new(move |req| {
            let service = ssr_service.clone();
            Box::pin(async move {
                // Skip API and static routes
                if req.path.starts_with("/api") || req.path.starts_with("/static") {
                    return Err(Error::NotFound("Not found".to_string()));
                }

                service.render(&req).await
            })
        }),
    });

    Application::new(container, router)
}
