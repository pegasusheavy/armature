//! Telemetry builder for easy setup

use crate::{
    config::TelemetryConfig,
    error::{TelemetryError, TelemetryResult},
    metrics::{HttpMetrics, init_metrics, shutdown_metrics},
    middleware::TelemetryMiddleware,
    tracing_setup::{init_tracing, shutdown_tracing},
};
use opentelemetry::metrics::MeterProvider;
use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::TracerProvider};
use std::sync::Arc;

/// Telemetry system manager
pub struct Telemetry {
    config: TelemetryConfig,
    tracer_provider: Option<TracerProvider>,
    meter_provider: Option<SdkMeterProvider>,
    http_metrics: Option<Arc<HttpMetrics>>,
}

impl Telemetry {
    /// Get the middleware for automatic instrumentation
    pub fn middleware(&self) -> TelemetryMiddleware {
        let mut middleware = TelemetryMiddleware::new(&self.config.service_name);

        if let Some(ref metrics) = self.http_metrics {
            middleware = middleware.with_metrics(Arc::clone(metrics));
        }

        middleware
    }

    /// Get HTTP metrics
    pub fn http_metrics(&self) -> Option<&HttpMetrics> {
        self.http_metrics.as_deref()
    }

    /// Shutdown telemetry gracefully
    pub async fn shutdown(self) -> TelemetryResult<()> {
        if let Some(provider) = self.tracer_provider {
            shutdown_tracing(provider).await?;
        }

        if let Some(provider) = self.meter_provider {
            shutdown_metrics(provider).await?;
        }

        Ok(())
    }
}

/// Builder for telemetry setup
pub struct TelemetryBuilder {
    config: TelemetryConfig,
}

impl TelemetryBuilder {
    /// Create a new telemetry builder
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            config: TelemetryConfig::new(service_name),
        }
    }

    /// Set service version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.config = self.config.with_version(version);
        self
    }

    /// Set service namespace
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.config = self.config.with_namespace(namespace);
        self
    }

    /// Set environment
    pub fn with_environment(mut self, env: impl Into<String>) -> Self {
        self.config = self.config.with_environment(env);
        self
    }

    /// Enable tracing
    pub fn with_tracing(mut self) -> Self {
        self.config.enable_tracing = true;
        self
    }

    /// Disable tracing
    pub fn without_tracing(mut self) -> Self {
        self.config.enable_tracing = false;
        self
    }

    /// Enable metrics
    pub fn with_metrics(mut self) -> Self {
        self.config.enable_metrics = true;
        self
    }

    /// Disable metrics
    pub fn without_metrics(mut self) -> Self {
        self.config.enable_metrics = false;
        self
    }

    /// Set OTLP endpoint for both tracing and metrics
    pub fn with_otlp_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        self.config.tracing.otlp_endpoint = Some(endpoint.clone());
        self.config.metrics.otlp_endpoint = Some(endpoint);
        self
    }

    /// Set Jaeger endpoint
    #[cfg(feature = "jaeger")]
    pub fn with_jaeger_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.tracing.jaeger_endpoint = Some(endpoint.into());
        self
    }

    /// Set Zipkin endpoint
    #[cfg(feature = "zipkin")]
    pub fn with_zipkin_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.tracing.zipkin_endpoint = Some(endpoint.into());
        self
    }

    /// Set Prometheus endpoint
    #[cfg(feature = "prometheus")]
    pub fn with_prometheus_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.metrics.prometheus_endpoint = Some(endpoint.into());
        self
    }

    /// Set sampling ratio (0.0 to 1.0)
    pub fn with_sampling_ratio(mut self, ratio: f64) -> Self {
        self.config.tracing.sampling_ratio = ratio;
        self
    }

    /// Add a resource attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config = self.config.with_attribute(key, value);
        self
    }

    /// Use custom configuration
    pub fn with_config(mut self, config: TelemetryConfig) -> Self {
        self.config = config;
        self
    }

    /// Build and initialize telemetry
    pub async fn build(self) -> TelemetryResult<Telemetry> {
        // Validate configuration
        self.config.validate()?;

        let mut tracer_provider = None;
        let mut meter_provider = None;
        let mut http_metrics = None;

        // Initialize tracing
        if self.config.enable_tracing {
            tracer_provider = Some(init_tracing(&self.config).await?);
        }

        // Initialize metrics
        if self.config.enable_metrics {
            let provider = init_metrics(&self.config).await?;
            let meter = provider.meter(self.config.service_name.clone());

            // Create HTTP metrics
            http_metrics =
                Some(Arc::new(HttpMetrics::new(&meter).map_err(|e| {
                    TelemetryError::Initialization(e.to_string())
                })?));

            meter_provider = Some(provider);
        }

        Ok(Telemetry {
            config: self.config,
            tracer_provider,
            meter_provider,
            http_metrics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_builder() {
        let telemetry = TelemetryBuilder::new("test-service")
            .with_version("1.0.0")
            .with_environment("test")
            .without_tracing()
            .without_metrics()
            .build()
            .await;

        assert!(telemetry.is_ok());
    }
}
