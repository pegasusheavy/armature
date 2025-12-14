//! Prometheus Metrics Example
//!
//! This example demonstrates how to use Armature's metrics module
//! to collect and expose Prometheus metrics.
//!
//! Run with:
//! ```bash
//! cargo run --example metrics_example
//! ```
//!
//! Test with:
//! ```bash
//! # Make some requests
//! curl http://localhost:3000/
//! curl http://localhost:3000/api/users
//! curl http://localhost:3000/api/posts
//!
//! # View metrics
//! curl http://localhost:3000/metrics
//! ```

use armature_core::*;
use armature_metrics::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let _guard = LogConfig::default().init();

    info!("Prometheus Metrics Example");
    info!("===========================");

    // Create custom business metrics
    info!("\nCreating custom metrics...");

    let page_views = CounterBuilder::new("page_views_total", "Total page views")
        .register()?;

    let active_users = GaugeBuilder::new("active_users", "Number of active users")
        .register()?;

    let api_latency = HistogramBuilder::new("api_latency_seconds", "API call latency")
        .latency_buckets()
        .register()?;

    info!("✓ Registered custom metrics");

    // Create router
    let mut router = Router::new();

    // Home endpoint
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/".to_string(),
        handler: Arc::new(move |_req| {
            let page_views = page_views.clone();
            Box::pin(async move {
                // Increment page views counter
                page_views.inc();

                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "message": "Metrics Example API",
                    "endpoints": {
                        "/": "Home",
                        "/api/users": "List users",
                        "/api/posts": "List posts",
                        "/metrics": "Prometheus metrics"
                    }
                }))?)
            })
        }),
        constraints: None,
    });

    // Users endpoint
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/users".to_string(),
        handler: Arc::new(move |_req| {
            let active_users = active_users.clone();
            let api_latency = api_latency.clone();

            Box::pin(async move {
                let start = std::time::Instant::now();

                // Simulate some work
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

                // Update active users gauge
                active_users.inc();

                // Record API latency
                let duration = start.elapsed().as_secs_f64();
                api_latency.observe(duration);

                // Decrease active users after response
                active_users.dec();

                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "users": [
                        {"id": 1, "name": "Alice"},
                        {"id": 2, "name": "Bob"},
                        {"id": 3, "name": "Charlie"}
                    ]
                }))?)
            })
        }),
        constraints: None,
    });

    // Posts endpoint
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/posts".to_string(),
        handler: Arc::new(|_req| {
            Box::pin(async move {
                // Simulate some work
                tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;

                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "posts": [
                        {"id": 1, "title": "First Post"},
                        {"id": 2, "title": "Second Post"}
                    ]
                }))?)
            })
        }),
        constraints: None,
    });

    // Metrics endpoint
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/metrics".to_string(),
        handler: create_metrics_handler(),
        constraints: None,
    });

    // Add request metrics middleware
    info!("Adding request metrics middleware...");
    let _metrics_middleware = Arc::new(RequestMetricsMiddleware::new());

    // Build application
    let container = Container::new();
    let app = Application::new(container, router);

    info!("\n✓ Server started on http://localhost:3000");
    info!("\nAvailable endpoints:");
    info!("  GET http://localhost:3000/");
    info!("  GET http://localhost:3000/api/users");
    info!("  GET http://localhost:3000/api/posts");
    info!("  GET http://localhost:3000/metrics  ← Prometheus metrics");
    info!("\nMetrics being collected:");
    info!("  - page_views_total (Counter)");
    info!("  - active_users (Gauge)");
    info!("  - api_latency_seconds (Histogram)");
    info!("  - http_requests_total (Counter)");
    info!("  - http_request_duration_seconds (Histogram)");
    info!("  - http_requests_in_flight (Gauge)");
    info!("  - http_request_size_bytes (Histogram)");
    info!("  - http_response_size_bytes (Histogram)");
    info!("\nPress Ctrl+C to stop");

    app.listen(3000).await?;

    Ok(())
}

