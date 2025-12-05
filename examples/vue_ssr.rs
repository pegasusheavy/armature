// Vue.js SSR Example

use armature::armature_vue::{VueConfig, VueService};
use armature::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

// ========== DTOs ==========

#[derive(Debug, Serialize, Deserialize)]
struct PageData {
    title: String,
    content: String,
    items: Vec<String>,
}

// ========== Services ==========

#[injectable]
#[derive(Clone, Default)]
struct DataService;

impl DataService {
    fn get_home_data(&self) -> PageData {
        PageData {
            title: "Welcome to Vue SSR".to_string(),
            content: "This page is server-side rendered with Vue.js!".to_string(),
            items: vec![
                "Fast initial page load".to_string(),
                "SEO-friendly".to_string(),
                "Client-side hydration".to_string(),
            ],
        }
    }

    fn get_about_data(&self) -> PageData {
        PageData {
            title: "About Us".to_string(),
            content: "Learn more about this application".to_string(),
            items: vec![
                "Built with Armature".to_string(),
                "Powered by Vue.js".to_string(),
                "Rust + JavaScript".to_string(),
            ],
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
    fn get_home_data(&self) -> Result<Json<PageData>, Error> {
        Ok(Json(self.data_service.get_home_data()))
    }

    fn get_about_data(&self) -> Result<Json<PageData>, Error> {
        Ok(Json(self.data_service.get_about_data()))
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
    println!("🖖 Armature Vue.js SSR Example");
    println!("==============================\n");

    let app = create_app();

    println!("Server running on http://localhost:3016");
    println!();
    println!("Setup Instructions:");
    println!();
    println!("1. Create a Vue 3 app:");
    println!("   npm create vue@latest my-vue-app");
    println!("   cd my-vue-app");
    println!();
    println!("2. Install Vue SSR dependencies:");
    println!("   npm install vue@latest vue-router@latest");
    println!("   npm install --save-dev @vitejs/plugin-vue");
    println!();
    println!("3. Create server entry (server-bundle.js):");
    println!("   ```javascript");
    println!("   const {{ createSSRApp }} = require('vue');");
    println!("   const {{ renderToString }} = require('vue/server-renderer');");
    println!("   ");
    println!("   // Read from stdin");
    println!("   let input = '';");
    println!("   process.stdin.on('data', (chunk) => {{");
    println!("     input += chunk;");
    println!("   }});");
    println!("   ");
    println!("   process.stdin.on('end', async () => {{");
    println!("     const {{ url, context }} = JSON.parse(input);");
    println!("     ");
    println!("     const app = createSSRApp({{");
    println!("       data: () => context,");
    println!("       template: '<div>{{ title }}</div>'");
    println!("     }});");
    println!("     ");
    println!("     const html = await renderToString(app);");
    println!("     console.log(html);");
    println!("   }});");
    println!("   ```");
    println!();
    println!("4. Build your Vue app:");
    println!("   npm run build");
    println!();
    println!("Endpoints:");
    println!("  GET /                   - Vue SSR home page");
    println!("  GET /about              - Vue SSR about page");
    println!("  GET /assets/*           - Static files (JS, CSS, images)");
    println!("  GET /api/home           - JSON API for home data");
    println!("  GET /api/about          - JSON API for about data");
    println!();
    println!("Features:");
    println!("  ✓ Server-Side Rendering with Vue 3");
    println!("  ✓ Vue Router integration");
    println!("  ✓ Static file serving");
    println!("  ✓ SEO-friendly pages");
    println!("  ✓ Fast initial page load");
    println!("  ✓ Client-side hydration");
    println!("  ✓ API data integration");
    println!("  ✓ Composition API support");
    println!();

    if let Err(e) = app.listen(3016).await {
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

    // Configure Vue SSR
    let vue_config = VueConfig::new(PathBuf::from("dist"))
        .with_static_dir(PathBuf::from("dist/client"))
        .with_server_entry("server-bundle.js".to_string())
        .with_template(PathBuf::from("dist/index.html"))
        .with_cache(false)
        .with_compression(true);

    let vue_service = VueService::new(vue_config);

    // API routes
    let home_ctrl = api_controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/home".to_string(),
        handler: Arc::new(move |_req| {
            let ctrl = home_ctrl.clone();
            Box::pin(async move { ctrl.get_home_data()?.into_response() })
        }),
    });

    let about_ctrl = api_controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/about".to_string(),
        handler: Arc::new(move |_req| {
            let ctrl = about_ctrl.clone();
            Box::pin(async move { ctrl.get_about_data()?.into_response() })
        }),
    });

    // Static files route
    let static_service = vue_service.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/assets".to_string(),
        handler: Arc::new(move |req| {
            let service = static_service.clone();
            Box::pin(async move {
                let path = req.path.trim_start_matches("/assets");
                service.serve_static(path).await
            })
        }),
    });

    // Vue SSR route with pre-fetched data (home page)
    let ssr_home = vue_service.clone();
    let home_data_service = data_service.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/".to_string(),
        handler: Arc::new(move |_req| {
            let service = ssr_home.clone();
            let data_svc = home_data_service.clone();
            Box::pin(async move {
                let data = data_svc.get_home_data();
                let json_data =
                    serde_json::to_value(data).map_err(|e| Error::Serialization(e.to_string()))?;
                service.render_with_data("/", json_data).await
            })
        }),
    });

    // Vue SSR route with pre-fetched data (about page)
    let ssr_about = vue_service.clone();
    let about_data_service = data_service.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/about".to_string(),
        handler: Arc::new(move |_req| {
            let service = ssr_about.clone();
            let data_svc = about_data_service.clone();
            Box::pin(async move {
                let data = data_svc.get_about_data();
                let json_data =
                    serde_json::to_value(data).map_err(|e| Error::Serialization(e.to_string()))?;
                service.render_with_data("/about", json_data).await
            })
        }),
    });

    Application::new(container, router)
}
