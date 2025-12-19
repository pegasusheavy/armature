import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-health-check-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class HealthCheckGuideComponent {
  page: DocPage = {
    title: 'Health Checks & Probes',
    subtitle: 'Kubernetes-ready health checks with liveness, readiness, and startup probes. Monitor database connections, external services, and system resources.',
    icon: '💚',
    badge: 'DevOps',
    features: [
      {
        icon: '❤️',
        title: 'Liveness Probes',
        description: 'Is the application alive and should be restarted?'
      },
      {
        icon: '✅',
        title: 'Readiness Probes',
        description: 'Is the application ready to receive traffic?'
      },
      {
        icon: '🚀',
        title: 'Startup Probes',
        description: 'Has the application finished starting up?'
      },
      {
        icon: '📊',
        title: 'Custom Indicators',
        description: 'Monitor any dependency or resource'
      }
    ],
    sections: [
      {
        id: 'quick-start',
        title: 'Quick Start',
        content: `<p>Add health checks to your application in minutes:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'main.rs',
            code: `use armature::prelude::*;
use armature_core::{HealthService, HealthServiceBuilder};

#[tokio::main]
async fn main() {
    // Build health service with default indicators
    let health_service = HealthServiceBuilder::new()
        .with_defaults()  // Memory, disk, uptime
        .with_info(|info| {
            info.name("my-api")
                .version("1.0.0")
                .description("Production API")
        })
        .build();

    // Register as provider
    let app = Application::create::<AppModule>().await;
    app.listen(3000).await.unwrap();
}

// Health endpoints are automatically available:
// GET /health       - Full health check
// GET /health/live  - Liveness probe
// GET /health/ready - Readiness probe`
          }
        ]
      },
      {
        id: 'health-endpoints',
        title: 'Health Endpoints',
        content: `<p>Three endpoints for different purposes:</p>`,
        subsections: [
          {
            id: 'full-health',
            title: '/health - Full Health Check',
            content: `<p>Returns detailed status of all health indicators:</p>`,
            codeBlocks: [
              {
                language: 'json',
                filename: 'Response (200 OK)',
                code: `{
  "status": "UP",
  "info": {
    "name": "my-api",
    "version": "1.0.0",
    "description": "Production API"
  },
  "details": {
    "memory": {
      "status": "UP",
      "used_mb": 128,
      "total_mb": 512,
      "percent": 25
    },
    "disk": {
      "status": "UP",
      "free_gb": 50,
      "total_gb": 100,
      "percent": 50
    },
    "uptime": {
      "status": "UP",
      "seconds": 86400,
      "human": "1 day"
    },
    "database": {
      "status": "UP",
      "latency_ms": 2
    }
  }
}`
              }
            ]
          },
          {
            id: 'liveness',
            title: '/health/live - Liveness Probe',
            content: `<p>Quick check if the application process is alive. Kubernetes restarts the pod if this fails.</p>`,
            codeBlocks: [
              {
                language: 'json',
                filename: 'Response (200 OK)',
                code: `{
  "status": "UP"
}`
              }
            ]
          },
          {
            id: 'readiness',
            title: '/health/ready - Readiness Probe',
            content: `<p>Check if the application can handle requests. Kubernetes removes from load balancer if this fails.</p>`,
            codeBlocks: [
              {
                language: 'json',
                filename: 'Response (503 Service Unavailable)',
                code: `{
  "status": "DOWN",
  "reason": "Database connection pool exhausted"
}`
              }
            ]
          }
        ]
      },
      {
        id: 'built-in-indicators',
        title: 'Built-in Health Indicators',
        content: `<p>Pre-built indicators for common checks:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature_core::{
    HealthServiceBuilder,
    MemoryHealthIndicator,
    DiskHealthIndicator,
    UptimeHealthIndicator,
};

let health = HealthServiceBuilder::new()
    // Memory usage (fail if > 90% used)
    .with_indicator(MemoryHealthIndicator::new(0.9))

    // Disk space (fail if < 10% free)
    .with_indicator(DiskHealthIndicator::new("/", 0.1))

    // Uptime tracking
    .with_indicator(UptimeHealthIndicator::default())

    .build();`
          }
        ]
      },
      {
        id: 'custom-indicators',
        title: 'Custom Health Indicators',
        content: `<p>Create custom indicators for your dependencies:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'database_indicator.rs',
            code: `use armature_core::{HealthIndicator, HealthStatus, HealthDetails};
use async_trait::async_trait;

struct DatabaseHealthIndicator {
    pool: DatabasePool,
}

#[async_trait]
impl HealthIndicator for DatabaseHealthIndicator {
    fn name(&self) -> &str {
        "database"
    }

    async fn check(&self) -> HealthStatus {
        let start = std::time::Instant::now();

        match self.pool.execute("SELECT 1").await {
            Ok(_) => {
                let latency = start.elapsed().as_millis();
                HealthStatus::up()
                    .with_detail("latency_ms", latency)
                    .with_detail("connections", self.pool.size())
            }
            Err(e) => {
                HealthStatus::down()
                    .with_detail("error", e.to_string())
            }
        }
    }
}

// Use it
let health = HealthServiceBuilder::new()
    .with_indicator(DatabaseHealthIndicator { pool })
    .build();`
          }
        ]
      },
      {
        id: 'external-services',
        title: 'External Service Checks',
        content: `<p>Monitor external APIs and services:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `struct ExternalApiIndicator {
    client: HttpClient,
    url: String,
    timeout: Duration,
}

#[async_trait]
impl HealthIndicator for ExternalApiIndicator {
    fn name(&self) -> &str {
        "external_api"
    }

    async fn check(&self) -> HealthStatus {
        let start = std::time::Instant::now();

        match tokio::time::timeout(
            self.timeout,
            self.client.get(&self.url).send()
        ).await {
            Ok(Ok(response)) if response.status().is_success() => {
                HealthStatus::up()
                    .with_detail("latency_ms", start.elapsed().as_millis())
            }
            Ok(Ok(response)) => {
                HealthStatus::down()
                    .with_detail("status", response.status().as_u16())
            }
            Ok(Err(e)) => {
                HealthStatus::down()
                    .with_detail("error", e.to_string())
            }
            Err(_) => {
                HealthStatus::down()
                    .with_detail("error", "Timeout")
            }
        }
    }
}`
          }
        ]
      },
      {
        id: 'kubernetes',
        title: 'Kubernetes Configuration',
        content: `<p>Configure probes in your Kubernetes deployment:</p>`,
        codeBlocks: [
          {
            language: 'yaml',
            filename: 'deployment.yaml',
            code: `apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-api
spec:
  template:
    spec:
      containers:
      - name: api
        image: my-api:latest
        ports:
        - containerPort: 3000

        # Liveness: restart if dead
        livenessProbe:
          httpGet:
            path: /health/live
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
          failureThreshold: 3

        # Readiness: remove from LB if not ready
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
          failureThreshold: 3

        # Startup: wait for slow starts
        startupProbe:
          httpGet:
            path: /health/live
            port: 3000
          initialDelaySeconds: 0
          periodSeconds: 5
          failureThreshold: 30`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Keep liveness probes fast</strong> — Don't check external dependencies</li>
          <li><strong>Use readiness for dependencies</strong> — Check DB, cache, external APIs</li>
          <li><strong>Set appropriate thresholds</strong> — Memory 90%, disk 10% free</li>
          <li><strong>Include timeouts</strong> — Don't let health checks hang</li>
          <li><strong>Monitor health endpoint</strong> — Alert on degraded status</li>
          <li><strong>Graceful degradation</strong> — Return DEGRADED instead of DOWN when possible</li>
        </ul>`
      }
    ],
    relatedDocs: [
      {
        id: 'metrics-guide',
        title: 'Prometheus Metrics',
        description: 'Export metrics for monitoring and alerting'
      },
      {
        id: 'graceful-shutdown',
        title: 'Graceful Shutdown',
        description: 'Handle shutdown signals properly'
      },
      {
        id: 'logging-guide',
        title: 'Logging Guide',
        description: 'Structured logging for observability'
      }
    ],
    seeAlso: [
      { title: 'OpenTelemetry', id: 'opentelemetry-guide' },
      { title: 'Error Tracking', id: 'error-correlation' },
      { title: 'Configuration', id: 'config-guide' }
    ]
  };
}

