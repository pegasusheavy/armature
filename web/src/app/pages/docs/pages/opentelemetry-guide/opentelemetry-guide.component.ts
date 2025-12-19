import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-opentelemetry-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class OpenTelemetryGuideComponent {
  page: DocPage = {
    title: 'OpenTelemetry',
    subtitle: 'Distributed tracing and observability with automatic instrumentation for requests, database calls, and external services.',
    icon: '🔭',
    badge: 'Observability',
    features: [
      { icon: '📊', title: 'Distributed Tracing', description: 'Track requests across services' },
      { icon: '🏷️', title: 'Auto-instrumentation', description: 'HTTP, DB, Redis traced' },
      { icon: '📈', title: 'Metrics Export', description: 'Prometheus, OTLP, Jaeger' },
      { icon: '🔗', title: 'Context Propagation', description: 'W3C Trace Context' }
    ],
    sections: [
      {
        id: 'setup',
        title: 'Basic Setup',
        content: `<p>Enable OpenTelemetry with automatic instrumentation:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature::prelude::*;
use armature_telemetry::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize OpenTelemetry
    let telemetry = TelemetryConfig::builder()
        .service_name("my-service")
        .otlp_endpoint("http://localhost:4317")  // Jaeger/OTLP collector
        .enable_tracing(true)
        .enable_metrics(true)
        .build()?;

    init_telemetry(telemetry)?;

    Application::new()
        .middleware(TracingMiddleware::new())
        .run()
        .await
}`
          }
        ]
      },
      {
        id: 'auto-instrumentation',
        title: 'Automatic Instrumentation',
        content: `<p>These are automatically traced:</p>
        <ul>
          <li><strong>HTTP requests</strong> — Incoming and outgoing</li>
          <li><strong>Database queries</strong> — PostgreSQL, MySQL, SQLite</li>
          <li><strong>Redis operations</strong> — Commands and pipelines</li>
          <li><strong>gRPC calls</strong> — Client and server</li>
          <li><strong>Message queues</strong> — Producer and consumer</li>
        </ul>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `# Example trace output:
[trace_id=abc123] POST /api/orders (23ms)
  └── [span] validate_request (1ms)
  └── [span] database.query (5ms)
      └── SELECT * FROM products WHERE id = $1
  └── [span] redis.get (2ms)
      └── GET cache:product:123
  └── [span] http.client (12ms)
      └── POST https://payment.example.com/charge`
          }
        ]
      },
      {
        id: 'custom-spans',
        title: 'Custom Spans',
        content: `<p>Add custom spans for business logic:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature_telemetry::tracing;

#[instrument(name = "process_order")]
async fn process_order(order_id: i64) -> Result<Order, Error> {
    // Add custom attributes
    tracing::Span::current()
        .set_attribute("order.id", order_id)
        .set_attribute("order.type", "subscription");

    // Create child span
    let _validation = tracing::span!("validate_inventory");
    validate_inventory(&order).await?;
    drop(_validation);

    // Record events
    tracing::info!(
        order_id = order_id,
        "Order processing started"
    );

    let result = charge_payment(&order).await?;

    tracing::info!(
        order_id = order_id,
        amount = result.amount,
        "Payment successful"
    );

    Ok(result)
}`
          }
        ]
      },
      {
        id: 'context-propagation',
        title: 'Context Propagation',
        content: `<p>Trace context is automatically propagated across services:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// Incoming request headers are automatically parsed:
// traceparent: 00-abc123-def456-01
// tracestate: key=value

// When making outgoing HTTP calls, context is injected:
let response = http_client
    .get("https://other-service.internal/api/data")
    // traceparent header automatically added
    .send()
    .await?;

// For custom propagation:
use armature_telemetry::propagation::inject_context;

let mut headers = HashMap::new();
inject_context(&mut headers);
// headers now contains traceparent and tracestate`
          }
        ]
      },
      {
        id: 'exporters',
        title: 'Exporters',
        content: `<p>Send traces to various backends:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// OTLP (Jaeger, Tempo, etc.)
let config = TelemetryConfig::builder()
    .otlp_endpoint("http://jaeger:4317")
    .build()?;

// Jaeger native
let config = TelemetryConfig::builder()
    .jaeger_endpoint("http://jaeger:14268/api/traces")
    .build()?;

// Zipkin
let config = TelemetryConfig::builder()
    .zipkin_endpoint("http://zipkin:9411/api/v2/spans")
    .build()?;

// Multiple exporters
let config = TelemetryConfig::builder()
    .otlp_endpoint("http://collector:4317")
    .stdout(true)  // Also print to console
    .build()?;`
          }
        ]
      },
      {
        id: 'sampling',
        title: 'Sampling',
        content: `<p>Control trace sampling for high-traffic services:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `let config = TelemetryConfig::builder()
    .sampler(Sampler::TraceIdRatioBased(0.1))  // 10% of traces
    .build()?;

// Or use parent-based sampling
let config = TelemetryConfig::builder()
    .sampler(Sampler::ParentBased {
        root: Box::new(Sampler::TraceIdRatioBased(0.1)),
    })
    .build()?;

// Always sample errors
let config = TelemetryConfig::builder()
    .sampler(Sampler::AlwaysOnForErrors)
    .build()?;`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Use semantic conventions</strong> — Follow OpenTelemetry naming standards</li>
          <li><strong>Sample in production</strong> — 100% sampling is expensive</li>
          <li><strong>Add business context</strong> — user_id, order_id, etc.</li>
          <li><strong>Propagate context</strong> — Enable cross-service tracing</li>
          <li><strong>Don't over-instrument</strong> — Focus on important paths</li>
          <li><strong>Set up alerts</strong> — High latency, error rates</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'metrics-guide', title: 'Metrics', description: 'Prometheus metrics export' },
      { id: 'logging-guide', title: 'Logging', description: 'Correlate logs with traces' }
    ],
    seeAlso: [
      { title: 'Grafana Dashboards', id: 'grafana-dashboards' },
      { title: 'Error Tracking', id: 'error-correlation' }
    ]
  };
}

