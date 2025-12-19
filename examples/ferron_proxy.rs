//! Ferron Reverse Proxy Integration Example
//!
//! This example demonstrates how to integrate Ferron as a reverse proxy
//! in front of an Armature application for production deployments.
//!
//! ## Features Demonstrated
//!
//! - Generating Ferron configuration from Armature routes
//! - Load balancing across multiple backend instances
//! - Dynamic service discovery
//! - Health checking
//! - Process management
//!
//! ## Running This Example
//!
//! Note: Ferron must be installed on your system for process management.
//!
//! ```bash
//! cargo run --example ferron_proxy
//! ```

use armature_ferron::{
    Backend, FerronConfig, FerronManager, HealthCheckConfig, LoadBalanceStrategy, LoadBalancer,
    Location, ProcessConfig, ProxyRoute, RateLimitConfig, ServiceRegistry,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Ferron Reverse Proxy Integration Example");
    println!("============================================\n");

    // Example 1: Simple Reverse Proxy Configuration
    println!("📝 Example 1: Simple Reverse Proxy");
    println!("-----------------------------------");

    let simple_config = FerronConfig::builder()
        .domain("api.example.com")
        .backend_url("http://localhost:3000")
        .tls_auto(true)
        .gzip(true)
        .header("X-Powered-By", "Armature")
        .build()?;

    println!("Generated configuration:\n");
    println!("{}", simple_config.to_kdl()?);

    // Example 2: Load Balanced Configuration
    println!("\n📝 Example 2: Load Balanced Configuration");
    println!("------------------------------------------");

    let lb_config = FerronConfig::builder()
        .domain("api.example.com")
        .load_balancer(
            LoadBalancer::new()
                .strategy(LoadBalanceStrategy::RoundRobin)
                .backend(Backend::new("http://backend1:3000").weight(3))
                .backend(Backend::new("http://backend2:3000").weight(2))
                .backend(Backend::new("http://backend3:3000").weight(1).backup())
                .health_check_interval(30)
                .health_check_path("/health")
                .health_check_threshold(3),
        )
        .tls_auto(true)
        .gzip(true)
        .build()?;

    println!("Generated configuration:\n");
    println!("{}", lb_config.to_kdl()?);

    // Example 3: Path-based Routing
    println!("\n📝 Example 3: Path-based Routing");
    println!("---------------------------------");

    let path_config = FerronConfig::builder()
        .domain("example.com")
        .backend_url("http://localhost:3000")
        // API routes with rate limiting
        .location(
            Location::new("/api")
                .remove_base(true)
                .proxy("http://localhost:3000/api")
                .rate_limit(RateLimitConfig::new(100).burst(200)),
        )
        // Static files
        .location(Location::new("/static").root("/var/www/static"))
        // WebSocket endpoint
        .route(
            ProxyRoute::new("/ws", "http://localhost:3000/ws")
                .websocket()
                .timeout(300),
        )
        .tls_auto(true)
        .build()?;

    println!("Generated configuration:\n");
    println!("{}", path_config.to_kdl()?);

    // Example 4: Service Discovery
    println!("\n📝 Example 4: Service Discovery");
    println!("--------------------------------");

    let registry = ServiceRegistry::new();

    // Register service instances
    let id1 = registry
        .register("api-service", "http://localhost:3001")
        .await?;
    let id2 = registry
        .register("api-service", "http://localhost:3002")
        .await?;
    let _id3 = registry
        .register("api-service", "http://localhost:3003")
        .await?;

    println!("Registered instances: {}, {}, ...", id1, id2);

    // Get all instances
    let instances = registry.get_instances("api-service").await;
    println!("Total instances: {}", instances.len());

    // Mark one as unhealthy
    registry.mark_unhealthy("api-service", &id1).await?;

    // Get only healthy instances
    let healthy = registry.get_healthy_urls("api-service").await;
    println!("Healthy URLs: {:?}", healthy);

    // Get registry stats
    let stats = registry.stats().await;
    println!(
        "Registry stats: {} services, {} total, {} healthy",
        stats.service_count, stats.total_instances, stats.healthy_instances
    );

    // Example 5: Health Checking
    println!("\n📝 Example 5: Health Check Configuration");
    println!("-----------------------------------------");

    let health_config = HealthCheckConfig::new()
        .path("/health")
        .method("GET")
        .timeout(Duration::from_secs(5))
        .interval(Duration::from_secs(30))
        .unhealthy_threshold(3)
        .healthy_threshold(2)
        .header("User-Agent", "Armature-HealthCheck/1.0");

    println!("Health check configuration:");
    println!("  Path: {}", health_config.path);
    println!("  Method: {}", health_config.method);
    println!("  Timeout: {:?}", health_config.timeout);
    println!("  Interval: {:?}", health_config.interval);
    println!(
        "  Unhealthy threshold: {}",
        health_config.unhealthy_threshold
    );
    println!("  Healthy threshold: {}", health_config.healthy_threshold);

    // Example 6: Armature App Configuration Helper
    println!("\n📝 Example 6: Armature App Configuration");
    println!("-----------------------------------------");

    let armature_config =
        armature_ferron::manager::helpers::armature_app_config("api.myapp.com", 3000)?;

    println!("Generated Armature-optimized configuration:\n");
    println!("{}", armature_config.to_kdl()?);

    // Example 7: Process Configuration (requires Ferron installed)
    println!("\n📝 Example 7: Process Configuration");
    println!("------------------------------------");

    let process_config = ProcessConfig::new("/usr/bin/ferron", "/etc/ferron/ferron.conf")
        .working_dir("/var/www")
        .arg("--verbose")
        .env("RUST_LOG", "debug")
        .auto_restart(true)
        .max_restarts(5)
        .restart_delay(2000);

    println!("Process configuration:");
    println!("  Binary: {:?}", process_config.binary_path);
    println!("  Config: {:?}", process_config.config_path);
    println!("  Working dir: {:?}", process_config.working_dir);
    println!("  Auto restart: {}", process_config.auto_restart);
    println!("  Max restarts: {}", process_config.max_restarts);

    // Example 8: Manager with Service Discovery
    println!("\n📝 Example 8: Full Manager Setup");
    println!("---------------------------------");

    // Create a configuration
    let base_config = FerronConfig::builder()
        .domain("api.example.com")
        .backend_url("http://localhost:3000")
        .tls_auto(true)
        .build()?;

    // Create manager with all features (without starting Ferron)
    let _manager = FerronManager::builder()
        .binary_path("/usr/bin/ferron")
        .config_path("/tmp/ferron_example.conf")
        .config(base_config)
        .service_registry(ServiceRegistry::new())
        .health_check(HealthCheckConfig::default())
        .auto_reload(true)
        .auto_restart(true)
        .build()?;

    println!("Manager created successfully!");
    println!("  - Service discovery: enabled");
    println!("  - Health checking: enabled");
    println!("  - Auto reload: enabled");
    println!("  - Auto restart: enabled");

    // Note: We don't start the manager since Ferron may not be installed
    // In production, you would call: manager.start().await?

    println!("\n✅ All examples completed successfully!");
    println!("\n💡 To use in production:");
    println!("   1. Install Ferron: https://ferron.sh");
    println!("   2. Generate config using FerronConfig::builder()");
    println!("   3. Use FerronManager for process lifecycle");
    println!("   4. Enable service discovery for dynamic backends");
    println!("   5. Enable health checking for reliability");

    Ok(())
}
