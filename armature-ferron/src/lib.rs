// Allow clippy warnings while crate is under development
#![allow(dead_code)]
#![allow(clippy::type_complexity)]
#![allow(clippy::new_without_default)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::derivable_impls)]

//! # Armature Ferron Integration
//!
//! This crate provides integration between Armature web applications and the
//! [Ferron](https://ferron.sh) reverse proxy server.
//!
//! ## Features
//!
//! - **Configuration Generation**: Generate Ferron config files from Armature app metadata
//! - **Process Management**: Start, stop, and reload Ferron instances
//! - **Health Checking**: Monitor backend services and Ferron health
//! - **Service Discovery**: Dynamic backend registration and discovery
//! - **Load Balancing**: Configure load balancing strategies
//! - **TLS Management**: Automatic certificate configuration
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use armature_ferron::{FerronConfig, Backend, ProxyRoute};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a Ferron configuration
//!     let config = FerronConfig::builder()
//!         .domain("api.example.com")
//!         .backend(Backend::new("http://localhost:3000"))
//!         .tls_auto(true)
//!         .build()?;
//!
//!     // Generate the configuration file
//!     let kdl_config = config.to_kdl()?;
//!     println!("{}", kdl_config);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Load Balancing
//!
//! ```rust,no_run
//! use armature_ferron::{FerronConfig, Backend, LoadBalancer, LoadBalanceStrategy};
//!
//! let config = FerronConfig::builder()
//!     .domain("api.example.com")
//!     .load_balancer(
//!         LoadBalancer::new()
//!             .strategy(LoadBalanceStrategy::RoundRobin)
//!             .backend(Backend::new("http://backend1:3000").weight(3))
//!             .backend(Backend::new("http://backend2:3000").weight(1))
//!             .health_check_interval(30)
//!     )
//!     .build()
//!     .unwrap();
//! ```
//!
//! ## Service Discovery Integration
//!
//! ```rust,no_run
//! use armature_ferron::{FerronManager, ServiceRegistry};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a service registry
//! let registry = ServiceRegistry::new();
//!
//! // Register services dynamically
//! registry.register("api-service", "http://localhost:3000").await?;
//! registry.register("api-service", "http://localhost:3001").await?;
//!
//! // Create manager with auto-discovery
//! let manager = FerronManager::builder()
//!     .config_path("/etc/ferron/ferron.conf")
//!     .service_registry(registry)
//!     .auto_reload(true)
//!     .build()?;
//!
//! // Start Ferron with discovered backends
//! manager.start().await?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod error;
pub mod health;
pub mod manager;
pub mod process;
pub mod registry;

// Re-export main types
pub use config::{
    Backend, FerronConfig, FerronConfigBuilder, LoadBalanceStrategy, LoadBalancer, Location,
    ProxyRoute, RateLimitConfig, TlsConfig,
};
pub use error::{FerronError, Result};
pub use health::{HealthCheck, HealthCheckConfig, HealthStatus};
pub use manager::FerronManager;
pub use process::{FerronProcess, ProcessConfig, ProcessStatus};
pub use registry::{ServiceInstance, ServiceRegistry};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::config::{
        Backend, FerronConfig, FerronConfigBuilder, LoadBalanceStrategy, LoadBalancer, Location,
        ProxyRoute, RateLimitConfig, TlsConfig,
    };
    pub use crate::error::{FerronError, Result};
    pub use crate::health::{HealthCheck, HealthCheckConfig, HealthStatus};
    pub use crate::manager::FerronManager;
    pub use crate::process::{FerronProcess, ProcessConfig, ProcessStatus};
    pub use crate::registry::{ServiceInstance, ServiceRegistry};
}
