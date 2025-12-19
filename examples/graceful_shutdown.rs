#![allow(clippy::needless_question_mark)]
//! Graceful Shutdown Example
//!
//! This example demonstrates graceful shutdown with:
//! - Connection draining
//! - Shutdown hooks
//! - Signal handling (SIGTERM, SIGINT)
//!
//! Run with:
//! ```bash
//! cargo run --example graceful_shutdown
//! ```
//!
//! Test shutdown with:
//! ```bash
//! # Start the server
//! cargo run --example graceful_shutdown &
//!
//! # Make some requests
//! curl http://localhost:3000/slow  # Takes 5 seconds
//!
//! # Trigger shutdown (Ctrl+C or kill)
//! # Server will wait for in-flight requests to complete
//! ```

use armature_core::*;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let _guard = LogConfig::default().init();

    info!("Graceful Shutdown Example");
    info!("========================");

    // Create shutdown manager
    let shutdown_manager = Arc::new(ShutdownManager::new());
    info!("✓ Shutdown manager created");

    // Configure shutdown timeout
    shutdown_manager.set_timeout(Duration::from_secs(30)).await;
    info!("✓ Shutdown timeout: 30 seconds");

    // Register shutdown hooks
    info!("\nRegistering shutdown hooks...");

    shutdown_manager
        .add_hook(Box::new(|| {
            Box::pin(async {
                info!("Hook 1: Closing database connections");
                tokio::time::sleep(Duration::from_millis(500)).await;
                info!("Hook 1: Database connections closed");
                Ok(())
            })
        }))
        .await;

    shutdown_manager
        .add_hook(Box::new(|| {
            Box::pin(async {
                info!("Hook 2: Flushing cache");
                tokio::time::sleep(Duration::from_millis(300)).await;
                info!("Hook 2: Cache flushed");
                Ok(())
            })
        }))
        .await;

    shutdown_manager
        .add_hook(Box::new(|| {
            Box::pin(async {
                info!("Hook 3: Cleaning up temporary files");
                tokio::time::sleep(Duration::from_millis(200)).await;
                info!("Hook 3: Temporary files cleaned");
                Ok(())
            })
        }))
        .await;

    info!("✓ 3 shutdown hooks registered");

    // Get connection tracker
    let tracker = shutdown_manager.tracker().clone();

    // Create router
    let mut router = Router::new();

    // Fast endpoint
    let tracker_fast = tracker.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/fast".to_string(),
        handler: Arc::new(move |_req| {
            let tracker = tracker_fast.clone();
            Box::pin(async move {
                // Track this connection
                let _guard = tracker.increment();

                info!(
                    "Processing fast request (active: {})",
                    tracker.active_count()
                );

                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "message": "Fast response",
                    "active_connections": tracker.active_count()
                }))?)
            })
        }),
        constraints: None,
    });

    // Slow endpoint (simulates long-running request)
    let tracker_slow = tracker.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/slow".to_string(),
        handler: Arc::new(move |_req| {
            let tracker = tracker_slow.clone();
            Box::pin(async move {
                // Track this connection
                let _guard = tracker.increment();

                info!(
                    "Processing slow request (active: {})",
                    tracker.active_count()
                );
                info!("This will take 5 seconds...");

                // Simulate long processing
                tokio::time::sleep(Duration::from_secs(5)).await;

                info!("Slow request completed");

                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "message": "Slow response completed",
                    "duration": "5s"
                }))?)
            })
        }),
        constraints: None,
    });

    // Status endpoint
    let tracker_status = tracker.clone();
    let shutdown_status = shutdown_manager.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/status".to_string(),
        handler: Arc::new(move |_req| {
            let tracker = tracker_status.clone();
            let shutdown = shutdown_status.clone();
            Box::pin(async move {
                Ok(HttpResponse::ok().with_json(&serde_json::json!({
                    "active_connections": tracker.active_count(),
                    "accepting_connections": tracker.is_accepting(),
                    "shutting_down": shutdown.is_shutting_down()
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
                    "message": "Graceful Shutdown Example",
                    "endpoints": {
                        "GET /fast": "Fast endpoint (instant)",
                        "GET /slow": "Slow endpoint (5 seconds)",
                        "GET /status": "Server status"
                    },
                    "instructions": "Press Ctrl+C to trigger graceful shutdown"
                }))?)
            })
        }),
        constraints: None,
    });

    // Build application
    let container = Container::new();
    let app = Application::new(container, router);

    // Setup signal handling
    let shutdown_signal = shutdown_manager.clone();
    tokio::spawn(async move {
        // Wait for SIGTERM or SIGINT
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("\nReceived SIGINT (Ctrl+C)");
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
                info!("\nReceived SIGTERM");
            }
        }

        info!("Initiating graceful shutdown...");
        shutdown_signal.initiate_shutdown().await;

        info!("\n✓ Shutdown complete - exiting");
        std::process::exit(0);
    });

    info!("\n✓ Server started on http://localhost:3000");
    info!("\nFeatures:");
    info!("  - Connection draining (waits for in-flight requests)");
    info!("  - Shutdown hooks (cleanup on exit)");
    info!("  - Signal handling (SIGTERM, SIGINT)");
    info!("\nTry these requests:");
    info!("  curl http://localhost:3000/fast");
    info!("  curl http://localhost:3000/slow   # Takes 5 seconds");
    info!("  curl http://localhost:3000/status");
    info!("\nPress Ctrl+C to trigger graceful shutdown");
    info!("During shutdown, slow requests will complete before exit\n");

    app.listen(3000).await?;

    Ok(())
}
