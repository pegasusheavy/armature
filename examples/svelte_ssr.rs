// Svelte SSR Example

use armature::armature_svelte::{SvelteConfig, SvelteService};
use armature::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

// ========== DTOs ==========

#[derive(Debug, Serialize, Deserialize)]
struct PageProps {
    title: String,
    message: String,
    items: Vec<String>,
    count: u32,
}

// ========== Services ==========

#[injectable]
#[derive(Clone, Default)]
struct DataService;

impl DataService {
    fn get_home_props(&self) -> PageProps {
        PageProps {
            title: "Welcome to Svelte SSR".to_string(),
            message: "This page is server-side rendered with Svelte!".to_string(),
            items: vec![
                "⚡ Lightning-fast SSR".to_string(),
                "🎨 Scoped CSS".to_string(),
                "🚀 Small bundle size".to_string(),
                "✨ Reactive by default".to_string(),
            ],
            count: 42,
        }
    }

    fn get_about_props(&self) -> PageProps {
        PageProps {
            title: "About Svelte SSR".to_string(),
            message: "Learn more about this application".to_string(),
            items: vec![
                "Built with Armature".to_string(),
                "Powered by Svelte".to_string(),
                "Compiled, not interpreted".to_string(),
                "True reactivity".to_string(),
            ],
            count: 24,
        }
    }
}

// ========== Controllers ==========

#[controller("/api")]
#[derive(Default, Clone)]
struct ApiController {
    data_service: DataService,
}

impl ApiController {
    fn get_home(&self) -> Result<Json<PageProps>, Error> {
        Ok(Json(self.data_service.get_home_props()))
    }

    fn get_about(&self) -> Result<Json<PageProps>, Error> {
        Ok(Json(self.data_service.get_about_props()))
    }
}

// ========== Module ==========

#[module(
    providers: [DataService],
    controllers: [ApiController]
)]
#[derive(Default)]
struct AppModule;

#[tokio::main]
async fn main() {
    println!("🔥 Armature Svelte SSR Example");
    println!("==============================\n");

    let app = create_app();

    println!("Server running on http://localhost:3017");
    println!();
    println!("Setup Instructions:");
    println!();
    println!("1. Create a SvelteKit app:");
    println!("   npm create svelte@latest my-app");
    println!("   cd my-app");
    println!("   npm install");
    println!();
    println!("2. Create server entry (server/index.js):");
    println!("   ```javascript");
    println!("   import {{ render }} from '../build/server/app.js';");
    println!("   ");
    println!("   let input = '';");
    println!("   process.stdin.on('data', (chunk) => {{");
    println!("     input += chunk;");
    println!("   }});");
    println!("   ");
    println!("   process.stdin.on('end', async () => {{");
    println!("     const {{ url, props }} = JSON.parse(input);");
    println!("     const result = await render(url, {{ props }});");
    println!("     console.log(JSON.stringify(result));");
    println!("   }});");
    println!("   ```");
    println!();
    println!("3. Or for standalone Svelte (no SvelteKit):");
    println!("   ```javascript");
    println!("   const App = require('./App.svelte').default;");
    println!("   ");
    println!("   process.stdin.on('end', async () => {{");
    println!("     const {{ props }} = JSON.parse(input);");
    println!("     const {{ html, css, head }} = App.render(props);");
    println!("     console.log(JSON.stringify({{ html, css, head }}));");
    println!("   }});");
    println!("   ```");
    println!();
    println!("4. Build your Svelte app:");
    println!("   npm run build");
    println!();
    println!("Endpoints:");
    println!("  GET /                   - Svelte SSR home page");
    println!("  GET /about              - Svelte SSR about page");
    println!("  GET /client/*           - Static files (JS, CSS)");
    println!("  GET /api/home           - JSON API for home data");
    println!("  GET /api/about          - JSON API for about data");
    println!();
    println!("Features:");
    println!("  ✓ Server-Side Rendering with Svelte");
    println!("  ✓ SvelteKit integration");
    println!("  ✓ Scoped CSS injection");
    println!("  ✓ SEO-friendly pages");
    println!("  ✓ Tiny bundle sizes");
    println!("  ✓ Client-side hydration");
    println!("  ✓ Reactive updates");
    println!("  ✓ Compile-time optimization");
    println!();

    if let Err(e) = app.listen(3017).await {
        eprintln!("Server error: {}", e);
    }
}

fn create_app() -> Application {
    let container = Container::new();
    let mut router = Router::new();

    // Register services
    let data_service = DataService::default();
    container.register(data_service.clone());

    let api_controller = ApiController {
        data_service: data_service.clone(),
    };

    // Configure Svelte SSR
    let svelte_config = SvelteConfig::new(PathBuf::from("build"))
        .with_static_dir(PathBuf::from("build/client"))
        .with_server_entry("server/index.js".to_string())
        .with_hydration(true)
        .with_compression(true);

    let svelte_service = SvelteService::new(svelte_config);

    // API routes
    let home_ctrl = api_controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/home".to_string(),
        handler: Arc::new(move |_req| {
            let ctrl = home_ctrl.clone();
            Box::pin(async move { ctrl.get_home()?.into_response() })
        }),
    });

    let about_ctrl = api_controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/about".to_string(),
        handler: Arc::new(move |_req| {
            let ctrl = about_ctrl.clone();
            Box::pin(async move { ctrl.get_about()?.into_response() })
        }),
    });

    // Static files route
    let static_service = svelte_service.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/client".to_string(),
        handler: Arc::new(move |req| {
            let service = static_service.clone();
            Box::pin(async move {
                let path = req.path.trim_start_matches("/client");
                service.serve_static(path).await
            })
        }),
    });

    // Svelte SSR route (home page)
    let ssr_home = svelte_service.clone();
    let home_data = data_service.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/".to_string(),
        handler: Arc::new(move |mut req| {
            let service = ssr_home.clone();
            let data_svc = home_data.clone();
            Box::pin(async move {
                // Inject props into request body for rendering
                let props = data_svc.get_home_props();
                req.body =
                    serde_json::to_vec(&props).map_err(|e| Error::Serialization(e.to_string()))?;
                service.render(&req).await
            })
        }),
    });

    // Svelte SSR route (about page)
    let ssr_about = svelte_service.clone();
    let about_data = data_service.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/about".to_string(),
        handler: Arc::new(move |mut req| {
            let service = ssr_about.clone();
            let data_svc = about_data.clone();
            Box::pin(async move {
                // Inject props into request body for rendering
                let props = data_svc.get_about_props();
                req.body =
                    serde_json::to_vec(&props).map_err(|e| Error::Serialization(e.to_string()))?;
                service.render(&req).await
            })
        }),
    });

    Application::new(container, router)
}
