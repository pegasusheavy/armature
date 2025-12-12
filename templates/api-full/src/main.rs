//! Armature API - Full Template
//!
//! A production-ready REST API with authentication, validation, and OpenAPI documentation.
//!
//! Run with: cargo run
//! API Docs: http://localhost:3000/docs

mod config;
mod controllers;
mod middleware;
mod models;
mod services;

use armature::prelude::*;
use std::sync::Arc;
use tracing::info;

use crate::config::AppConfig;
use crate::controllers::{AuthController, HealthController, UserController};
use crate::middleware::AuthMiddleware;
use crate::services::{AuthService, UserService};

// =============================================================================
// Application Module
// =============================================================================

pub struct AppModule {
    config: Arc<AppConfig>,
    auth_service: Arc<AuthService>,
    user_service: Arc<UserService>,
}

impl AppModule {
    pub fn new(config: AppConfig) -> Self {
        let config = Arc::new(config);
        let auth_service = Arc::new(AuthService::new(config.jwt_secret.clone()));
        let user_service = Arc::new(UserService::new());

        Self {
            config,
            auth_service,
            user_service,
        }
    }
}

impl Module for AppModule {
    fn name(&self) -> &'static str {
        "AppModule"
    }

    fn providers(&self) -> Vec<Arc<dyn Provider>> {
        vec![
            self.auth_service.clone() as Arc<dyn Provider>,
            self.user_service.clone() as Arc<dyn Provider>,
        ]
    }

    fn controllers(&self) -> Vec<Box<dyn Controller>> {
        vec![
            Box::new(HealthController),
            Box::new(AuthController::new(
                self.auth_service.clone(),
                self.user_service.clone(),
            )),
            Box::new(UserController::new(
                self.user_service.clone(),
                self.auth_service.clone(),
            )),
        ]
    }
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
        .with_thread_ids(true)
        .init();

    info!("🚀 Starting Armature API (Full Template)");

    // Load configuration
    let config = AppConfig::from_env();

    info!(host = %config.host, port = %config.port, "Configuration loaded");

    // Create application
    let module = AppModule::new(config.clone());
    let app = Application::create(Box::new(module));

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    info!(addr = %addr, "Server starting");

    println!("");
    println!("Available endpoints:");
    println!("  GET  /health           - Health check");
    println!("  POST /api/auth/login   - Login");
    println!("  POST /api/auth/register - Register");
    println!("  GET  /api/users        - List users (auth required)");
    println!("  GET  /api/users/:id    - Get user (auth required)");
    println!("  GET  /docs             - API documentation");
    println!("");

    app.listen(&addr).await?;

    Ok(())
}

