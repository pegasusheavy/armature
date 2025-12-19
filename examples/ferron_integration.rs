//! Ferron Integration Example
//!
//! This example demonstrates a complete setup of an Armature application
//! with Ferron reverse proxy configuration.
//!
//! ## What This Example Does
//!
//! 1. Starts an Armature HTTP API server
//! 2. Generates Ferron reverse proxy configuration
//! 3. Shows how to use service discovery for dynamic backends
//! 4. Demonstrates health check integration
//!
//! ## Running This Example
//!
//! ```bash
//! cargo run --example ferron_integration
//! ```
//!
//! Then test the API:
//! ```bash
//! curl http://localhost:<port>/api/status
//! curl http://localhost:<port>/api/users
//! curl http://localhost:<port>/health
//! ```

#[path = "common/mod.rs"]
mod common;

use armature::logging::LogConfig;
use armature::prelude::*;
use armature_ferron::{
    Backend, FerronConfig, HealthCheckConfig, LoadBalanceStrategy, LoadBalancer, Location,
    RateLimitConfig, ServiceRegistry,
};
use common::find_available_port;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// ============================================================================
// Domain Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiStatus {
    status: String,
    version: String,
    uptime_seconds: u64,
    requests_served: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HealthResponse {
    healthy: bool,
    checks: Vec<HealthCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HealthCheck {
    name: String,
    status: String,
    latency_ms: u64,
}

// ============================================================================
// Services
// ============================================================================

#[injectable]
#[derive(Clone, Debug)]
struct UserService {
    users: Arc<RwLock<Vec<User>>>,
}

impl Default for UserService {
    fn default() -> Self {
        Self::new()
    }
}

impl UserService {
    fn new() -> Self {
        let users = vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                role: "admin".to_string(),
            },
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
                role: "user".to_string(),
            },
            User {
                id: 3,
                name: "Charlie".to_string(),
                email: "charlie@example.com".to_string(),
                role: "user".to_string(),
            },
        ];
        Self {
            users: Arc::new(RwLock::new(users)),
        }
    }

    async fn get_all(&self) -> Vec<User> {
        self.users.read().await.clone()
    }

    async fn get_by_id(&self, id: u64) -> Option<User> {
        self.users.read().await.iter().find(|u| u.id == id).cloned()
    }
}

#[injectable]
#[derive(Clone, Debug)]
struct StatsService {
    start_time: std::time::Instant,
    request_count: Arc<RwLock<u64>>,
}

impl Default for StatsService {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsService {
    fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            request_count: Arc::new(RwLock::new(0)),
        }
    }

    async fn increment_requests(&self) {
        let mut count = self.request_count.write().await;
        *count += 1;
    }

    async fn get_status(&self) -> ApiStatus {
        ApiStatus {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            requests_served: *self.request_count.read().await,
        }
    }
}

// ============================================================================
// Controllers
// ============================================================================

#[controller("/api")]
#[derive(Default, Clone)]
struct ApiController;

#[routes]
impl ApiController {
    #[get("/status")]
    async fn get_status() -> Result<HttpResponse, Error> {
        let status = ApiStatus {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            uptime_seconds: 0,
            requests_served: 1,
        };
        HttpResponse::json(&status)
    }

    #[get("/users")]
    async fn get_users() -> Result<HttpResponse, Error> {
        let users = vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                role: "admin".to_string(),
            },
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
                role: "user".to_string(),
            },
            User {
                id: 3,
                name: "Charlie".to_string(),
                email: "charlie@example.com".to_string(),
                role: "user".to_string(),
            },
        ];
        HttpResponse::json(&users)
    }

    #[get("/users/:id")]
    async fn get_user(req: HttpRequest) -> Result<HttpResponse, Error> {
        let id: u64 = req
            .param("id")
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| Error::validation("Invalid user ID"))?;

        let users = vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                role: "admin".to_string(),
            },
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
                role: "user".to_string(),
            },
            User {
                id: 3,
                name: "Charlie".to_string(),
                email: "charlie@example.com".to_string(),
                role: "user".to_string(),
            },
        ];

        match users.into_iter().find(|u| u.id == id) {
            Some(user) => HttpResponse::json(&user),
            None => Err(Error::not_found(format!("User {} not found", id))),
        }
    }
}

#[controller("")]
#[derive(Default, Clone)]
struct HealthController;

#[routes]
impl HealthController {
    #[get("/health")]
    async fn health() -> Result<HttpResponse, Error> {
        let response = HealthResponse {
            healthy: true,
            checks: vec![
                HealthCheck {
                    name: "database".to_string(),
                    status: "ok".to_string(),
                    latency_ms: 5,
                },
                HealthCheck {
                    name: "cache".to_string(),
                    status: "ok".to_string(),
                    latency_ms: 1,
                },
            ],
        };
        HttpResponse::json(&response)
    }

    #[get("/ready")]
    async fn ready() -> Result<HttpResponse, Error> {
        HttpResponse::json(&serde_json::json!({
            "ready": true
        }))
    }

    #[get("/live")]
    async fn live() -> Result<HttpResponse, Error> {
        Ok(HttpResponse::ok().with_body(b"OK".to_vec()))
    }
}

// ============================================================================
// Module Configuration
// ============================================================================

#[module(
    controllers: [ApiController, HealthController]
)]
#[derive(Default)]
struct AppModule;

// ============================================================================
// Ferron Configuration Generator
// ============================================================================

fn generate_ferron_config(
    domain: &str,
    app_port: u16,
) -> std::result::Result<FerronConfig, armature_ferron::FerronError> {
    FerronConfig::builder()
        .domain(domain)
        .backend_url(format!("http://127.0.0.1:{}", app_port))
        .tls_auto(true)
        // API routes with rate limiting
        .location(
            Location::new("/api")
                .proxy(format!("http://127.0.0.1:{}/api", app_port))
                .rate_limit(RateLimitConfig::new(100).burst(200)),
        )
        // Health endpoints (no rate limit for probes)
        .location(Location::new("/health").proxy(format!("http://127.0.0.1:{}/health", app_port)))
        .location(Location::new("/ready").proxy(format!("http://127.0.0.1:{}/ready", app_port)))
        .location(Location::new("/live").proxy(format!("http://127.0.0.1:{}/live", app_port)))
        // Security headers
        .header("X-Frame-Options", "DENY")
        .header("X-Content-Type-Options", "nosniff")
        .header("X-XSS-Protection", "1; mode=block")
        .header("Referrer-Policy", "strict-origin-when-cross-origin")
        .gzip(true)
        .build()
}

fn generate_load_balanced_config(
    domain: &str,
    backend_ports: &[u16],
) -> std::result::Result<FerronConfig, armature_ferron::FerronError> {
    let mut lb = LoadBalancer::new()
        .strategy(LoadBalanceStrategy::RoundRobin)
        .health_check_interval(30)
        .health_check_path("/health")
        .health_check_threshold(3);

    for port in backend_ports {
        lb = lb.backend(Backend::new(format!("http://127.0.0.1:{}", port)));
    }

    FerronConfig::builder()
        .domain(domain)
        .load_balancer(lb)
        .tls_auto(true)
        .header("X-Frame-Options", "DENY")
        .header("X-Content-Type-Options", "nosniff")
        .gzip(true)
        .build()
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let _guard = LogConfig::default().init();

    let port = find_available_port();

    println!("🔄 Ferron Integration Example");
    println!("==============================\n");

    // 1. Generate Ferron configuration
    println!("📝 Step 1: Generate Ferron Configuration");
    println!("-----------------------------------------\n");

    let ferron_config = generate_ferron_config("api.example.com", port)?;
    println!("Single-backend configuration:\n");
    println!("{}", ferron_config.to_kdl()?);

    // 2. Generate load-balanced configuration
    println!("\n📝 Step 2: Load-Balanced Configuration");
    println!("---------------------------------------\n");

    let lb_config = generate_load_balanced_config("api.example.com", &[port, port + 1, port + 2])?;
    println!("Load-balanced configuration:\n");
    println!("{}", lb_config.to_kdl()?);

    // 3. Service Discovery Demo
    println!("\n📝 Step 3: Service Discovery");
    println!("-----------------------------\n");

    let registry = ServiceRegistry::new();

    // Register this instance
    let instance_id = registry
        .register("armature-api", &format!("http://127.0.0.1:{}", port))
        .await?;
    println!("Registered instance: {}", instance_id);

    // Simulate additional instances
    registry
        .register("armature-api", &format!("http://127.0.0.1:{}", port + 1))
        .await?;
    registry
        .register("armature-api", &format!("http://127.0.0.1:{}", port + 2))
        .await?;

    let instances = registry.get_instances("armature-api").await;
    println!("Total registered instances: {}", instances.len());

    let urls = registry.get_urls("armature-api").await;
    println!("Backend URLs: {:?}", urls);

    // 4. Health Check Configuration
    println!("\n📝 Step 4: Health Check Setup");
    println!("------------------------------\n");

    let health_config = HealthCheckConfig::new()
        .path("/health")
        .timeout(Duration::from_secs(5))
        .interval(Duration::from_secs(30))
        .unhealthy_threshold(3)
        .healthy_threshold(2);

    println!("Health check configuration:");
    println!("  Endpoint: /health");
    println!("  Timeout: {:?}", health_config.timeout);
    println!("  Interval: {:?}", health_config.interval);
    println!(
        "  Unhealthy after: {} failures",
        health_config.unhealthy_threshold
    );
    println!(
        "  Healthy after: {} successes",
        health_config.healthy_threshold
    );

    // 5. Start the Armature server
    println!("\n🚀 Step 5: Starting Armature Server");
    println!("------------------------------------\n");

    println!("Server running on http://127.0.0.1:{}", port);
    println!("\n📋 Available Endpoints:");
    println!("  GET  /api/status    - API status and statistics");
    println!("  GET  /api/users     - List all users");
    println!("  GET  /api/users/:id - Get user by ID");
    println!("  GET  /health        - Health check (for Ferron)");
    println!("  GET  /ready         - Readiness probe");
    println!("  GET  /live          - Liveness probe");

    println!("\n💡 Test Commands:");
    println!("  curl http://127.0.0.1:{}/api/status", port);
    println!("  curl http://127.0.0.1:{}/api/users", port);
    println!("  curl http://127.0.0.1:{}/api/users/1", port);
    println!("  curl http://127.0.0.1:{}/health", port);

    println!("\n📄 To use with Ferron:");
    println!("  1. Save the configuration above to /etc/ferron/ferron.conf");
    println!("  2. Start Ferron: ferron -c /etc/ferron/ferron.conf");
    println!("  3. Access via: https://api.example.com");

    println!("\nPress Ctrl+C to stop\n");

    // Start the server
    let app = Application::create::<AppModule>().await;
    app.listen(port).await?;

    Ok(())
}
