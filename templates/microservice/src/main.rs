//! Armature Microservice Template
//!
//! A queue-connected microservice for background job processing.
//!
//! Run with: cargo run

mod config;
mod handlers;
mod jobs;

use armature::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::signal;
use tracing::{error, info, warn};

use crate::config::ServiceConfig;
use crate::handlers::JobHandlers;

// =============================================================================
// Service State
// =============================================================================

#[derive(Default)]
pub struct ServiceState {
    pub jobs_processed: AtomicU64,
    pub jobs_failed: AtomicU64,
    pub is_healthy: AtomicBool,
    pub start_time: Option<Instant>,
}

impl ServiceState {
    pub fn new() -> Self {
        Self {
            jobs_processed: AtomicU64::new(0),
            jobs_failed: AtomicU64::new(0),
            is_healthy: AtomicBool::new(true),
            start_time: Some(Instant::now()),
        }
    }

    pub fn record_success(&self) {
        self.jobs_processed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.jobs_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_unhealthy(&self) {
        self.is_healthy.store(false, Ordering::Relaxed);
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0)
    }

    pub fn stats(&self) -> ServiceStats {
        ServiceStats {
            jobs_processed: self.jobs_processed.load(Ordering::Relaxed),
            jobs_failed: self.jobs_failed.load(Ordering::Relaxed),
            is_healthy: self.is_healthy.load(Ordering::Relaxed),
            uptime_seconds: self.uptime_seconds(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ServiceStats {
    pub jobs_processed: u64,
    pub jobs_failed: u64,
    pub is_healthy: bool,
    pub uptime_seconds: u64,
}

// =============================================================================
// Health Controller
// =============================================================================

pub struct HealthController {
    state: Arc<ServiceState>,
}

impl HealthController {
    pub fn new(state: Arc<ServiceState>) -> Self {
        Self { state }
    }
}

impl Controller for HealthController {
    fn routes(&self) -> Vec<Route> {
        vec![
            Route::new(HttpMethod::GET, "/health", "health"),
            Route::new(HttpMethod::GET, "/health/live", "liveness"),
            Route::new(HttpMethod::GET, "/health/ready", "readiness"),
            Route::new(HttpMethod::GET, "/metrics", "metrics"),
        ]
    }

    fn handle(&self, route_name: &str, _request: &HttpRequest) -> HttpResponse {
        let stats = self.state.stats();

        match route_name {
            "health" => HttpResponse::json(serde_json::json!({
                "status": if stats.is_healthy { "healthy" } else { "unhealthy" },
                "version": env!("CARGO_PKG_VERSION"),
                "uptime_seconds": stats.uptime_seconds,
                "jobs_processed": stats.jobs_processed,
                "jobs_failed": stats.jobs_failed,
            })),
            "liveness" => HttpResponse::json(serde_json::json!({
                "status": "alive"
            })),
            "readiness" => {
                if stats.is_healthy {
                    HttpResponse::json(serde_json::json!({
                        "status": "ready"
                    }))
                } else {
                    HttpResponse::service_unavailable().json(serde_json::json!({
                        "status": "not_ready"
                    }))
                }
            }
            "metrics" => {
                // Prometheus-style metrics
                let metrics = format!(
                    "# HELP jobs_processed_total Total number of jobs processed\n\
                     # TYPE jobs_processed_total counter\n\
                     jobs_processed_total {}\n\
                     # HELP jobs_failed_total Total number of jobs that failed\n\
                     # TYPE jobs_failed_total counter\n\
                     jobs_failed_total {}\n\
                     # HELP uptime_seconds Service uptime in seconds\n\
                     # TYPE uptime_seconds gauge\n\
                     uptime_seconds {}\n",
                    stats.jobs_processed, stats.jobs_failed, stats.uptime_seconds
                );
                HttpResponse::ok()
                    .header("Content-Type", "text/plain; charset=utf-8")
                    .body(metrics)
            }
            _ => HttpResponse::not_found(),
        }
    }
}

// =============================================================================
// Service Module
// =============================================================================

pub struct ServiceModule {
    state: Arc<ServiceState>,
}

impl ServiceModule {
    pub fn new(state: Arc<ServiceState>) -> Self {
        Self { state }
    }
}

impl Module for ServiceModule {
    fn name(&self) -> &'static str {
        "ServiceModule"
    }

    fn providers(&self) -> Vec<Arc<dyn Provider>> {
        vec![]
    }

    fn controllers(&self) -> Vec<Box<dyn Controller>> {
        vec![Box::new(HealthController::new(self.state.clone()))]
    }
}

// =============================================================================
// Job Worker (Simulated)
// =============================================================================

async fn run_worker(state: Arc<ServiceState>, config: ServiceConfig) {
    info!("Starting job worker");

    // Simulated job processing loop
    // In production, connect to Redis/RabbitMQ/etc.
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

    loop {
        interval.tick().await;

        // Simulate processing a job
        info!("Processing job...");

        // Simulate random success/failure
        if rand_bool() {
            state.record_success();
            info!("Job completed successfully");
        } else {
            state.record_failure();
            warn!("Job failed");
        }

        // Log stats periodically
        let stats = state.stats();
        info!(
            processed = stats.jobs_processed,
            failed = stats.jobs_failed,
            "Worker stats"
        );
    }
}

fn rand_bool() -> bool {
    // Simple randomness without rand crate
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
        % 10
        > 2 // 70% success rate
}

// =============================================================================
// Graceful Shutdown
// =============================================================================

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown...");
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_target(true)
        .init();

    info!("🚀 Starting Armature Microservice");

    // Load configuration
    let config = ServiceConfig::from_env();
    info!(
        service_name = %config.service_name,
        "Configuration loaded"
    );

    // Create service state
    let state = Arc::new(ServiceState::new());

    // Create application for health endpoints
    let module = ServiceModule::new(state.clone());
    let app = Application::create(Box::new(module));

    // Start worker in background
    let worker_state = state.clone();
    let worker_config = config.clone();
    let worker_handle = tokio::spawn(async move {
        run_worker(worker_state, worker_config).await;
    });

    // Start HTTP server for health checks
    let addr = format!("{}:{}", config.host, config.port);
    info!(addr = %addr, "Health endpoint starting");

    println!("");
    println!("Available endpoints:");
    println!("  GET /health       - Service health");
    println!("  GET /health/live  - Liveness probe");
    println!("  GET /health/ready - Readiness probe");
    println!("  GET /metrics      - Prometheus metrics");
    println!("");

    // Run server with graceful shutdown
    tokio::select! {
        result = app.listen(&addr) => {
            if let Err(e) = result {
                error!(error = %e, "Server error");
            }
        }
        _ = shutdown_signal() => {
            info!("Shutting down...");
        }
    }

    // Cancel worker
    worker_handle.abort();

    // Final stats
    let final_stats = state.stats();
    info!(
        processed = final_stats.jobs_processed,
        failed = final_stats.jobs_failed,
        uptime = final_stats.uptime_seconds,
        "Final stats"
    );

    info!("Shutdown complete");
    Ok(())
}

