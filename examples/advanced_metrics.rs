//! Advanced Metrics Example
//!
//! This example demonstrates advanced metric features including:
//! - Metric labels
//! - Multiple metric types
//! - Custom buckets
//! - Business metrics
//!
//! Run with:
//! ```bash
//! cargo run --example advanced_metrics
//! ```

use armature_core::*;
use armature_metrics::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let _guard = LogConfig::default().init();

    info!("Advanced Metrics Example");
    info!("=========================");

    // Create metrics with labels
    info!("\nCreating labeled metrics...");

    // Counter with labels
    let http_errors = CounterVecBuilder::new("http_errors_total", "Total HTTP errors")
        .labels(&["error_type", "endpoint"])
        .register()?;

    // Gauge with labels
    let db_connections = GaugeVecBuilder::new("database_connections", "Database connections")
        .labels(&["pool", "state"])
        .register()?;

    // Histogram with labels and custom buckets
    let db_query_duration = HistogramVecBuilder::new(
        "db_query_duration_seconds",
        "Database query duration"
    )
        .labels(&["operation", "table"])
        .buckets(vec![0.0001, 0.001, 0.01, 0.1, 0.5, 1.0])
        .register()?;

    // Business metrics
    let orders_total = CounterVecBuilder::new("orders_total", "Total orders")
        .labels(&["status", "payment_method"])
        .register()?;

    let revenue_total = CounterBuilder::new("revenue_dollars_total", "Total revenue in dollars")
        .register()?;

    let cart_size = HistogramVecBuilder::new("shopping_cart_items", "Shopping cart size")
        .labels(&["user_segment"])
        .buckets(vec![1.0, 5.0, 10.0, 20.0, 50.0])
        .register()?;

    info!("✓ Registered all metrics");

    // Create router
    let mut router = Router::new();

    // Clone for handlers
    let db_connections_users = db_connections.clone();
    let db_query_duration_users = db_query_duration.clone();

    // Simulate database operations
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/users".to_string(),
        handler: Arc::new(move |_req| {
            let db_connections = db_connections_users.clone();
            let db_query_duration = db_query_duration_users.clone();

            Box::pin(async move {
                let start = std::time::Instant::now();

                // Simulate DB connection
                db_connections.with_label_values(&["default", "active"]).inc();

                // Simulate query
                tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;

                // Record query duration
                let duration = start.elapsed().as_secs_f64();
                db_query_duration.with_label_values(&["SELECT", "users"]).observe(duration);

                // Release connection
                db_connections.with_label_values(&["default", "active"]).dec();
                db_connections.with_label_values(&["default", "idle"]).inc();

                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "users": [
                        {"id": 1, "name": "Alice"},
                        {"id": 2, "name": "Bob"}
                    ]
                }))?)
            })
        }),
        constraints: None,
    });

    // Clone for order handler
    let orders_total_clone = orders_total.clone();
    let revenue_total_clone = revenue_total.clone();
    let http_errors_clone = http_errors.clone();

    // Simulate order processing
    router.add_route(Route {
        method: HttpMethod::POST,
        path: "/api/orders".to_string(),
        handler: Arc::new(move |_req| {
            let orders_total = orders_total_clone.clone();
            let revenue_total = revenue_total_clone.clone();
            let http_errors = http_errors_clone.clone();

            Box::pin(async move {
                // Simulate order processing - always succeed for demo
                let success = true;

                if success {
                    // Record successful order
                    orders_total.with_label_values(&["completed", "credit_card"]).inc();

                    // Record revenue (simulate $50 order)
                    revenue_total.inc_by(50.0);

                    Ok(HttpResponse::ok().with_json(&serde_json::json!({
                        "order_id": "ORD-12345",
                        "status": "completed",
                        "amount": 50.0
                    }))?)
                } else {
                    // Record error
                    http_errors.with_label_values(&["payment_failed", "/api/orders"]).inc();
                    orders_total.with_label_values(&["failed", "credit_card"]).inc();

                    Err(Error::BadRequest("Payment failed".to_string()))
                }
            })
        }),
        constraints: None,
    });

    // Clone for cart handler
    let cart_size_clone = cart_size.clone();

    // Shopping cart endpoint
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/cart".to_string(),
        handler: Arc::new(move |_req| {
            let cart_size = cart_size_clone.clone();

            Box::pin(async move {
                // Alternate between segments for demo
                let segment = "regular";
                let items = 5.0;

                cart_size.with_label_values(&[segment]).observe(items);

                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "items": items as u32,
                    "segment": segment
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

    // Root endpoint
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/".to_string(),
        handler: Arc::new(|_req| {
            Box::pin(async move {
                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "message": "Advanced Metrics Example",
                    "endpoints": {
                        "GET /api/users": "Query users (DB metrics)",
                        "POST /api/orders": "Create order (business metrics)",
                        "GET /api/cart": "View cart (user behavior metrics)",
                        "GET /metrics": "Prometheus metrics"
                    },
                    "metrics": {
                        "http_errors_total": "Errors by type and endpoint",
                        "database_connections": "DB connections by pool and state",
                        "db_query_duration_seconds": "Query duration by operation and table",
                        "orders_total": "Orders by status and payment method",
                        "revenue_dollars_total": "Total revenue",
                        "shopping_cart_items": "Cart size by user segment"
                    }
                }))?)
            })
        }),
        constraints: None,
    });

    // Build application
    let container = Container::new();
    let app = Application::new(container, router);

    info!("\n✓ Server started on http://localhost:3000");
    info!("\nTry these requests:");
    info!("  curl http://localhost:3000/");
    info!("  curl http://localhost:3000/api/users");
    info!("  curl -X POST http://localhost:3000/api/orders");
    info!("  curl http://localhost:3000/api/cart");
    info!("  curl http://localhost:3000/metrics");
    info!("\nPress Ctrl+C to stop");

    app.listen(3000).await?;

    Ok(())
}
