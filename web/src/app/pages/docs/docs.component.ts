import { Component, OnInit, signal, effect } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';
import { HttpClient } from '@angular/common/http';
import { marked } from 'marked';
import { DomSanitizer, SafeHtml } from '@angular/platform-browser';
import { DocsOverviewComponent } from './overview/overview.component';

interface DocMetadata {
  id: string;
  title: string;
  filename: string;
  category: string;
}

@Component({
  selector: 'app-docs',
  standalone: true,
  imports: [CommonModule, RouterModule, DocsOverviewComponent],
  templateUrl: './docs.component.html',
  styleUrls: ['./docs.component.scss'],
})
export class DocsComponent implements OnInit {
  content = signal<SafeHtml>('');
  loading = signal(true);
  error = signal<string | null>(null);
  currentDoc = signal<DocMetadata | null>(null);
  isOverview = signal(false);

  // Documentation metadata with user-friendly titles
  docs: DocMetadata[] = [
    // Getting Started
    { id: 'readme', title: 'Overview', filename: '', category: 'Getting Started' },
    { id: 'di-guide', title: 'Dependency Injection', filename: 'di-guide.md', category: 'Getting Started' },
    { id: 'config-guide', title: 'Configuration Management', filename: 'config-guide.md', category: 'Getting Started' },
    { id: 'project-templates', title: 'Project Templates', filename: 'project-templates.md', category: 'Getting Started' },
    { id: 'lifecycle-hooks', title: 'Lifecycle Hooks', filename: 'lifecycle-hooks.md', category: 'Getting Started' },
    { id: 'macro-overview', title: 'Macros Overview', filename: 'macro-overview.md', category: 'Getting Started' },

    // Authentication & Security
    { id: 'auth-guide', title: 'Authentication Basics', filename: 'auth-guide.md', category: 'Authentication & Security' },
    { id: 'oauth2-providers', title: 'OAuth2 & Social Login', filename: 'oauth2-providers-guide.md', category: 'Authentication & Security' },
    { id: 'security-guide', title: 'Security Best Practices', filename: 'security-guide.md', category: 'Authentication & Security' },
    { id: 'security-advanced', title: 'Advanced Security (2FA, WebAuthn)', filename: 'security-advanced-guide.md', category: 'Authentication & Security' },
    { id: 'session-guide', title: 'Session Management', filename: 'session-guide.md', category: 'Authentication & Security' },
    { id: 'rate-limiting', title: 'Rate Limiting & Throttling', filename: 'rate-limiting-guide.md', category: 'Authentication & Security' },

    // Routing & Controllers
    { id: 'route-groups', title: 'Organizing Routes', filename: 'route-groups-guide.md', category: 'Routing & Controllers' },
    { id: 'route-constraints', title: 'Route Constraints & Validation', filename: 'route-constraints-guide.md', category: 'Routing & Controllers' },
    { id: 'guards-interceptors', title: 'Guards & Interceptors', filename: 'guards-interceptors.md', category: 'Routing & Controllers' },
    { id: 'use-guard', title: 'Using Guards', filename: 'use-guard-guide.md', category: 'Routing & Controllers' },
    { id: 'use-middleware', title: 'Using Middleware', filename: 'use-middleware-guide.md', category: 'Routing & Controllers' },
    { id: 'request-extractors', title: 'Request Extractors', filename: 'request-extractors.md', category: 'Routing & Controllers' },

    // API Features
    { id: 'api-versioning', title: 'Versioning Your API', filename: 'api-versioning-guide.md', category: 'API Features' },
    { id: 'content-negotiation', title: 'Content Negotiation', filename: 'content-negotiation-guide.md', category: 'API Features' },
    { id: 'pagination-filtering', title: 'Pagination, Sorting & Filtering', filename: 'pagination-filtering-guide.md', category: 'API Features' },
    { id: 'response-caching', title: 'HTTP Caching Headers', filename: 'response-caching-guide.md', category: 'API Features' },
    { id: 'etag-conditional', title: 'ETags & Conditional Requests', filename: 'etag-conditional-requests-guide.md', category: 'API Features' },

    // HTTP & Networking
    { id: 'https-guide', title: 'HTTPS & TLS Setup', filename: 'https-guide.md', category: 'HTTP & Networking' },
    { id: 'acme-certificates', title: 'Auto SSL Certificates', filename: 'acme-certificates.md', category: 'HTTP & Networking' },
    { id: 'compression', title: 'Response Compression', filename: 'compression.md', category: 'HTTP & Networking' },
    { id: 'http-status-errors', title: 'HTTP Status Codes & Errors', filename: 'http-status-errors.md', category: 'HTTP & Networking' },
    { id: 'request-timeouts', title: 'Request Timeouts', filename: 'request-timeouts-guide.md', category: 'HTTP & Networking' },
    { id: 'streaming-responses', title: 'Streaming Large Responses', filename: 'streaming-responses-guide.md', category: 'HTTP & Networking' },

    // Real-Time Communication
    { id: 'websocket-sse', title: 'WebSockets & Server-Sent Events', filename: 'websocket-sse-guide.md', category: 'Real-Time Communication' },
    { id: 'webhooks', title: 'Webhook Integration', filename: 'webhooks.md', category: 'Real-Time Communication' },

    // GraphQL
    { id: 'graphql-guide', title: 'Getting Started with GraphQL', filename: 'graphql-guide.md', category: 'GraphQL' },
    { id: 'graphql-config', title: 'Advanced GraphQL Configuration', filename: 'graphql-configuration.md', category: 'GraphQL' },

    // OpenAPI
    { id: 'openapi-guide', title: 'OpenAPI & Swagger Documentation', filename: 'openapi-guide.md', category: 'OpenAPI' },

    // Background Processing
    { id: 'queue-guide', title: 'Background Jobs & Queues', filename: 'queue-guide.md', category: 'Background Processing' },
    { id: 'cron-guide', title: 'Scheduled Tasks (Cron)', filename: 'cron-guide.md', category: 'Background Processing' },
    { id: 'graceful-shutdown', title: 'Graceful Shutdown', filename: 'graceful-shutdown-guide.md', category: 'Background Processing' },

    // Caching
    { id: 'cache-improvements', title: 'Caching Strategies', filename: 'cache-improvements-guide.md', category: 'Caching' },
    { id: 'redis-guide', title: 'Redis Integration', filename: 'redis-guide.md', category: 'Caching' },

    // Cloud Providers
    { id: 'cloud-providers', title: 'Cloud Provider SDKs', filename: 'cloud-providers-guide.md', category: 'Cloud Providers' },

    // Observability
    { id: 'logging-guide', title: 'Structured Logging', filename: 'logging-guide.md', category: 'Observability' },
    { id: 'debug-logging', title: 'Debug & Development Logging', filename: 'debug-logging-guide.md', category: 'Observability' },
    { id: 'opentelemetry-guide', title: 'Distributed Tracing (OpenTelemetry)', filename: 'opentelemetry-guide.md', category: 'Observability' },
    { id: 'metrics-guide', title: 'Prometheus Metrics', filename: 'metrics-guide.md', category: 'Observability' },
    { id: 'health-check', title: 'Health Checks & Probes', filename: 'health-check-guide.md', category: 'Observability' },
    { id: 'error-correlation', title: 'Error Tracking & Correlation', filename: 'error-correlation-guide.md', category: 'Observability' },
    { id: 'audit-guide', title: 'Audit Logging', filename: 'audit-guide.md', category: 'Observability' },

    // Architecture
    { id: 'stateless-architecture', title: 'Building Stateless Services', filename: 'stateless-architecture.md', category: 'Architecture' },
    { id: 'server-integration', title: 'Server & Runtime Integration', filename: 'server-integration.md', category: 'Architecture' },

    // Macros
    { id: 'macros-guide', title: 'Macro System Deep Dive', filename: 'macros-guide.md', category: 'Macros' },

    // Error Handling
    { id: 'error-transformation', title: 'Custom Error Handling', filename: 'error-transformation-guide.md', category: 'Error Handling' },

    // Testing & Quality
    { id: 'testing-guide', title: 'Testing Your Application', filename: 'testing-guide.md', category: 'Testing & Quality' },
    { id: 'testing-coverage', title: 'Code Coverage', filename: 'testing-coverage.md', category: 'Testing & Quality' },
    { id: 'testing-documentation', title: 'Testing Best Practices', filename: 'testing-documentation.md', category: 'Testing & Quality' },
    { id: 'documentation-testing', title: 'Documentation Examples', filename: 'documentation-testing.md', category: 'Testing & Quality' },

    // Benchmarks
    { id: 'armature-vs-nodejs', title: 'Performance vs Node.js', filename: 'armature-vs-nodejs-benchmark.md', category: 'Benchmarks' },
    { id: 'armature-vs-nextjs', title: 'Performance vs Next.js', filename: 'armature-vs-nextjs-benchmark.md', category: 'Benchmarks' },
  ];

  // Group docs by category
  docsByCategory = signal<{ [key: string]: DocMetadata[] }>({});

  constructor(
    private route: ActivatedRoute,
    private router: Router,
    private http: HttpClient,
    private sanitizer: DomSanitizer
  ) {
    // Configure marked options
    marked.setOptions({
      gfm: true,
      breaks: true,
    });

    // Group docs by category
    const grouped: { [key: string]: DocMetadata[] } = {};
    this.docs.forEach((doc) => {
      if (!grouped[doc.category]) {
        grouped[doc.category] = [];
      }
      grouped[doc.category].push(doc);
    });
    this.docsByCategory.set(grouped);

    // Watch for route changes
    effect(() => {
      const docId = this.route.snapshot.paramMap.get('id');
      if (docId) {
        this.loadDoc(docId);
      }
    });
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

  async loadDoc(docId: string) {
    this.loading.set(true);
    this.error.set(null);
    this.isOverview.set(false);

    const doc = this.docs.find((d) => d.id === docId);
    if (!doc) {
      this.error.set('Documentation not found');
      this.loading.set(false);
      return;
    }

    this.currentDoc.set(doc);

    // Handle overview page specially - it uses a component, not markdown
    if (docId === 'readme') {
      this.isOverview.set(true);
      this.loading.set(false);
      return;
    }

    try {
      const markdown = await this.http
        .get(`docs/${doc.filename}`, { responseType: 'text' })
        .toPromise();

      if (markdown) {
        const html = await marked.parse(markdown);
        this.content.set(this.sanitizer.bypassSecurityTrustHtml(html as string));
      }
    } catch (err) {
      this.error.set('Failed to load documentation');
      console.error('Error loading doc:', err);
    } finally {
      this.loading.set(false);
    }
  }

  getCategories(): string[] {
    return Object.keys(this.docsByCategory());
  }
}
