import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-graceful-shutdown',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class GracefulShutdownComponent {
  page: DocPage = {
    title: 'Graceful Shutdown',
    subtitle: 'Safely shut down your application by draining connections, completing in-flight requests, and cleaning up resources.',
    icon: '🛑',
    badge: 'Background',
    features: [
      { icon: '⏳', title: 'Connection Draining', description: 'Wait for active requests' },
      { icon: '🔌', title: 'Shutdown Hooks', description: 'Custom cleanup logic' },
      { icon: '💚', title: 'Health Update', description: 'Mark unhealthy during shutdown' },
      { icon: '⏱️', title: 'Timeout', description: 'Force shutdown after timeout' }
    ],
    sections: [
      {
        id: 'basic-shutdown',
        title: 'Basic Graceful Shutdown',
        content: `<p>Armature handles graceful shutdown automatically:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Application::new()
        .shutdown_timeout(Duration::from_secs(30))  // Max wait time
        .run()
        .await
}

// When SIGTERM/SIGINT is received:
// 1. Stop accepting new connections
// 2. Wait for in-flight requests (up to timeout)
// 3. Close all connections
// 4. Exit`
          }
        ]
      },
      {
        id: 'shutdown-hooks',
        title: 'Shutdown Hooks',
        content: `<p>Run custom cleanup during shutdown:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `#[injectable]
pub struct CleanupService {
    db: DatabasePool,
    cache: CacheClient,
    queue: JobQueue,
}

#[async_trait]
impl OnApplicationShutdown for CleanupService {
    async fn on_application_shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting graceful shutdown...");

        // Stop accepting new jobs
        self.queue.stop().await;

        // Wait for in-flight jobs (with timeout)
        let _ = tokio::time::timeout(
            Duration::from_secs(10),
            self.queue.drain()
        ).await;

        // Flush cache writes
        self.cache.flush().await?;

        // Close database connections
        self.db.close().await;

        // Deregister from service discovery
        deregister_from_consul().await?;

        info!("Shutdown complete");
        Ok(())
    }
}`
          }
        ]
      },
      {
        id: 'health-integration',
        title: 'Health Check Integration',
        content: `<p>Update health status during shutdown:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// Health check automatically returns unhealthy during shutdown

// GET /health/ready during normal operation:
{
    "status": "healthy",
    "checks": { ... }
}

// GET /health/ready during shutdown:
{
    "status": "shutting_down",
    "message": "Application is shutting down"
}

// Kubernetes will stop sending traffic when readiness fails`
          }
        ]
      },
      {
        id: 'connection-draining',
        title: 'Connection Draining',
        content: `<p>Configure how connections are drained:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `Application::new()
    .shutdown_config(ShutdownConfig {
        // Maximum time to wait for requests to complete
        timeout: Duration::from_secs(30),

        // Delay before stopping to accept connections
        // (allows load balancer to update)
        delay: Duration::from_secs(5),

        // Force close connections after timeout
        force_close: true,
    })
    .run()
    .await`
          }
        ]
      },
      {
        id: 'kubernetes',
        title: 'Kubernetes Integration',
        content: `<p>Configure proper shutdown for Kubernetes:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            filename: 'deployment.yaml',
            code: `apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      terminationGracePeriodSeconds: 60  # Must be > app timeout
      containers:
        - name: app
          lifecycle:
            preStop:
              exec:
                # Give time for endpoints to update
                command: ["sleep", "5"]
          readinessProbe:
            httpGet:
              path: /health/ready
              port: 3000
            periodSeconds: 5`
          }
        ]
      },
      {
        id: 'signals',
        title: 'Signal Handling',
        content: `<p>Armature responds to standard Unix signals:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `# Graceful shutdown (allows draining)
$ kill -SIGTERM <pid>

# Also triggers graceful shutdown
$ kill -SIGINT <pid>  # Ctrl+C

# Force immediate shutdown (skip draining)
$ kill -SIGKILL <pid>  # Not recommended

# From Docker
$ docker stop <container>  # Sends SIGTERM, then SIGKILL after 10s

# From Kubernetes
$ kubectl delete pod <pod>  # Sends SIGTERM, respects terminationGracePeriodSeconds`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Set reasonable timeouts</strong> — Long enough for requests, short enough for deploys</li>
          <li><strong>Update health check first</strong> — Stop traffic before draining</li>
          <li><strong>Close resources properly</strong> — Don't leak connections or file handles</li>
          <li><strong>Log shutdown progress</strong> — Helps debug stuck shutdowns</li>
          <li><strong>Handle stuck requests</strong> — Force close after timeout</li>
          <li><strong>Test shutdown behavior</strong> — Simulate in staging</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'health-check', title: 'Health Checks', description: 'Readiness probes' },
      { id: 'lifecycle-hooks', title: 'Lifecycle Hooks', description: 'OnShutdown hook' }
    ],
    seeAlso: [
      { title: 'Kubernetes', id: 'kubernetes-guide' },
      { title: 'Docker', id: 'docker-guide' }
    ]
  };
}

