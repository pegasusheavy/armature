//! Armature GraphQL API Template
//!
//! A production-ready GraphQL API with queries, mutations, and subscriptions.
//!
//! Run with: cargo run
//! GraphQL Playground: http://localhost:3000/playground

mod config;
mod schema;
mod services;

use armature::prelude::*;
use armature::armature_graphql::graphiql_html;
use async_graphql::{EmptySubscription, Schema};
use serde::Deserialize;
use tracing::info;

use crate::config::AppConfig;
use crate::schema::{MutationRoot, QueryRoot};
use crate::services::{BookService, UserService};

// =============================================================================
// GraphQL Controller
// =============================================================================

#[controller("/graphql")]
#[derive(Default)]
struct GraphQLController;

impl GraphQLController {
    #[post("")]
    async fn execute(req: HttpRequest) -> Result<HttpResponse, Error> {
        // Create services
        let user_service = UserService::new();
        let book_service = BookService::new();

        // Build schema
        let query = QueryRoot::new(user_service.clone(), book_service.clone());
        let mutation = MutationRoot::new(user_service, book_service);
        let schema = Schema::build(query, mutation, EmptySubscription).finish();

        // Parse GraphQL request
        #[derive(Deserialize)]
        struct GraphQLRequest {
            query: String,
            #[serde(default)]
            variables: Option<serde_json::Value>,
            #[serde(default, rename = "operationName")]
            operation_name: Option<String>,
        }

        let gql_req: GraphQLRequest = req.json()?;

        // Build async-graphql request
        let mut request = async_graphql::Request::new(gql_req.query);
        if let Some(vars) = gql_req.variables {
            request = request.variables(async_graphql::Variables::from_json(vars));
        }
        if let Some(op_name) = gql_req.operation_name {
            request = request.operation_name(op_name);
        }

        // Execute query
        let response = schema.execute(request).await;

        // Convert to JSON response
        let json =
            serde_json::to_value(&response).map_err(|e| Error::Serialization(e.to_string()))?;

        HttpResponse::ok().with_json(&json)
    }

    #[get("/schema")]
    async fn get_schema() -> Result<HttpResponse, Error> {
        let user_service = UserService::new();
        let book_service = BookService::new();

        let query = QueryRoot::new(user_service.clone(), book_service.clone());
        let mutation = MutationRoot::new(user_service, book_service);
        let schema = Schema::build(query, mutation, EmptySubscription).finish();
        let sdl = schema.sdl();

        Ok(HttpResponse::ok()
            .with_header("Content-Type".to_string(), "text/plain".to_string())
            .with_body(sdl.into_bytes()))
    }
}

// =============================================================================
// Playground Controller
// =============================================================================

#[controller("/playground")]
#[derive(Default)]
struct PlaygroundController;

impl PlaygroundController {
    #[get("")]
    async fn playground() -> Result<HttpResponse, Error> {
        let html = graphiql_html("/graphql");
        Ok(HttpResponse::ok()
            .with_header("Content-Type".to_string(), "text/html".to_string())
            .with_body(html.into_bytes()))
    }
}

// =============================================================================
// Health Controller
// =============================================================================

#[controller("/health")]
#[derive(Default)]
struct HealthController;

impl HealthController {
    #[get("")]
    async fn check() -> Result<HttpResponse, Error> {
        HttpResponse::ok().with_json(&serde_json::json!({
            "status": "healthy",
            "service": "graphql-api"
        }))
    }

    #[get("/live")]
    async fn liveness() -> Result<HttpResponse, Error> {
        HttpResponse::ok().with_json(&serde_json::json!({ "status": "alive" }))
    }

    #[get("/ready")]
    async fn readiness() -> Result<HttpResponse, Error> {
        HttpResponse::ok().with_json(&serde_json::json!({ "status": "ready" }))
    }
}

// =============================================================================
// Application Module
// =============================================================================

#[module(
    providers: [UserService, BookService],
    controllers: [GraphQLController, PlaygroundController, HealthController]
)]
#[derive(Default)]
struct AppModule;

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

    info!("🚀 Starting Armature GraphQL API");

    // Load configuration
    let config = AppConfig::from_env();

    info!(host = %config.host, port = %config.port, "Configuration loaded");

    // Create application
    let app = Application::create::<AppModule>().await;

    println!();
    println!("🎯 GraphQL API ready!");
    println!();
    println!("Endpoints:");
    println!("  POST /graphql        - GraphQL endpoint");
    println!("  GET  /graphql/schema - GraphQL SDL");
    println!("  GET  /playground     - GraphiQL Playground");
    println!("  GET  /health         - Health check");
    println!("  GET  /health/live    - Liveness probe");
    println!("  GET  /health/ready   - Readiness probe");
    println!();
    println!("Example queries:");
    println!();
    println!("  # List all users");
    println!("  query {{ users {{ id name email role }} }}");
    println!();
    println!("  # Get a specific book with author");
    println!("  query {{ book(id: \"1\") {{ id title authorId }} }}");
    println!();
    println!("  # Create a user");
    println!("  mutation {{ createUser(name: \"Alice\", email: \"alice@example.com\") {{ id name }} }}");
    println!();
    println!("  # Search books");
    println!("  query {{ searchBooks(query: \"Rust\") {{ id title }} }}");
    println!();

    let port = config.port;
    if let Err(e) = app.listen(port).await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
