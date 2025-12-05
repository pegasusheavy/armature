//! HTTPS server example with TLS
//!
//! This example demonstrates how to run an Armature application with HTTPS support.
//!
//! # Running with self-signed certificates (development only)
//!
//! ```bash
//! cargo run --example https_server --features self-signed-certs
//! ```
//!
//! # Running with your own certificates
//!
//! First, generate certificates (for testing):
//! ```bash
//! openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
//! ```
//!
//! Then run:
//! ```bash
//! cargo run --example https_server
//! ```
//!
//! Test with:
//! ```bash
//! curl -k https://localhost:8443/
//! ```

use armature::prelude::*;

#[derive(Default, Clone)]
#[injectable]
pub struct ApiService;

impl ApiService {
    pub fn get_message(&self) -> String {
        "Hello from HTTPS server!".to_string()
    }
}

#[controller("/")]
pub struct ApiController {
    api_service: ApiService,
}

impl ApiController {
    pub fn new(api_service: ApiService) -> Self {
        Self { api_service }
    }

    #[get("/")]
    pub async fn index(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::ok().with_body(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>HTTPS Server</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
        }
        .secure { color: green; }
        .info { background: #f0f0f0; padding: 15px; border-radius: 5px; }
    </style>
</head>
<body>
    <h1>🔒 HTTPS Server Running</h1>
    <p class="secure">✅ This connection is secure</p>
    <div class="info">
        <h2>Server Information</h2>
        <ul>
            <li>Protocol: HTTPS (TLS)</li>
            <li>Framework: Armature</li>
            <li>Status: Active</li>
        </ul>
    </div>
    <h2>API Endpoints</h2>
    <ul>
        <li><a href="/message">GET /message</a> - Get a message</li>
        <li><a href="/health">GET /health</a> - Health check</li>
    </ul>
</body>
</html>
            "#
            .as_bytes()
            .to_vec(),
        ))
    }

    #[get("/message")]
    pub async fn message(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        let msg = self.api_service.get_message();
        Ok(HttpResponse::ok().with_json(&serde_json::json!({
            "message": msg,
            "secure": true,
            "protocol": "HTTPS"
        }))?)
    }

    #[get("/health")]
    pub async fn health(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::ok().with_json(&serde_json::json!({
            "status": "healthy",
            "tls": "enabled"
        }))?)
    }
}

#[derive(Default)]
#[module(
    providers: [ApiService],
    controllers: [ApiController]
)]
pub struct AppModule {}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("🔒 Starting HTTPS server example\n");

    let app = Application::create::<AppModule>();

    // Option 1: Use self-signed certificate (development only)
    #[cfg(feature = "self-signed-certs")]
    {
        println!("⚠️  Using self-signed certificate (development only)");
        println!("   In production, use real certificates from a trusted CA!\n");

        let tls_config = armature_core::TlsConfig::self_signed(&["localhost", "127.0.0.1"])?;

        println!("📝 Test with: curl -k https://localhost:8443/\n");

        app.listen_https(8443, tls_config).await?;
    }

    // Option 2: Use certificate files
    #[cfg(not(feature = "self-signed-certs"))]
    {
        use std::path::Path;

        let cert_path = "cert.pem";
        let key_path = "key.pem";

        if !Path::new(cert_path).exists() || !Path::new(key_path).exists() {
            eprintln!("❌ Certificate files not found!");
            eprintln!("\nTo run this example, you need:");
            eprintln!("1. Generate certificates:");
            eprintln!(
                "   openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes\n"
            );
            eprintln!("2. Or run with self-signed certs:");
            eprintln!("   cargo run --example https_server --features self-signed-certs\n");
            std::process::exit(1);
        }

        println!("📜 Loading certificates from files...");
        let tls_config = armature_core::TlsConfig::from_pem_files(cert_path, key_path)?;

        println!("📝 Test with: curl -k https://localhost:8443/\n");

        app.listen_https(8443, tls_config).await?;
    }

    Ok(())
}
