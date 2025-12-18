#![allow(clippy::needless_question_mark)]
//! Advanced Graceful Shutdown Example
//!
//! Demonstrates advanced shutdown features:
//! - Custom shutdown phases
//! - Multiple shutdown hooks with different priorities
//! - Connection draining with timeout
//!
//! Run with:
//! ```bash
//! cargo run --example shutdown_advanced
//! ```

use armature_core::*;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let _guard = LogConfig::default().init();

    info!("Advanced Graceful Shutdown Example");
    info!("==================================");

    // Create shutdown manager
    let shutdown_manager = Arc::new(ShutdownManager::new());

    // Configure shutdown
    shutdown_manager.set_timeout(Duration::from_secs(45)).await;

    info!("✓ Shutdown manager configured");

    // Register shutdown hooks with different purposes
    info!("\nRegistering shutdown hooks...");

    // Critical shutdown hook (database)
    shutdown_manager
        .add_hook(Box::new(|| {
            Box::pin(async {
                info!("🔴 CRITICAL: Closing database connection pool");
                tokio::time::sleep(Duration::from_millis(1000)).await;
                info!("✓ Database connections closed");
                Ok(())
            })
        }))
        .await;

    // Important shutdown hook (cache)
    shutdown_manager
        .add_hook(Box::new(|| {
            Box::pin(async {
                info!("🟠 IMPORTANT: Flushing cache to disk");
                tokio::time::sleep(Duration::from_millis(800)).await;
                info!("✓ Cache flushed");
                Ok(())
            })
        }))
        .await;

    // Normal shutdown hook (metrics)
    shutdown_manager
        .add_hook(Box::new(|| {
            Box::pin(async {
                info!("🟡 NORMAL: Sending final metrics");
                tokio::time::sleep(Duration::from_millis(500)).await;
                info!("✓ Metrics sent");
                Ok(())
            })
        }))
        .await;

    // Low priority shutdown hook (logs)
    shutdown_manager
        .add_hook(Box::new(|| {
            Box::pin(async {
                info!("🟢 LOW: Rotating log files");
                tokio::time::sleep(Duration::from_millis(300)).await;
                info!("✓ Logs rotated");
                Ok(())
            })
        }))
        .await;

    info!("✓ 4 shutdown hooks registered");

    // Get connection tracker
    let tracker = shutdown_manager.tracker().clone();

    // Create router
    let mut router = Router::new();

    // Health check endpoint
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/health".to_string(),
        handler: Arc::new(|_req| {
            Box::pin(async move {
                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "status": "healthy",
                    "checks": {
                        "database": "connected",
                        "redis": "connected"
                    }
                }))?)
            })
        }),
        constraints: None,
    });

    // Readiness check
    let shutdown_clone = shutdown_manager.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/ready".to_string(),
        handler: Arc::new(move |_req| {
            let shutdown = shutdown_clone.clone();
            Box::pin(async move {
                if shutdown.is_shutting_down() {
                    Ok(HttpResponse::new(503).with_json(&serde_json::json!({
                        "status": "shutting_down",
                        "message": "Server is shutting down"
                    }))?)
                } else {
                    Ok(HttpResponse::ok().with_json(&serde_json::json!({
                        "status": "ready",
                        "message": "Server is ready"
                    }))?)
                }
            })
        }),
        constraints: None,
    });

    // Simulate work endpoint
    let tracker_clone = tracker.clone();
    router.add_route(Route {
        method: HttpMethod::POST,
        path: "/work".to_string(),
        handler: Arc::new(move |req| {
            let tracker = tracker_clone.clone();
            Box::pin(async move {
                // Track this connection
                let guard = match tracker.increment() {
                    Some(g) => g,
                    None => {
                        return Ok(HttpResponse::new(503).with_json(&serde_json::json!({
                            "error": "Server is shutting down"
                        }))?);
                    }
                };

                // Parse duration from request
                let body_str = String::from_utf8_lossy(&req.body);
                let duration_secs: u64 = body_str.parse().unwrap_or(3);

                info!(
                    "Processing work request ({} seconds, active: {})",
                    duration_secs,
                    tracker.active_count()
                );

                // Simulate work
                tokio::time::sleep(Duration::from_secs(duration_secs)).await;

                drop(guard); // Explicitly drop to track completion

                info!("Work completed");

                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "message": "Work completed",
                    "duration": duration_secs
                }))?)
            })
        }),
        constraints: None,
    });

    // Status endpoint
    let tracker_clone2 = tracker.clone();
    let shutdown_clone2 = shutdown_manager.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/status".to_string(),
        handler: Arc::new(move |_req| {
            let tracker = tracker_clone2.clone();
            let shutdown = shutdown_clone2.clone();
            Box::pin(async move {
                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "server": {
                        "accepting": tracker.is_accepting(),
                        "shutting_down": shutdown.is_shutting_down(),
                        "active_connections": tracker.active_count()
                    },
                    "health_checks": "See /health",
                    "readiness": "See /ready"
                }))?)
            })
        }),
        constraints: None,
    });

    // Home endpoint
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/".to_string(),
        handler: Arc::new(|_req| {
            Box::pin(async move {
                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "message": "Advanced Graceful Shutdown Example",
                    "endpoints": {
                        "GET /health": "Health check",
                        "GET /ready": "Readiness check",
                        "POST /work": "Simulate work (POST duration in seconds)",
                        "GET /status": "Server status"
                    },
                    "features": {
                        "connection_draining": true,
                        "shutdown_hooks": 4,
                        "timeout": "45 seconds"
                    }
                }))?)
            })
        }),
        constraints: None,
    });

    // Build application
    let container = Container::new();
    let app = Application::new(container, router);

    // Setup signal handling with custom shutdown phases
    let shutdown_signal = shutdown_manager.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("\n🛑 Received shutdown signal (Ctrl+C)");
            }
            _ = async {
                #[cfg(unix)]
                {
                    let mut sigterm = tokio::signal::unix::signal(
                        tokio::signal::unix::SignalKind::terminate()
                    ).expect("Failed to setup SIGTERM handler");
                    sigterm.recv().await;
                }
                #[cfg(not(unix))]
                {
                    std::future::pending::<()>().await;
                }
            } => {
                info!("\n🛑 Received SIGTERM");
            }
        }

        info!("\n╔════════════════════════════════════════╗");
        info!("║   GRACEFUL SHUTDOWN SEQUENCE STARTED   ║");
        info!("╚════════════════════════════════════════╝\n");

        shutdown_signal.initiate_shutdown().await;

        info!("\n╔════════════════════════════════════════╗");
        info!("║   GRACEFUL SHUTDOWN COMPLETE           ║");
        info!("╚════════════════════════════════════════╝\n");

        std::process::exit(0);
    });

    info!("\n✓ Server started on http://localhost:3000");
    info!("\n📊 Advanced Features Enabled:");
    info!("  ✓ Connection draining with 45s timeout");
    info!("  ✓ 4 prioritized shutdown hooks");
    info!("  ✓ Graceful rejection of new connections during shutdown");
    info!("\n🧪 Test Commands:");
    info!("  # Check health");
    info!("  curl http://localhost:3000/health");
    info!("\n  # Start long work (10 seconds)");
    info!("  curl -X POST http://localhost:3000/work -d '10' &");
    info!("\n  # Trigger shutdown while work is running");
    info!("  # Press Ctrl+C");
    info!("\n  # Server will wait for work to complete before shutting down");
    info!("\nPress Ctrl+C to trigger graceful shutdown\n");

    app.listen(3000).await?;

    Ok(())
}
