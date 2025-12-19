import { Component, OnInit, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';
import { DocsOverviewComponent } from './overview/overview.component';

// Import doc page components
import { AuthGuideComponent } from './pages/auth-guide/auth-guide.component';
import { DiGuideComponent } from './pages/di-guide/di-guide.component';
import { GraphqlGuideComponent } from './pages/graphql-guide/graphql-guide.component';
import { WebsocketSseGuideComponent } from './pages/websocket-sse-guide/websocket-sse-guide.component';
import { CacheGuideComponent } from './pages/cache-guide/cache-guide.component';
import { LoggingGuideComponent } from './pages/logging-guide/logging-guide.component';
import { HealthCheckGuideComponent } from './pages/health-check-guide/health-check-guide.component';
import { GrafanaDashboardsComponent } from './pages/grafana-dashboards/grafana-dashboards.component';
import { DocFerronComponent } from './ferron/ferron.component';

// New doc page components
import { ProjectTemplatesComponent } from './pages/project-templates/project-templates.component';
import { ConfigGuideComponent } from './pages/config-guide/config-guide.component';
import { LifecycleHooksComponent } from './pages/lifecycle-hooks/lifecycle-hooks.component';
import { RouteGroupsComponent } from './pages/route-groups/route-groups.component';
import { MiddlewareGuideComponent } from './pages/middleware-guide/middleware-guide.component';
import { RateLimitingComponent } from './pages/rate-limiting/rate-limiting.component';
import { ApiVersioningComponent } from './pages/api-versioning/api-versioning.component';
import { RedisGuideComponent } from './pages/redis-guide/redis-guide.component';
import { QueueGuideComponent } from './pages/queue-guide/queue-guide.component';
import { CronGuideComponent } from './pages/cron-guide/cron-guide.component';
import { GracefulShutdownComponent } from './pages/graceful-shutdown/graceful-shutdown.component';
import { DockerGuideComponent } from './pages/docker-guide/docker-guide.component';
import { MetricsGuideComponent } from './pages/metrics-guide/metrics-guide.component';
import { TestingGuideComponent } from './pages/testing-guide/testing-guide.component';
import { KubernetesGuideComponent } from './pages/kubernetes-guide/kubernetes-guide.component';
import { SessionGuideComponent } from './pages/session-guide/session-guide.component';
import { WebhooksGuideComponent } from './pages/webhooks-guide/webhooks-guide.component';
import { OpenTelemetryGuideComponent } from './pages/opentelemetry-guide/opentelemetry-guide.component';
import { ProfilingGuideComponent } from './pages/profiling-guide/profiling-guide.component';

interface DocMetadata {
  id: string;
  title: string;
  category: string;
  component?: any;
  hasComponent?: boolean;
}

@Component({
  selector: 'app-docs',
  standalone: true,
  imports: [
    CommonModule,
    RouterModule,
    DocsOverviewComponent,
    AuthGuideComponent,
    DiGuideComponent,
    GraphqlGuideComponent,
    WebsocketSseGuideComponent,
    CacheGuideComponent,
    LoggingGuideComponent,
    HealthCheckGuideComponent,
    GrafanaDashboardsComponent,
    DocFerronComponent,
    ProjectTemplatesComponent,
    ConfigGuideComponent,
    LifecycleHooksComponent,
    RouteGroupsComponent,
    MiddlewareGuideComponent,
    RateLimitingComponent,
    ApiVersioningComponent,
    RedisGuideComponent,
    QueueGuideComponent,
    CronGuideComponent,
    GracefulShutdownComponent,
    DockerGuideComponent,
    MetricsGuideComponent,
    TestingGuideComponent,
    KubernetesGuideComponent,
    SessionGuideComponent,
    WebhooksGuideComponent,
    OpenTelemetryGuideComponent,
    ProfilingGuideComponent,
  ],
  templateUrl: './docs.component.html',
  styleUrls: ['./docs.component.scss'],
})
export class DocsComponent implements OnInit {
  loading = signal(true);
  error = signal<string | null>(null);
  currentDoc = signal<DocMetadata | null>(null);
  activeComponent = signal<string>('overview');

  // Documentation metadata with user-friendly titles
  docs: DocMetadata[] = [
    // 1. Getting Started - First things first
    { id: 'readme', title: 'Overview', category: 'Getting Started', hasComponent: true },
    { id: 'project-templates', title: 'Project Templates', category: 'Getting Started', hasComponent: true },
    { id: 'config-guide', title: 'Configuration', category: 'Getting Started', hasComponent: true },
    { id: 'macro-overview', title: 'Macros Overview', category: 'Getting Started' },

    // 2. Core Concepts - Foundation before building
    { id: 'di-guide', title: 'Dependency Injection', category: 'Core Concepts', hasComponent: true },
    { id: 'lifecycle-hooks', title: 'Lifecycle Hooks', category: 'Core Concepts', hasComponent: true },

    // 3. Routing & Controllers - Learn to handle requests first
    { id: 'route-groups', title: 'Route Groups', category: 'Routing', hasComponent: true },
    { id: 'route-constraints', title: 'Route Constraints', category: 'Routing' },
    { id: 'request-extractors', title: 'Request Extractors', category: 'Routing' },
    { id: 'use-middleware', title: 'Middleware', category: 'Routing', hasComponent: true },
    { id: 'guards-interceptors', title: 'Guards & Interceptors', category: 'Routing' },
    { id: 'use-guard', title: 'Using Guards', category: 'Routing' },

    // 4. Request & Response - HTTP handling
    { id: 'http-status-errors', title: 'HTTP Errors', category: 'HTTP' },
    { id: 'request-timeouts', title: 'Timeouts', category: 'HTTP' },
    { id: 'streaming-responses', title: 'Streaming', category: 'HTTP' },
    { id: 'compression', title: 'Compression', category: 'HTTP' },
    { id: 'https-guide', title: 'HTTPS & TLS', category: 'HTTP' },
    { id: 'acme-certificates', title: 'SSL Certificates', category: 'HTTP' },

    // 5. Security - After understanding routing
    { id: 'auth-guide', title: 'Authentication', category: 'Security', hasComponent: true },
    { id: 'oauth2-providers', title: 'OAuth2 & Social Login', category: 'Security' },
    { id: 'session-guide', title: 'Sessions', category: 'Security', hasComponent: true },
    { id: 'rate-limiting', title: 'Rate Limiting', category: 'Security', hasComponent: true },
    { id: 'security-guide', title: 'Security Best Practices', category: 'Security' },
    { id: 'security-advanced', title: 'Advanced Security', category: 'Security' },

    // 6. API Features - Building APIs
    { id: 'api-versioning', title: 'API Versioning', category: 'API Features', hasComponent: true },
    { id: 'pagination-filtering', title: 'Pagination & Filtering', category: 'API Features' },
    { id: 'content-negotiation', title: 'Content Negotiation', category: 'API Features' },
    { id: 'response-caching', title: 'HTTP Caching', category: 'API Features' },
    { id: 'etag-conditional', title: 'ETags', category: 'API Features' },

    // 7. Data & Caching - Persistence layer
    { id: 'cache-improvements', title: 'Caching Strategies', category: 'Data & Caching', hasComponent: true },
    { id: 'redis-guide', title: 'Redis', category: 'Data & Caching', hasComponent: true },

    // 8. Background Processing - Async work
    { id: 'queue-guide', title: 'Job Queues', category: 'Background Jobs', hasComponent: true },
    { id: 'cron-guide', title: 'Scheduled Tasks', category: 'Background Jobs', hasComponent: true },
    { id: 'graceful-shutdown', title: 'Graceful Shutdown', category: 'Background Jobs', hasComponent: true },

    // 9. Real-Time - Live communication
    { id: 'websocket-sse', title: 'WebSockets & SSE', category: 'Real-Time', hasComponent: true },
    { id: 'webhooks', title: 'Webhooks', category: 'Real-Time', hasComponent: true },

    // 10. GraphQL - Alternative API paradigm
    { id: 'graphql-guide', title: 'GraphQL', category: 'GraphQL', hasComponent: true },
    { id: 'graphql-config', title: 'GraphQL Config', category: 'GraphQL' },

    // 11. Cloud & Deployment - Production readiness
    { id: 'cloud-providers', title: 'Cloud SDKs', category: 'Cloud & Deployment' },
    { id: 'deployment-guide', title: 'Deployment Guide', category: 'Cloud & Deployment' },
    { id: 'docker-guide', title: 'Docker', category: 'Cloud & Deployment', hasComponent: true },
    { id: 'kubernetes-guide', title: 'Kubernetes', category: 'Cloud & Deployment', hasComponent: true },
    { id: 'ferron-guide', title: 'Ferron Reverse Proxy', category: 'Cloud & Deployment', hasComponent: true },
    { id: 'scaling-guide', title: 'Scaling', category: 'Cloud & Deployment' },

    // 12. Observability - Monitoring and debugging
    { id: 'logging-guide', title: 'Logging', category: 'Observability', hasComponent: true },
    { id: 'debug-logging', title: 'Debug Logging', category: 'Observability' },
    { id: 'health-check', title: 'Health Checks', category: 'Observability', hasComponent: true },
    { id: 'metrics-guide', title: 'Metrics', category: 'Observability', hasComponent: true },
    { id: 'grafana-dashboards', title: 'Grafana Dashboards', category: 'Observability', hasComponent: true },
    { id: 'opentelemetry-guide', title: 'OpenTelemetry', category: 'Observability', hasComponent: true },
    { id: 'error-correlation', title: 'Error Tracking', category: 'Observability' },
    { id: 'audit-guide', title: 'Audit Logging', category: 'Observability' },

    // 13. Testing - Quality assurance
    { id: 'testing-guide', title: 'Testing', category: 'Testing', hasComponent: true },
    { id: 'testing-coverage', title: 'Coverage', category: 'Testing' },

    // 14. Performance - Optimization tools
    { id: 'profiling-guide', title: 'CPU Profiling', category: 'Performance', hasComponent: true },

    // 15. Benchmarks - Performance comparisons
    { id: 'armature-vs-nodejs', title: 'vs Node.js', category: 'Benchmarks' },
    { id: 'armature-vs-nextjs', title: 'vs Next.js', category: 'Benchmarks' },
  ];

  // Group docs by category
  docsByCategory = signal<{ [key: string]: DocMetadata[] }>({});

  constructor(
    private route: ActivatedRoute,
    private router: Router
  ) {
    // Group docs by category
    const grouped: { [key: string]: DocMetadata[] } = {};
    this.docs.forEach((doc) => {
      if (!grouped[doc.category]) {
        grouped[doc.category] = [];
      }
      grouped[doc.category].push(doc);
    });
    this.docsByCategory.set(grouped);
  }

  ngOnInit() {
    this.route.paramMap.subscribe((params) => {
      const docId = params.get('id');
      if (docId) {
        this.loadDoc(docId);
      } else {
        // Default to README
        this.router.navigate(['/docs/readme']);
      }
    });
  }

  loadDoc(docId: string) {
    this.loading.set(true);
    this.error.set(null);

    const doc = this.docs.find((d) => d.id === docId);
    if (!doc) {
      this.error.set('Documentation not found');
      this.loading.set(false);
      return;
    }

    this.currentDoc.set(doc);
    this.activeComponent.set(docId);
    this.loading.set(false);

    // Scroll to top on navigation
    window.scrollTo({ top: 0, behavior: 'instant' });
  }

  getCategories(): string[] {
    return Object.keys(this.docsByCategory());
  }

  isActiveDoc(docId: string): boolean {
    return this.currentDoc()?.id === docId;
  }
}
