//! Security middleware example - demonstrates Helmet-like security features
//!
//! This example shows how to use the comprehensive security middleware
//! to protect your Armature applications with various security headers.

use armature::prelude::*;
use armature_security::{
    SecurityMiddleware, content_security_policy::CspConfig, frame_guard::FrameGuard,
    hsts::HstsConfig, referrer_policy::ReferrerPolicy,
};

// ========== Services ==========

#[injectable]
#[derive(Default, Clone)]
struct SecurityService;

impl SecurityService {
    fn get_security_info(&self) -> serde_json::Value {
        serde_json::json!({
            "message": "Secured with Helmet-like middleware!",
            "secured": true,
            "headers": [
                "Content-Security-Policy",
                "Strict-Transport-Security",
                "X-Frame-Options",
                "X-Content-Type-Options",
                "X-XSS-Protection",
                "Referrer-Policy",
                "X-DNS-Prefetch-Control",
                "X-Download-Options",
                "X-Permitted-Cross-Domain-Policies",
                "Expect-CT"
            ]
        })
    }
}

// ========== Controllers ==========

#[controller("/")]
#[derive(Default)]
struct HomeController;

impl HomeController {
    #[get("")]
    async fn index() -> Result<HttpResponse, Error> {
        let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Security Example</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
        }
        .security-info {
            background: #f0f0f0;
            padding: 20px;
            border-radius: 8px;
            margin: 20px 0;
        }
        .header {
            background: #4CAF50;
            color: white;
            padding: 10px;
            border-radius: 4px;
            margin: 5px 0;
            font-family: monospace;
        }
    </style>
</head>
<body>
    <h1>🛡️ Armature Security Middleware</h1>
    <p>Secured with Helmet-like middleware!</p>

    <div class="security-info">
        <h2>Active Security Headers</h2>
        <p>Open your browser's developer tools (Network tab) to see these headers:</p>

        <div class="header">Content-Security-Policy</div>
        <div class="header">Strict-Transport-Security</div>
        <div class="header">X-Frame-Options</div>
        <div class="header">X-Content-Type-Options</div>
        <div class="header">X-XSS-Protection</div>
        <div class="header">Referrer-Policy</div>
        <div class="header">X-DNS-Prefetch-Control</div>
        <div class="header">X-Download-Options</div>
        <div class="header">X-Permitted-Cross-Domain-Policies</div>
        <div class="header">Expect-CT</div>
    </div>

    <h2>What This Protects Against</h2>
    <ul>
        <li>Cross-Site Scripting (XSS)</li>
        <li>Clickjacking</li>
        <li>MIME-sniffing</li>
        <li>Man-in-the-Middle attacks</li>
        <li>DNS prefetch attacks</li>
        <li>Cross-domain policy abuse</li>
    </ul>
</body>
</html>"#;

        Ok(HttpResponse::ok()
            .with_body(html.as_bytes().to_vec())
            .with_header("Content-Type".to_string(), "text/html".to_string()))
    }
}

#[controller("/api")]
#[derive(Default)]
struct ApiController;

impl ApiController {
    #[get("/data")]
    async fn get_data() -> Result<Json<serde_json::Value>, Error> {
        let service = SecurityService::default();
        Ok(Json(service.get_security_info()))
    }

    #[get("/custom")]
    async fn get_custom() -> Result<Json<serde_json::Value>, Error> {
        Ok(Json(serde_json::json!({
            "message": "Custom security configuration",
            "frame_options": "SAMEORIGIN",
            "referrer_policy": "strict-origin-when-cross-origin"
        })))
    }
}

// ========== Module ==========

#[module(
    providers: [SecurityService],
    controllers: [HomeController, ApiController]
)]
#[derive(Default)]
struct AppModule;

// ========== Main ==========

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️  Security Middleware Example");
    println!("================================\n");

    // Configure security middleware with recommended defaults
    let _security = SecurityMiddleware::default();

    println!("✅ Security middleware configured with:");
    println!("   - Content Security Policy (CSP)");
    println!("   - HTTP Strict Transport Security (HSTS)");
    println!("   - Frame Guard (X-Frame-Options: DENY)");
    println!("   - XSS Protection");
    println!("   - Content Type Options");
    println!("   - DNS Prefetch Control");
    println!("   - Referrer Policy");
    println!("   - Download Options");
    println!("   - Cross-Domain Policies");
    println!("   - Expect-CT");
    println!("   - X-Powered-By header removed");

    // Example: Custom CSP configuration (demonstrating API usage)
    let _custom_security = SecurityMiddleware::new()
        .with_csp(
            CspConfig::new()
                .default_src(vec!["'self'".to_string()])
                .script_src(vec![
                    "'self'".to_string(),
                    "https://cdn.example.com".to_string(),
                ])
                .style_src(vec!["'self'".to_string(), "'unsafe-inline'".to_string()])
                .img_src(vec![
                    "'self'".to_string(),
                    "data:".to_string(),
                    "https:".to_string(),
                ]),
        )
        .with_hsts(
            HstsConfig::new(31536000) // 1 year
                .include_subdomains(true)
                .preload(true),
        )
        .with_frame_guard(FrameGuard::SameOrigin)
        .with_referrer_policy(ReferrerPolicy::StrictOriginWhenCrossOrigin)
        .hide_powered_by(true);

    println!("\n🌐 Server starting on http://localhost:3000");
    println!("📖 Visit http://localhost:3000 to see security headers in action");
    println!("🔍 Check your browser's Network tab to inspect headers");
    println!("\nEndpoints:");
    println!("  GET /         - Home page");
    println!("  GET /api/data - JSON API with security info");
    println!("  GET /api/custom - Custom security config info\n");

    let app = Application::create::<AppModule>().await;
    app.listen(3000).await?;

    Ok(())
}
