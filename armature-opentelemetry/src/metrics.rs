//! Metrics collection and export

use crate::{
    config::{MetricsExporter, TelemetryConfig},
    error::{TelemetryError, TelemetryResult},
};
use opentelemetry::{KeyValue, global};
use opentelemetry_sdk::metrics::SdkMeterProvider;

/// Initialize metrics based on configuration
pub async fn init_metrics(config: &TelemetryConfig) -> TelemetryResult<SdkMeterProvider> {
    if !config.enable_metrics {
        return Err(TelemetryError::Config(
            "Metrics are not enabled".to_string(),
        ));
    }

    let resource = config.create_resource()?;

    let provider = match config.metrics.exporter {
        #[cfg(feature = "otlp")]
        MetricsExporter::Otlp => {
            // TODO: OTLP metrics exporter requires complex setup with opentelemetry 0.22
            // For now, return a basic provider
            return Err(TelemetryError::Config(
                "OTLP metrics exporter not yet implemented for opentelemetry 0.22".to_string(),
            ));
        }

        #[cfg(feature = "prometheus")]
        MetricsExporter::Prometheus => {
            let exporter = opentelemetry_prometheus::exporter()
                .build()
                .map_err(|e| TelemetryError::Exporter(e.to_string()))?;

            SdkMeterProvider::builder()
                .with_resource(resource)
                .with_reader(exporter)
                .build()
        }

        MetricsExporter::None => SdkMeterProvider::builder().with_resource(resource).build(),

        #[allow(unreachable_patterns)]
        _ => {
            return Err(TelemetryError::Config(format!(
                "Metrics exporter {:?} not available (feature not enabled)",
                config.metrics.exporter
            )));
        }
    };

    // Set as global provider
    global::set_meter_provider(provider.clone());

    Ok(provider)
}

/// Shutdown metrics gracefully
pub async fn shutdown_metrics(provider: SdkMeterProvider) -> TelemetryResult<()> {
    provider
        .shutdown()
        .map_err(|e| TelemetryError::Shutdown(e.to_string()))?;
    Ok(())
}

/// Get a meter for the current service
pub fn get_meter(name: &'static str) -> opentelemetry::metrics::Meter {
    global::meter(name)
}

/// Common HTTP metrics
pub struct HttpMetrics {
    pub request_count: opentelemetry::metrics::Counter<u64>,
    pub request_duration: opentelemetry::metrics::Histogram<f64>,
    pub active_requests: opentelemetry::metrics::UpDownCounter<i64>,
}

impl HttpMetrics {
    /// Create HTTP metrics
    pub fn new(meter: &opentelemetry::metrics::Meter) -> TelemetryResult<Self> {
        let request_count = meter
            .u64_counter("http.server.request.count")
            .with_description("Total number of HTTP requests")
            .init();

        let request_duration = meter
            .f64_histogram("http.server.request.duration")
            .with_description("HTTP request duration in seconds")
            .with_unit(opentelemetry::metrics::Unit::new("s"))
            .init();

        let active_requests = meter
            .i64_up_down_counter("http.server.active_requests")
            .with_description("Number of active HTTP requests")
            .init();

        Ok(Self {
            request_count,
            request_duration,
            active_requests,
        })
    }

    /// Record a request
    pub fn record_request(&self, method: &str, path: &str, status: u16, duration: f64) {
        let attributes = vec![
            KeyValue::new("http.method", method.to_string()),
            KeyValue::new("http.route", path.to_string()),
            KeyValue::new("http.status_code", status.to_string()),
        ];

        self.request_count.add(1, &attributes);
        self.request_duration.record(duration, &attributes);
    }

    /// Increment active requests
    pub fn increment_active(&self) {
        self.active_requests.add(1, &[]);
    }

    /// Decrement active requests
    pub fn decrement_active(&self) {
        self.active_requests.add(-1, &[]);
    }
}
