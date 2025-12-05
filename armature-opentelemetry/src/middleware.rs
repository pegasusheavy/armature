//! OpenTelemetry middleware for automatic instrumentation

use armature_core::{Error, HttpRequest, HttpResponse, Middleware, Next};
use async_trait::async_trait;
use opentelemetry::{
    Context as OtelContext, KeyValue, global,
    trace::{FutureExt, SpanKind, TraceContextExt, Tracer},
};
use std::sync::Arc;
use std::time::Instant;

/// OpenTelemetry middleware for automatic tracing and metrics
pub struct TelemetryMiddleware {
    service_name: String,
    metrics: Option<Arc<crate::metrics::HttpMetrics>>,
}

impl TelemetryMiddleware {
    /// Create a new telemetry middleware
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            metrics: None,
        }
    }

    /// Create with metrics collection
    pub fn with_metrics(mut self, metrics: Arc<crate::metrics::HttpMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Extract span attributes from request
    fn extract_attributes(&self, req: &HttpRequest) -> Vec<KeyValue> {
        let mut attributes = vec![
            KeyValue::new("http.method", req.method.to_string()),
            KeyValue::new("http.target", req.path.clone()),
            KeyValue::new("http.scheme", "http"),
        ];

        // Add host header if present
        if let Some(host) = req.headers.get("host") {
            attributes.push(KeyValue::new("http.host", host.clone()));
        }

        // Add user agent if present
        if let Some(ua) = req.headers.get("user-agent") {
            attributes.push(KeyValue::new("http.user_agent", ua.clone()));
        }

        attributes
    }

    /// Extract response attributes
    fn extract_response_attributes(&self, res: &HttpResponse) -> Vec<KeyValue> {
        vec![
            KeyValue::new("http.status_code", res.status.to_string()),
            KeyValue::new("http.response_content_length", res.body.len() as i64),
        ]
    }
}

#[async_trait]
impl Middleware for TelemetryMiddleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        let start_time = Instant::now();

        // Increment active requests
        if let Some(ref metrics) = self.metrics {
            metrics.increment_active();
        }

        // Extract span context from headers (for distributed tracing)
        let parent_context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(&req.headers))
        });

        // Create a span for this request
        let tracer = global::tracer(&self.service_name);
        let span = tracer
            .span_builder(format!("{} {}", req.method, req.path))
            .with_kind(SpanKind::Server)
            .with_attributes(self.extract_attributes(&req))
            .start_with_context(&tracer, &parent_context);

        let cx = OtelContext::current_with_span(span);

        // Execute request with tracing context
        let result = async move {
            let response = next(req).await;

            match &response {
                Ok(res) => {
                    // Add response attributes to span
                    let span = cx.span();
                    for attr in self.extract_response_attributes(res) {
                        span.set_attribute(attr);
                    }

                    // Set span status based on HTTP status
                    if res.status >= 400 {
                        span.set_status(opentelemetry::trace::Status::error(format!(
                            "HTTP {}",
                            res.status
                        )));
                    } else {
                        span.set_status(opentelemetry::trace::Status::Ok);
                    }
                }
                Err(e) => {
                    // Record error
                    let span = cx.span();
                    span.set_status(opentelemetry::trace::Status::error(e.to_string()));
                    span.record_error(e);
                }
            }

            response
        }
        .with_context(cx)
        .await;

        // Record metrics
        let duration = start_time.elapsed().as_secs_f64();

        if let Some(ref metrics) = self.metrics {
            metrics.decrement_active();

            if let Ok(ref res) = result {
                metrics.record_request(
                    &result.as_ref().ok().map(|_| "method").unwrap_or("UNKNOWN"),
                    "path",
                    res.status,
                    duration,
                );
            }
        }

        result
    }
}

/// Helper to extract trace context from HTTP headers
struct HeaderExtractor<'a>(&'a std::collections::HashMap<String, String>);

impl<'a> opentelemetry::propagation::Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

/// Helper to inject trace context into HTTP headers
pub struct HeaderInjector<'a>(pub &'a mut std::collections::HashMap<String, String>);

impl<'a> opentelemetry::propagation::Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key.to_string(), value);
    }
}

/// Create a span for a function
#[macro_export]
macro_rules! trace_span {
    ($name:expr) => {{
        use opentelemetry::trace::Tracer;
        let tracer = opentelemetry::global::tracer("");
        tracer.start($name)
    }};
    ($name:expr, $($key:expr => $value:expr),*) => {{
        use opentelemetry::trace::Tracer;
        use opentelemetry::KeyValue;
        let tracer = opentelemetry::global::tracer("");
        let mut span = tracer.start($name);
        $(
            span.set_attribute(KeyValue::new($key, $value));
        )*
        span
    }};
}

/// Add an attribute to the current span
#[macro_export]
macro_rules! span_attribute {
    ($key:expr, $value:expr) => {{
        use opentelemetry::{Context, KeyValue, trace::TraceContextExt};
        let cx = Context::current();
        let span = cx.span();
        span.set_attribute(KeyValue::new($key, $value));
    }};
}

/// Record an event in the current span
#[macro_export]
macro_rules! span_event {
    ($name:expr) => {{
        use opentelemetry::{trace::TraceContextExt, Context};
        let cx = Context::current();
        let span = cx.span();
        span.add_event($name, vec![]);
    }};
    ($name:expr, $($key:expr => $value:expr),*) => {{
        use opentelemetry::{trace::TraceContextExt, Context, KeyValue};
        let cx = Context::current();
        let span = cx.span();
        let attributes = vec![
            $(KeyValue::new($key, $value)),*
        ];
        span.add_event($name, attributes);
    }};
}
