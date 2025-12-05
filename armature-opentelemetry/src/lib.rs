//! OpenTelemetry integration for Armature
//!
//! This crate provides observability features for Armature applications including:
//! - Distributed tracing with automatic span creation
//! - Metrics collection (counters, gauges, histograms)
//! - Logging integration
//! - Multiple exporters (OTLP, Jaeger, Zipkin, Prometheus)
//! - Middleware for automatic instrumentation
//!
//! # Examples
//!
//! ## Basic Tracing
//!
//! ```no_run
//! use armature_opentelemetry::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), TelemetryError> {
//!     // Initialize OpenTelemetry with OTLP exporter
//!     let telemetry = TelemetryBuilder::new("my-service")
//!         .with_otlp_endpoint("http://localhost:4317")
//!         .with_tracing()
//!         .with_metrics()
//!         .build()
//!         .await?;
//!
//!     // Use middleware for automatic instrumentation
//!     // let app = Application::new(container, router)
//!     //     .with_middleware(telemetry.middleware());
//!
//!     // Shutdown telemetry on exit
//!     telemetry.shutdown().await?;
//!     Ok(())
//! }
//! ```

pub mod builder;
pub mod config;
pub mod error;
pub mod metrics;
pub mod middleware;
pub mod tracing_setup;

pub use builder::*;
pub use config::*;
pub use error::{TelemetryError, TelemetryResult};
pub use metrics::*;
pub use middleware::*;
pub use tracing_setup::*;

// Re-export commonly used OpenTelemetry types
pub use opentelemetry::{
    Context as OtelContext, KeyValue, StringValue, Value, global, trace::TraceError,
};
pub use opentelemetry_sdk::{
    Resource,
    export::metrics::aggregation::Aggregation,
    metrics::{MeterProvider, PeriodicReader},
    runtime,
    trace::{RandomIdGenerator, Sampler, Tracer, TracerProvider},
};
