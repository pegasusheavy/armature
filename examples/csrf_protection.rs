use armature::prelude::*;
use armature_csrf::{CsrfConfig, CsrfMiddleware};

#[injectable]
struct FormService {
    csrf: CsrfMiddleware,
}

impl FormService {
    fn new() -> Self {
        let config = CsrfConfig::default()
            .with_token_ttl(3600) // 1 hour
            .with_cookie_name("_csrf")
            .with_header_name("X-CSRF-Token")
            .with_cookie_secure(true);

        Self {
            csrf: CsrfMiddleware::new(config),
        }
    }

    fn generate_form_html(&self, csrf_token: &str) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>CSRF Protected Form</title>
</head>
<body>
    <h1>Submit Form</h1>
    <form method="POST" action="/api/submit">
        <input type="hidden" name="csrf_token" value="{}" />
        <input type="text" name="username" placeholder="Username" />
        <input type="email" name="email" placeholder="Email" />
        <button type="submit">Submit</button>
    </form>
</body>
</html>
            "#,
            csrf_token
        )
    }
}

#[controller("/api")]
struct ApiController {
    form_service: FormService,
}

#[module(
    controllers = [ApiController],
    providers = [FormService]
)]
struct AppModule;

impl ApiController {
    #[get("/form")]
    async fn show_form(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        // Generate CSRF token
        let token = self
            .form_service
            .csrf
            .generate_token()
            .map_err(|e| Error::Internal(e.to_string()))?;

        // Create response with form HTML
        let html = self.form_service.generate_form_html(&token.value);
        let mut response = HttpResponse::ok().with_body(html.into_bytes());

        // Add CSRF token as cookie
        response = self
            .form_service
            .csrf
            .add_token_cookie(response, &token)
            .map_err(|e| Error::Internal(e.to_string()))?;

        Ok(response)
    }

    #[post("/submit")]
    async fn submit_form(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // Validate CSRF token
        self.form_service.csrf.validate_request(&req)?;

        // Process form submission
        println!("✅ CSRF validation passed!");
        println!("Processing form data...");

        Ok(HttpResponse::ok()
            .with_body(b"Form submitted successfully!".to_vec()))
    }

    #[get("/api")]
    async fn api_endpoint(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // API endpoints should also be protected for state-changing operations
        if req.method == HttpMethod::Post
            || req.method == HttpMethod::Put
            || req.method == HttpMethod::Delete
        {
            self.form_service.csrf.validate_request(&req)?;
        }

        Ok(HttpResponse::ok()
            .with_json(serde_json::json!({
                "status": "success",
                "message": "API endpoint protected by CSRF"
            }))
            .unwrap())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Armature CSRF Protection Example");
    println!("===================================\n");

    println!("CSRF Protection Features:");
    println!("  ✓ Token-based synchronizer pattern");
    println!("  ✓ HMAC-SHA256 signed tokens");
    println!("  ✓ Configurable token TTL");
    println!("  ✓ Cookie and header support");
    println!("  ✓ Path exclusion support");
    println!("\nStarting server on http://localhost:3000\n");

    println!("Try it out:");
    println!("  1. GET  http://localhost:3000/api/form");
    println!("     → Returns HTML form with CSRF token");
    println!("\n  2. POST http://localhost:3000/api/submit");
    println!("     → Requires valid CSRF token in cookie + form field");
    println!("\n  3. POST without token:");
    println!("     curl -X POST http://localhost:3000/api/submit");
    println!("     → Returns 403 Forbidden");
    println!("\n  4. GET the form, extract token, then POST:");
    println!("     TOKEN=$(curl -c cookies.txt http://localhost:3000/api/form | grep -oP 'value=\"\\K[^\"]+')");
    println!("     curl -b cookies.txt -X POST http://localhost:3000/api/submit -d \"csrf_token=$TOKEN\"");
    println!("     → Returns 200 OK\n");

    let app = Application::create::<AppModule>();
    app.listen(3000).await?;

    Ok(())
}

