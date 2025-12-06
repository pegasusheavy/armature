use armature_core::*;
use armature_macro::*;
use std::sync::Arc;
use std::time::Duration;

#[injectable]
#[derive(Clone, Default)]
struct ApiService;

impl ApiService {
    fn new() -> Self {
        Self
    }
}

#[controller("/api")]
struct ApiController {
    service: ApiService,
}

impl ApiController {
    #[get("/data")]
    async fn get_data(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::ok().with_json(&serde_json::json!({
            "message": "Hello from API",
            "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        }))?)
    }
}

#[module(
    providers: [ApiService],
    controllers: [ApiController]
)]
#[derive(Default)]
struct AppModule;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🗂️  Armature Static Assets Example");
    println!("====================================\n");

    // Create a demo public directory structure
    println!("📁 Setting up demo directory structure...");
    setup_demo_directory().await?;

    let container = Container::new();
    let mut router = Router::new();

    // Example 1: Basic static asset serving
    println!("\n📦 Example 1: Basic Static Assets");
    println!("   URL: http://localhost:3000/static/");
    
    let basic_config = StaticAssetsConfig::new("demo/public")
        .with_default_strategy(CacheStrategy::Public(Duration::from_secs(3600)));
    
    let basic_server = StaticAssetServer::new(basic_config)?;
    
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/static/*".to_string(),
        handler: Arc::new(move |req| {
            let server = basic_server.clone();
            Box::pin(async move {
                let path = req.path.trim_start_matches("/static");
                server.serve(&HttpRequest::new("GET".to_string(), path.to_string())).await
            })
        }),
    });

    // Example 2: SPA mode (fallback to index.html)
    println!("📦 Example 2: SPA Mode with Fallback");
    println!("   URL: http://localhost:3000/app/");
    
    let spa_config = StaticAssetsConfig::new("demo/spa")
        .spa_mode();  // Automatically configures for SPAs
    
    let spa_server = StaticAssetServer::new(spa_config)?;
    
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/app/*".to_string(),
        handler: Arc::new(move |req| {
            let server = spa_server.clone();
            Box::pin(async move {
                let path = req.path.trim_start_matches("/app");
                let mut spa_req = HttpRequest::new("GET".to_string(), path.to_string());
                // Copy headers for conditional requests
                spa_req.headers = req.headers.clone();
                server.serve(&spa_req).await
            })
        }),
    });

    // Example 3: Maximum performance (immutable assets)
    println!("📦 Example 3: Maximum Performance Mode");
    println!("   URL: http://localhost:3000/cdn/");
    
    let cdn_config = StaticAssetsConfig::new("demo/cdn")
        .max_performance()  // Aggressive caching for CDN-like behavior
        .with_cors_origin("*");
    
    let cdn_server = StaticAssetServer::new(cdn_config)?;
    
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/cdn/*".to_string(),
        handler: Arc::new(move |req| {
            let server = cdn_server.clone();
            Box::pin(async move {
                let path = req.path.trim_start_matches("/cdn");
                let mut cdn_req = HttpRequest::new("GET".to_string(), path.to_string());
                cdn_req.headers = req.headers.clone();
                server.serve(&cdn_req).await
            })
        }),
    });

    // Example 4: Development mode (no caching)
    println!("📦 Example 4: Development Mode (No Caching)");
    println!("   URL: http://localhost:3000/dev/");
    
    let dev_config = StaticAssetsConfig::new("demo/dev")
        .development();  // No caching for development
    
    let dev_server = StaticAssetServer::new(dev_config)?;
    
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/dev/*".to_string(),
        handler: Arc::new(move |req| {
            let server = dev_server.clone();
            Box::pin(async move {
                let path = req.path.trim_start_matches("/dev");
                server.serve(&HttpRequest::new("GET".to_string(), path.to_string())).await
            })
        }),
    });

    // Example 5: Custom per-filetype caching
    println!("📦 Example 5: Custom Per-FileType Caching");
    println!("   URL: http://localhost:3000/custom/");
    
    let custom_config = StaticAssetsConfig::new("demo/custom")
        .with_type_strategy(FileType::JavaScript, CacheStrategy::Immutable)
        .with_type_strategy(FileType::Stylesheet, CacheStrategy::Immutable)
        .with_type_strategy(FileType::Image, CacheStrategy::Public(Duration::from_secs(86400)))
        .with_type_strategy(FileType::Html, CacheStrategy::NoCache)
        .with_etag(true)
        .with_last_modified(true);
    
    let custom_server = StaticAssetServer::new(custom_config)?;
    
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/custom/*".to_string(),
        handler: Arc::new(move |req| {
            let server = custom_server.clone();
            Box::pin(async move {
                let path = req.path.trim_start_matches("/custom");
                let mut custom_req = HttpRequest::new("GET".to_string(), path.to_string());
                custom_req.headers = req.headers.clone();
                server.serve(&custom_req).await
            })
        }),
    });

    // API route for comparison
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/data".to_string(),
        handler: Arc::new(|_req| {
            Box::pin(async {
                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "message": "This is an API endpoint, not a static asset",
                    "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                }))?)
            })
        }),
    });

    // Info page
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/".to_string(),
        handler: Arc::new(|_req| {
            Box::pin(async {
                let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Static Assets Demo</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 1200px; margin: 50px auto; padding: 20px; }
        h1 { color: #333; }
        .example { background: #f5f5f5; padding: 20px; margin: 20px 0; border-radius: 5px; }
        .example h2 { margin-top: 0; color: #007bff; }
        .example ul { margin: 10px 0; }
        .example li { margin: 5px 0; }
        code { background: #e0e0e0; padding: 2px 6px; border-radius: 3px; }
        .cache-info { font-size: 0.9em; color: #666; }
    </style>
</head>
<body>
    <h1>🗂️ Armature Static Assets Demo</h1>
    <p>This example demonstrates different static asset serving strategies with configurable caching.</p>

    <div class="example">
        <h2>📦 Example 1: Basic Static Assets</h2>
        <p><strong>URL:</strong> <code>/static/*</code></p>
        <p class="cache-info">Cache: Public, 1 hour max-age</p>
        <ul>
            <li><a href="/static/index.html">index.html</a> - Main page</li>
            <li><a href="/static/styles.css">styles.css</a> - Stylesheet</li>
            <li><a href="/static/script.js">script.js</a> - JavaScript</li>
            <li><a href="/static/logo.svg">logo.svg</a> - Image</li>
        </ul>
    </div>

    <div class="example">
        <h2>📦 Example 2: SPA Mode</h2>
        <p><strong>URL:</strong> <code>/app/*</code></p>
        <p class="cache-info">Cache: Fallback to index.html, immutable JS/CSS, no-cache HTML</p>
        <ul>
            <li><a href="/app/">Root</a> → Falls back to index.html</li>
            <li><a href="/app/dashboard">Dashboard route</a> → Falls back to index.html</li>
            <li><a href="/app/unknown-route">Unknown route</a> → Falls back to index.html</li>
        </ul>
    </div>

    <div class="example">
        <h2>📦 Example 3: CDN Mode (Max Performance)</h2>
        <p><strong>URL:</strong> <code>/cdn/*</code></p>
        <p class="cache-info">Cache: Immutable, 1 year max-age, CORS enabled</p>
        <ul>
            <li><a href="/cdn/bundle.123456.js">bundle.123456.js</a> - Hashed JS</li>
            <li><a href="/cdn/styles.abc123.css">styles.abc123.css</a> - Hashed CSS</li>
        </ul>
    </div>

    <div class="example">
        <h2>📦 Example 4: Development Mode</h2>
        <p><strong>URL:</strong> <code>/dev/*</code></p>
        <p class="cache-info">Cache: No caching, no ETags</p>
        <ul>
            <li><a href="/dev/test.js">test.js</a> - Always fresh</li>
            <li><a href="/dev/test.css">test.css</a> - Always fresh</li>
        </ul>
    </div>

    <div class="example">
        <h2>📦 Example 5: Custom Caching</h2>
        <p><strong>URL:</strong> <code>/custom/*</code></p>
        <p class="cache-info">Cache: Mixed strategies, ETags enabled</p>
        <ul>
            <li>JS/CSS: Immutable</li>
            <li>Images: 24 hours</li>
            <li>HTML: No cache</li>
        </ul>
    </div>

    <div class="example">
        <h2>🔍 Testing Cache Headers</h2>
        <p>Use browser DevTools (Network tab) or curl to inspect cache headers:</p>
        <pre><code>curl -I http://localhost:3000/static/script.js
curl -I http://localhost:3000/cdn/bundle.123456.js</code></pre>
    </div>

    <div class="example">
        <h2>🎯 Key Features</h2>
        <ul>
            <li>✅ Multiple cache strategies (NoCache, Public, Private, Immutable)</li>
            <li>✅ Per-filetype cache configuration</li>
            <li>✅ ETag support for conditional requests</li>
            <li>✅ Last-Modified headers</li>
            <li>✅ 304 Not Modified responses</li>
            <li>✅ SPA fallback routing</li>
            <li>✅ CORS support</li>
            <li>✅ Path traversal prevention</li>
            <li>✅ Automatic Content-Type detection</li>
        </ul>
    </div>
</body>
</html>
                "#;
                Ok(HttpResponse::ok()
                    .with_header("Content-Type".to_string(), "text/html".to_string())
                    .with_body(html.as_bytes().to_vec()))
            })
        }),
    });

    println!("\n🚀 Server starting on http://localhost:3000");
    println!("\nExamples:");
    println!("  • http://localhost:3000/              → Info page");
    println!("  • http://localhost:3000/static/       → Basic static files");
    println!("  • http://localhost:3000/app/          → SPA mode");
    println!("  • http://localhost:3000/cdn/          → CDN mode");
    println!("  • http://localhost:3000/dev/          → Dev mode");
    println!("  • http://localhost:3000/custom/       → Custom caching");
    println!("  • http://localhost:3000/api/data      → API endpoint\n");

    let app = Application {
        router: Arc::new(router),
        container,
    };

    app.listen(3000).await?;

    Ok(())
}

/// Setup demo directory structure with sample files
async fn setup_demo_directory() -> std::io::Result<()> {
    use tokio::fs;

    // Create directories
    fs::create_dir_all("demo/public").await?;
    fs::create_dir_all("demo/spa").await?;
    fs::create_dir_all("demo/cdn").await?;
    fs::create_dir_all("demo/dev").await?;
    fs::create_dir_all("demo/custom").await?;

    // Sample files for basic static serving
    fs::write("demo/public/index.html", "<html><body><h1>Static Index</h1></body></html>").await?;
    fs::write("demo/public/styles.css", "body { margin: 0; padding: 20px; }").await?;
    fs::write("demo/public/script.js", "console.log('Static script loaded');").await?;
    fs::write("demo/public/logo.svg", "<svg></svg>").await?;

    // SPA files
    fs::write("demo/spa/index.html", "<html><body><div id=\"app\">SPA Root</div></body></html>").await?;

    // CDN files (with hashed names)
    fs::write("demo/cdn/bundle.123456.js", "// Hashed bundle").await?;
    fs::write("demo/cdn/styles.abc123.css", "/* Hashed styles */").await?;

    // Dev files
    fs::write("demo/dev/test.js", "// Development file").await?;
    fs::write("demo/dev/test.css", "/* Development styles */").await?;

    // Custom files
    fs::write("demo/custom/index.html", "<html><body><h1>Custom</h1></body></html>").await?;
    fs::write("demo/custom/app.js", "// Custom JS").await?;

    println!("✅ Demo files created");

    Ok(())
}

