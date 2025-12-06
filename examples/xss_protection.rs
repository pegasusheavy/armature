use armature::prelude::*;
use armature_xss::{XssConfig, XssEncoder, XssMiddleware, XssSanitizer, XssValidator};

#[injectable]
struct ContentService {
    xss: XssMiddleware,
    sanitizer: XssSanitizer,
}

impl ContentService {
    fn new() -> Self {
        let config = XssConfig::default()
            .with_validation(true) // Enable validation
            .with_auto_sanitize(false); // Manual sanitization for demo

        Self {
            xss: XssMiddleware::new(config),
            sanitizer: XssSanitizer::new(),
        }
    }

    fn generate_demo_page(&self) -> String {
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>XSS Protection Demo</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        .form-group { margin: 20px 0; }
        label { display: block; font-weight: bold; margin-bottom: 5px; }
        input, textarea { width: 100%; padding: 10px; margin-bottom: 10px; }
        button { padding: 10px 20px; background: #007bff; color: white; border: none; cursor: pointer; }
        .result { margin: 20px 0; padding: 15px; background: #f0f0f0; border-radius: 5px; }
        .error { background: #ffeeee; color: #cc0000; }
        .success { background: #eeffee; color: #00cc00; }
    </style>
</head>
<body>
    <h1>🔒 XSS Protection Demo</h1>

    <div class="form-group">
        <label>Test XSS Validation</label>
        <textarea id="content" rows="5" placeholder="Try entering: <script>alert('XSS')</script>"></textarea>
        <button onclick="testValidation()">Validate</button>
    </div>

    <div id="result" class="result"></div>

    <h2>Common XSS Patterns (Blocked)</h2>
    <ul>
        <li><code>&lt;script&gt;alert('XSS')&lt;/script&gt;</code></li>
        <li><code>&lt;img src="x" onerror="alert('XSS')"&gt;</code></li>
        <li><code>&lt;a href="javascript:alert('XSS')"&gt;Click&lt;/a&gt;</code></li>
        <li><code>&lt;iframe src="javascript:alert('XSS')"&gt;&lt;/iframe&gt;</code></li>
    </ul>

    <script>
        async function testValidation() {
            const content = document.getElementById('content').value;
            const result = document.getElementById('result');

            try {
                const response = await fetch('/api/validate', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ content })
                });

                const data = await response.json();

                if (response.ok) {
                    result.className = 'result success';
                    result.innerHTML = `<strong>✅ Validation Passed</strong><br>` +
                        `Sanitized: <pre>${escapeHtml(data.sanitized)}</pre>`;
                } else {
                    result.className = 'result error';
                    result.innerHTML = `<strong>❌ XSS Detected</strong><br>${data.error}`;
                }
            } catch (error) {
                result.className = 'result error';
                result.innerHTML = `<strong>Error:</strong> ${error.message}`;
            }
        }

        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }
    </script>
</body>
</html>
        "#.to_string()
    }
}

#[controller("/api")]
struct ApiController {
    content_service: ContentService,
}

#[module(
    controllers = [ApiController],
    providers = [ContentService]
)]
struct AppModule;

impl ApiController {
    #[get("/")]
    async fn demo_page(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        let html = self.content_service.generate_demo_page();
        let mut response = HttpResponse::ok().with_body(html.into_bytes());

        // Add XSS protection headers
        response = self.content_service.xss.add_protection_headers(response);

        Ok(response)
    }

    #[post("/validate")]
    async fn validate_content(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // Parse request body
        let body: serde_json::Value = serde_json::from_slice(&req.body())
            .map_err(|e| Error::BadRequest(e.to_string()))?;

        let content = body
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::BadRequest("Missing 'content' field".to_string()))?;

        // Validate for XSS
        if let Some(attack_type) = XssValidator::detect_attack_type(content) {
            return Ok(HttpResponse::bad_request().with_json(serde_json::json!({
                "error": format!("XSS attack detected: {}", attack_type),
                "attack_type": attack_type
            }))?);
        }

        // Sanitize content
        let sanitized = self
            .content_service
            .sanitizer
            .sanitize(content)
            .map_err(|e| Error::Internal(e.to_string()))?;

        Ok(HttpResponse::ok().with_json(serde_json::json!({
            "status": "success",
            "original": content,
            "sanitized": sanitized
        }))?)
    }

    #[post("/encode")]
    async fn encode_content(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        let body: serde_json::Value = serde_json::from_slice(&req.body())
            .map_err(|e| Error::BadRequest(e.to_string()))?;

        let content = body
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::BadRequest("Missing 'content' field".to_string()))?;

        let context = body
            .get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("html");

        let encoded = match context {
            "html" => XssEncoder::encode_html(content),
            "javascript" | "js" => XssEncoder::encode_javascript(content),
            "url" => XssEncoder::encode_url(content),
            "css" => XssEncoder::encode_css(content),
            _ => return Err(Error::BadRequest("Invalid context".to_string())),
        };

        Ok(HttpResponse::ok().with_json(serde_json::json!({
            "context": context,
            "original": content,
            "encoded": encoded
        }))?)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Armature XSS Protection Example");
    println!("==================================\n");

    println!("XSS Protection Features:");
    println!("  ✓ HTML sanitization with ammonia");
    println!("  ✓ Context-aware encoding (HTML, JS, URL, CSS)");
    println!("  ✓ Pattern-based validation");
    println!("  ✓ Attack type detection");
    println!("  ✓ Protection headers");
    println!("\nStarting server on http://localhost:3000\n");

    println!("Try it out:");
    println!("  1. Visit http://localhost:3000/api/");
    println!("     → Interactive demo page");
    println!("\n  2. Test validation:");
    println!("     curl -X POST http://localhost:3000/api/validate \\");
    println!("       -H 'Content-Type: application/json' \\");
    println!("       -d '{{\"content\":\"<script>alert('XSS')</script>\"}}'");
    println!("     → Detects and reports XSS attack");
    println!("\n  3. Test encoding:");
    println!("     curl -X POST http://localhost:3000/api/encode \\");
    println!("       -H 'Content-Type: application/json' \\");
    println!("       -d '{{\"content\":\"<script>alert('XSS')</script>\",\"context\":\"html\"}}'");
    println!("     → Returns HTML-encoded string\n");

    let app = Application::create::<AppModule>();
    app.listen(3000).await?;

    Ok(())
}


