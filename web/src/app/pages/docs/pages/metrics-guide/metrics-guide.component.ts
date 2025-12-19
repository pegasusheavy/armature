import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-metrics-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class MetricsGuideComponent {
  page: DocPage = {
    title: 'Prometheus Metrics',
    subtitle: 'Export application metrics for monitoring with Prometheus, including request metrics, custom counters, and business KPIs.',
    icon: '📊',
    badge: 'Observability',
    features: [
      { icon: '📈', title: 'Request Metrics', description: 'Latency, throughput, errors' },
      { icon: '🔢', title: 'Custom Counters', description: 'Track business events' },
      { icon: '📉', title: 'Histograms', description: 'Distribution statistics' },
      { icon: '🎯', title: 'Labels', description: 'Dimensional metrics' }
    ],
    sections: [
      {
        id: 'setup',
        title: 'Basic Setup',
        content: `<p>Enable the metrics endpoint:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature::prelude::*;
use armature_metrics::*;

Application::new()
    .configure(|app| {
        // Enable /metrics endpoint
        app.metrics(MetricsConfig::default());
    })
    .run()
    .await`
          },
          {
            language: 'bash',
            code: `# Access metrics
$ curl http://localhost:3000/metrics

# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="GET",path="/api/users",status="200"} 1523
http_requests_total{method="POST",path="/api/users",status="201"} 89

# HELP http_request_duration_seconds HTTP request duration
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{le="0.005"} 1200
http_request_duration_seconds_bucket{le="0.01"} 1450
...`
          }
        ]
      },
      {
        id: 'request-metrics',
        title: 'Automatic Request Metrics',
        content: `<p>The following metrics are collected automatically:</p>
        <ul>
          <li><code>http_requests_total</code> — Counter of requests by method, path, status</li>
          <li><code>http_request_duration_seconds</code> — Histogram of request latency</li>
          <li><code>http_requests_in_flight</code> — Gauge of concurrent requests</li>
          <li><code>http_request_size_bytes</code> — Histogram of request body sizes</li>
          <li><code>http_response_size_bytes</code> — Histogram of response body sizes</li>
        </ul>`
      },
      {
        id: 'custom-counters',
        title: 'Custom Counters',
        content: `<p>Track custom events with counters:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature_metrics::*;

#[injectable]
pub struct UserService {
    metrics: MetricsRegistry,
}

impl UserService {
    pub async fn register_user(&self, user: CreateUser) -> Result<User, Error> {
        let result = self.do_registration(user).await;

        match &result {
            Ok(_) => {
                // Increment success counter
                self.metrics.counter("user_registrations_total")
                    .with_label("status", "success")
                    .inc();
            }
            Err(e) => {
                // Increment failure counter with reason
                self.metrics.counter("user_registrations_total")
                    .with_label("status", "failure")
                    .with_label("reason", e.code())
                    .inc();
            }
        }

        result
    }
}`
          }
        ]
      },
      {
        id: 'histograms',
        title: 'Histograms',
        content: `<p>Track value distributions:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[injectable]
pub struct PaymentService {
    metrics: MetricsRegistry,
}

impl PaymentService {
    pub async fn process_payment(&self, amount: f64) -> Result<Payment, Error> {
        let start = Instant::now();

        let result = self.charge(amount).await;

        // Record processing time
        self.metrics.histogram("payment_processing_seconds")
            .with_label("provider", "stripe")
            .observe(start.elapsed().as_secs_f64());

        // Record payment amount
        self.metrics.histogram("payment_amount_dollars")
            .with_label("currency", "USD")
            .observe(amount);

        result
    }
}`
          }
        ]
      },
      {
        id: 'gauges',
        title: 'Gauges',
        content: `<p>Track current values that can go up or down:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[injectable]
pub struct ConnectionPool {
    metrics: MetricsRegistry,
}

impl ConnectionPool {
    pub async fn acquire(&self) -> Connection {
        self.metrics.gauge("db_connections_active").inc();
        let conn = self.pool.acquire().await;
        conn
    }

    pub fn release(&self, _conn: Connection) {
        self.metrics.gauge("db_connections_active").dec();
    }

    pub fn update_pool_stats(&self) {
        let stats = self.pool.status();
        self.metrics.gauge("db_connections_idle")
            .set(stats.idle_connections as f64);
        self.metrics.gauge("db_connections_total")
            .set(stats.total_connections as f64);
    }
}`
          }
        ]
      },
      {
        id: 'labels',
        title: 'Using Labels',
        content: `<p>Add dimensions to metrics with labels:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// Good: Low cardinality labels
metrics.counter("api_requests_total")
    .with_label("method", "GET")        // ~5 values
    .with_label("status", "200")        // ~10 values
    .with_label("endpoint", "/users")   // ~50 values
    .inc();

// Bad: High cardinality labels (avoid!)
metrics.counter("api_requests_total")
    .with_label("user_id", &user.id)    // Millions of values!
    .with_label("request_id", &req_id)  // Infinite values!
    .inc();`
          }
        ]
      },
      {
        id: 'prometheus-config',
        title: 'Prometheus Configuration',
        content: `<p>Configure Prometheus to scrape your application:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            filename: 'prometheus.yml',
            code: `global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'armature-app'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: /metrics
    scheme: http`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Use consistent naming</strong> — <code>noun_verb_unit</code> (e.g., <code>http_requests_total</code>)</li>
          <li><strong>Keep label cardinality low</strong> — Avoid user IDs, request IDs as labels</li>
          <li><strong>Use histograms for latency</strong> — Not gauges or summaries</li>
          <li><strong>Document your metrics</strong> — Add HELP descriptions</li>
          <li><strong>Set up alerts</strong> — Metrics are useless without alerting</li>
          <li><strong>Use Grafana dashboards</strong> — Visualize your metrics</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'grafana-dashboards', title: 'Grafana Dashboards', description: 'Pre-built dashboards' },
      { id: 'opentelemetry-guide', title: 'OpenTelemetry', description: 'Distributed tracing' }
    ],
    seeAlso: [
      { title: 'Health Checks', id: 'health-check' },
      { title: 'Logging', id: 'logging-guide' }
    ]
  };
}

