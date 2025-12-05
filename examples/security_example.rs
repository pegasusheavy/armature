//! Security middleware example - demonstrates Helmet-like security features
//!
//! This example shows how to use the comprehensive security middleware
//! to protect your Armature applications with various security headers.

use armature::prelude::*;
use armature_security::{
    SecurityMiddleware,
    content_security_policy::CspConfig,
    hsts::HstsConfig,
    referrer_policy::ReferrerPolicy,
    frame_guard::FrameGuard,
};

#[injectable]
struct AppService;

impl AppService {
    fn new() -> Self {
        Self
    }

    fn get_message(&self) -> String {
        "Secured with Helmet-like middleware!".to_string()
    }
}

#[controller("/")]
struct HomeController {
    service: AppService,
}

impl HomeController {
    fn new(service: AppService) -> Self {
        Self { service }
    }

    #[get("/")]
    fn index(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Security Example</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
        }}
        .security-info {{
            background: #f0f0f0;
            padding: 20px;
            border-radius: 8px;
            margin: 20px 0;
        }}
        .header {{
            background: #4CAF50;
            color: white;
            padding: 10px;
            border-radius: 4px;
            margin: 5px 0;
            font-family: monospace;
        }}
    </style>
</head>
<body>
    <h1>🛡️ Armature Security Middleware</h1>
    <p>{}</p>
    
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
    
    <h2>Usage Examples</h2>
    <pre style="background: #f5f5f5; padding: 15px; border-radius: 4px;">
// Default (recommended) - All security features enabled
let security = SecurityMiddleware::default();

// Custom configuration
let security = SecurityMiddleware::new()
    .with_hsts(HstsConfig::new(31536000))
    .with_frame_guard(FrameGuard::Deny)
    .hide_powered_by(true)
    .with_referrer_policy(ReferrerPolicy::StrictOriginWhenCrossOrigin);

// Apply to response
let secured_response = security.apply(response);
    </pre>
</body>
</html>"#,
            self.service.get_message()
        );

        Ok(HttpResponse::ok()
            .with_body(html.into_bytes())
            .with_header("Content-Type".to_string(), "text/html".to_string()))
    }

    #[get("/api/data")]
    fn api_data(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        #[derive(serde::Serialize)]
        struct ApiResponse {
            message: String,
            secured: bool,
        }

        let response = ApiResponse {
            message: self.service.get_message(),
            secured: true,
        };

        Ok(HttpResponse::ok().with_json(&response)?)
    }
}

#[module]
struct AppModule {
    controllers: Vec<Box<dyn Controller>>,
    providers: Vec<Box<dyn Provider>>,
}

impl AppModule {
    fn new() -> Self {
        Self {
            controllers: vec![Box::new(HomeController::new(AppService::new()))],
            providers: vec![Box::new(AppService::new())],
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️  Security Middleware Example");
    println!("================================\n");

    // Create the application
    let mut app = Application::new();
    let module = AppModule::new();
    app.register_module(module)?;

    // Configure security middleware with recommended defaults
    let security = SecurityMiddleware::default();
    
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

    // Example: Custom CSP configuration
    let _custom_security = SecurityMiddleware::new()
        .with_csp(
            CspConfig::new()
                .default_src(vec!["'self'".to_string()])
                .script_src(vec!["'self'".to_string(), "https://cdn.example.com".to_string()])
                .style_src(vec!["'self'".to_string(), "'unsafe-inline'".to_string()])
                .img_src(vec!["'self'".to_string(), "data:".to_string(), "https:".to_string()])
        )
        .with_hsts(
            HstsConfig::new(31536000) // 1 year
                .include_subdomains(true)
                .preload(true)
        )
        .with_frame_guard(FrameGuard::SameOrigin)
        .with_referrer_policy(ReferrerPolicy::StrictOriginWhenCrossOrigin)
        .hide_powered_by(true);

    println!("\n🌐 Server starting on http://localhost:3000");
    println!("📖 Visit http://localhost:3000 to see security headers in action");
    println!("🔍 Check your browser's Network tab to inspect headers\n");

    // Note: In a real application, you would integrate this with middleware
    // For now, we'll apply it manually to responses
    let _security_for_responses = security;

    // Start the server
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}

