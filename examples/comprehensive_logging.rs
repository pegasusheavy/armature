//! Comprehensive Logging Example
//!
//! Demonstrates all features of Armature's logging system including:
//! - Multiple log formats (JSON, Pretty, Plain, Compact)
//! - Multiple outputs (STDOUT, STDERR, File, Rolling File)
//! - Log levels (TRACE, DEBUG, INFO, WARN, ERROR)
//! - Structured logging with context
//! - Integration with Application

use armature_core::*;

// ============================================================================
// Main Function - Demonstrates Different Logging Configurations
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                                                              ║");
    println!("║          Armature Comprehensive Logging Example             ║");
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // ========================================================================
    // DEMO 1: JSON Logging (Default)
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  DEMO 1: JSON Logging (Default Configuration)                ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Initializing JSON logging to STDOUT...\n");

    {
        // Initialize default JSON logging
        let _guard = LogConfig::default().init();

        info!("Application started with JSON logging");
        debug!("Debug message - will not appear (INFO level)");
        warn!("Warning message example");
        error!("Error message example");

        // Structured logging
        info!(
            user_id = 123,
            action = "login",
            ip_address = "192.168.1.1",
            "User authentication successful"
        );

        println!("\n");
    }

    // ========================================================================
    // DEMO 2: Pretty Logging (Development)
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  DEMO 2: Pretty Logging (Development Mode)                   ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Switching to pretty logging with colors...\n");

    {
        let _guard = LogConfig::new()
            .level(LogLevel::Debug)
            .format(LogFormat::Pretty)
            .with_colors(true)
            .with_targets(true)
            .with_file_line(true)
            .init();

        info!("Application started with pretty logging");
        debug!("This debug message IS visible now");
        warn!("Warning with pretty colors");

        info!(
            module = "auth",
            function = "validate_token",
            "Token validation in progress"
        );

        println!("\n");
    }

    // ========================================================================
    // DEMO 3: File Logging
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  DEMO 3: File Logging                                        ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Writing logs to 'app.log'...");

    {
        let _guard = LogConfig::new()
            .format(LogFormat::Json)
            .output(LogOutput::File("app.log".to_string()))
            .init();

        info!("This message goes to app.log");
        warn!("Warning message in file");
        error!("Error message in file");

        info!(
            event = "file_logging_test",
            file = "app.log",
            "File logging demonstration"
        );

        println!("✅ Logs written to 'app.log'\n");
    }

    // ========================================================================
    // DEMO 4: Rolling File Logging
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  DEMO 4: Rolling File Logging                                ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Creating logs directory and writing with daily rotation...");

    {
        std::fs::create_dir_all("logs").ok();

        let _guard = LogConfig::new()
            .format(LogFormat::Json)
            .output(LogOutput::RollingFile {
                directory: "logs".to_string(),
                prefix: "armature".to_string(),
                rotation: Rotation::Daily,
            })
            .init();

        info!("Application started with rolling file logging");
        
        for i in 0..5 {
            info!(iteration = i, "Processing batch");
        }

        println!("✅ Logs written to 'logs/armature.<date>'\n");
    }

    // ========================================================================
    // DEMO 5: HTTP Request Logging Simulation
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  DEMO 5: HTTP Request Logging                                ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Simulating HTTP request logging...\n");

    {
        // Initialize JSON logging for server
        let _guard = LogConfig::new()
            .level(LogLevel::Info)
            .format(LogFormat::Json)
            .init();

        info!("Armature HTTP server starting");
        info!(
            port = 3000,
            routes = 4,
            "Server configuration loaded"
        );

        println!("\nSimulating HTTP requests:\n");

        // Simulate GET request
        info!(
            method = "GET",
            path = "/api/users",
            status = 200,
            duration_ms = 45,
            response_size = 256,
            "HTTP request completed"
        );

        // Simulate POST request
        info!(
            method = "POST",
            path = "/api/users",
            status = 201,
            duration_ms = 123,
            body_size = 58,
            user_id = 4,
            "User created via API"
        );

        // Simulate error
        error!(
            method = "GET",
            path = "/api/error",
            status = 500,
            duration_ms = 12,
            error = "Database connection failed",
            "Request failed"
        );

        // Complex operation
        info!(
            method = "GET",
            path = "/api/analytics",
            status = 200,
            duration_ms = 350,
            database_queries = 3,
            cache_hits = 2,
            cache_misses = 1,
            "Analytics request completed"
        );

        println!("\n");
    }

    // ========================================================================
    // DEMO 6: Different Log Levels
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  DEMO 6: Log Level Filtering                                 ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Demonstrating TRACE level (most verbose)...\n");

    {
        let _guard = LogConfig::new()
            .level(LogLevel::Trace)
            .format(LogFormat::Compact)
            .init();

        trace!("Very detailed trace information");
        debug!("Debug information for development");
        info!("General information");
        warn!("Warning about potential issues");
        error!("Error that needs attention");

        println!("\n");
    }

    // ========================================================================
    // DEMO 7: Custom Environment Filter
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("  DEMO 7: Custom Environment Filter                           ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("Using custom filter: armature=debug,hyper=warn...\n");

    {
        let _guard = LogConfig::new()
            .format(LogFormat::Plain)
            .with_env_filter("armature=debug,hyper=warn")
            .init();

        info!("This shows: armature at debug level");
        debug!("This also shows: armature debug messages");
        // hyper logs would be filtered to warn level only

        println!("\n");
    }

    // ========================================================================
    // Summary
    // ========================================================================

    println!("═══════════════════════════════════════════════════════════════");
    println!("                         SUMMARY                               ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("✅ Demonstrated Features:");
    println!("   • JSON logging (default)");
    println!("   • Pretty logging (development)");
    println!("   • Plain and compact formats");
    println!("   • File logging");
    println!("   • Rolling file logging");
    println!("   • HTTP request logging");
    println!("   • Multiple log levels (TRACE to ERROR)");
    println!("   • Structured logging with context fields");
    println!("   • Custom environment filters");
    println!();

    println!("📖 Key Features:");
    println!("   • Highly configurable");
    println!("   • Multiple output formats");
    println!("   • Multiple output destinations");
    println!("   • Structured logging support");
    println!("   • Low overhead");
    println!("   • Production-ready");
    println!();

    println!("═══════════════════════════════════════════════════════════════");
    println!("✅ Logging demonstration complete!");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Cleanup
    std::fs::remove_file("app.log").ok();
    std::fs::remove_dir_all("logs").ok();

    Ok(())
}

