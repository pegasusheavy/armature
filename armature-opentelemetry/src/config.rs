//! Configuration for OpenTelemetry

use crate::error::{TelemetryError, TelemetryResult};
use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use serde::{Deserialize, Serialize};

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Service name
    pub service_name: String,

    /// Service version
    pub service_version: Option<String>,

    /// Service namespace
    pub service_namespace: Option<String>,

    /// Environment (e.g., "production", "staging", "development")
    pub environment: Option<String>,

    /// Enable tracing
    pub enable_tracing: bool,

    /// Enable metrics
    pub enable_metrics: bool,

    /// Enable logging
    pub enable_logging: bool,

    /// Tracing configuration
    pub tracing: TracingConfig,

    /// Metrics configuration
    pub metrics: MetricsConfig,

    /// Additional resource attributes
    pub resource_attributes: Vec<(String, String)>,
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Exporter type
    pub exporter: TracingExporter,

    /// OTLP endpoint (if using OTLP)
    pub otlp_endpoint: Option<String>,

    /// Jaeger endpoint (if using Jaeger)
    pub jaeger_endpoint: Option<String>,

    /// Zipkin endpoint (if using Zipkin)
    pub zipkin_endpoint: Option<String>,

    /// Sampling ratio (0.0 to 1.0)
    pub sampling_ratio: f64,

    /// Maximum attributes per span
    pub max_attributes_per_span: u32,

    /// Maximum events per span
    pub max_events_per_span: u32,
}

/// Tracing exporter type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TracingExporter {
    /// OTLP exporter (OpenTelemetry Protocol)
    Otlp,
    /// Jaeger exporter
    Jaeger,
    /// Zipkin exporter
    Zipkin,
    /// No exporter (testing)
    None,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Exporter type
    pub exporter: MetricsExporter,

    /// OTLP endpoint (if using OTLP)
    pub otlp_endpoint: Option<String>,

    /// Prometheus endpoint (if using Prometheus)
    pub prometheus_endpoint: Option<String>,

    /// Metrics collection interval in seconds
    pub collection_interval_secs: u64,
}

/// Metrics exporter type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricsExporter {
    /// OTLP exporter
    Otlp,
    /// Prometheus exporter
    Prometheus,
    /// No exporter (testing)
    None,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: "armature-service".to_string(),
            service_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            service_namespace: None,
            environment: Some("development".to_string()),
            enable_tracing: true,
            enable_metrics: true,
            enable_logging: false,
            tracing: TracingConfig::default(),
            metrics: MetricsConfig::default(),
            resource_attributes: Vec::new(),
        }
    }
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            exporter: TracingExporter::Otlp,
            otlp_endpoint: Some("http://localhost:4317".to_string()),
            jaeger_endpoint: None,
            zipkin_endpoint: None,
            sampling_ratio: 1.0,
            max_attributes_per_span: 128,
            max_events_per_span: 128,
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            exporter: MetricsExporter::Otlp,
            otlp_endpoint: Some("http://localhost:4317".to_string()),
            prometheus_endpoint: None,
            collection_interval_secs: 60,
        }
    }
}

impl TelemetryConfig {
    /// Create a new configuration with a service name
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            ..Default::default()
        }
    }

    /// Set service version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.service_version = Some(version.into());
        self
    }

    /// Set service namespace
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.service_namespace = Some(namespace.into());
        self
    }

    /// Set environment
    pub fn with_environment(mut self, env: impl Into<String>) -> Self {
        self.environment = Some(env.into());
        self
    }

    /// Enable tracing
    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.enable_tracing = enabled;
        self
    }

    /// Enable metrics
    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.enable_metrics = enabled;
        self
    }

    /// Add a resource attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.resource_attributes.push((key.into(), value.into()));
        self
    }

    /// Create OpenTelemetry resource from configuration
    pub fn create_resource(&self) -> TelemetryResult<Resource> {
        let mut attributes = vec![KeyValue::new("service.name", self.service_name.clone())];

        if let Some(ref version) = self.service_version {
            attributes.push(KeyValue::new("service.version", version.clone()));
        }

        if let Some(ref namespace) = self.service_namespace {
            attributes.push(KeyValue::new("service.namespace", namespace.clone()));
        }

        if let Some(ref env) = self.environment {
            attributes.push(KeyValue::new("deployment.environment", env.clone()));
        }

        // Add custom attributes
        for (key, value) in &self.resource_attributes {
            attributes.push(KeyValue::new(key.clone(), value.clone()));
        }

        Ok(Resource::new(attributes))
    }

    /// Validate configuration
    pub fn validate(&self) -> TelemetryResult<()> {
        if self.service_name.is_empty() {
            return Err(TelemetryError::Config(
                "Service name cannot be empty".to_string(),
            ));
        }

        if self.enable_tracing {
            match self.tracing.exporter {
                TracingExporter::Otlp => {
                    if self.tracing.otlp_endpoint.is_none() {
                        return Err(TelemetryError::Config(
                            "OTLP endpoint required for OTLP tracing exporter".to_string(),
                        ));
                    }
                }
                TracingExporter::Jaeger => {
                    if self.tracing.jaeger_endpoint.is_none() {
                        return Err(TelemetryError::Config(
                            "Jaeger endpoint required for Jaeger exporter".to_string(),
                        ));
                    }
                }
                TracingExporter::Zipkin => {
                    if self.tracing.zipkin_endpoint.is_none() {
                        return Err(TelemetryError::Config(
                            "Zipkin endpoint required for Zipkin exporter".to_string(),
                        ));
                    }
                }
                TracingExporter::None => {}
            }
        }

        if self.tracing.sampling_ratio < 0.0 || self.tracing.sampling_ratio > 1.0 {
            return Err(TelemetryError::Config(
                "Sampling ratio must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(())
    }
}
