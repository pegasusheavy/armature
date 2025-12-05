//! HTTPS server with HTTP to HTTPS redirect
//!
//! This example demonstrates how to run an HTTPS server with automatic
//! HTTP to HTTPS redirection.
//!
//! # Running
//!
//! ```bash
//! cargo run --example https_with_redirect --features self-signed-certs
//! ```
//!
//! # Testing
//!
//! ```bash
//! # HTTPS (direct)
//! curl -k https://localhost:8443/
//!
//! # HTTP (will redirect to HTTPS)
//! curl -L -k http://localhost:8080/
//! ```

use armature::prelude::*;

#[derive(Default, Clone)]
#[injectable]
pub struct SecureService;

impl SecureService {
    pub fn process_secure_request(&self, path: &str) -> String {
        format!("Securely processed request to: {}", path)
    }
}

#[controller("/api")]
pub struct SecureController {
    secure_service: SecureService,
}

impl SecureController {
    pub fn new(secure_service: SecureService) -> Self {
        Self { secure_service }
    }

    #[get("/")]
    pub async fn index(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::ok().with_json(&serde_json::json!({
            "message": "Welcome to the secure API",
            "endpoints": [
                "/api/secure",
                "/api/status"
            ]
        }))?)
    }

    #[get("/secure")]
    pub async fn secure(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        let result = self.secure_service.process_secure_request(&req.path);

        Ok(HttpResponse::ok().with_json(&serde_json::json!({
            "result": result,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "secure": true
        }))?)
    }

    #[get("/status")]
    pub async fn status(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::ok().with_json(&serde_json::json!({
            "status": "operational",
            "tls": "enabled",
            "redirect": "active"
        }))?)
    }
}

#[module(
    providers: [SecureService],
    controllers: [SecureController]
)]
pub struct AppModule {}

#[cfg(feature = "self-signed-certs")]
#[tokio::main]
async fn main() -> Result<()> {
    println!("🔒 Starting HTTPS server with HTTP redirect\n");

    let app = Application::create::<AppModule>();

    // Create self-signed certificate
    println!("🔐 Generating self-signed certificate...");
    let tls_config = armature_core::TlsConfig::self_signed(&["localhost", "127.0.0.1"])?;

    // Configure HTTPS with HTTP redirect
    let https_config = armature_core::HttpsConfig::new("0.0.0.0:8443", tls_config)
        .with_http_redirect("0.0.0.0:8080");

    println!("📝 Server Configuration:");
    println!("   • HTTPS: https://localhost:8443");
    println!("   • HTTP:  http://localhost:8080 (redirects to HTTPS)");
    println!("\n🧪 Test with:");
    println!("   curl -k https://localhost:8443/api/");
    println!("   curl -L -k http://localhost:8080/api/");
    println!();

    app.listen_with_config(https_config).await?;

    Ok(())
}

#[cfg(not(feature = "self-signed-certs"))]
fn main() {
    eprintln!("❌ This example requires the 'self-signed-certs' feature");
    eprintln!("\nRun with:");
    eprintln!("  cargo run --example https_with_redirect --features self-signed-certs");
    std::process::exit(1);
}
