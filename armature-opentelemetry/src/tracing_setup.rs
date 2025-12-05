//! Tracing setup and management

use crate::{
    config::{TelemetryConfig, TracingExporter},
    error::{TelemetryError, TelemetryResult},
};
use opentelemetry::global;
use opentelemetry_sdk::{
    runtime,
    trace::{Config, RandomIdGenerator, Sampler, TracerProvider},
};

/// Initialize tracing based on configuration
pub async fn init_tracing(config: &TelemetryConfig) -> TelemetryResult<TracerProvider> {
    if !config.enable_tracing {
        return Err(TelemetryError::Config("Tracing is not enabled".to_string()));
    }

    let resource = config.create_resource()?;

    let sampler = if config.tracing.sampling_ratio >= 1.0 {
        Sampler::AlwaysOn
    } else if config.tracing.sampling_ratio <= 0.0 {
        Sampler::AlwaysOff
    } else {
        Sampler::TraceIdRatioBased(config.tracing.sampling_ratio)
    };

    let provider = match config.tracing.exporter {
        #[cfg(feature = "otlp")]
        TracingExporter::Otlp => {
            let endpoint = config.tracing.otlp_endpoint.as_ref().ok_or_else(|| {
                TelemetryError::Config("OTLP endpoint not configured".to_string())
            })?;

            use opentelemetry_otlp::WithExportConfig;

            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint)
                .build_span_exporter()
                .map_err(|e| TelemetryError::Exporter(e.to_string()))?;

            let trace_config = Config::default()
                .with_resource(resource)
                .with_id_generator(RandomIdGenerator::default())
                .with_sampler(sampler)
                .with_max_attributes_per_span(config.tracing.max_attributes_per_span)
                .with_max_events_per_span(config.tracing.max_events_per_span);

            TracerProvider::builder()
                .with_batch_exporter(exporter, runtime::Tokio)
                .with_config(trace_config)
                .build()
        }

        #[cfg(feature = "jaeger")]
        TracingExporter::Jaeger => {
            let endpoint = config.tracing.jaeger_endpoint.as_ref().ok_or_else(|| {
                TelemetryError::Config("Jaeger endpoint not configured".to_string())
            })?;

            let exporter = opentelemetry_jaeger::new_agent_pipeline()
                .with_endpoint(endpoint)
                .with_service_name(&config.service_name)
                .build_async_agent_exporter(runtime::Tokio)
                .map_err(|e| TelemetryError::Exporter(e.to_string()))?;

            let trace_config = Config::default()
                .with_resource(resource)
                .with_id_generator(RandomIdGenerator::default())
                .with_sampler(sampler);

            TracerProvider::builder()
                .with_batch_exporter(exporter, runtime::Tokio)
                .with_config(trace_config)
                .build()
        }

        #[cfg(feature = "zipkin")]
        TracingExporter::Zipkin => {
            let endpoint = config.tracing.zipkin_endpoint.as_ref().ok_or_else(|| {
                TelemetryError::Config("Zipkin endpoint not configured".to_string())
            })?;

            let exporter = opentelemetry_zipkin::new_pipeline()
                .with_service_name(&config.service_name)
                .with_collector_endpoint(endpoint)
                .init_exporter()
                .map_err(|e| TelemetryError::Exporter(e.to_string()))?;

            let trace_config = Config::default()
                .with_resource(resource)
                .with_id_generator(RandomIdGenerator::default())
                .with_sampler(sampler);

            TracerProvider::builder()
                .with_batch_exporter(exporter, runtime::Tokio)
                .with_config(trace_config)
                .build()
        }

        TracingExporter::None => {
            let trace_config = Config::default()
                .with_resource(resource)
                .with_id_generator(RandomIdGenerator::default())
                .with_sampler(sampler);

            TracerProvider::builder().with_config(trace_config).build()
        }

        #[allow(unreachable_patterns)]
        _ => {
            return Err(TelemetryError::Config(format!(
                "Tracing exporter {:?} not available (feature not enabled)",
                config.tracing.exporter
            )));
        }
    };

    // Set as global provider
    global::set_tracer_provider(provider.clone());

    Ok(provider)
}

/// Shutdown tracing gracefully
pub async fn shutdown_tracing(provider: TracerProvider) -> TelemetryResult<()> {
    provider.force_flush();
    Ok(())
}

/// Get a tracer for the current service
pub fn get_tracer(name: &'static str) -> impl opentelemetry::trace::Tracer {
    global::tracer(name)
}
