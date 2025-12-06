//! SQLx Database Integration Example
//!
//! Demonstrates how to integrate SQLx database connections with Armature's
//! dependency injection system and lifecycle hooks.
//!
//! This example shows:
//! - Creating a database service with connection pool
//! - Using lifecycle hooks (OnModuleInit, OnModuleDestroy)
//! - Injecting database service into controllers
//! - Performing CRUD operations
//! - Proper error handling and connection management

use armature_core::lifecycle::{LifecycleManager, OnApplicationShutdown, OnModuleInit};
use armature_core::{Error as ArmatureError, HttpRequest, HttpResponse, Provider};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{FromRow, Row};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Domain Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

// ============================================================================
// Database Service with Lifecycle Hooks
// ============================================================================

/// Database service that manages SQLx connection pool
///
/// This service:
/// - Initializes connection pool on module init
/// - Closes pool gracefully on shutdown
/// - Provides database operations to other services
#[derive(Clone)]
pub struct DatabaseService {
    /// Connection string for the database
    connection_string: String,
    
    /// SQLx connection pool (initialized after OnModuleInit)
    pool: Arc<RwLock<Option<PgPool>>>,
}

impl DatabaseService {
    /// Create a new database service
    pub fn new(connection_string: String) -> Self {
        Self {
            connection_string,
            pool: Arc::new(RwLock::new(None)),
        }
    }

    /// Get a reference to the connection pool
    ///
    /// Returns an error if the pool hasn't been initialized yet
    pub async fn pool(&self) -> Result<PgPool, sqlx::Error> {
        let pool_guard = self.pool.read().await;
        pool_guard
            .clone()
            .ok_or_else(|| sqlx::Error::PoolClosed)
    }

    /// Check if database is connected
    pub async fn is_connected(&self) -> bool {
        self.pool.read().await.is_some()
    }
}

// Mark as injectable provider
impl Provider for DatabaseService {}

// Lifecycle hook: Initialize database connection on module init
#[async_trait]
impl OnModuleInit for DatabaseService {
    async fn on_module_init(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("📊 DatabaseService: Connecting to PostgreSQL...");
        println!("   Connection: {}", mask_connection_string(&self.connection_string));

        // Create connection pool with configuration
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .idle_timeout(std::time::Duration::from_secs(600))
            .connect(&self.connection_string)
            .await?;

        // Test the connection
        sqlx::query("SELECT 1")
            .fetch_one(&pool)
            .await?;

        // Store the pool
        *self.pool.write().await = Some(pool);

        println!("   ✅ Database connected successfully!");
        println!("   📋 Running migrations...");

        // Run migrations (optional)
        if let Some(pool) = self.pool.read().await.as_ref() {
            self.create_tables(pool).await?;
            println!("   ✅ Migrations complete!");
        }

        Ok(())
    }
}

// Lifecycle hook: Close database connection on shutdown
#[async_trait]
impl OnApplicationShutdown for DatabaseService {
    async fn on_application_shutdown(
        &self,
        signal: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(sig) = signal {
            println!("📊 DatabaseService: Closing connections (signal: {})...", sig);
        } else {
            println!("📊 DatabaseService: Closing connections...");
        }

        // Close the pool
        if let Some(pool) = self.pool.write().await.take() {
            pool.close().await;
            println!("   ✅ Database connections closed!");
        }

        Ok(())
    }
}

// Database operations
impl DatabaseService {
    /// Create tables (simple migration)
    async fn create_tables(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                username VARCHAR(255) NOT NULL UNIQUE,
                email VARCHAR(255) NOT NULL UNIQUE,
                created_at TIMESTAMP NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get all users
    pub async fn get_all_users(&self) -> Result<Vec<User>, sqlx::Error> {
        let pool = self.pool().await?;

        let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY id")
            .fetch_all(&pool)
            .await?;

        Ok(users)
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error> {
        let pool = self.pool().await?;

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&pool)
            .await?;

        Ok(user)
    }

    /// Create a new user
    pub async fn create_user(
        &self,
        username: String,
        email: String,
    ) -> Result<User, sqlx::Error> {
        let pool = self.pool().await?;

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email)
            VALUES ($1, $2)
            RETURNING *
            "#,
        )
        .bind(username)
        .bind(email)
        .fetch_one(&pool)
        .await?;

        Ok(user)
    }

    /// Update a user
    pub async fn update_user(
        &self,
        id: i32,
        username: Option<String>,
        email: Option<String>,
    ) -> Result<Option<User>, sqlx::Error> {
        let pool = self.pool().await?;

        // Build dynamic update query
        let mut query = String::from("UPDATE users SET ");
        let mut updates = Vec::new();
        let mut param_count = 1;

        if username.is_some() {
            updates.push(format!("username = ${}", param_count));
            param_count += 1;
        }

        if email.is_some() {
            updates.push(format!("email = ${}", param_count));
            param_count += 1;
        }

        if updates.is_empty() {
            // No updates, just return existing user
            return self.get_user_by_id(id).await;
        }

        query.push_str(&updates.join(", "));
        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

        let mut query_builder = sqlx::query_as::<_, User>(&query);

        if let Some(uname) = username {
            query_builder = query_builder.bind(uname);
        }
        if let Some(mail) = email {
            query_builder = query_builder.bind(mail);
        }
        query_builder = query_builder.bind(id);

        let user = query_builder.fetch_optional(&pool).await?;

        Ok(user)
    }

    /// Delete a user
    pub async fn delete_user(&self, id: i32) -> Result<bool, sqlx::Error> {
        let pool = self.pool().await?;

        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get user count
    pub async fn count_users(&self) -> Result<i64, sqlx::Error> {
        let pool = self.pool().await?;

        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(&pool)
            .await?
            .try_get("count")?;

        Ok(count)
    }
}

// ============================================================================
// User Service (Business Logic Layer)
// ============================================================================

/// User service that handles business logic
///
/// This service depends on DatabaseService via DI
#[derive(Clone)]
pub struct UserService {
    db: Arc<DatabaseService>,
}

impl UserService {
    pub fn new(db: DatabaseService) -> Self {
        Self { db: Arc::new(db) }
    }

    pub async fn list_users(&self) -> Result<Vec<User>, String> {
        self.db
            .get_all_users()
            .await
            .map_err(|e| format!("Failed to fetch users: {}", e))
    }

    pub async fn get_user(&self, id: i32) -> Result<Option<User>, String> {
        self.db
            .get_user_by_id(id)
            .await
            .map_err(|e| format!("Failed to fetch user: {}", e))
    }

    pub async fn create_user(&self, req: CreateUserRequest) -> Result<User, String> {
        // Validation
        if req.username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }

        if !req.email.contains('@') {
            return Err("Invalid email address".to_string());
        }

        self.db
            .create_user(req.username, req.email)
            .await
            .map_err(|e| format!("Failed to create user: {}", e))
    }

    pub async fn update_user(&self, id: i32, req: UpdateUserRequest) -> Result<Option<User>, String> {
        // Validation
        if let Some(ref username) = req.username {
            if username.is_empty() {
                return Err("Username cannot be empty".to_string());
            }
        }

        if let Some(ref email) = req.email {
            if !email.contains('@') {
                return Err("Invalid email address".to_string());
            }
        }

        self.db
            .update_user(id, req.username, req.email)
            .await
            .map_err(|e| format!("Failed to update user: {}", e))
    }

    pub async fn delete_user(&self, id: i32) -> Result<bool, String> {
        self.db
            .delete_user(id)
            .await
            .map_err(|e| format!("Failed to delete user: {}", e))
    }

    pub async fn stats(&self) -> Result<serde_json::Value, String> {
        let count = self.db
            .count_users()
            .await
            .map_err(|e| format!("Failed to get stats: {}", e))?;

        Ok(serde_json::json!({
            "total_users": count,
            "database": "PostgreSQL",
            "connected": self.db.is_connected().await
        }))
    }
}

impl Provider for UserService {}

// ============================================================================
// Controller (Simulated - would use #[controller] macro in real app)
// ============================================================================

/// User controller that handles HTTP requests
pub struct UserController {
    user_service: Arc<UserService>,
}

impl UserController {
    pub fn new(user_service: UserService) -> Self {
        Self {
            user_service: Arc::new(user_service),
        }
    }

    /// GET /users - List all users
    pub async fn list_users(&self, _req: HttpRequest) -> Result<HttpResponse, ArmatureError> {
        match self.user_service.list_users().await {
            Ok(users) => {
                let json = serde_json::to_string(&users)
                    .map_err(|e| ArmatureError::Internal(e.to_string()))?;
                Ok(HttpResponse::ok()
                    .with_header("Content-Type".to_string(), "application/json".to_string())
                    .with_body(json.into_bytes()))
            }
            Err(e) => Ok(HttpResponse::internal_server_error()
                .with_body(format!(r#"{{"error": "{}"}}"#, e).into_bytes())),
        }
    }

    /// GET /users/:id - Get user by ID
    pub async fn get_user(&self, req: HttpRequest) -> Result<HttpResponse, ArmatureError> {
        let id: i32 = req
            .path_params
            .get("id")
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| ArmatureError::BadRequest("Invalid user ID".to_string()))?;

        match self.user_service.get_user(id).await {
            Ok(Some(user)) => {
                let json = serde_json::to_string(&user)
                    .map_err(|e| ArmatureError::Internal(e.to_string()))?;
                Ok(HttpResponse::ok()
                    .with_header("Content-Type".to_string(), "application/json".to_string())
                    .with_body(json.into_bytes()))
            }
            Ok(None) => Ok(HttpResponse::not_found()
                .with_body(format!(r#"{{"error": "User not found"}}"#).into_bytes())),
            Err(e) => Ok(HttpResponse::internal_server_error()
                .with_body(format!(r#"{{"error": "{}"}}"#, e).into_bytes())),
        }
    }

    /// POST /users - Create user
    pub async fn create_user(&self, req: HttpRequest) -> Result<HttpResponse, ArmatureError> {
        let create_req: CreateUserRequest = serde_json::from_slice(&req.body)
            .map_err(|e| ArmatureError::BadRequest(format!("Invalid JSON: {}", e)))?;

        match self.user_service.create_user(create_req).await {
            Ok(user) => {
                let json = serde_json::to_string(&user)
                    .map_err(|e| ArmatureError::Internal(e.to_string()))?;
                Ok(HttpResponse::new(201)
                    .with_header("Content-Type".to_string(), "application/json".to_string())
                    .with_body(json.into_bytes()))
            }
            Err(e) => Ok(HttpResponse::new(400)
                .with_body(format!(r#"{{"error": "{}"}}"#, e).into_bytes())),
        }
    }

    /// GET /stats - Get database stats
    pub async fn stats(&self, _req: HttpRequest) -> Result<HttpResponse, ArmatureError> {
        match self.user_service.stats().await {
            Ok(stats) => {
                let json = serde_json::to_string(&stats)
                    .map_err(|e| ArmatureError::Internal(e.to_string()))?;
                Ok(HttpResponse::ok()
                    .with_header("Content-Type".to_string(), "application/json".to_string())
                    .with_body(json.into_bytes()))
            }
            Err(e) => Ok(HttpResponse::internal_server_error()
                .with_body(format!(r#"{{"error": "{}"}}"#, e).into_bytes())),
        }
    }
}

// ============================================================================
// Main Example
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         SQLx Database Integration Example                  ║");
    println!("║       Armature DI + Lifecycle Hooks + PostgreSQL           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Get database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            println!("⚠️  DATABASE_URL not set, using default (demo mode)");
            "postgres://postgres:password@localhost/armature_demo".to_string()
        });

    println!("📋 Configuration:");
    println!("   Database: {}\n", mask_connection_string(&database_url));

    // Create lifecycle manager
    let lifecycle = LifecycleManager::new();

    // Create database service
    let db_service = Arc::new(DatabaseService::new(database_url.clone()));

    // Register lifecycle hooks
    println!("🔧 Registering lifecycle hooks...");
    lifecycle
        .register_on_init("DatabaseService".to_string(), db_service.clone())
        .await;
    lifecycle
        .register_on_shutdown("DatabaseService".to_string(), db_service.clone())
        .await;
    println!("   ✅ Hooks registered\n");

    // Initialize database (calls OnModuleInit)
    println!("═══════════════════════════════════════════════════════════════");
    println!("                  INITIALIZATION PHASE                         ");
    println!("═══════════════════════════════════════════════════════════════\n");

    if let Err(errors) = lifecycle.call_module_init_hooks().await {
        eprintln!("\n❌ Failed to initialize database:");
        for (name, error) in errors {
            eprintln!("   {} - {}", name, error);
        }
        eprintln!("\n💡 Make sure PostgreSQL is running:");
        eprintln!("   docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=password postgres");
        eprintln!("   export DATABASE_URL=postgres://postgres:password@localhost/armature_demo");
        return Ok(());
    }

    println!();

    // Create services with dependency injection
    println!("═══════════════════════════════════════════════════════════════");
    println!("                  DEPENDENCY INJECTION                         ");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("🔌 Injecting DatabaseService into UserService...");
    let user_service = Arc::new(UserService::new((*db_service).clone()));
    println!("   ✅ UserService created\n");

    println!("🔌 Injecting UserService into UserController...");
    let user_controller = UserController::new((*user_service).clone());
    println!("   ✅ UserController created\n");

    // Demonstrate CRUD operations
    println!("═══════════════════════════════════════════════════════════════");
    println!("                  DATABASE OPERATIONS                          ");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Create test data
    println!("📝 Creating test users...");
    match user_service
        .create_user(CreateUserRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        })
        .await
    {
        Ok(user) => println!("   ✅ Created user: {} (ID: {})", user.username, user.id),
        Err(e) => println!("   ⚠️  {}", e),
    }

    match user_service
        .create_user(CreateUserRequest {
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
        })
        .await
    {
        Ok(user) => println!("   ✅ Created user: {} (ID: {})", user.username, user.id),
        Err(e) => println!("   ⚠️  {}", e),
    }

    println!();

    // List users
    println!("📋 Listing all users...");
    match user_service.list_users().await {
        Ok(users) => {
            println!("   Found {} users:", users.len());
            for user in users {
                println!("   - {} <{}> (ID: {})", user.username, user.email, user.id);
            }
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }

    println!();

    // Get stats
    println!("📊 Database statistics...");
    match user_service.stats().await {
        Ok(stats) => println!("   {}", serde_json::to_string_pretty(&stats).unwrap()),
        Err(e) => println!("   ❌ Error: {}", e),
    }

    println!();

    // Graceful shutdown
    println!("═══════════════════════════════════════════════════════════════");
    println!("                  GRACEFUL SHUTDOWN                            ");
    println!("═══════════════════════════════════════════════════════════════\n");

    lifecycle
        .call_shutdown_hooks(Some("SIGTERM".to_string()))
        .await
        .ok();

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("✅ Example complete! Database integration successful.");
    println!("═══════════════════════════════════════════════════════════════\n");

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Mask sensitive parts of connection string for logging
fn mask_connection_string(conn_str: &str) -> String {
    if let Some(at_pos) = conn_str.find('@') {
        if let Some(colon_pos) = conn_str[..at_pos].rfind(':') {
            let mut masked = conn_str.to_string();
            masked.replace_range(colon_pos + 1..at_pos, "****");
            return masked;
        }
    }
    conn_str.to_string()
}

