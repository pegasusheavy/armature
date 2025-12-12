// Configuration management example

use armature::prelude::*;
use armature_config::{ConfigService, FileFormat, Validate};
use serde::{Deserialize, Serialize};

// ========== Configuration Structures ==========

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AppConfig {
    app: ApplicationConfig,
    database: DatabaseConfig,
    server: ServerConfig,
}

impl Validate for AppConfig {
    fn validate(&self) -> armature_config::Result<()> {
        self.app.validate()?;
        self.database.validate()?;
        self.server.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ApplicationConfig {
    name: String,
    version: String,
    environment: String,
}

impl Validate for ApplicationConfig {
    fn validate(&self) -> armature_config::Result<()> {
        armature_config::ConfigValidator::not_empty(&self.name, "app.name")?;
        armature_config::ConfigValidator::one_of(
            &self.environment.as_str(),
            &["development", "staging", "production"],
            "app.environment",
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

impl Validate for DatabaseConfig {
    fn validate(&self) -> armature_config::Result<()> {
        armature_config::ConfigValidator::not_empty(&self.host, "database.host")?;
        armature_config::ConfigValidator::is_port(self.port, "database.port")?;
        armature_config::ConfigValidator::not_empty(&self.database, "database.database")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ServerConfig {
    host: String,
    port: u16,
    cors_enabled: bool,
}

impl Validate for ServerConfig {
    fn validate(&self) -> armature_config::Result<()> {
        armature_config::ConfigValidator::not_empty(&self.host, "server.host")?;
        armature_config::ConfigValidator::is_port(self.port, "server.port")?;
        Ok(())
    }
}

// ========== Services ==========

#[injectable]
#[derive(Clone)]
struct AppService {
    config: ConfigService,
}

impl Default for AppService {
    fn default() -> Self {
        Self {
            config: ConfigService::default(),
        }
    }
}

impl AppService {
    fn get_app_info(&self) -> Result<serde_json::Value, Error> {
        let app_name = self
            .config
            .get_string("app.name")
            .unwrap_or_else(|_| "Unknown".to_string());
        let app_version = self
            .config
            .get_string("app.version")
            .unwrap_or_else(|_| "0.0.0".to_string());
        let environment = self
            .config
            .get_string("app.environment")
            .unwrap_or_else(|_| "development".to_string());

        Ok(serde_json::json!({
            "name": app_name,
            "version": app_version,
            "environment": environment
        }))
    }

    fn get_database_config(&self) -> Result<serde_json::Value, Error> {
        let host = self
            .config
            .get_string("database.host")
            .unwrap_or_else(|_| "localhost".to_string());
        let port = self.config.get_int("database.port").unwrap_or(5432);

        Ok(serde_json::json!({
            "host": host,
            "port": port
        }))
    }
}

// ========== Controllers ==========

#[controller("/config")]
#[derive(Default, Clone)]
struct ConfigController {
    app_service: AppService,
}

impl ConfigController {
    fn get_info(&self) -> Result<Json<serde_json::Value>, Error> {
        let info = self.app_service.get_app_info()?;
        Ok(Json(info))
    }

    fn get_database_info(&self) -> Result<Json<serde_json::Value>, Error> {
        let db_config = self.app_service.get_database_config()?;
        Ok(Json(db_config))
    }
}

// ========== Module ==========

#[module(
    providers: [AppService],
    controllers: [ConfigController]
)]
#[derive(Default)]
struct AppModule;

// ========== Main ==========

#[tokio::main]
async fn main() {
    println!("⚙️  Armature Configuration Example");
    println!("=================================\n");

    // Create configuration service with builder
    let config_service = create_config_service();

    // Display loaded configuration
    display_configuration(&config_service);

    // Create application
    let app = create_app_with_config(config_service);

    println!("\nAvailable endpoints:");
    println!("  GET /config/info     - Application info");
    println!("  GET /config/database - Database config");
    println!();

    if let Err(e) = app.listen(3008).await {
        eprintln!("Server error: {}", e);
    }
}

fn create_config_service() -> ConfigService {
    println!("📋 Loading configuration...\n");

    // Build configuration service
    let config_service = ConfigService::builder()
        .with_prefix("APP".to_string())
        .load_env()
        .load_dotenv(Some(".env.example".to_string()))
        .build()
        .unwrap_or_else(|_| {
            // If loading fails, create with defaults
            let service = ConfigService::new();
            set_defaults(&service);
            service
        });

    // If no config loaded, set some defaults
    if !config_service.has("app.name") {
        set_defaults(&config_service);
    }

    config_service
}

fn set_defaults(config: &ConfigService) {
    let manager = config.manager();

    // Application defaults
    let _ = manager.set("app.name", "Armature Config Example");
    let _ = manager.set("app.version", "0.1.0");
    let _ = manager.set("app.environment", "development");

    // Database defaults
    let _ = manager.set("database.host", "localhost");
    let _ = manager.set("database.port", 5432i64);
    let _ = manager.set("database.username", "postgres");
    let _ = manager.set("database.password", "password");
    let _ = manager.set("database.database", "armature_db");

    // Server defaults
    let _ = manager.set("server.host", "0.0.0.0");
    let _ = manager.set("server.port", 3008i64);
    let _ = manager.set("server.cors_enabled", true);
}

fn display_configuration(config: &ConfigService) {
    println!("✅ Configuration loaded:\n");

    println!("Application:");
    println!(
        "  Name: {}",
        config.get_string("app.name").unwrap_or_default()
    );
    println!(
        "  Version: {}",
        config.get_string("app.version").unwrap_or_default()
    );
    println!(
        "  Environment: {}",
        config.get_string("app.environment").unwrap_or_default()
    );

    println!("\nDatabase:");
    println!(
        "  Host: {}",
        config.get_string("database.host").unwrap_or_default()
    );
    println!(
        "  Port: {}",
        config.get_int("database.port").unwrap_or_default()
    );
    println!(
        "  Database: {}",
        config.get_string("database.database").unwrap_or_default()
    );

    println!("\nServer:");
    println!(
        "  Host: {}",
        config.get_string("server.host").unwrap_or_default()
    );
    println!(
        "  Port: {}",
        config.get_int("server.port").unwrap_or_default()
    );
    println!(
        "  CORS: {}",
        config.get_bool("server.cors_enabled").unwrap_or_default()
    );
}

fn create_app_with_config(config_service: ConfigService) -> Application {
    let container = Container::new();
    let mut router = Router::new();

    // Register config service
    container.register(config_service.clone());

    // Register app service with config
    let app_service = AppService {
        config: config_service,
    };
    container.register(app_service.clone());

    // Create controller
    let controller = ConfigController {
        app_service: app_service.clone(),
    };

    // Register routes
    let controller_clone = controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/config/info".to_string(),
        handler: std::sync::Arc::new(move |_req| {
            let ctrl = controller_clone.clone();
            Box::pin(async move { ctrl.get_info()?.into_response() })
        }),
    });

    let controller_clone2 = controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/config/database".to_string(),
        handler: std::sync::Arc::new(move |_req| {
            let ctrl = controller_clone2.clone();
            Box::pin(async move { ctrl.get_database_info()?.into_response() })
        }),
    });

    Application::new(container, router)
}
