//! Health check controller

use crate::models::HealthResponse;
use armature::prelude::*;
use chrono::Utc;
use std::sync::OnceLock;
use std::time::Instant;

static START_TIME: OnceLock<Instant> = OnceLock::new();

pub struct HealthController;

impl Controller for HealthController {
    fn routes(&self) -> Vec<Route> {
        // Initialize start time on first route registration
        START_TIME.get_or_init(Instant::now);

        vec![
            Route::new(HttpMethod::GET, "/health", "health"),
            Route::new(HttpMethod::GET, "/health/live", "liveness"),
            Route::new(HttpMethod::GET, "/health/ready", "readiness"),
        ]
    }

    fn handle(&self, route_name: &str, _request: &HttpRequest) -> HttpResponse {
        match route_name {
            "health" => {
                let uptime = START_TIME
                    .get()
                    .map(|t| t.elapsed().as_secs())
                    .unwrap_or(0);

                let response = HealthResponse {
                    status: "healthy".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    timestamp: Utc::now(),
                    uptime_seconds: uptime,
                };

                HttpResponse::json(response)
            }
            "liveness" => HttpResponse::json(serde_json::json!({
                "status": "alive"
            })),
            "readiness" => {
                // In production, check database connectivity, etc.
                HttpResponse::json(serde_json::json!({
                    "status": "ready",
                    "checks": {
                        "database": "ok",
                        "cache": "ok"
                    }
                }))
            }
            _ => HttpResponse::not_found(),
        }
    }
}

