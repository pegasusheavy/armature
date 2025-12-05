// Simplified Angular SSR example

use armature_angular::{AngularConfig, AngularService};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    println!("🅰️  Armature Angular SSR Example");
    println!("=================================\n");

    // Configure Angular SSR
    let config = AngularConfig::new()
        .with_node_path(PathBuf::from("node"))
        .with_server_bundle(PathBuf::from("dist/my-app/server/main.js"))
        .with_browser_dist(PathBuf::from("dist/my-app/browser"))
        .exclude_route("/api".to_string())
        .with_cache(true, 300);

    println!("Angular SSR Configuration:");
    println!("  Node: {:?}", config.node_path);
    println!("  Server Bundle: {:?}", config.server_bundle_path);
    println!("  Browser Dist: {:?}", config.browser_dist_path);
    println!("  Excluded Routes: {:?}", config.excluded_routes);
    println!("  Cache Enabled: {}", config.enable_cache);
    println!();

    // Create Angular service
    match AngularService::new(config) {
        Ok(service) => {
            println!("✅ Angular SSR service created successfully!");
            println!();
            println!("Service capabilities:");
            println!("  - Server-side rendering of Angular routes");
            println!("  - Static file serving");
            println!("  - Automatic route detection");
            println!("  - Caching support");
            println!();
            println!("Testing route detection:");
            println!("  / -> SSR: {}", service.should_render("/"));
            println!("  /home -> SSR: {}", service.should_render("/home"));
            println!(
                "  /api/users -> SSR: {}",
                service.should_render("/api/users")
            );
            println!(
                "  /assets/logo.png -> SSR: {}",
                service.should_render("/assets/logo.png")
            );
            println!("  /main.js -> SSR: {}", service.should_render("/main.js"));
        }
        Err(e) => {
            println!("⚠️  Angular SSR not configured: {}", e);
            println!();
            println!("Setup steps:");
            println!("  1. Create Angular app: ng new my-app");
            println!("  2. Add SSR: ng add @angular/ssr");
            println!("  3. Build: ng build && ng run my-app:server");
            println!("  4. Update paths in this example");
            println!();
            println!("This is a demonstration of the Angular SSR configuration API.");
        }
    }

    println!();
    println!("For a full integration example, see:");
    println!("  - docs/ANGULAR_SSR_GUIDE.md");
    println!("  - examples/angular_ssr.rs (requires Angular app)");
}
