#![allow(dead_code)]
// Example: running a Rhai application from Rust code.
//
// This shows how to embed the Rhai app runner in a Rust binary.
// Most users will prefer `armature run app.rhai` via the CLI instead.
//
// Run:
//   export API_TOKEN=$(openssl rand -hex 32)
//   cargo run --example rhai_app
//
// Then test:
//   curl http://localhost:3001/health
//   curl -H "Authorization: Bearer $API_TOKEN" http://localhost:3001/api/tasks
//   curl http://localhost:3001/api/greetings/world

use std::path::Path;

#[tokio::main]
async fn main() {
    println!("Starting Rhai application from examples/rhai_app/app.rhai");
    println!();
    println!("Endpoints:");
    println!("  GET    /health              — health check");
    println!(
        "  GET    /api/tasks           — list tasks   (needs Authorization: Bearer $API_TOKEN)"
    );
    println!(
        "  GET    /api/tasks/:id       — get task     (needs Authorization: Bearer $API_TOKEN)"
    );
    println!(
        "  POST   /api/tasks           — create task  (needs Authorization: Bearer $API_TOKEN)"
    );
    println!(
        "  DELETE /api/tasks/:id       — delete task  (needs Authorization: Bearer $API_TOKEN)"
    );
    println!("  GET    /api/greetings/:name — greet someone");
    println!();

    // The script's AuthGuard compares against API_TOKEN and refuses every
    // protected request without it.
    if std::env::var("API_TOKEN")
        .map(|t| t.len() < 32)
        .unwrap_or(true)
    {
        eprintln!("API_TOKEN must be set to a random value of at least 32 characters, e.g.");
        eprintln!("  export API_TOKEN=$(openssl rand -hex 32)");
        std::process::exit(1);
    }

    let script = Path::new("examples/rhai_app/app.rhai");

    let config = armature_app::RunConfig {
        port: Some(3001),                    // override script port to avoid conflicts
        host: Some("127.0.0.1".to_string()), // loopback only for the demo
    };

    if let Err(e) = armature_app::run(script, config).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
