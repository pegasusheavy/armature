//! Service Discovery for Armature
//!
//! This crate provides service discovery and registration capabilities.
//!
//! ## Features
//!
//! - **Service Registration** - Register service instances
//! - **Service Discovery** - Find healthy service instances
//! - **Health Checks** - Automated health checking
//! - **Load Balancing** - Round-robin, random, or custom strategies
//! - **Multiple Backends** - Consul, etcd, or in-memory
//!
//! ## Quick Start
//!
//! ### In-Memory Discovery (Testing)
//!
//! ```rust,ignore
//! use armature_discovery::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let discovery = InMemoryDiscovery::new();
//!
//!     // Register a service
//!     let service = ServiceInstance::new("api-1", "api", "localhost", 8080)
//!         .with_tag("v1")
//!         .with_health_check("http://localhost:8080/health");
//!
//!     discovery.register(&service).await?;
//!
//!     // Discover services
//!     let instances = discovery.discover("api").await?;
//!     for instance in instances {
//!         println!("Found: {}", instance.url());
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Consul Discovery
//!
//! ```rust,ignore
//! use armature_discovery::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let consul = ConsulDiscovery::new("http://localhost:8500")?;
//!
//!     let service = ServiceInstance::new("api-1", "api", "localhost", 8080);
//!     consul.register(&service).await?;
//!
//!     let instances = consul.discover("api").await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Service Resolver with Load Balancing
//!
//! ```rust,ignore
//! use armature_discovery::*;
//!
//! let discovery = InMemoryDiscovery::new();
//! let resolver = ServiceResolver::new(discovery, LoadBalancingStrategy::RoundRobin);
//!
//! // Automatically selects instance using round-robin
//! let instance = resolver.resolve("api").await?;
//! ```

pub mod service;
pub mod memory;
pub mod consul;
pub mod etcd;

pub use service::{
    DiscoveryError, LoadBalancingStrategy, ServiceDiscovery, ServiceInstance, ServiceResolver,
};
pub use memory::InMemoryDiscovery;
pub use consul::ConsulDiscovery;
pub use etcd::EtcdDiscovery;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        // Ensure module compiles
    }
}

