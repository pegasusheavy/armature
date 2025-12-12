//! Lifecycle Hooks Example
//!
//! Demonstrates how to use lifecycle hooks in Armature framework.
//! This example shows OnModuleInit, OnModuleDestroy, OnApplicationBootstrap,
//! and OnApplicationShutdown hooks.

use armature_core::Provider;
use armature_core::lifecycle::{
    LifecycleManager, OnApplicationBootstrap, OnApplicationShutdown, OnModuleDestroy, OnModuleInit,
};
use async_trait::async_trait;
use std::sync::Arc;

// Example: Database connection service with lifecycle hooks
struct DatabaseService {
    connection_string: String,
    connected: Arc<tokio::sync::RwLock<bool>>,
}

impl Provider for DatabaseService {}

#[async_trait]
impl OnModuleInit for DatabaseService {
    async fn on_module_init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("  📊 DatabaseService: Connecting to database...");
        println!("     Connection string: {}", self.connection_string);

        // Simulate database connection
        *self.connected.write().await = true;

        println!("     ✅ Database connected!");
        Ok(())
    }
}

#[async_trait]
impl OnModuleDestroy for DatabaseService {
    async fn on_module_destroy(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("  📊 DatabaseService: Closing database connection...");

        // Simulate database disconnection
        *self.connected.write().await = false;

        println!("     ✅ Database connection closed!");
        Ok(())
    }
}

// Example: Cache service with lifecycle hooks
struct CacheService {
    redis_url: String,
    initialized: Arc<tokio::sync::RwLock<bool>>,
}

impl Provider for CacheService {}

#[async_trait]
impl OnModuleInit for CacheService {
    async fn on_module_init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("  💾 CacheService: Initializing Redis cache...");
        println!("     Redis URL: {}", self.redis_url);

        // Simulate Redis connection
        *self.initialized.write().await = true;

        println!("     ✅ Cache initialized!");
        Ok(())
    }
}

#[async_trait]
impl OnModuleDestroy for CacheService {
    async fn on_module_destroy(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("  💾 CacheService: Flushing and closing cache...");

        // Simulate cache cleanup
        *self.initialized.write().await = false;

        println!("     ✅ Cache closed!");
        Ok(())
    }
}

// Example: Logger service with application-level hooks
struct LoggerService {
    log_level: String,
}

impl Provider for LoggerService {}

#[async_trait]
impl OnApplicationBootstrap for LoggerService {
    async fn on_application_bootstrap(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("  📝 LoggerService: Application fully bootstrapped!");
        println!("     Log level: {}", self.log_level);
        println!("     ✅ Logger ready for requests");
        Ok(())
    }
}

#[async_trait]
impl OnApplicationShutdown for LoggerService {
    async fn on_application_shutdown(
        &self,
        signal: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(sig) = signal {
            println!("  📝 LoggerService: Received shutdown signal: {}", sig);
        } else {
            println!("  📝 LoggerService: Graceful shutdown initiated");
        }
        println!("     ✅ Final logs flushed");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         Armature Lifecycle Hooks Example                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Create lifecycle manager
    let lifecycle = LifecycleManager::new();

    // Create services
    let db_service = Arc::new(DatabaseService {
        connection_string: "postgresql://localhost:5432/mydb".to_string(),
        connected: Arc::new(tokio::sync::RwLock::new(false)),
    });

    let cache_service = Arc::new(CacheService {
        redis_url: "redis://localhost:6379".to_string(),
        initialized: Arc::new(tokio::sync::RwLock::new(false)),
    });

    let logger_service = Arc::new(LoggerService {
        log_level: "info".to_string(),
    });

    // Register lifecycle hooks
    println!("🔧 Registering lifecycle hooks...\n");

    lifecycle
        .register_on_init("DatabaseService".to_string(), db_service.clone())
        .await;
    lifecycle
        .register_on_destroy("DatabaseService".to_string(), db_service.clone())
        .await;

    lifecycle
        .register_on_init("CacheService".to_string(), cache_service.clone())
        .await;
    lifecycle
        .register_on_destroy("CacheService".to_string(), cache_service.clone())
        .await;

    lifecycle
        .register_on_bootstrap("LoggerService".to_string(), logger_service.clone())
        .await;
    lifecycle
        .register_on_shutdown("LoggerService".to_string(), logger_service.clone())
        .await;

    println!("✅ Hooks registered!\n");

    // Show hook counts
    let counts = lifecycle.hook_counts().await;
    println!("📊 Registered hooks:");
    println!("  - OnModuleInit: {}", counts.init);
    println!("  - OnModuleDestroy: {}", counts.destroy);
    println!("  - OnApplicationBootstrap: {}", counts.bootstrap);
    println!("  - OnApplicationShutdown: {}", counts.shutdown);
    println!();

    // Simulate application lifecycle
    println!("═══════════════════════════════════════════════════════════════");
    println!("                  APPLICATION LIFECYCLE                        ");
    println!("═══════════════════════════════════════════════════════════════\n");

    // 1. Module initialization
    println!("STEP 1: Module Initialization");
    println!("─────────────────────────────");
    lifecycle.call_module_init_hooks().await.unwrap();
    println!();

    // 2. Application bootstrap
    println!("STEP 2: Application Bootstrap");
    println!("─────────────────────────────");
    lifecycle.call_bootstrap_hooks().await.unwrap();
    println!();

    // Simulate application running
    println!("STEP 3: Application Running");
    println!("───────────────────────────");
    println!("  🌐 Application is now serving requests...");
    println!("  ⏱️  Simulating 2 seconds of runtime...\n");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 4. Graceful shutdown
    println!("STEP 4: Graceful Shutdown");
    println!("─────────────────────────");
    lifecycle
        .call_shutdown_hooks(Some("SIGTERM".to_string()))
        .await
        .unwrap();
    println!();

    // 5. Module destruction
    println!("STEP 5: Module Destruction");
    println!("──────────────────────────");
    lifecycle.call_module_destroy_hooks().await.unwrap();
    println!();

    println!("═══════════════════════════════════════════════════════════════");
    println!("✅ Lifecycle demonstration complete!");
    println!("═══════════════════════════════════════════════════════════════\n");

    Ok(())
}
