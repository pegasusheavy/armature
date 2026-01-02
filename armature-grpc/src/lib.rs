// Allow dead_code for now as this crate is still under development
#![allow(dead_code)]
// Status from tonic is inherently large; this is acceptable for error handling
#![allow(clippy::result_large_err)]

//! # Armature gRPC
//!
//! gRPC server and client support for Armature applications.
//!
//! ## Features
//!
//! - **Server**: Build gRPC servers with middleware support
//! - **Client**: Type-safe gRPC client with retry and load balancing
//! - **Interceptors**: Request/response interceptors for auth, logging, etc.
//! - **Health Checking**: Built-in gRPC health checking service
//! - **Reflection**: Server reflection for tools like grpcurl
//! - **Compression**: gzip and zstd compression support
//!
//! ## Quick Start
//!
//! ### Server
//!
//! ```rust,ignore
//! use armature_grpc::{GrpcServer, GrpcServerConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = GrpcServerConfig::builder()
//!         .bind_address("0.0.0.0:50051")
//!         .enable_health_check()
//!         .enable_reflection()
//!         .build();
//!
//!     let server = GrpcServer::new(config)
//!         .add_service(MyServiceServer::new(MyServiceImpl))
//!         .serve()
//!         .await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Client
//!
//! ```rust,ignore
//! use armature_grpc::{GrpcClient, GrpcClientConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = GrpcClientConfig::builder()
//!         .endpoint("http://localhost:50051")
//!         .timeout(std::time::Duration::from_secs(30))
//!         .build();
//!
//!     let client = GrpcClient::connect(config).await?;
//!
//!     // Use the client...
//!     Ok(())
//! }
//! ```

mod client;
mod config;
mod error;
mod interceptor;
mod middleware;
mod server;

pub use client::{GrpcChannel, GrpcClient};
pub use config::{GrpcClientConfig, GrpcServerConfig};
pub use error::{GrpcError, Result};
pub use interceptor::{
    AuthInterceptor, Interceptor, LoggingInterceptor, MetricsInterceptor, RequestInterceptor,
    ResponseInterceptor,
};
pub use middleware::{GrpcMiddleware, MiddlewareLayer};
pub use server::{GrpcServer, GrpcServerBuilder};

// Re-export tonic types
pub use tonic::{
    metadata::{MetadataMap, MetadataValue},
    transport::{Channel, Endpoint, Server},
    Code, Request, Response, Status,
};

#[cfg(feature = "health")]
pub use tonic_health;

#[cfg(feature = "reflection")]
pub use tonic_reflection;

/// Prelude for common imports.
///
/// ```
/// use armature_grpc::prelude::*;
/// ```
pub mod prelude {
    pub use crate::client::{GrpcChannel, GrpcClient};
    pub use crate::config::{GrpcClientConfig, GrpcServerConfig};
    pub use crate::error::{GrpcError, Result};
    pub use crate::interceptor::{
        AuthInterceptor, Interceptor, LoggingInterceptor, MetricsInterceptor,
    };
    pub use crate::middleware::{GrpcMiddleware, MiddlewareLayer};
    pub use crate::server::{GrpcServer, GrpcServerBuilder};
    pub use tonic::{
        metadata::{MetadataMap, MetadataValue},
        transport::{Channel, Endpoint, Server},
        Code, Request, Response, Status,
    };
}
