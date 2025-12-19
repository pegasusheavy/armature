//! # Armature Lambda
//!
//! AWS Lambda runtime adapter for Armature applications.
//!
//! This crate allows you to deploy Armature applications to AWS Lambda
//! with API Gateway, ALB, or Lambda Function URLs.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use armature::prelude::*;
//! use armature_lambda::LambdaRuntime;
//!
//! #[controller("/")]
//! struct HelloController;
//!
//! #[controller_impl]
//! impl HelloController {
//!     #[get("/")]
//!     async fn hello() -> &'static str {
//!         "Hello from Lambda!"
//!     }
//! }
//!
//! #[module(controllers: [HelloController])]
//! struct AppModule;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), lambda_runtime::Error> {
//!     // Initialize tracing for CloudWatch
//!     armature_lambda::init_tracing();
//!
//!     // Create Armature application
//!     let app = Application::create::<AppModule>();
//!
//!     // Run on Lambda
//!     LambdaRuntime::new(app).run().await
//! }
//! ```
//!
//! ## With AWS Services
//!
//! ```rust,ignore
//! use armature_lambda::LambdaRuntime;
//! use armature_aws::{AwsServices, AwsConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), lambda_runtime::Error> {
//!     armature_lambda::init_tracing();
//!
//!     // Initialize AWS services
//!     let aws = AwsServices::new(
//!         AwsConfig::from_env()
//!             .enable_dynamodb()
//!             .enable_s3()
//!             .build()
//!     ).await?;
//!
//!     // Register in DI container
//!     let app = Application::create::<AppModule>();
//!     app.container().register(aws);
//!
//!     LambdaRuntime::new(app).run().await
//! }
//! ```
//!
//! ## Deployment
//!
//! Build for Lambda with:
//!
//! ```bash
//! # Install cargo-lambda
//! cargo install cargo-lambda
//!
//! # Build for Lambda
//! cargo lambda build --release
//!
//! # Deploy
//! cargo lambda deploy
//! ```

mod error;
mod request;
mod response;
mod runtime;

pub use error::{LambdaError, Result};
pub use request::LambdaRequest;
pub use response::LambdaResponse;
pub use runtime::{LambdaConfig, LambdaRuntime};

// Re-export lambda types
pub use lambda_http;
pub use lambda_runtime;

/// Initialize tracing for Lambda/CloudWatch.
///
/// This sets up structured JSON logging suitable for CloudWatch Logs.
pub fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().json().flatten_event(true))
        .init();
}

/// Initialize tracing with a custom log level.
pub fn init_tracing_with_level(level: &str) {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let filter = tracing_subscriber::EnvFilter::new(level);

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().json().flatten_event(true))
        .init();
}
